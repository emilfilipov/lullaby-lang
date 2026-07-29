//! Native backend: the two **hardware-entry calling conventions** —
//! `interrupt fn` and `naked fn` (`documents/freestanding_tier_design.md` §6).
//!
//! An ordinary Lullaby function is entered by a `call` and leaves by `ret`. These
//! two kinds are entered by the *hardware* (an IDT vector) or by a bootloader, so
//! neither wraps its body in the ordinary prologue/epilogue.
//!
//! # `naked fn`
//!
//! Nothing is emitted around the body: no `push rbp`, no `sub rsp`, no `ret`, not
//! even the fallthrough `xor eax, eax`. The `asm` bytes the author wrote are the
//! entire function. Semantics guarantees the shape this relies on (`L0446`): the
//! body is `asm` only, binds no operands or clobbers (both need frame slots), and
//! the function takes no parameters and returns `void`. Lowering therefore needs
//! no `NativeCtx` at all — see [`lower_naked_function`].
//!
//! # `interrupt fn`
//!
//! The body is ordinary Lullaby code with a real frame, so it reuses the whole
//! statement/expression lowering. Only the wrapper differs:
//!
//! ```text
//!   push rax … push r15      ; 14 GPRs, everything but rsp/rbp
//!   sub  rsp, 256            ; XMM save area
//!   movups [rsp + 16*i], xmm(i)   ; i = 0..15
//!   push rbp                 ; the 15th saved GPR, and the frame link
//!   mov  rbp, rsp
//!   sub  rsp, frame_size
//!   and  rsp, -16            ; realign for calls out of the body
//!   cld                      ; DF = 0, assumed by the `rep movsb` helpers
//!   lea  <frame param>, [rbp + 376 (+8 on an error-code vector)]
//!   mov  <error_code param>, [rbp + 376]      ; error-code vectors only
//!   … body …
//!   mov  rsp, rbp
//!   pop  rbp
//!   movups xmm(i), [rsp + 16*i]
//!   add  rsp, 256
//!   pop  r15 … pop rax
//!   add  rsp, 8              ; discard the error code, error-code vectors only
//!   iretq
//! ```
//!
//! ## Why each piece is there
//!
//! * **All 14 GPRs plus `rbp`.** An interrupt can arrive between any two
//!   instructions of the interrupted context, so *every* register it might hold a
//!   live value in must come back unchanged. `rsp` is excluded because `iretq`
//!   restores it from the CPU-pushed frame.
//! * **All 16 XMM registers.** Not defensive padding: this emitter puts `xmm0`
//!   through `xmm15` to work for `f64`/`f32` values *and* for the SSE integer
//!   reduction/vectorization paths (`native_object_simd.rs`,
//!   `native_object_reduce.rs`), so even an all-integer ISR body can clobber
//!   them. Saving only the ones a given body touches is a post-1.0 refinement;
//!   saving all of them is correct regardless of what the body — or anything it
//!   calls — does. `movups` (unaligned) rather than `movaps` because the save area
//!   inherits the interrupted context's `rsp` alignment.
//! * **`and rsp, -16`.** The Win64/SysV ABI requires `rsp ≡ 0 (mod 16)` at a
//!   `call`. The ordinary prologue gets that for free (entry `rsp ≡ 8`, `push rbp`
//!   makes it 0, and `frame_size` is a multiple of 16), but an ISR's entry
//!   alignment depends on whether the vector pushed an error code — 5 CPU-pushed
//!   words for most vectors, 6 for the error-code ones — so it differs by variant.
//!   Forcing the alignment is variant-independent and provably safe: `and` only
//!   ever *lowers* `rsp`, and every local is addressed through `rbp`, so nothing
//!   in the frame moves — the whole `[rsp, rbp)` span stays ours, including the
//!   shadow space and outgoing-argument area, which the call-argument emitter
//!   addresses `rsp`-relative. It is why the epilogue restores with
//!   `mov rsp, rbp` rather than `add rsp, frame_size`.
//!
//!   Note the converse is *not* required: `rbp` itself need not be 16-aligned in
//!   an ISR, because no rbp-relative access here has an alignment requirement —
//!   the emitter's only SSE memory moves are `movdqu`/`movups`/`movsd`, all
//!   unaligned (`movapd` appears once, register-to-register).
//! * **`cld`.** The shared `.text` helpers copy aggregates with `rep movsb`, which
//!   reads the direction flag. Both ABIs require DF = 0 at a call boundary, but an
//!   interrupt is not a call boundary — it can land inside a `std`/`cld` window.
//!   `iretq` restores the interrupted `RFLAGS`, so clearing DF here cannot leak.
//! * **`iretq` (`48 CF`), not `iret` (`CF`).** In long mode the operand size must
//!   be forced to 64 bits with `REX.W`; the un-prefixed form pops 32-bit values
//!   and corrupts the resumed context.
//!
//! ## The two invariants this convention needs, made explicit
//!
//! **Neither is reachable from Lullaby source today** — say so plainly, because an
//! earlier revision of this header claimed both "were reachable by accident" and
//! that was simply false. What is true is narrower and still worth the guards:
//!
//! 1. **No register promotion.** `plan` is told the function kind and refuses to
//!    promote any local for an ISR ([`NativeFnKind::promotes_registers`]).
//!    Promotion spills `rbx`/`rsi` in the *ordinary* prologue and restores them in
//!    the *ordinary* epilogue — neither of which an ISR runs — so a promoted ISR
//!    would return `rbx` holding a Lullaby local to the interrupted context. It
//!    could not happen without the guard either: `plan_register_promotion`
//!    (`native_object_regalloc.rs`) returns nothing unless the return type is
//!    `i64` and every parameter is `i64`, while `L0446` requires an
//!    `interrupt fn` to return `void` and take a `ptr<T>`. The guard's value is
//!    that it makes this invariant independent of *that* rule, which is about
//!    scalar codegen and could widen without anyone thinking about interrupts.
//! 2. **No arena prologue.** The arena-first path saves and restores the global
//!    heap bump pointer `__lullaby_heap_next` around the body. An ISR can
//!    interrupt a *different* function that is mid-arena, so writing that global
//!    would corrupt the interrupted function's region. Again unreachable today —
//!    the arena set only admits heap-using functions, and a `no-runtime` module
//!    (the only tier where these keywords are legal) allocates nothing — so the
//!    guard exists so the invariant does not rest on the tier gate. The program
//!    driver drops both kinds from the arena set and [`lower_native_function`]
//!    re-checks rather than trusting it.
//!
//! Both guards are therefore **defence-in-depth over a currently-closed path**,
//! not live protection. `native_object_isr_tests.rs` pins each by reaching past
//! the frontend to build the shape the frontend cannot produce and lowering it
//! both ways, with the ordinary arm as a control — which is what keeps those
//! tests from being vacuous. The byte pins for the convention itself live there
//! too.

