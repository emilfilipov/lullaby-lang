//! Native backend: inline-`asm` operand marshalling.
//!
//! Extends the raw-byte `asm` statement so it can move Lullaby values into and
//! out of explicitly-named 64-bit registers and preserve the ABI across a body
//! that clobbers callee-saved registers. There is no assembler here — the bytes
//! are still emitted verbatim; this module only wraps them with the register
//! marshalling.
//!
//! ## The marshalling sequence
//!
//! For an `asm` with inputs `I`, outputs `O`, and the set `CS` of callee-saved
//! registers it *touches* (via any `in`/`out`/`clobber`), the emitted sequence is:
//!
//! 1. **Save** each register in `CS` into a reserved frame scratch slot.
//! 2. **Stage** every input: evaluate its expression into `rax`, spill `rax` to a
//!    reserved scratch slot. All inputs are staged *before* any register is
//!    loaded, so a later input's evaluation cannot clobber an earlier input's
//!    register.
//! 3. **Load** each staged input from its scratch slot into its target register.
//! 4. Emit the machine-code **bytes** verbatim.
//! 5. **Write** each output: `mov [rbp - localslot], <reg>` (read *before* the
//!    clobber restore, so an output in a callee-saved register is captured first).
//! 6. **Restore** each register in `CS` from its scratch slot.
//!
//! All staging/save slots are `rbp`-relative frame words reserved by
//! `NativeCtx::plan` (see `max_asm_scratch_words`), never `rsp` pushes — so the
//! marshalling never perturbs stack alignment and a call inside an input
//! expression stays 16-byte-aligned per the ABI.
//!
//! ## The two soundness invariants
//!
//! * **Register-promotion exclusion.** A function containing `asm` is never
//!   register-promoted (`instr_reg_promotable` has no `Asm` arm), so every local
//!   stays frame-resident. That is what lets `out <reg> = local` write
//!   `[rbp - slot]` authoritatively; a promoted output local would drop the write.
//!   This module asserts the output local is not promoted and refuses rather than
//!   miscompiles if it ever were.
//! * **Callee-saved preservation.** A non-promoting function otherwise saves no
//!   callee-saved registers, so an `asm` that touches one would return it
//!   corrupted to the caller. Steps 1/6 above save and restore every callee-saved
//!   register the `asm` touches, using the **union** of the Win64 and SysV
//!   callee-saved sets (`rbx`, `rsi`, `rdi`, `r12`–`r15`) so the preservation is
//!   correct regardless of target.

use super::super::*;
use crate::{BcAsmOperand, IrAsmClobber, IrAsmReg};

/// The frame slot displacement for asm scratch word `word` (0-based). Words are
/// laid out contiguously from `asm_scratch_base`, one 8-byte cell each, exactly
/// like the enum-scrutinee scratch region. `plan` reserves
/// `max_asm_scratch_words` of them (plus a guard word), so every index a single
/// `asm` uses is in range.
fn asm_scratch_slot(ctx: &NativeCtx, word: usize) -> i32 {
    ctx.asm_scratch_base + 8 * (word as i32 + 1)
}

/// `mov [rbp - slot], <reg64>` — store a 64-bit register into a frame slot.
/// `REX.W` plus `REX.R` when `reg >= 8`; ModRM `mod=10, reg=<reg>, rm=101 (rbp)`
/// with a 32-bit displacement of `-slot`. For `reg == 0` (rax) this is byte-identical
/// to the backend's `store_local`.
fn mov_slot_from_reg(code: &mut Vec<u8>, reg: u8, slot: i32) {
    let rex = 0x48 | if reg >= 8 { 0x04 } else { 0 };
    code.push(rex);
    code.push(0x89);
    code.push(0x85 | ((reg & 7) << 3));
    code.extend_from_slice(&(-slot).to_le_bytes());
}

/// `mov <reg64>, [rbp - slot]` — load a frame slot into a 64-bit register.
/// `REX.W` plus `REX.R` when `reg >= 8`; opcode `8B`; ModRM `mod=10, reg=<reg>,
/// rm=101 (rbp)`. For `reg == 0` (rax) this is byte-identical to `load_local`.
fn mov_reg_from_slot(code: &mut Vec<u8>, reg: u8, slot: i32) {
    let rex = 0x48 | if reg >= 8 { 0x04 } else { 0 };
    code.push(rex);
    code.push(0x8B);
    code.push(0x85 | ((reg & 7) << 3));
    code.extend_from_slice(&(-slot).to_le_bytes());
}

/// The register an operand binds.
fn operand_reg(operand: &BcAsmOperand) -> &IrAsmReg {
    match operand {
        BcAsmOperand::In { reg, .. } | BcAsmOperand::Out { reg, .. } => reg,
    }
}

