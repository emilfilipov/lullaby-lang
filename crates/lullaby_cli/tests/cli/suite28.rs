//! CLI integration tests, part 28 — the freestanding tier's **hardware entry
//! points**: `interrupt fn` and `naked fn`
//! (`documents/freestanding_tier_design.md` §6, stage 8).
//!
//! # What can and cannot be executed here, and why
//!
//! Neither construct can be *entered* from a hosted test process, and that is a
//! property of the constructs rather than a gap in the harness:
//!
//! * an `interrupt fn` is reached by the CPU from an **IDT entry** and leaves
//!   with `iretq`. Installing an IDT entry is a ring-0 operation; and calling one
//!   from Lullaby is refused at compile time (`L0446`) precisely because `iretq`
//!   would pop three words of the caller's frame as `rip`/`cs`/`rflags`.
//! * a `naked fn` has no compiler-generated prologue, epilogue, or `ret` at all —
//!   its control flow is whatever its `asm` writes — and is likewise not callable.
//!
//! So the proof is split, exactly as it is for port I/O (`suite15`, whose `in`/
//! `out` fault at CPL 3):
//!
//! * **Executed here:** the surrounding program. The valid fixture's `main`
//!   returns 21 on all three interpreters and natively, so the mere *presence* of
//!   the two kinds is proven not to disturb the rest of the module — which is the
//!   testable content of §6's "on the interpreters an `interrupt fn` body runs as
//!   an ordinary function". Also executed: `lullaby native --freestanding` really
//!   emits a runnable image containing them.
//! * **Pinned byte-exactly elsewhere:** the ISR prologue/epilogue, the frame and
//!   error-code materialization, `iretq`, and the *absence* of any emitted code
//!   around a naked body — `crates/lullaby_ir/src/native_object_isr_tests.rs`.
//! * **Rejected here:** every `L0446` signature/body constraint, the `L0441` tier
//!   gate, the `L0201` modifier-exclusivity rules, and the not-callable rule.
//!
//! INJECT-THE-BUG TEETH (measured on this branch): deleting the `L0446`
//! signature checks in `crates/lullaby_semantics/src/semantics_isr.rs` makes the
//! `interrupt_*`/`naked_*` rejection tests below fail; deleting the not-callable
//! check in `semantics_checker_calls.rs` makes
//! [`calling_an_interrupt_fn_is_rejected`] fail; and removing the two
//! `eat_keyword` calls from the parser's modifier loop makes every test here fail
//! at parse time.

use crate::*;

/// The four-tier valid fixture.
const VALID_FIXTURE: &str = "tests/fixtures/valid/no_runtime/freestanding_interrupt_naked.lby";

/// `lullaby check` a fixture, asserting it is rejected with `code` and that the
/// message mentions `needle` (so a test cannot pass on the *wrong* violation).
fn assert_rejected_with(fixture: &str, code: &str, needle: &str) {
    let path = workspace_root().join(fixture);
    let output = lullaby()
        .args(["check", path.to_str().expect("fixture path")])
        .output()
        .expect("run cli");
    let stderr = stderr(&output);
    assert!(
        !output.status.success(),
        "{fixture} should be rejected but exited 0. stderr: {stderr}"
    );
    assert!(
        stderr.contains(code),
        "{fixture} should report {code}. stderr: {stderr}"
    );
    assert!(
        stderr.contains(needle),
        "{fixture}'s {code} should be the {needle:?} violation. stderr: {stderr}"
    );
}

/// Reject `source` (written to a scratch file) with `code`, mentioning `needle`.
fn assert_source_rejected_with(source: &str, tag: &str, code: &str, needle: &str) {
    let dir = ScratchDir::new("isr_reject");
    let src = dir.join(format!("{tag}.lby"));
    std::fs::write(&src, source).expect("write source");
    let output = lullaby()
        .args(["check", src.to_str().expect("src path")])
        .output()
        .expect("run cli");
    let stderr = stderr(&output);
    assert!(
        !output.status.success(),
        "{tag} should be rejected but exited 0. stderr: {stderr}"
    );
    assert!(
        stderr.contains(code),
        "{tag} should report {code}. stderr: {stderr}"
    );
    assert!(
        stderr.contains(needle),
        "{tag}'s {code} should be the {needle:?} violation. stderr: {stderr}"
    );
}

// -- The tier gate (`L0441`) -------------------------------------------------

