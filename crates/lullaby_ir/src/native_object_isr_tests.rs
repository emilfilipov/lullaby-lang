//! Codegen tests for the two hardware-entry calling conventions, `interrupt fn`
//! and `naked fn` (`documents/freestanding_tier_design.md` §6).
//!
//! A real IDT round-trip cannot run in a hosted process — the CPU, not a caller,
//! enters an ISR, and installing an IDT entry needs ring 0 — so the convention is
//! pinned **byte-exactly** here instead: the save-all prologue, the stack
//! realignment, the frame/error-code materialization, the mirror-image restore,
//! and `iretq` (`48 CF`, not the 32-bit `CF`). A `naked fn` is pinned by what is
//! *absent*: its `.text` contains its `asm` bytes and nothing else.
//!
//! ## The two ISR invariants, and what these tests actually prove about them
//!
//! Be precise about the scope, because an earlier revision of this header was not
//! and the tests underneath it were vacuous as a result. **Neither invariant is
//! reachable from Lullaby source today**:
//!
//! * `plan_register_promotion` returns nothing unless the return type is `i64`
//!   and every parameter is `i64` (`native_object_regalloc.rs`), while `L0446`
//!   requires an `interrupt fn` to return `void` and take a `ptr<T>` — so no
//!   source-level ISR can be promoted regardless of the guard;
//! * the arena-first path only ever admits heap-using functions, and a
//!   `no-runtime` module — the only tier in which these keywords are legal —
//!   allocates nothing.
//!
//! The guards are kept anyway, as defence-in-depth: each makes the invariant
//! independent of an *unrelated* rule that could widen without anyone thinking
//! about interrupt handlers (promotion admitting `void` returns or pointer
//! parameters; the tier gate being relaxed). What the two tests below pin is
//! therefore the **guard's own behaviour against the shape the frontend cannot
//! currently produce** — each reaches past the frontend, builds that shape, and
//! lowers it BOTH ways, with the ordinary arm as a control proving the shape is
//! genuinely affected. Without that control they would be assertions about the
//! body rather than about the guard.
//!
//! INJECT-THE-BUG TEETH (all measured on this branch by reverting the guard,
//! running this file, and restoring): dropping the `and rsp, -16` realignment, the
//! `cld`, the XMM save area, the error-code `add rsp, 8`, or the error-code frame
//! shift each fails exactly one pin below; emitting `CF` instead of `48 CF` fails
//! three; making `promotes_registers`/`allows_arena` unconditionally true each
//! fails one; routing an ISR's return edges through the ordinary epilogue fails
//! five; and removing the `Naked` early return from `lower_native_function` fails
//! [`naked_function_lowers_to_exactly_its_asm_bytes`].
//!
//! That last one is a cautionary case worth keeping in view. This test originally
//! searched for `[0x55, 0x48, 0x89, 0xE5, <body>]` — "the prologue immediately
//! before the naked body" — and stayed **green** with the early return removed,
//! because a real prologue is `push rbp; mov rbp, rsp` followed by
//! `sub rsp, <frame_size>` and the planner always reserves at least one scratch
//! word, so the searched-for window never occurs either way. It now asserts exact
//! equality against the lowered bytes, which cannot pass vacuously.

use super::*;
use crate::{lower, lower_to_bytecode};
use lullaby_lexer::lex;
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
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// A minimal `no-runtime` program with one no-error-code ISR plus a `main` (so the
/// program has an entry point and something to emit).
const TIMER_ISR: &str = concat!(
    "no-runtime\n",
    "\n",
    "interrupt fn timer_isr frame ptr<i64>\n",
    "    unsafe\n",
    "        ptr_write(frame, 1)\n",
    "\n",
    "fn main -> i64\n",
    "    0\n",
);

/// The error-code variant: the second `error_code u64` parameter selects the
/// vector convention on which the CPU pushes an extra word.
const PAGE_FAULT_ISR: &str = concat!(
    "no-runtime\n",
    "\n",
    "interrupt fn page_fault frame ptr<i64> error_code u64\n",
    "    unsafe\n",
    "        ptr_write(frame, to_i64(error_code))\n",
    "\n",
    "fn main -> i64\n",
    "    0\n",
);

