//! CLI integration tests, part 6 (`match` as an expression, with block arm
//! bodies). Split out of tests/cli.rs so it does not overlap the native / fuzz /
//! socket / stdin suites, and kept in a `match_expr/` fixture subdirectory so the
//! `ir_lib_tests.rs` whole-`valid`-directory backend-parity harness does not also
//! pick these up (that harness runs every backend variant against the flat
//! fixture set; these programs are deliberately exercised only on the three
//! interpreters here). Each test runs a pure, deterministic `.lby` program that
//! uses `match` in value position — a `let`/assignment RHS, a `return`, a nested
//! arm value, or a multi-statement block arm — and asserts the captured stdout is
//! byte-for-byte identical on every interpreter backend (`ast`, `ir`,
//! `bytecode`). This pins that a value-position `match` (and block arm bodies)
//! evaluates identically across the three tiers, including the arm-body outer
//! mutation that the AST interpreter must run against the real environment.

use crate::*;

/// Run `lullaby run --backend <backend> <program>` and return the captured
/// output. The match-expression fixtures are pure (no stdin, no I/O), so a plain
/// `Command::output` suffices.
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

/// Assert every interpreter backend runs `fixture` successfully and prints
/// exactly `expected`.
fn assert_all_backends(fixture: &str, expected: &str) {
    let path = workspace_root().join(format!("tests/fixtures/valid/match_expr/{fixture}"));
    for backend in ["ast", "ir", "bytecode"] {
        let output = run_backend(backend, &path);
        assert!(output.status.success(), "{backend}: {output:?}");
        assert_eq!(stdout(&output), expected, "{backend}");
    }
}

/// `match` in `let`-binding position, yielding `i64`. The binding annotation
/// flows in as the expected arm-result type; `classify(4)` -> 104,
/// `classify(3)` -> 203, and `main` returns their sum (307), printed last.
#[test]
pub(crate) fn match_expression_in_let_position_on_all_backends() {
    assert_all_backends("let_position.lby", "104\n203\n307\n");
}

/// Block (multi-statement) arm bodies through a `return match ...`: an arm runs
/// its statements (including a `print` side effect and a payload binding) then
/// yields its final expression as the arm's value. `main` returns
/// `len("empty")` (5), printed last.
#[test]
pub(crate) fn match_expression_block_arm_bodies_on_all_backends() {
    assert_all_backends(
        "block_arms.lby",
        "saw circle\ncircle:9\nsaw rect\nrect:10\nempty\n5\n",
    );
}

/// A `match` expression nested as another `match` expression's arm value, with a
/// user-enum result type. `pick(4)` -> Red (1), `pick(14)` -> Green (2),
/// `pick(3)` -> Blue (3); `main` returns `1*100 + 2*10 + 3` (123), printed last.
#[test]
pub(crate) fn nested_match_expression_enum_result_on_all_backends() {
    assert_all_backends("nested_enum_result.lby", "1\n2\n3\n123\n");
}

/// A block arm body that mutates an enclosing-scope binding, plus an arm-internal
/// `return` that diverts the whole function. `f(A)` -> 51 (the `total` mutation
/// survives), `f(B)` -> 999 (early return), `f(C)` -> 3; `main` returns their sum
/// (1053), printed last. This is the case where the AST interpreter must evaluate
/// the value-position `match` against the real environment to match the IR /
/// bytecode desugaring.
#[test]
pub(crate) fn match_expression_arm_outer_mutation_on_all_backends() {
    assert_all_backends("outer_mutation.lby", "51\n999\n3\n1053\n");
}