#[test]
fn hardware_entry_points_require_the_no_runtime_tier() {
    // §6: "using them in a non-`no-runtime` module is `L0441`". The check must
    // live OUTSIDE the `no-runtime` enforcement pass, which returns early for a
    // module without the directive — which is exactly the case being caught.
    assert_rejected_with(
        "tests/fixtures/invalid/interrupt_outside_no_runtime.lby",
        "L0441",
        "freestanding-tier construct",
    );
    assert_rejected_with(
        "tests/fixtures/invalid/naked_outside_no_runtime.lby",
        "L0441",
        "freestanding-tier construct",
    );
}

// -- `interrupt fn` signature constraints (`L0446`) --------------------------

#[test]
fn interrupt_signature_constraints_are_enforced() {
    for (fixture, needle) in [
        (
            "tests/fixtures/invalid/no_runtime/interrupt_wrong_arity.lby",
            "declares 3 parameters",
        ),
        (
            "tests/fixtures/invalid/no_runtime/interrupt_no_params.lby",
            "declares 0 parameters",
        ),
        (
            "tests/fixtures/invalid/no_runtime/interrupt_non_pointer_frame.lby",
            "must be a raw pointer",
        ),
        (
            "tests/fixtures/invalid/no_runtime/interrupt_bad_error_code_type.lby",
            "must be declared `u64`",
        ),
        (
            "tests/fixtures/invalid/no_runtime/interrupt_non_void_return.lby",
            "must return `void`",
        ),
    ] {
        assert_rejected_with(fixture, "L0446", needle);
    }
}

#[test]
fn a_generic_interrupt_fn_is_rejected() {
    // A hardware entry point is named once, by an IDT entry — there is no call
    // site for a type argument to be inferred from, and monomorphization would
    // produce several symbols for one vector.
    assert_source_rejected_with(
        concat!(
            "no-runtime\n",
            "\n",
            "interrupt fn generic_isr<T> frame ptr<i64>\n",
            "    unsafe\n",
            "        asm 144\n",
            "\n",
            "fn main -> i64\n",
            "    0\n",
        ),
        "generic_isr",
        "L0446",
        "may not be generic",
    );
}

// -- `naked fn` body constraints (`L0446`) -----------------------------------

#[test]
fn naked_body_constraints_are_enforced() {
    for (fixture, needle) in [
        (
            "tests/fixtures/invalid/no_runtime/naked_has_parameters.lby",
            "takes no parameters",
        ),
        (
            "tests/fixtures/invalid/no_runtime/naked_non_asm_body.lby",
            "only `asm` statements",
        ),
        (
            "tests/fixtures/invalid/no_runtime/naked_control_flow.lby",
            "only `asm` statements",
        ),
        (
            "tests/fixtures/invalid/no_runtime/naked_asm_operands.lby",
            "may not bind `in`/`out` operands",
        ),
        (
            "tests/fixtures/invalid/no_runtime/naked_asm_clobbers.lby",
            "may not declare `clobber`s",
        ),
    ] {
        assert_rejected_with(fixture, "L0446", needle);
    }
}

#[test]
fn a_naked_fn_with_a_return_type_is_rejected() {
    assert_source_rejected_with(
        concat!(
            "no-runtime\n",
            "\n",
            "naked fn returns_a_value -> i64\n",
            "    unsafe\n",
            "        asm 195\n",
            "\n",
            "fn main -> i64\n",
            "    0\n",
        ),
        "naked_returns",
        "L0446",
        "must return `void`",
    );
}

// -- The not-callable rule (`L0446`) -----------------------------------------

#[test]
fn calling_an_interrupt_fn_is_rejected() {
    // The failure this prevents has no runtime diagnostic, only a wild jump: a
    // `call` pushes a return address, and the ISR's `iretq` pops THREE words as
    // `rip`/`cs`/`rflags` — the return address plus two words of the caller's own
    // frame.
    assert_rejected_with(
        "tests/fixtures/invalid/no_runtime/interrupt_called_from_lullaby.lby",
        "L0446",
        "cannot be called",
    );
}

#[test]
fn a_hardware_entry_point_cannot_be_used_as_a_function_value() {
    // The alias route around the not-callable rule, and the one that actually got
    // through: guarding only the direct-call path let the bare name be typed as a
    // `fn(...)` value, bound to a local (or passed to a higher-order function), and
    // invoked from there — which type-checked and **ran the ISR body**, returning 7
    // on all three interpreters. Native happened to decline, but for a wholly
    // unrelated reason (a fn-typed local that is not a closure literal or a factory
    // call is outside the native subset), so this was a silent tier divergence that
    // would have become a real `call`-into-`iretq` once native gains first-class
    // function values.
    //
    // The rule is therefore about the VALUE, not the call syntax, which is what
    // makes it cover every sink — a local, a higher-order argument, a returned
    // callable — with one check.
    assert_rejected_with(
        "tests/fixtures/invalid/no_runtime/interrupt_as_fn_value.lby",
        "L0446",
        "cannot be used as a function value",
    );
    assert_rejected_with(
        "tests/fixtures/invalid/no_runtime/naked_passed_to_higher_order_fn.lby",
        "L0446",
        "cannot be used as a function value",
    );
}

