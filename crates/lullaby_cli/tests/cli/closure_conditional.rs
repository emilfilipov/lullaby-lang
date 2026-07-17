//! CLI integration tests — an **inline conditional as a closure body**, across
//! every execution tier.
//!
//! # The divergence these pin
//!
//! A closure body is a single expression in the surface grammar (`expr_parser`
//! parses it with `parse_conditional`), but a *ternary* body does not lower to a
//! single IR expression: `desugar_conditional` hoists `let __cond_N` plus an `if`
//! and rewrites the position to `__cond_N`. That scaffolding reads the closure's
//! **parameters**.
//!
//! The closure lowerer used to lower its body with a plain `lower_expr`, leaving
//! those statements in the shared `try_prelude`, which the enclosing block lowerer
//! drained into the **enclosing function**. At runtime the enclosing frame has no
//! such parameter, so:
//!
//! | tier            | `fn x i64 -> 1 if x > 0 else 0`     |
//! |-----------------|-------------------------------------|
//! | AST interpreter | correct answer (never hoists)       |
//! | IR interpreter  | `L0403 unknown variable \`x\`` in `main` |
//! | bytecode VM     | `L0403 unknown variable \`x\`` in `main` |
//! | native          | clean `L0339` skip                  |
//!
//! Two tiers died at **runtime**, naming the user's own closure parameter, on a
//! program semantics had accepted and one tier answered correctly. The fix gives
//! `IrClosureDef` a `prelude` that travels with the closure and runs in the
//! closure's own frame, per call.
//!
//! # Why native is pinned as a refusal, not an answer
//!
//! Native does not compile a ternary-bodied closure: the body is `__cond_N`, which
//! its capture analysis cannot bind to a native local, so the function skips
//! cleanly to the interpreters (`L0339`). That is the correct-or-refuse contract —
//! a tier answers like the others or declines with a diagnostic. What it must never
//! do is compile the body while ignoring the prelude, which would answer
//! *differently*. `native_closure_conditional_body_skips_cleanly` pins the refusal
//! so that stays a deliberate boundary rather than something that quietly erodes.

use super::{lullaby, stderr, stdout, workspace_root};

/// An inline conditional as a closure body — captured, nested, passed to a
/// higher-order function, returned from one, and with both branches taken —
/// computes 45 identically on all three interpreter tiers.
///
/// Before the fix this printed 45 on the AST interpreter and failed with
/// `L0403 unknown variable \`x\`` on the other two.
#[test]
pub(crate) fn runs_closure_conditional_fixture_on_all_backends() {
    let fixture = workspace_root().join("tests/fixtures/valid/run_closure_conditional.lby");
    for backend in [None, Some("ir"), Some("bytecode")] {
        let mut args = vec!["run".to_string()];
        if let Some(backend) = backend {
            args.push("--backend".to_string());
            args.push(backend.to_string());
        }
        args.push(fixture.to_str().expect("fixture path").to_string());
        let output = lullaby().args(&args).output().expect("run cli");
        assert!(
            output.status.success(),
            "backend {backend:?}: a ternary closure body must run: {}",
            stderr(&output)
        );
        assert_eq!(
            stdout(&output).trim(),
            "45",
            "backend {backend:?}: every tier must agree on the ternary-closure result"
        );
    }
}

/// The ternary body is evaluated **lazily, at call time**: exactly the taken arm
/// runs, exactly once per call.
///
/// The transcript is the whole point. `TAKEN` is printed only by the then-arm, and
/// the closure is called once taking each branch, so the exact stdout distinguishes
/// correct laziness from every plausible wrong schedule:
///
/// - untaken arm evaluated too  -> `TAKEN` appears twice,
/// - prelude run eagerly at closure creation -> `TAKEN` precedes `start`, or the
///   parameter-reading prelude fails outright,
/// - prelude run once and cached -> the second call cannot re-decide.
#[test]
pub(crate) fn closure_conditional_body_is_lazy_on_all_backends() {
    let fixture = workspace_root().join("tests/fixtures/valid/run_closure_conditional_lazy.lby");
    for backend in [None, Some("ir"), Some("bytecode")] {
        let mut args = vec!["run".to_string()];
        if let Some(backend) = backend {
            args.push("--backend".to_string());
            args.push(backend.to_string());
        }
        args.push(fixture.to_str().expect("fixture path").to_string());
        let output = lullaby().args(&args).output().expect("run cli");
        assert!(
            output.status.success(),
            "backend {backend:?}: {}",
            stderr(&output)
        );
        let rendered = stdout(&output);
        let lines: Vec<&str> = rendered.lines().map(str::trim).collect();
        // `start` then one `TAKEN` (from `f(2)` only) then `end`, then `a + b` = 3.
        assert_eq!(
            lines,
            vec!["start", "TAKEN", "end", "3"],
            "backend {backend:?}: only the taken arm may run, once, at call time"
        );
    }
}

/// A closure whose *only* disqualifying trait is its ternary body skips cleanly to
/// the interpreters (`L0339`) rather than being miscompiled.
///
/// The fixture is otherwise squarely inside the native scalar subset — a direct
/// literal, no captures, scalar parameter and return, called directly — so this
/// isolates the conditional as the sole cause. Native must refuse, and the
/// interpreters must still compute 10.
#[test]
pub(crate) fn native_closure_conditional_body_skips_cleanly() {
    let fixture = workspace_root().join("tests/fixtures/valid/native_closure_conditional_skip.lby");

    let native = lullaby()
        .args([
            "native",
            "--verbose",
            fixture.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run cli");
    assert!(
        !native.status.success(),
        "a ternary-bodied closure must not compile natively"
    );
    let rendered = format!("{}{}", stdout(&native), stderr(&native));
    assert!(rendered.contains("L0339"), "expected L0339: {rendered}");
    assert!(
        rendered.contains("skipped main"),
        "expected `main` in the skip listing: {rendered}"
    );

    // The refusal is a boundary, not a dead end: the program still runs, and every
    // interpreter tier agrees on the answer native declined to produce.
    for backend in [None, Some("ir"), Some("bytecode")] {
        let mut args = vec!["run".to_string()];
        if let Some(backend) = backend {
            args.push("--backend".to_string());
            args.push(backend.to_string());
        }
        args.push(fixture.to_str().expect("fixture path").to_string());
        let run = lullaby().args(&args).output().expect("run cli");
        assert!(
            run.status.success(),
            "backend {backend:?}: {}",
            stderr(&run)
        );
        assert_eq!(
            stdout(&run).trim(),
            "10",
            "backend {backend:?}: the skipped program still runs on the interpreters"
        );
    }
}
