//! CLI integration tests, part 11 (multi-parameter and bounded generic types,
//! stage 5). Split out of tests/cli.rs so it does not overlap the native / fuzz /
//! socket / stdin / map / match / const / generic-struct / generic-enum /
//! generic-method suites.
//!
//! Stage 5 is the final generics-frontend increment. It covers two features:
//!
//! - **Multi-parameter generic types.** `struct Pair<K, V>` and `enum
//!   Either<L, R>` declare two independent type parameters; each is inferred
//!   separately from construction, annotations, and method return types.
//!   (Single-parameter structs/enums/methods shipped in stages 1–4; two-parameter
//!   inference already worked and is pinned here against regression.)
//!
//! - **Trait bounds on generic types.** `struct Ranked<T: Rank>` constrains its
//!   parameter, so the type's inherent-impl method may call the bound trait's
//!   methods on a `T` value — the declaration's bound propagates into `impl
//!   Ranked<T>`. Every instantiation must satisfy the bound, else the
//!   unsatisfied-bound diagnostic (`L0400`) fires.
//!
//! Generics are erased on the three interpreters, so both fixtures run
//! byte-for-byte identically on `ast`, `ir`, and `bytecode`. The native backend
//! is not yet monomorphized for user generics, so a function using one is cleanly
//! skipped via the existing `L0339` no-eligible-function gate — never
//! miscompiled. The positive fixtures live under `tests/fixtures/valid/generics/`
//! and the negatives under `tests/fixtures/invalid/generics/` so the
//! `ir_lib_tests` full-fixture harness and the formatter fixture sweep (both of
//! which scan only the top level of their fixture directory) do not also pick
//! them up.

use crate::*;

fn run_backend(backend: &str, program: &std::path::Path) -> std::process::Output {
    lullaby()
        .args([
            "run",
            "--backend",
            backend,
            program.to_str().expect("program path"),
        ])
        .output()
        .expect("run lullaby")
}

/// A multi-parameter generic struct `Pair<K, V>` (two independent parameters,
/// both used by fields and by two-parameter inherent-impl methods) over two
/// instantiations `Pair<i64, bool>` and `Pair<string, i64>`, plus a
/// multi-parameter generic `enum Either<L, R>` matched exhaustively. `main`
/// evaluates to `55`, identical on every interpreter backend because each type
/// parameter is inferred independently and generics are erased at run time.
#[test]
pub(crate) fn multi_param_generics_run_identically_on_all_backends() {
    let fixture = workspace_root().join("tests/fixtures/valid/generics/multi_param.lby");
    let mut results = Vec::new();
    for backend in ["ast", "ir", "bytecode"] {
        let output = run_backend(backend, &fixture);
        assert!(output.status.success(), "{backend}: {output:?}");
        results.push(stdout(&output));
    }
    assert_eq!(results[0].trim(), "55", "ast");
    assert_eq!(results[1], results[0], "ir output differs from ast");
    assert_eq!(results[2], results[0], "bytecode output differs from ast");
}

/// A trait bound on a generic type: `Ranked<T: Rank>` whose inherent-impl method
/// `score` calls the bound trait's `rank` on the `T` field, over two satisfying
/// instantiations `Ranked<Card>` (struct impl) and `Ranked<Coin>` (enum impl).
/// `main` evaluates to `242`, identical on every interpreter backend because
/// bound-trait dispatch is ordinary receiver dispatch under erasure.
#[test]
pub(crate) fn bounded_generic_type_runs_identically_on_all_backends() {
    let fixture = workspace_root().join("tests/fixtures/valid/generics/bounded.lby");
    let mut results = Vec::new();
    for backend in ["ast", "ir", "bytecode"] {
        let output = run_backend(backend, &fixture);
        assert!(output.status.success(), "{backend}: {output:?}");
        results.push(stdout(&output));
    }
    assert_eq!(results[0].trim(), "242", "ast");
    assert_eq!(results[1], results[0], "ir output differs from ast");
    assert_eq!(results[2], results[0], "bytecode output differs from ast");
}

/// A function that uses a stage-5 generic type is native-ineligible
/// (monomorphization on the native backend is a later stage), so the native
/// command must *cleanly skip* it via the existing `L0339` gate rather than
/// miscompiling. Because `main` uses `Ranked<Card>`, no function is eligible and
/// the native command surfaces `L0339` as a hard error, naming the skipped
/// function — a clean diagnostic, never a produced-but-wrong executable.
#[test]
pub(crate) fn bounded_generic_cleanly_skips_native() {
    let fixture = workspace_root().join("tests/fixtures/valid/generics/bounded.lby");
    let output = lullaby()
        .args([
            "native",
            "--verbose",
            fixture.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run cli");
    assert!(
        !output.status.success(),
        "expected the L0339 no-eligible-function gate: {output:?}"
    );
    let errors = stderr(&output);
    assert!(
        errors.contains("L0339"),
        "expected the no-eligible-function skip diagnostic: {errors}"
    );
    assert!(
        errors.contains("skipped main"),
        "expected `main` to be skipped natively: {errors}"
    );
}

/// The stage-5 semantic negatives, each rejected with its dedicated diagnostic:
/// a multi-parameter type spelled with the wrong number of type arguments
/// (`L0454`), and an instantiation whose type argument does not satisfy the
/// type's declared trait bound (`L0400`).
#[test]
pub(crate) fn multi_param_and_bound_negatives_are_rejected() {
    for (fixture_name, code) in [
        ("multi_param_wrong_arity", "L0454"),
        ("unsatisfied_bound", "L0400"),
    ] {
        let fixture = workspace_root()
            .join("tests/fixtures/invalid/generics")
            .join(format!("{fixture_name}.lby"));
        let output = lullaby()
            .args(["check", fixture.to_str().expect("fixture path")])
            .output()
            .expect("check cli");
        assert!(
            !output.status.success(),
            "{fixture_name}: expected rejection, got {output:?}"
        );
        let errors = stderr(&output);
        assert!(
            errors.contains(code),
            "{fixture_name}: expected `{code}` in diagnostics: {errors}"
        );
    }
}