// -- The pinned encodings ----------------------------------------------------

/// `push rax, rcx, rdx, rbx, rsi, rdi, r8 … r15` — the 14 GPRs the ISR prologue
/// saves, in push order. `rsp` is excluded (`iretq` restores it) and `rbp` is
/// pushed separately, by the frame link.
const PUSH_ALL_GPRS: [u8; 22] = [
    0x50, 0x51, 0x52, 0x53, 0x56, 0x57, // rax rcx rdx rbx rsi rdi
    0x41, 0x50, 0x41, 0x51, 0x41, 0x52, 0x41, 0x53, // r8 r9 r10 r11
    0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57, // r12 r13 r14 r15
];

/// The mirror image: `pop r15 … pop rax`.
const POP_ALL_GPRS: [u8; 22] = [
    0x41, 0x5F, 0x41, 0x5E, 0x41, 0x5D, 0x41, 0x5C, // r15 r14 r13 r12
    0x41, 0x5B, 0x41, 0x5A, 0x41, 0x59, 0x41, 0x58, // r11 r10 r9 r8
    0x5F, 0x5E, 0x5B, 0x5A, 0x59, 0x58, // rdi rsi rbx rdx rcx rax
];

/// `sub rsp, 256` — reserve the 16×16-byte XMM save area.
const SUB_RSP_256: [u8; 7] = [0x48, 0x81, 0xEC, 0x00, 0x01, 0x00, 0x00];
/// `add rsp, 256` — release it.
const ADD_RSP_256: [u8; 7] = [0x48, 0x81, 0xC4, 0x00, 0x01, 0x00, 0x00];
/// `movups [rsp + 0], xmm0`.
const SAVE_XMM0: [u8; 8] = [0x0F, 0x11, 0x84, 0x24, 0, 0, 0, 0];
/// `movups [rsp + 240], xmm15` — the highest save slot, with the REX.R prefix.
const SAVE_XMM15: [u8; 9] = [0x44, 0x0F, 0x11, 0xBC, 0x24, 0xF0, 0x00, 0x00, 0x00];
/// `movups xmm0, [rsp + 0]`.
const RESTORE_XMM0: [u8; 8] = [0x0F, 0x10, 0x84, 0x24, 0, 0, 0, 0];
/// `movups xmm15, [rsp + 240]`.
const RESTORE_XMM15: [u8; 9] = [0x44, 0x0F, 0x10, 0xBC, 0x24, 0xF0, 0x00, 0x00, 0x00];
/// `push rbp; mov rbp, rsp` — the ordinary frame link, kept so local
/// displacements match an ordinary function's.
const FRAME_LINK: [u8; 4] = [0x55, 0x48, 0x89, 0xE5];
/// `and rsp, -16` — force the ABI's 16-byte alignment regardless of how many
/// words the vector pushed.
const AND_RSP_16: [u8; 4] = [0x48, 0x83, 0xE4, 0xF0];
/// `cld` — the aggregate helpers' `rep movsb` reads DF.
const CLD: [u8; 1] = [0xFC];
/// `mov rsp, rbp; pop rbp` — undoes the realignment as well as the frame.
const UNWIND_FRAME: [u8; 4] = [0x48, 0x89, 0xEC, 0x5D];
/// `iretq` = REX.W + `CF`. The bare `CF` is `iretd` and would pop 32-bit words.
const IRETQ: [u8; 2] = [0x48, 0xCF];
/// `add rsp, 8` — discard the CPU-pushed error code, which `iretq` does not pop.
const DISCARD_ERROR_CODE: [u8; 4] = [0x48, 0x83, 0xC4, 0x08];