#[test]
fn a_hardware_entry_point_used_as_a_value_never_runs_on_an_interpreter() {
    // The teeth that matter: the compile-time refusal above must actually prevent
    // EXECUTION. Before the value route was closed, this exact fixture printed `7`
    // — the ISR body's write, observed through the pointer — on ast, ir, and
    // bytecode alike. Asserting the diagnostic alone would not have caught that,
    // because the diagnostic was simply absent.
    let path = workspace_root().join("tests/fixtures/invalid/no_runtime/interrupt_as_fn_value.lby");
    for backend in ["ast", "ir", "bytecode"] {
        let output = lullaby()
            .args([
                "run",
                "--backend",
                backend,
                path.to_str().expect("fixture path"),
            ])
            .output()
            .expect("run cli");
        let stderr = stderr(&output);
        assert!(
            !output.status.success(),
            "[{backend}] an `interrupt fn` reached through a fn value must not run. \
             stdout: {} stderr: {stderr}",
            stdout(&output)
        );
        assert!(
            stderr.contains("L0446"),
            "[{backend}] the refusal must be L0446. stderr: {stderr}"
        );
        assert!(
            !stdout(&output).contains('7'),
            "[{backend}] the ISR body must NOT have executed (it writes 7 through the \
             frame pointer). stdout: {}",
            stdout(&output)
        );
    }
}

#[test]
fn calling_a_naked_fn_is_rejected() {
    assert_source_rejected_with(
        concat!(
            "no-runtime\n",
            "\n",
            "naked fn boot_entry\n",
            "    unsafe\n",
            "        asm 195\n",
            "\n",
            "fn main -> i64\n",
            "    boot_entry()\n",
            "    0\n",
        ),
        "call_naked",
        "L0446",
        "cannot be called",
    );
}

// -- Modifier exclusivity (`L0201`) ------------------------------------------

#[test]
fn hardware_entry_modifiers_are_exclusive() {
    // `interrupt` and `naked` name two DIFFERENT calling conventions, and each is
    // exclusive with every visibility/ABI modifier: `pub`/`export` advertise a
    // function as callable (it is not), `async` needs the runtime the freestanding
    // tier removes, and `extern` is a body-less import rather than a definition.
    let cases: [(&str, &str, &str); 7] = [
        (
            "both",
            "interrupt naked fn f frame ptr<i64>\n    unsafe\n        asm 244\n",
            "`interrupt` and `naked` cannot be combined",
        ),
        (
            "pub_interrupt",
            "pub interrupt fn f frame ptr<i64>\n    unsafe\n        asm 244\n",
            "`pub` and `interrupt` cannot be combined",
        ),
        (
            "pub_naked",
            "pub naked fn f\n    unsafe\n        asm 244\n",
            "`pub` and `naked` cannot be combined",
        ),
        (
            "async_interrupt",
            "async interrupt fn f frame ptr<i64>\n    unsafe\n        asm 244\n",
            "`async` and `interrupt` cannot be combined",
        ),
        (
            "extern_naked",
            "extern naked fn f -> i64\n",
            "`extern` and `naked` cannot be combined",
        ),
        (
            "export_interrupt",
            "export interrupt fn f frame ptr<i64>\n    unsafe\n        asm 244\n",
            "`export` and `interrupt` cannot be combined",
        ),
        (
            "export_naked",
            "export naked fn f\n    unsafe\n        asm 244\n",
            "`export` and `naked` cannot be combined",
        ),
    ];
    for (tag, decl, needle) in cases {
        let source = format!("no-runtime\n\n{decl}\nfn main -> i64\n    0\n");
        assert_source_rejected_with(&source, tag, "L0201", needle);
    }
}

#[test]
fn a_stray_hardware_entry_modifier_must_prefix_a_fn() {
    for (tag, keyword) in [("stray_interrupt", "interrupt"), ("stray_naked", "naked")] {
        let source = format!("no-runtime\n\n{keyword} const X i64 = 1\n\nfn main -> i64\n    0\n");
        assert_source_rejected_with(
            &source,
            tag,
            "L0201",
            &format!("`{keyword}` must prefix a `fn` declaration"),
        );
    }
}