use super::*;

/// Which calling convention a function is lowered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum NativeFnKind {
    /// The ordinary Lullaby/Win64 convention: `push rbp` … `ret`.
    #[default]
    Ordinary,
    /// A hardware interrupt-service routine. `has_error_code` is true for the
    /// vectors on which the CPU pushes an error code word below the interrupt
    /// frame (spelled as the declaration's second `error_code u64` parameter).
    Interrupt { has_error_code: bool },
    /// A naked function: the body's `asm` and nothing else.
    Naked,
}

impl NativeFnKind {
    /// Whether a function of this kind may keep locals in callee-saved registers.
    /// Only the ordinary convention may: see invariant 1 in the module docs.
    pub(crate) fn promotes_registers(self) -> bool {
        matches!(self, NativeFnKind::Ordinary)
    }

    /// Whether a function of this kind may use the arena-first heap path. Only
    /// the ordinary convention may: see invariant 2 in the module docs.
    pub(crate) fn allows_arena(self) -> bool {
        matches!(self, NativeFnKind::Ordinary)
    }

    /// Resolve a function's kind from the module's interrupt/naked tables.
    pub(crate) fn for_function(module: &BytecodeModule, name: &str) -> Self {
        if let Some(isr) = module
            .interrupt_functions
            .iter()
            .find(|isr| isr.name == name)
        {
            return NativeFnKind::Interrupt {
                has_error_code: isr.has_error_code,
            };
        }
        if module.naked_functions.iter().any(|n| n == name) {
            return NativeFnKind::Naked;
        }
        NativeFnKind::Ordinary
    }
}