/// `lea rax, [rbp + 376]` — the interrupt-frame pointer on a NO-error-code
/// vector. 376 = 8 (saved rbp) + 256 (XMM area) + 112 (14 GPRs).
const LEA_FRAME_NO_ERROR: [u8; 7] = [0x48, 0x8D, 0x85, 0x78, 0x01, 0x00, 0x00];
/// `lea rax, [rbp + 384]` — the frame pointer on an error-code vector, one word
/// higher because the error code sits below the frame.
const LEA_FRAME_WITH_ERROR: [u8; 7] = [0x48, 0x8D, 0x85, 0x80, 0x01, 0x00, 0x00];
/// `mov rax, [rbp + 376]` — read the CPU-pushed error code.
const LOAD_ERROR_CODE: [u8; 7] = [0x48, 0x8B, 0x85, 0x78, 0x01, 0x00, 0x00];

/// A promoted local's `rbx` spill / reload (`mov [rbp-slot], rbx` /
/// `mov rbx, [rbp-slot]`), matched by the 3-byte opcode+ModRM prefix that is
/// unique to an rbx spill against `[rbp + disp32]`.
const SPILL_RBX: [u8; 3] = [0x48, 0x89, 0x9D];
const RELOAD_RBX: [u8; 3] = [0x48, 0x8B, 0x9D];

fn emit(source: &str) -> NativeProgram {
    emit_native_program(&module_for(source)).expect("emit native program")
}

#[test]
fn interrupt_prologue_saves_every_general_purpose_register() {
    let program = emit(TIMER_ISR);
    assert!(
        program.compiled.contains(&"timer_isr".to_string()),
        "an `interrupt fn` must COMPILE natively, not skip: {:?}",
        program.skipped
    );
    assert!(
        contains_seq(&program.bytes, &PUSH_ALL_GPRS),
        "the ISR prologue must push all 14 general-purpose registers in order \
         (rax rcx rdx rbx rsi rdi r8..r15); an interrupt can arrive between any two \
         instructions, so every one may hold a live value"
    );
    assert!(
        contains_seq(&program.bytes, &FRAME_LINK),
        "the ISR prologue must establish the ordinary rbp frame link after the pushes"
    );
}

#[test]
fn interrupt_prologue_saves_every_xmm_register() {
    let program = emit(TIMER_ISR);
    assert!(
        contains_seq(&program.bytes, &SUB_RSP_256),
        "the ISR prologue must reserve the 256-byte XMM save area"
    );
    assert!(
        contains_seq(&program.bytes, &SAVE_XMM0) && contains_seq(&program.bytes, &SAVE_XMM15),
        "the ISR prologue must `movups`-save xmm0 through xmm15: the SSE reduction \
         and vectorization paths clobber them even for an all-integer body"
    );
    assert!(
        contains_seq(&program.bytes, &RESTORE_XMM0)
            && contains_seq(&program.bytes, &RESTORE_XMM15)
            && contains_seq(&program.bytes, &ADD_RSP_256),
        "the ISR epilogue must restore every XMM register and release the save area"
    );
}

#[test]
fn interrupt_prologue_realigns_the_stack_and_clears_the_direction_flag() {
    let program = emit(TIMER_ISR);
    assert!(
        contains_seq(&program.bytes, &AND_RSP_16),
        "the ISR prologue must force 16-byte stack alignment: the number of words \
         the CPU pushed differs between the error-code and no-error-code vectors, \
         so the entry alignment is not fixed the way an ordinary call's is"
    );
    assert!(
        contains_seq(&program.bytes, &CLD),
        "the ISR prologue must clear the direction flag: the aggregate-copy helpers \
         use `rep movsb`, and an interrupt can land inside a window where the \
         interrupted code has set DF"
    );
    assert!(
        contains_seq(&program.bytes, &UNWIND_FRAME),
        "the ISR epilogue must restore rsp from rbp (`mov rsp, rbp`), not by adding \
         back frame_size — the prologue's realignment moved rsp by an amount the \
         epilogue does not know"
    );
}