/// Lower an inline-`asm` statement with operand marshalling. Emits the six-step
/// sequence documented at the module level. The raw-byte fast path (no operands,
/// no clobbers) is byte-identical to the legacy emission (`code.extend_from_slice`).
pub(crate) fn lower_native_asm(
    ctx: &mut NativeCtx,
    bytes: &[u8],
    operands: &[BcAsmOperand],
    clobbers: &[IrAsmClobber],
    code: &mut Vec<u8>,
) -> Result<(), String> {
    // The distinct callee-saved registers this asm touches, via any in/out/clobber.
    let mut callee_saved: Vec<u8> = Vec::new();
    let note_cs = |reg: &IrAsmReg, out: &mut Vec<u8>| {
        if reg.callee_saved && !out.contains(&reg.code) {
            out.push(reg.code);
        }
    };
    for operand in operands {
        note_cs(operand_reg(operand), &mut callee_saved);
    }
    for clobber in clobbers {
        if let IrAsmClobber::Reg(reg) = clobber {
            note_cs(reg, &mut callee_saved);
        }
    }

    // 1. Save each touched callee-saved register into its scratch slot.
    for (j, &reg_code) in callee_saved.iter().enumerate() {
        let slot = asm_scratch_slot(ctx, j);
        mov_slot_from_reg(code, reg_code, slot);
    }

    // 2. Stage every input to a scratch slot (all evaluated before any is loaded).
    let input_base = callee_saved.len();
    let inputs: Vec<(&IrAsmReg, &BytecodeExpr)> = operands
        .iter()
        .filter_map(|operand| match operand {
            BcAsmOperand::In { reg, value } => Some((reg, value)),
            BcAsmOperand::Out { .. } => None,
        })
        .collect();
    for (i, (_reg, value)) in inputs.iter().enumerate() {
        lower_native_expr(ctx, value, code)?; // value -> rax
        let slot = asm_scratch_slot(ctx, input_base + i);
        mov_slot_from_reg(code, 0, slot); // mov [rbp - slot], rax
    }

    // 3. Load each staged input into its target register.
    for (i, (reg, _value)) in inputs.iter().enumerate() {
        let slot = asm_scratch_slot(ctx, input_base + i);
        mov_reg_from_slot(code, reg.code, slot);
    }

    // 4. Emit the machine-code bytes verbatim.
    code.extend_from_slice(bytes);

    // 5. Write outputs into their local slots (before any clobber restore).
    for operand in operands {
        if let BcAsmOperand::Out { reg, place } = operand {
            let name = match &place.kind {
                BytecodeExprKind::Variable(name) => name,
                // Semantics guarantees a bare-local `out` target; refuse rather
                // than miscompile if some earlier pass ever produced otherwise.
                _ => return Err("asm `out` target must be a local variable".to_string()),
            };
            let slot = ctx.local_slot(name)?;
            // Soundness: an asm function is never register-promoted, so the output
            // local is frame-resident. If it were promoted, writing the frame slot
            // would silently drop the value the register holds — refuse instead.
            if ctx.promoted_reg(slot).is_some() {
                return Err(format!(
                    "asm `out` target `{name}` is register-promoted; the asm \
                     register-promotion exclusion should have prevented this"
                ));
            }
            mov_slot_from_reg(code, reg.code, slot);
        }
    }

    // 6. Restore each touched callee-saved register.
    for (j, &reg_code) in callee_saved.iter().enumerate() {
        let slot = asm_scratch_slot(ctx, j);
        mov_reg_from_slot(code, reg_code, slot);
    }

    Ok(())
}

/// The number of frame scratch words the `asm` statements in a function body
/// need: the maximum over every `asm` of (callee-saved registers it touches +
/// input operands). A function with no operand `asm` returns 0, so its frame is
/// byte-identical to before this feature.
pub(crate) fn max_asm_scratch_words(body: &[BytecodeInstruction]) -> i32 {
    let mut max = 0i32;
    for instruction in body {
        let here = match instruction {
            BytecodeInstruction::Asm {
                operands, clobbers, ..
            } => asm_scratch_words(operands, clobbers),
            BytecodeInstruction::If {
                branches,
                else_body,
                ..
            } => {
                let mut h = max_asm_scratch_words(else_body);
                for branch in branches {
                    h = h.max(max_asm_scratch_words(&branch.body));
                }
                h
            }
            BytecodeInstruction::While { body, .. }
            | BytecodeInstruction::For { body, .. }
            | BytecodeInstruction::Loop { body, .. }
            | BytecodeInstruction::RegionBlock { body, .. } => max_asm_scratch_words(body),
            BytecodeInstruction::Match { arms, .. } => arms
                .iter()
                .map(|arm| max_asm_scratch_words(&arm.body))
                .max()
                .unwrap_or(0),
            BytecodeInstruction::Try {
                body, catch_body, ..
            } => max_asm_scratch_words(body).max(max_asm_scratch_words(catch_body)),
            _ => 0,
        };
        max = max.max(here);
    }
    max
}

/// The scratch words one `asm` statement needs: the callee-saved registers it
/// touches (each needs a save slot) plus its input operands (each needs a
/// staging slot).
fn asm_scratch_words(operands: &[BcAsmOperand], clobbers: &[IrAsmClobber]) -> i32 {
    let mut callee_saved: Vec<u8> = Vec::new();
    let mut inputs = 0i32;
    let note = |reg: &IrAsmReg, out: &mut Vec<u8>| {
        if reg.callee_saved && !out.contains(&reg.code) {
            out.push(reg.code);
        }
    };
    for operand in operands {
        note(operand_reg(operand), &mut callee_saved);
        if matches!(operand, BcAsmOperand::In { .. }) {
            inputs += 1;
        }
    }
    for clobber in clobbers {
        if let IrAsmClobber::Reg(reg) = clobber {
            note(reg, &mut callee_saved);
        }
    }
    callee_saved.len() as i32 + inputs
}