/// The general-purpose registers the ISR prologue pushes, in push order, by
/// encoding. `rsp` (4) is excluded — `iretq` restores it from the CPU-pushed
/// frame — and `rbp` (5) is excluded here because it is pushed last, by the frame
/// link, so that `[rbp]` still holds the caller's `rbp` exactly as in an ordinary
/// frame.
const ISR_SAVED_GPRS: [u8; 14] = [0, 1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

/// Bytes of the XMM save area: 16 registers × 16 bytes.
const XMM_SAVE_BYTES: i32 = 256;

/// Distance from `rbp` to the lowest CPU-pushed word on an ISR frame.
///
/// After the prologue's pushes and `mov rbp, rsp`, the stack above `rbp` holds,
/// in ascending address order: the saved `rbp` (at `[rbp]`), the 256-byte XMM save
/// area, then the 14 saved GPRs — so the word the CPU pushed last sits at
/// `rbp + 8 + 256 + 112`. On a no-error-code vector that word is `RIP` (the start
/// of the interrupt frame); on an error-code vector it is the error code, and the
/// frame proper starts 8 bytes higher.
const ISR_CPU_STACK_DISP: i32 = 8 + XMM_SAVE_BYTES + 8 * ISR_SAVED_GPRS.len() as i32;

/// Emit `push <reg64>`: `50+r`, with a `41` REX.B prefix for `r8`–`r15`.
fn push_reg(code: &mut Vec<u8>, reg: u8) {
    if reg >= 8 {
        code.push(0x41);
    }
    code.push(0x50 + (reg & 7));
}

/// Emit `pop <reg64>`: `58+r`, with a `41` REX.B prefix for `r8`–`r15`.
fn pop_reg(code: &mut Vec<u8>, reg: u8) {
    if reg >= 8 {
        code.push(0x41);
    }
    code.push(0x58 + (reg & 7));
}

/// Emit `movups [rsp + disp32], xmm<n>` (`store`) or `movups xmm<n>, [rsp +
/// disp32]` (`!store`).
///
/// Opcode `0F 11` stores, `0F 10` loads; `REX.R` (`44`) selects `xmm8`–`xmm15`.
/// ModRM uses `mod=10` (disp32) with `rm=100` to select a SIB byte, and SIB `24`
/// encodes "base = rsp, no index". A disp32 is used for every slot rather than
/// switching to disp8 for the low offsets, so all 16 encodings are one uniform
/// shape.
fn movups_xmm_rsp(code: &mut Vec<u8>, xmm: u8, disp: i32, store: bool) {
    if xmm >= 8 {
        code.push(0x44);
    }
    code.push(0x0F);
    code.push(if store { 0x11 } else { 0x10 });
    code.push(0x84 | ((xmm & 7) << 3));
    code.push(0x24);
    code.extend_from_slice(&disp.to_le_bytes());
}

/// Emit the ISR prologue up to and including `cld`, leaving `rbp` established and
/// `rsp` 16-byte aligned below a `frame_size` local area.
pub(crate) fn emit_isr_prologue(code: &mut Vec<u8>, frame_size: i32) {
    for reg in ISR_SAVED_GPRS {
        push_reg(code, reg);
    }
    // sub rsp, 256 — the XMM save area (imm32 form; 256 does not fit imm8).
    code.extend_from_slice(&[0x48, 0x81, 0xEC]);
    code.extend_from_slice(&XMM_SAVE_BYTES.to_le_bytes());
    for xmm in 0..16u8 {
        movups_xmm_rsp(code, xmm, xmm as i32 * 16, true);
    }
    // push rbp; mov rbp, rsp — the ordinary frame link, so `[rbp]` is the saved
    // rbp and every local displacement matches an ordinary function's.
    code.extend_from_slice(&[0x55, 0x48, 0x89, 0xE5]);
    emit_sub_rsp(code, frame_size);
    // and rsp, -16 — see the module docs. imm8 form, sign-extended.
    code.extend_from_slice(&[0x48, 0x83, 0xE4, 0xF0]);
    // cld — the aggregate helpers' `rep movsb` reads DF, and an interrupt can land
    // inside a window where the interrupted code has set it.
    code.push(0xFC);
}

/// Emit `lea rax, [rbp + disp32]` then store `rax` into the frame slot `slot`.
/// Used to seat the interrupt-frame pointer parameter, which is not passed in a
/// register — it is the address of the CPU-pushed frame itself.
fn seat_frame_pointer(code: &mut Vec<u8>, disp: i32, slot: i32) {
    // lea rax, [rbp + disp32]: REX.W 8D /r, mod=10 reg=rax rm=rbp.
    code.extend_from_slice(&[0x48, 0x8D, 0x85]);
    code.extend_from_slice(&disp.to_le_bytes());
    store_local(code, slot);
}

/// Emit `mov rax, [rbp + disp32]` then store `rax` into the frame slot `slot`.
/// Used to seat the CPU-pushed error code.
fn seat_stack_word(code: &mut Vec<u8>, disp: i32, slot: i32) {
    // mov rax, [rbp + disp32]: REX.W 8B /r, mod=10 reg=rax rm=rbp.
    code.extend_from_slice(&[0x48, 0x8B, 0x85]);
    code.extend_from_slice(&disp.to_le_bytes());
    store_local(code, slot);
}

/// Seat an `interrupt fn`'s parameters. Unlike every other function, an ISR
/// receives nothing in registers: the first parameter is the *address* of the
/// CPU-pushed interrupt frame, and the optional second is the CPU-pushed error
/// code word that sits just below it.
///
/// `ctx` must already have been planned, so each parameter has a frame slot.
/// Semantics has pinned the shape (`L0446`): one or two parameters, the first a
/// raw pointer, the second (if present) a `u64`.
pub(crate) fn seat_isr_params(
    ctx: &NativeCtx,
    function: &BytecodeFunction,
    has_error_code: bool,
    code: &mut Vec<u8>,
) -> Result<(), String> {
    // On an error-code vector the CPU pushed the code *below* the frame, so the
    // frame proper starts one word higher.
    let frame_disp = ISR_CPU_STACK_DISP + if has_error_code { 8 } else { 0 };
    let Some(frame_param) = function.params.first() else {
        return Err(
            "an `interrupt fn` must declare the CPU-pushed interrupt frame parameter".to_string(),
        );
    };
    let frame_local = ctx.local(&frame_param.name)?.clone();
    seat_frame_pointer(code, frame_disp, frame_local.slot);

    if let Some(error_param) = function.params.get(1) {
        if !has_error_code {
            return Err(
                "an `interrupt fn` with a second parameter must be lowered as an \
                 error-code vector"
                    .to_string(),
            );
        }
        let error_local = ctx.local(&error_param.name)?.clone();
        seat_stack_word(code, ISR_CPU_STACK_DISP, error_local.slot);
    }
    Ok(())
}

/// Emit the ISR epilogue: unwind the frame, restore the XMM and general-purpose
/// registers, discard the CPU-pushed error code if there is one, and `iretq`.
/// The exact mirror of [`emit_isr_prologue`].
pub(crate) fn emit_isr_epilogue(code: &mut Vec<u8>, has_error_code: bool) {
    // mov rsp, rbp; pop rbp — restores rsp regardless of the `and rsp, -16`
    // realignment the prologue applied.
    code.extend_from_slice(&[0x48, 0x89, 0xEC, 0x5D]);
    for xmm in 0..16u8 {
        movups_xmm_rsp(code, xmm, xmm as i32 * 16, false);
    }
    // add rsp, 256 — release the XMM save area.
    code.extend_from_slice(&[0x48, 0x81, 0xC4]);
    code.extend_from_slice(&XMM_SAVE_BYTES.to_le_bytes());
    for reg in ISR_SAVED_GPRS.iter().rev() {
        pop_reg(code, *reg);
    }
    if has_error_code {
        // add rsp, 8 — the CPU pushes the error code but `iretq` does not pop it.
        code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x08]);
    }
    // iretq — REX.W + CF. The un-prefixed `CF` is `iretd` and pops 32-bit words.
    code.extend_from_slice(&[0x48, 0xCF]);
}