#[test]
fn interrupt_epilogue_ends_in_iretq() {
    let program = emit(TIMER_ISR);
    assert!(
        contains_seq(&program.bytes, &POP_ALL_GPRS),
        "the ISR epilogue must pop the general-purpose registers in exact reverse order"
    );
    // The pops must be immediately followed by `iretq` on a no-error-code vector.
    let tail: Vec<u8> = POP_ALL_GPRS.iter().chain(IRETQ.iter()).copied().collect();
    assert!(
        contains_seq(&program.bytes, &tail),
        "a no-error-code ISR must terminate with `iretq` (48 CF) directly after the \
         register restore; the un-prefixed `CF` is `iretd` and pops 32-bit words, \
         corrupting the resumed context"
    );
}

#[test]
fn interrupt_materializes_the_frame_pointer_from_the_stack() {
    let program = emit(TIMER_ISR);
    assert!(
        contains_seq(&program.bytes, &LEA_FRAME_NO_ERROR),
        "the interrupt-frame parameter is not passed in a register: it must be \
         computed as `lea rax, [rbp + 376]` (8 saved rbp + 256 XMM + 112 GPRs)"
    );
    assert!(
        !contains_seq(&program.bytes, &LOAD_ERROR_CODE),
        "a no-error-code ISR must not read an error code word"
    );
}

#[test]
fn error_code_interrupt_shifts_the_frame_and_discards_the_code() {
    let program = emit(PAGE_FAULT_ISR);
    assert!(
        program.compiled.contains(&"page_fault".to_string()),
        "the error-code `interrupt fn` variant must compile: {:?}",
        program.skipped
    );
    assert!(
        contains_seq(&program.bytes, &LEA_FRAME_WITH_ERROR),
        "on an error-code vector the CPU pushes the code BELOW the interrupt frame, \
         so the frame pointer must be 8 bytes higher (`lea rax, [rbp + 384]`)"
    );
    assert!(
        contains_seq(&program.bytes, &LOAD_ERROR_CODE),
        "the error-code parameter must be read from `[rbp + 376]`"
    );
    let tail: Vec<u8> = DISCARD_ERROR_CODE
        .iter()
        .chain(IRETQ.iter())
        .copied()
        .collect();
    assert!(
        contains_seq(&program.bytes, &tail),
        "an error-code ISR must `add rsp, 8` to discard the CPU-pushed code before \
         `iretq`, which does not pop it — leaving it would resume at the wrong address"
    );
    // The no-error-code shape must NOT appear for this program's ISR.
    assert!(
        !contains_seq(&program.bytes, &LEA_FRAME_NO_ERROR),
        "the error-code variant must not also emit the no-error-code frame offset"
    );
}

