//! CLI integration tests, part 13 — safe-tier failure semantics (decision A5).
//!
//! The safe-tier guarantee (see `documents/lullaby_error_handling.md` and
//! `documents/execution_tiers_and_1_0_scope.md`) splits runtime failure into two
//! disjoint families:
//!
//!   * A **contract / memory-safety violation** (index out of bounds, `pop` of an
//!     empty list, divide-by-zero) is a *bug*. It **aborts** the program with a
//!     clear `L####` diagnostic and a non-zero exit; it does **not** unwind and is
//!     **not** catchable by `try`/`catch`.
//!   * A **modeled / expected failure** flows through `result`/`?`/`throw`/`catch`
//!     and is **recoverable** — the program keeps running to a normal exit.
//!
//! These tests pin BOTH halves and, crucially, assert the same behavior on all
//! three interpreters (`ast`, `ir`, `bytecode`) so no backend can silently drift
//! (e.g. swallow an abort, or return a wrong value instead of aborting).

use crate::*;

/// The three interpreter backends `lullaby run` accepts.
const BACKENDS: [&str; 3] = ["ast", "ir", "bytecode"];

/// Run a fixture through `lullaby run --backend <backend>` and return the output.
fn run_backend(fixture: &str, backend: &str) -> std::process::Output {
    let path = workspace_root().join(fixture);
    lullaby()
        .args([
            "run",
            "--backend",
            backend,
            path.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run cli")
}

/// Assert a fixture ABORTS with `code` on every interpreter backend: non-zero
/// exit and the diagnostic code on stderr, never a success and never a different
/// code. This is the abort half of the A5 guarantee, checked for consistency.
fn assert_aborts_on_all_backends(fixture: &str, code: &str) {
    for backend in BACKENDS {
        let output = run_backend(fixture, backend);
        let stderr = stderr(&output);
        assert!(
            !output.status.success(),
            "[{backend}] {fixture} should abort, but it exited 0. stderr: {stderr}"
        );
        assert!(
            stderr.contains(code),
            "[{backend}] {fixture} should abort with {code}. stderr: {stderr}"
        );
    }
}

/// Assert a fixture COMPLETES normally (exit 0) on every interpreter backend —
/// the recoverable half of the A5 guarantee. Returns nothing; a modeled failure
/// handled by `catch`/`?` must not be turned into an abort.
fn assert_recovers_on_all_backends(fixture: &str) {
    for backend in BACKENDS {
        let output = run_backend(fixture, backend);
        assert!(
            output.status.success(),
            "[{backend}] {fixture} should recover and exit 0. stderr: {}",
            stderr(&output)
        );
    }
}

// -- Abort family: contract violations abort with a clear diagnostic ----------

#[test]
pub(crate) fn array_index_out_of_bounds_aborts_l0413_on_all_backends() {
    assert_aborts_on_all_backends(
        "tests/fixtures/invalid/array_index_out_of_bounds.lby",
        "L0413",
    );
}

#[test]
pub(crate) fn list_get_out_of_bounds_aborts_l0413_on_all_backends() {
    assert_aborts_on_all_backends("tests/fixtures/invalid/list_get_out_of_bounds.lby", "L0413");
}

#[test]
pub(crate) fn pop_empty_list_aborts_l0413_on_all_backends() {
    assert_aborts_on_all_backends("tests/fixtures/invalid/pop_empty_list.lby", "L0413");
}

#[test]
pub(crate) fn divide_by_zero_aborts_l0404_on_all_backends() {
    assert_aborts_on_all_backends("tests/fixtures/invalid/div_by_zero.lby", "L0404");
}

/// A contract violation is NOT catchable: wrapping a divide-by-zero in a
/// `try`/`catch` must still abort with the same `L0404` — only user `throw`s are
/// recoverable. This is the key "no unwinding through a safety abort" assertion.
#[test]
pub(crate) fn abort_is_not_catchable_by_try_catch_on_all_backends() {
    // A `try` body whose divisor is a runtime zero: the catch handler must NOT
    // run, and the program must abort with the div-by-zero diagnostic.
    let source = concat!(
        "fn main -> i64\n",
        "    let zero i64 = len(\"\")\n",
        "    try\n",
        "        10 / zero\n",
        "    catch message\n",
        "        999\n",
    );
    let (dir, base) = fs_temp_dir("a5_uncatchable");
    let path = format!("{base}/uncatchable.lby");
    std::fs::write(&path, source).expect("write temp source");
    for backend in BACKENDS {
        let output = lullaby()
            .args(["run", "--backend", backend, &path])
            .output()
            .expect("run cli");
        let stderr = stderr(&output);
        let stdout = stdout(&output);
        assert!(
            !output.status.success(),
            "[{backend}] a caught div-by-zero must still abort. stderr: {stderr}"
        );
        assert!(
            stderr.contains("L0404"),
            "[{backend}] expected L0404 (division by zero) to escape the catch. stderr: {stderr}"
        );
        // The catch handler's sentinel value must never be produced.
        assert!(
            !stdout.contains("999"),
            "[{backend}] the catch handler ran on a safety abort — it must not. stdout: {stdout}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// An UNCAUGHT `throw` aborts with `L0420` on every backend — the boundary of the
/// recoverable model: `throw` is catchable, but if nothing catches it the program
/// terminates with a clear diagnostic rather than continuing.
#[test]
pub(crate) fn uncaught_throw_aborts_l0420_on_all_backends() {
    assert_aborts_on_all_backends("tests/fixtures/invalid/uncaught_throw.lby", "L0420");
}

// -- Recoverable family: modeled failures keep running -------------------------

/// A caught `throw` and a `?`-propagated `none` both let the program run to a
/// clean exit — proving contract-violation aborts and modeled failures are truly
/// distinct paths. Lullaby has no forced "unwrap" that panics on `none`.
#[test]
pub(crate) fn recoverable_throw_and_question_mark_complete_on_all_backends() {
    assert_recovers_on_all_backends("tests/fixtures/valid/run_recoverable_not_abort.lby");
}

/// The existing `?`-propagation fixture (result + option, success and failure
/// paths folded via `match`) stays a clean exit on every backend — a `none`/`err`
/// flowing through `?` is recovered, never an abort.
#[test]
pub(crate) fn error_propagation_stays_recoverable_on_all_backends() {
    assert_recovers_on_all_backends("tests/fixtures/valid/run_error_propagation.lby");
}