/// Emit the prologue `fn_kind` calls for, and report whether the caller should
/// then run the ordinary Win64 register-argument seating.
///
/// `false` for an ISR, which receives **nothing** in registers: its parameters are
/// the address of the CPU-pushed interrupt frame and, on an error-code vector, the
/// pushed error code, and both are materialized from the stack here. `true` for
/// the ordinary convention. (A `naked fn` never reaches this — it is emitted whole
/// by [`hardware_entry_preflight`] before a frame is even planned.)
pub(crate) fn emit_prologue_for(
    ctx: &NativeCtx,
    function: &BytecodeFunction,
    fn_kind: NativeFnKind,
    code: &mut Vec<u8>,
) -> Result<bool, String> {
    let NativeFnKind::Interrupt { has_error_code } = fn_kind else {
        // Ordinary prologue: push rbp; mov rbp, rsp; sub rsp, frame_size.
        code.extend_from_slice(&[0x55, 0x48, 0x89, 0xE5]);
        emit_sub_rsp(code, ctx.frame_size);
        // Preserve the callee-saved registers holding promoted locals, spilling each
        // caller value into its reserved frame slot (restored in the epilogue). Done
        // before parameters are seated so a promoted parameter can overwrite its
        // register next.
        for (reg, slot) in &ctx.saved_reg_slots {
            reg.spill_to_slot(code, *slot);
        }
        return Ok(true);
    };

    // The ISR prologue saves the full register set (14 GPRs + rbp + all 16 XMM
    // registers), establishes the frame, and forces 16-byte stack alignment. It
    // replaces BOTH the ordinary prologue and the promoted-register spill above.
    //
    // The `saved_reg_slots` check below CANNOT fire today, and is kept deliberately
    // rather than dressed up as load-bearing: `NativeCtx::plan` is handed `fn_kind`
    // and skips `plan_register_promotion` entirely for this kind, so
    // `saved_reg_slots` is empty two ways over (and no source-level ISR is
    // promotable anyway — promotion needs an `i64` return and `i64` parameters,
    // while `L0446` requires `void` and a `ptr<T>`). It stays because the guarantee
    // lives in a different file from the code that depends on it: if `plan`'s slot
    // bookkeeping ever populated `saved_reg_slots` from another source, an ISR would
    // silently emit a spill whose matching restore lives in an epilogue it never
    // runs, handing the interrupted context a callee-saved register full of a
    // Lullaby local. Refusing costs one `is_empty()` and turns that into a clean
    // demotion.
    if !ctx.saved_reg_slots.is_empty() {
        return Err(format!(
            "`{}` is an interrupt handler but was planned with register promotion; \
             its promoted registers would be restored by an epilogue it never runs",
            function.name
        ));
    }
    emit_isr_prologue(code, ctx.frame_size);
    seat_isr_params(ctx, function, has_error_code, code)?;
    Ok(false)
}