#[test]
fn the_promotion_guard_holds_against_a_promotable_shape() {
    // HONEST SCOPE, stated because the first version of this test did not have
    // it and was therefore vacuous. No *source-level* `interrupt fn` can be
    // register-promoted anyway: `plan_register_promotion` returns nothing unless
    // the return type is `i64` and every parameter is `i64`, while `L0446`
    // requires an ISR to return `void` and take a `ptr<T>`. So a test written over
    // real Lullaby source cannot fail no matter what the guard does — which is
    // exactly what the earlier `interrupt_is_never_register_promoted` was.
    //
    // The guard is still worth keeping and worth pinning: it makes the invariant
    // independent of `plan_register_promotion`'s *unrelated* type-shape rules,
    // which could widen (to `void` returns, or to pointer parameters) without
    // anyone thinking about interrupt handlers. This test reaches past the
    // frontend to build the shape the frontend cannot produce — an `i64`-returning,
    // `i64`-parameter function with a hot loop — and lowers it BOTH ways.
    //
    // The `Ordinary` arm is the control, and it is what gives the test teeth: it
    // proves the constructed function really is promotable, so the `Interrupt`
    // arm's absence of a spill is evidence about the guard rather than about the
    // body. Flipping `NativeFnKind::promotes_registers` to `true` makes the
    // Interrupt arm fail.
    let module = module_for(concat!(
        // One parameter, so it matches an `interrupt fn`'s arity and the ISR arm
        // seats it as the frame pointer (a synthetic shape — the point is the
        // guard, not the signature). Two hot `i64` locals in a loop are what the
        // promoter targets.
        "fn hot a i64 -> i64
",
        "    let acc i64 = a
",
        "    let step i64 = 1
",
        "    for i from 0 to 8
",
        "        acc = acc + step
",
        "        step = step + i
",
        "    acc + step
",
        "
",
        "fn main -> i64
",
        "    hot(1)
",
    ));
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "hot")
        .expect("the promotable function is in the module");

    let lower_as = |kind: NativeFnKind| {
        let callable = std::collections::HashSet::new();
        let extern_sigs = HashMap::new();
        let signatures = HashMap::new();
        let array_lengths = ArrayLengths::new();
        let closure_layouts = HashMap::new();
        let hof_index = HashMap::new();
        let mut strings = StringPool::default();
        lower_native_function(
            function,
            &callable,
            &extern_sigs,
            &module.structs,
            &module.enums,
            &mut strings,
            &signatures,
            &array_lengths,
            false,
            false,
            &closure_layouts,
            &hof_index,
            kind,
        )
        .unwrap_or_else(|e| panic!("the function must lower under both conventions: {e}"))
        .code
    };

    let ordinary = lower_as(NativeFnKind::Ordinary);
    assert!(
        contains_seq(&ordinary, &SPILL_RBX) && contains_seq(&ordinary, &RELOAD_RBX),
        "CONTROL: this body must genuinely be register-promoted under the ordinary \
         convention, or the Interrupt arm below proves nothing about the guard"
    );

    let isr = lower_as(NativeFnKind::Interrupt {
        has_error_code: false,
    });
    assert!(
        !contains_seq(&isr, &SPILL_RBX) && !contains_seq(&isr, &RELOAD_RBX),
        "the SAME body lowered as an `interrupt fn` must not be register-promoted: \
         the promoted register's spill and restore live in the ordinary prologue \
         and epilogue, neither of which an ISR runs, so the caller's `rbx` would be \
         handed back to the interrupted context holding a Lullaby local"
    );
}

#[test]
fn every_interrupt_return_edge_uses_iretq() {
    // An explicit `return` inside the body emits its own epilogue. It must be the
    // ISR epilogue too — the defect this pins is a body that leaves through `ret`
    // on one path and `iretq` on another.
    let program = emit(concat!(
        "no-runtime\n",
        "\n",
        "interrupt fn early_out frame ptr<i64>\n",
        "    unsafe\n",
        "        let v i64 = ptr_read(frame)\n",
        "        if v == 0\n",
        "            return\n",
        "        ptr_write(frame, v - 1)\n",
        "\n",
        "fn main -> i64\n",
        "    0\n",
    ));
    assert!(
        program.compiled.contains(&"early_out".to_string()),
        "the ISR must compile: {:?}",
        program.skipped
    );
    // Two exits (the early `return` and the fallthrough) ⇒ two `iretq`s, and no
    // `pop rbp; ret` pair anywhere in this ISR-only program except `main`'s.
    let iretq_count = program.bytes.windows(2).filter(|w| *w == IRETQ).count();
    assert!(
        iretq_count >= 2,
        "each return edge of an `interrupt fn` must end in `iretq`; found {iretq_count}"
    );
}

/// The naked-function program used by the two tests below: `cli; hlt` in one
/// `asm`, then `xor eax, eax; ret` split across two more, so the concatenation is
/// observable as well as the absence of a wrapper.
const NAKED_PROGRAM: &str = concat!(
    "no-runtime\n",
    "\n",
    "naked fn boot_entry\n",
    "    unsafe\n",
    "        asm 250, 244\n", // cli; hlt
    "        asm 49, 192\n",  // xor eax, eax
    "        asm 195\n",      // ret
    "\n",
    "fn main -> i64\n",
    "    0\n",
);

/// The exact bytes `boot_entry` must lower to — its `asm` bodies concatenated,
/// with nothing before, between, or after them.
const NAKED_EXPECTED: [u8; 5] = [0xFA, 0xF4, 0x31, 0xC0, 0xC3];

