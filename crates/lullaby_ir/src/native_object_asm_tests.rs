//! Codegen tests for inline-`asm` operand marshalling and the two soundness
//! invariants it depends on.
//!
//! * **Callee-saved preservation** — an `asm` that touches a callee-saved
//!   register (via `in`/`out`/`clobber`) must save it before and restore it after
//!   the body, or the function returns that register corrupted to its caller. The
//!   byte pins below assert the save (`mov [rbp-slot], rbx`) and restore
//!   (`mov rbx, [rbp-slot]`) straddle the emitted body, and that an `asm` touching
//!   only caller-saved registers emits neither.
//! * **Register-promotion exclusion** — a function containing `asm` is never
//!   register-promoted, so every local stays frame-resident and `out <reg> =
//!   local` writes an authoritative slot. Pinned here at both the instruction
//!   guard (`instr_reg_promotable`) and the function planner
//!   (`plan_register_promotion`).
//!
//! The run-it-and-check-the-exit-code proofs (a real `.exe`: pure-register
//! round-trip, and a clobber-of-a-live-callee-saved-value test with its
//! inject-the-bug teeth) live in `crates/lullaby_cli/tests/cli/suite27.rs`.

use super::*;
use crate::{IrParam, lower, lower_to_bytecode};
use lullaby_lexer::{Span, lex};
use lullaby_parser::parse;
use lullaby_semantics::validate_executable;

/// Compile source through the full frontend into a `BytecodeModule`.
fn module_for(source: &str) -> BytecodeModule {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    let checked = validate_executable(&program).expect("semantic");
    let ir = lower(&checked).expect("lower");
    lower_to_bytecode(&ir)
}