/// Emit the return sequence appropriate to `ctx`'s calling convention. The single
/// place a return edge decides between `ret` and `iretq`, so adding a return path
/// to the lowering cannot accidentally emit the wrong one for an ISR — and every
/// return edge in `native_object_stmt.rs` (the tail, each value-position
/// `if`/`match`, and each explicit `return`) routes through it.
///
/// A `naked fn` never reaches here: it is lowered by [`lower_naked_function`],
/// which emits no epilogue at all.
pub(crate) fn emit_epilogue_for(ctx: &NativeCtx, code: &mut Vec<u8>) {
    match ctx.fn_kind {
        NativeFnKind::Interrupt { has_error_code } => emit_isr_epilogue(code, has_error_code),
        // Defensive: a naked function is lowered without a `NativeCtx`, so this arm
        // is unreachable; emitting the ordinary epilogue is the safe fallback and is
        // never observable.
        NativeFnKind::Ordinary | NativeFnKind::Naked => {
            emit_native_epilogue(code, ctx.frame_size, &ctx.saved_reg_slots)
        }
    }
}

/// The checks `lower_native_function` runs before planning a frame, for the two
/// kinds whose conventions the planner does not model.
///
/// Returns `Some(lowered)` when the function is fully handled here (a `naked fn`,
/// which needs no frame at all), `None` when lowering should proceed normally, and
/// `Err` when the function must be demoted to the interpreters.
pub(crate) fn hardware_entry_preflight(
    function: &BytecodeFunction,
    fn_kind: NativeFnKind,
    is_arena: bool,
) -> Result<Option<LoweredNativeFunction>, String> {
    // A `naked fn` has no frame at all, so it never reaches the planner: its `asm`
    // bodies are the entire emission.
    if matches!(fn_kind, NativeFnKind::Naked) {
        return Ok(Some(lower_naked_function(function)?));
    }
    // Invariant 2 of the hardware-entry conventions, checked rather than assumed:
    // an ISR must not carry the arena-first prologue, which saves and restores the
    // global heap bump pointer. An interrupt can land inside a *different*
    // function's arena region, so writing that global here would corrupt it. The
    // eligibility pass already excludes both kinds from the arena set; refusing
    // (rather than trusting) means a future change there demotes the function to
    // the interpreters instead of silently miscompiling it.
    if is_arena && !fn_kind.allows_arena() {
        return Err(format!(
            "`{}` is a hardware entry point and cannot use the arena-first heap path \
             (an interrupt can arrive inside another function's arena region)",
            function.name
        ));
    }
    Ok(None)
}