#[test]
fn naked_function_lowers_to_exactly_its_asm_bytes() {
    // The definitive pin, and the reason it is an EXACT equality rather than a
    // byte-window search: an ordinary prologue is `push rbp; mov rbp, rsp` followed
    // by `sub rsp, <frame_size>`, and the planner always reserves at least one
    // scratch word — so a `contains(&[0x55, 0x48, 0x89, 0xE5, <body>])` search
    // never matches even when the prologue IS emitted, and passes while proving
    // nothing. (Measured: with the `NativeFnKind::Naked` early return removed from
    // `lower_native_function`, the window-search version of this test stayed green.)
    let module = module_for(NAKED_PROGRAM);
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "boot_entry")
        .expect("the naked function is in the module");
    assert_eq!(
        NativeFnKind::for_function(&module, "boot_entry"),
        NativeFnKind::Naked,
        "the module's naked-function table must classify `boot_entry`"
    );

    let callable = std::collections::HashSet::new();
    let extern_sigs = HashMap::new();
    let signatures = HashMap::new();
    let array_lengths = ArrayLengths::new();
    let closure_layouts = HashMap::new();
    let hof_index = HashMap::new();
    let mut strings = StringPool::default();
    let lowered = lower_native_function(
        function,
        &callable,
        &extern_sigs,
        &module.structs,
        &module.enums,
        &mut strings,
        &signatures,
        &array_lengths,
        false,
        false,
        &closure_layouts,
        &hof_index,
        NativeFnKind::Naked,
    )
    .expect("a naked function must lower");

    assert_eq!(
        lowered.code.as_slice(),
        NAKED_EXPECTED.as_slice(),
        "a `naked fn` must lower to EXACTLY its `asm` bytes — no prologue, no \
         epilogue, no implicit `ret`, no fallthrough `xor rax, rax`. That absence is \
         the definition of the kind."
    );
    assert!(
        lowered.relocations.is_empty(),
        "a naked body emits raw bytes only and can reference no symbol"
    );
}

#[test]
fn naked_function_reaches_the_object_through_the_program_driver() {
    // The companion to the exact-bytes pin: the driver must actually ROUTE a naked
    // function to `lower_naked_function` and emit it, rather than skipping it (an
    // ISR or boot entry missing from `.text` is a symbol the bootloader jumps into
    // nothing at).
    let program = emit(NAKED_PROGRAM);
    assert!(
        program.compiled.contains(&"boot_entry".to_string()),
        "a `naked fn` must compile natively, not skip: {:?}",
        program.skipped
    );
    assert!(
        contains_seq(&program.bytes, &NAKED_EXPECTED),
        "the naked body's `asm` bytes must appear in `.text` contiguously, so the \
         three `asm` statements concatenate with nothing inserted between them"
    );
}