/// Whether `haystack` contains `needle` as a contiguous byte window.
fn contains_seq(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// `mov [rbp - disp32], rbx` and `mov rbx, [rbp - disp32]` (the callee-saved save
// and restore the marshaller emits around an `asm` that touches `rbx`). The
// displacement varies with the frame, so the pins match the 3-byte opcode+ModRM
// prefix that is unique to an rbx spill/reload against `[rbp + disp32]`.
const SAVE_RBX: [u8; 3] = [0x48, 0x89, 0x9D];
const RESTORE_RBX: [u8; 3] = [0x48, 0x8B, 0x9D];

/// A `clobber rbx` (callee-saved) forces a save before and a restore after the
/// asm body. Every function in the program contains an `asm`, so none is
/// register-promoted — the ONLY source of an rbx spill/reload is the clobber
/// marshalling, which makes this a precise pin.
#[test]
fn callee_saved_clobber_emits_save_and_restore() {
    let program = emit_native_program(&module_for(concat!(
        "fn f -> i64\n",
        "    unsafe\n",
        "        asm 72, 199, 192, 5, 0, 0, 0\n", // mov rax, 5
        "            clobber rbx\n",
        "\n",
        "fn main -> i64\n",
        "    f()\n",
    )))
    .expect("emit clobber-rbx program");
    assert!(
        program.compiled.contains(&"f".to_string()),
        "an operand `asm` must compile: {:?}",
        program.skipped
    );
    assert!(
        contains_seq(&program.bytes, &SAVE_RBX),
        "a `clobber rbx` must save the callee-saved register before the asm body \
         (mov [rbp-slot], rbx = 48 89 9D ..)"
    );
    assert!(
        contains_seq(&program.bytes, &RESTORE_RBX),
        "a `clobber rbx` must restore the callee-saved register after the asm body \
         (mov rbx, [rbp-slot] = 48 8B 9D ..)"
    );
    // The verbatim body bytes must still be present, unmodified, between them.
    assert!(
        contains_seq(&program.bytes, &[0x48, 0xC7, 0xC0, 5, 0, 0, 0]),
        "the asm body bytes (mov rax, 5) must be emitted verbatim"
    );
}

/// An `asm` that clobbers ONLY caller-saved registers (`rcx`, `r11` — the Linux
/// syscall clobbers) needs no callee-saved preservation, so no rbx spill/reload
/// appears. This is the negative control for the pin above.
#[test]
fn caller_saved_clobber_emits_no_preservation() {
    let program = emit_native_program(&module_for(concat!(
        "fn f -> i64\n",
        "    unsafe\n",
        "        asm 72, 199, 192, 5, 0, 0, 0\n", // mov rax, 5
        "            clobber rcx\n",
        "            clobber r11\n",
        "\n",
        "fn main -> i64\n",
        "    f()\n",
    )))
    .expect("emit caller-saved-clobber program");
    assert!(program.compiled.contains(&"f".to_string()));
    assert!(
        !contains_seq(&program.bytes, &SAVE_RBX) && !contains_seq(&program.bytes, &RESTORE_RBX),
        "clobbering only caller-saved registers must emit NO callee-saved \
         save/restore (rcx/r11 are volatile on both ABIs)"
    );
}

/// An `in`/`out` binding to a callee-saved register (`rbx`) — not just a clobber —
/// must ALSO be preserved: the caller relies on `rbx` surviving the whole
/// function, so a body that loads an input into it or reads an output from it
/// still corrupts the caller unless it is saved/restored.
#[test]
fn callee_saved_operand_binding_is_preserved() {
    let program = emit_native_program(&module_for(concat!(
        "fn f x i64 -> i64\n",
        "    let r i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 216\n", // mov rax, rbx
        "            in rbx = x\n",
        "            out rax = r\n",
        "    r\n",
        "\n",
        "fn main -> i64\n",
        "    f(9)\n",
    )))
    .expect("emit callee-saved-binding program");
    assert!(
        program.compiled.contains(&"f".to_string()),
        "{:?}",
        program.skipped
    );
    assert!(
        contains_seq(&program.bytes, &SAVE_RBX) && contains_seq(&program.bytes, &RESTORE_RBX),
        "an `in rbx = ..` binding must save/restore rbx (it is callee-saved) even \
         though it is an input, not a clobber"
    );
}

/// A raw-byte `asm` with no operands and no clobbers is byte-identical to the
/// legacy emission: the marshaller's fast path just copies the bytes, so no rbx
/// spill/reload (or any other preservation) appears.
#[test]
fn raw_byte_asm_has_no_marshalling() {
    let program = emit_native_program(&module_for(concat!(
        "fn f -> i64\n",
        "    unsafe\n",
        "        asm 72, 199, 192, 5, 0, 0, 0\n",
        "\n",
        "fn main -> i64\n",
        "    f()\n",
    )))
    .expect("emit raw-byte program");
    assert!(program.compiled.contains(&"f".to_string()));
    assert!(
        !contains_seq(&program.bytes, &SAVE_RBX) && !contains_seq(&program.bytes, &RESTORE_RBX),
        "a raw-byte `asm` (no operands/clobbers) must emit no operand marshalling"
    );
    assert!(
        contains_seq(&program.bytes, &[0x48, 0xC7, 0xC0, 5, 0, 0, 0]),
        "the raw asm bytes must be emitted verbatim"
    );
}

// -- The register-promotion exclusion pin -------------------------------------

/// An `Asm` instruction is NEVER register-promotable. This is the guard that
/// keeps every local of an asm-containing function frame-resident so `out <reg> =
/// local` and `in <reg> = local` address authoritative slots. Injecting an `Asm`
/// arm into `instr_reg_promotable` that returns `true` fails this test — and would
/// let a syscall wrapper promote a local into `rbx`, so `clobber rbx` (or an
/// `out rax = result` whose result the planner moved to `rbx`) would corrupt it.
#[test]
fn asm_instruction_is_never_reg_promotable() {
    let asm = BytecodeInstruction::Asm {
        bytes: vec![0x90],
        operands: vec![crate::BcAsmOperand::Out {
            reg: crate::IrAsmReg {
                code: 0,
                name: "rax".to_string(),
                callee_saved: false,
            },
            place: BytecodeExpr {
                kind: BytecodeExprKind::Variable("r".to_string()),
                ty: TypeRef::new("i64"),
                span: Span::new(1, 1),
            },
        }],
        clobbers: vec![crate::IrAsmClobber::Reg(crate::IrAsmReg {
            code: 3,
            name: "rbx".to_string(),
            callee_saved: true,
        })],
        span: Span::new(1, 1),
    };
    assert!(
        !instr_reg_promotable(&asm),
        "an `asm` instruction must never be register-promotable — the operand \
         marshalling depends on every local staying frame-resident"
    );
}

/// A purely-scalar `i64` function whose body contains an `asm` is NOT
/// register-promoted: the planner returns no promotions and no saved registers,
/// so no local is moved into `rbx`/`rsi` across the asm.
#[test]
fn function_with_asm_is_not_register_promoted() {
    let func = BytecodeFunction {
        name: "f".to_string(),
        params: vec![IrParam {
            name: "x".to_string(),
            ty: TypeRef::new("i64"),
        }],
        return_type: TypeRef::new("i64"),
        instructions: vec![BytecodeInstruction::Asm {
            bytes: vec![0x90],
            operands: vec![],
            clobbers: vec![crate::IrAsmClobber::Reg(crate::IrAsmReg {
                code: 3,
                name: "rbx".to_string(),
                callee_saved: true,
            })],
            span: Span::new(1, 1),
        }],
        span: Span::new(1, 1),
    };
    let (promoted, saved) = plan_register_promotion(&func, &std::collections::HashMap::new());
    assert!(
        promoted.is_empty() && saved.is_empty(),
        "a function containing `asm` must not be register-promoted (promoted={:?}, \
         saved.len()={})",
        promoted.len(),
        saved.len()
    );
}