// -- The valid fixture on every tier -----------------------------------------

#[test]
fn interrupt_and_naked_declarations_do_not_disturb_the_interpreters() {
    // §6: "on the interpreters an `interrupt fn` body runs as an ordinary
    // function". Because neither kind is callable, the observable content of that
    // statement is that their PRESENCE changes nothing: the module type-checks and
    // the rest of it runs with normal results on all three interpreters.
    let path = workspace_root().join(VALID_FIXTURE);
    for backend in ["ast", "ir", "bytecode"] {
        let output = lullaby()
            .args([
                "run",
                "--backend",
                backend,
                path.to_str().expect("fixture path"),
            ])
            .output()
            .expect("run cli");
        assert!(
            output.status.success(),
            "[{backend}] the fixture should run. stderr: {}",
            stderr(&output)
        );
        assert_eq!(
            stdout(&output).trim(),
            "21",
            "[{backend}] `main` must be unaffected by the hardware entry points. stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn interrupt_and_naked_compile_into_a_freestanding_image() {
    // The native half: `lullaby native --freestanding` must EMIT an image
    // containing both kinds — a skip would mean the ISR silently does not exist in
    // the binary the IDT is meant to point at. The resulting exe is run only to
    // confirm the ordinary `main` still works; the ISR and naked bodies are never
    // entered (see the module header).
    if !cfg!(windows) {
        eprintln!("not a Windows host; skipping the freestanding-image test (ran nothing)");
        return;
    }
    let dir = ScratchDir::new("isr_freestanding_image");
    let exe = dir.join("isr_freestanding.exe");
    let path = workspace_root().join(VALID_FIXTURE);
    let output = lullaby()
        .args([
            "native",
            "--freestanding",
            "--verbose",
            "-o",
            exe.to_str().expect("exe path"),
            path.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "the freestanding native emit must succeed. stderr: {}",
        stderr(&output)
    );
    assert!(
        exe.is_file(),
        "a freestanding exe must be written. stdout: {}",
        stdout(&output)
    );
    // A POSITIVE assertion (`compiled <name>`), not merely the absence of a skip
    // line: a function that vanished from the module entirely — dropped by a
    // future dead-code pass, say, since nothing in Lullaby can call these — would
    // be neither compiled nor skipped, and an absence-only check would pass while
    // the image lost the very symbol an IDT entry is meant to point at.
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    for name in [
        "timer_isr",
        "page_fault",
        "accumulate_isr",
        "boot_entry",
        "zero_and_return",
    ] {
        assert!(
            combined.contains(&format!("compiled {name}")),
            "`{name}` must be COMPILED into the image (not skipped, and not dropped \
             from the module): {combined}"
        );
    }
    let run = std::process::Command::new(&exe)
        .output()
        .expect("run the freestanding exe");
    assert_eq!(
        run.status.code(),
        Some(21),
        "the ordinary `main` of a module containing hardware entry points must still run"
    );
}

// -- Formatter round-trip ----------------------------------------------------

#[test]
fn the_formatter_round_trips_both_modifiers() {
    // A modifier that does not survive `lullaby fmt` is a defect: formatting would
    // silently turn an ISR into an ordinary function.
    let path = workspace_root().join(VALID_FIXTURE);
    let output = lullaby()
        .args(["fmt", path.to_str().expect("fixture path")])
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "fmt should succeed. stderr: {}",
        stderr(&output)
    );
    let formatted = stdout(&output);
    for line in [
        "interrupt fn timer_isr frame ptr<i64>",
        "interrupt fn page_fault frame ptr<i64> error_code u64",
        "naked fn boot_entry",
        "naked fn zero_and_return",
    ] {
        assert!(
            formatted.contains(line),
            "the formatter must render `{line}`:\n{formatted}"
        );
    }

    // And the formatted output must itself still check: a round-trip that produced
    // text the parser rejects would be caught here rather than by a reader.
    let dir = ScratchDir::new("isr_fmt_round_trip");
    let src = dir.join("formatted.lby");
    std::fs::write(&src, &formatted).expect("write formatted source");
    let recheck = lullaby()
        .args(["check", src.to_str().expect("src path")])
        .output()
        .expect("run cli");
    assert!(
        recheck.status.success(),
        "the formatter's output must re-check cleanly. stderr: {}",
        stderr(&recheck)
    );
}