/// Lower a `naked fn`: emit its `asm` bodies verbatim and nothing else.
///
/// No prologue, no epilogue, no implicit `ret`, no fallthrough zeroing — that
/// absence *is* the feature. The traversal descends through `unsafe` blocks
/// (which `asm` requires) and accepts nothing else; semantics has already refused
/// any other statement, any operand/clobber binding, and any parameter or non-void
/// return with `L0446`, so a violation reaching here is an internal inconsistency
/// and is refused (the function skips to the interpreters) rather than emitted as
/// a body that would fall through into whatever follows it in `.text`.
pub(crate) fn lower_naked_function(
    function: &BytecodeFunction,
) -> Result<LoweredNativeFunction, String> {
    let mut code = Vec::new();
    collect_naked_asm(&function.name, &function.instructions, &mut code)?;
    if code.is_empty() {
        return Err(format!(
            "`naked fn {}` lowered to no instructions; a naked function emits no \
             prologue, epilogue, or `ret`, so an empty body would fall through into \
             the next function in `.text`",
            function.name
        ));
    }
    Ok(LoweredNativeFunction {
        name: function.name.clone(),
        code,
        relocations: Vec::new(),
        line: function.span.line as u32,
    })
}

fn collect_naked_asm(
    name: &str,
    instructions: &[BytecodeInstruction],
    code: &mut Vec<u8>,
) -> Result<(), String> {
    for instruction in instructions {
        match instruction {
            BytecodeInstruction::Asm {
                bytes,
                operands,
                clobbers,
                ..
            } => {
                if !operands.is_empty() || !clobbers.is_empty() {
                    return Err(format!(
                        "`naked fn {name}` contains an `asm` with operands or clobbers; \
                         both are marshalled through rbp-relative frame slots and a naked \
                         function has no frame"
                    ));
                }
                code.extend_from_slice(bytes);
            }
            // An `unsafe` block does not survive IR lowering as its own node
            // (statements are flattened into the enclosing block), but a `region`
            // block does; neither can appear in a naked body after `L0446`.
            other => {
                return Err(format!(
                    "`naked fn {name}` contains a non-`asm` statement ({}); a naked \
                     function emits no prologue, so a Lullaby statement would address \
                     locals through a frame that does not exist",
                    naked_instruction_label(other)
                ));
            }
        }
    }
    Ok(())
}

/// A short label for the unexpected instruction kind found in a naked body.
fn naked_instruction_label(instruction: &BytecodeInstruction) -> &'static str {
    match instruction {
        BytecodeInstruction::Let { .. } => "a `let` binding",
        BytecodeInstruction::Assign { .. } => "an assignment",
        BytecodeInstruction::Return(_) => "a `return`",
        BytecodeInstruction::Break(_) => "a `break`",
        BytecodeInstruction::Continue(_) => "a `continue`",
        BytecodeInstruction::Expr(_) => "an expression statement",
        BytecodeInstruction::If { .. } => "an `if`",
        BytecodeInstruction::While { .. } => "a `while` loop",
        BytecodeInstruction::For { .. } => "a `for` loop",
        BytecodeInstruction::Loop { .. } => "a `loop`",
        BytecodeInstruction::RegionBlock { .. } => "a `region` block",
        BytecodeInstruction::Match { .. } => "a `match`",
        BytecodeInstruction::Throw { .. } => "a `throw`",
        BytecodeInstruction::Try { .. } => "a `try`",
        BytecodeInstruction::Asm { .. } => "an `asm` statement",
    }
}