#[test]
fn the_arena_guard_refuses_a_hardware_entry_point() {
    // Same honest scope as the promotion guard above: no source-level ISR can
    // reach the arena path today (a `no-runtime` module allocates nothing, and the
    // arena set only ever admits heap-using functions), so a test over real source
    // cannot fail. The guard exists so the invariant does not silently depend on
    // that — an interrupt can arrive while a *different* function is mid-region,
    // and the arena prologue/epilogue save and restore the GLOBAL bump pointer
    // `__lullaby_heap_next`, so an ISR that rewound it would reclaim memory the
    // interrupted function still owns.
    //
    // So this drives `lower_native_function` directly with `is_arena = true`,
    // which is the re-check the driver's set-filtering is backed by. The Ordinary
    // arm is the control: it proves the same function DOES lower on the arena path,
    // so the Interrupt arm's refusal is evidence about the guard.
    //
    // (What is emphatically NOT asserted here: `!Interrupt.allows_arena()`. An
    // earlier revision did exactly that — the predicate against itself — which is
    // a tautology and says nothing about emitted code.)
    let module = module_for(concat!(
        "fn adds a i64 -> i64
",
        "    a + 1
",
        "
",
        "fn main -> i64
",
        "    adds(1)
",
    ));
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "adds")
        .expect("the function is in the module");

    let lower_as = |kind: NativeFnKind| {
        let callable = std::collections::HashSet::new();
        let extern_sigs = HashMap::new();
        let signatures = HashMap::new();
        let array_lengths = ArrayLengths::new();
        let closure_layouts = HashMap::new();
        let hof_index = HashMap::new();
        let mut strings = StringPool::default();
        lower_native_function(
            function,
            &callable,
            &extern_sigs,
            &module.structs,
            &module.enums,
            &mut strings,
            &signatures,
            &array_lengths,
            false,
            /* is_arena */ true,
            &closure_layouts,
            &hof_index,
            kind,
        )
    };

    assert!(
        lower_as(NativeFnKind::Ordinary).is_ok(),
        "CONTROL: an ordinary function must lower fine with the arena path enabled, \
         or the Interrupt arm below proves nothing about the guard"
    );

    let refused = lower_as(NativeFnKind::Interrupt {
        has_error_code: false,
    });
    let Err(reason) = refused else {
        panic!(
            "an `interrupt fn` handed `is_arena` must be REFUSED (demoted to the \
             interpreters), not lowered with the arena prologue"
        );
    };
    assert!(
        reason.contains("hardware entry point") && reason.contains("arena"),
        "the refusal must name the reason, not fail incidentally: {reason}"
    );
}

#[test]
fn the_module_tables_classify_each_function_kind() {
    // Non-tautological companion to the two guard tests: `NativeFnKind::for_function`
    // is what turns the module's `interrupt_functions`/`naked_functions` tables into
    // the kind the emitter switches on, so a table that failed to carry a function
    // through IR lowering would silently give it the ordinary convention. Pinned by
    // classification, with an ordinary function as the negative control.
    let module = module_for(concat!(
        "no-runtime
",
        "
",
        "interrupt fn timer_isr frame ptr<i64>
",
        "    unsafe
",
        "        ptr_write(frame, 1)
",
        "
",
        "interrupt fn page_fault frame ptr<i64> error_code u64
",
        "    unsafe
",
        "        ptr_write(frame, to_i64(error_code))
",
        "
",
        "naked fn boot_entry
",
        "    unsafe
",
        "        asm 244
",
        "
",
        "fn main -> i64
",
        "    0
",
    ));
    assert_eq!(
        NativeFnKind::for_function(&module, "timer_isr"),
        NativeFnKind::Interrupt {
            has_error_code: false
        },
        "a one-parameter `interrupt fn` is a no-error-code vector"
    );
    assert_eq!(
        NativeFnKind::for_function(&module, "page_fault"),
        NativeFnKind::Interrupt {
            has_error_code: true
        },
        "the second `error_code u64` parameter selects the error-code vector \
         convention — the one bit of the ABI the emitter cannot recover from the body"
    );
    assert_eq!(
        NativeFnKind::for_function(&module, "boot_entry"),
        NativeFnKind::Naked
    );
    assert_eq!(
        NativeFnKind::for_function(&module, "main"),
        NativeFnKind::Ordinary,
        "an ordinary function in the same module must keep the ordinary convention"
    );
}

#[test]
fn hardware_entry_functions_are_skipped_by_the_wasm_backend() {
    let ir = {
        let tokens = lex(TIMER_ISR).expect("lex");
        let program = parse(&tokens).expect("parse");
        let checked = validate_executable(&program).expect("semantic");
        lower(&checked).expect("lower")
    };
    let artifact = crate::wasm::emit_wasm_module(&ir).expect("emit wasm");
    assert!(
        artifact
            .skipped
            .iter()
            .any(|s| s.name == "timer_isr" && s.reason.contains("hardware calling convention")),
        "an `interrupt fn` must be skipped by the WASM backend rather than emitted \
         as an ordinary WASM function: {:?}",
        artifact.skipped
    );
    assert!(
        !artifact.compiled.contains(&"timer_isr".to_string()),
        "an `interrupt fn` must not appear among the compiled WASM functions"
    );
}
