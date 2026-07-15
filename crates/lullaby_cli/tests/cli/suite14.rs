//! CLI integration tests, part 14 — the stage-1 actor concurrency model
//! (`actor`/`state`/`init`/`on`, `spawn`, and fire-and-forget `tell`).
//!
//! Stage 1 delivers actors on the **AST interpreter only**. The scheduler is
//! single-threaded, cooperative, and deterministic: `spawn` constructs an actor
//! (running its `init`) and returns an `Actor<T>` handle; `tell` enqueues a
//! fire-and-forget message; every outstanding message is drained
//! run-to-completion, one at a time, before `main` returns. So a `tell` to a
//! handler with an observable side effect (e.g. `print`) produces the **same
//! output on every run**, which is what these tests pin.
//!
//! The IR and bytecode backends **reject** an actor program (`L0355`) and the
//! native/WASM backends **cleanly skip** it (`L0339`/`L0338`), so no backend can
//! silently diverge from the AST semantics. The negative tests pin the
//! declaration/`spawn`/`tell` diagnostics: a `tell` to a reply (`ask`) handler
//! or an unknown handler (`L0352`), a non-sendable message argument (`L0353`),
//! and any external access to an actor's private `state` (`L0354`).

use crate::*;

/// Run a valid fixture on the default (AST) backend and return the captured
/// output. The fixture path is relative to the workspace root.
fn run_ast(fixture: &str) -> std::process::Output {
    let path = workspace_root().join(fixture);
    lullaby()
        .args(["run", path.to_str().expect("fixture path")])
        .output()
        .expect("run cli")
}

/// Assert a fixture reports `code` and exits non-zero on `lullaby check` — the
/// static-rejection path used for the negative (diagnostic) cases.
fn assert_check_rejects(fixture: &str, code: &str) {
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
}

#[test]
fn actor_counter_is_deterministic_on_ast() {
    // `main` prints its own line first; the drained handler output follows,
    // deterministically. 0 + 5 + 10 + 2 = 17.
    let output = run_ast("tests/fixtures/valid/actors/counter_logger.lby");
    assert!(
        output.status.success(),
        "counter should run: {}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "main done\nhits = 17\n0\n");
}

#[test]
fn actor_multiple_actors_are_deterministic_on_ast() {
    // The two `forward`s each enqueue a `log` on the shared logger during the
    // drain; the direct `log` was enqueued first, so it runs first. FIFO order
    // is deterministic: direct, one, two.
    let output = run_ast("tests/fixtures/valid/actors/multiple_actors.lby");
    assert!(
        output.status.success(),
        "multiple actors should run: {}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "log: direct\nlog: one\nlog: two\n0\n");
}

#[test]
fn actor_ir_and_bytecode_reject_cleanly() {
    // The IR interpreter and bytecode VM do not support actors yet; both reject
    // an actor program with the dedicated `L0355` rather than silently diverging.
    let path = workspace_root().join("tests/fixtures/valid/actors/counter_logger.lby");
    for backend in ["ir", "bytecode"] {
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
            "[{backend}] actor program should be rejected. stderr: {stderr}"
        );
        assert!(
            stderr.contains("L0355"),
            "[{backend}] actor program should report L0355. stderr: {stderr}"
        );
    }
}

#[test]
fn actor_native_skips_cleanly() {
    // A program using actors is native-ineligible: `lullaby native` skips it with
    // `L0339` (no eligible function), never miscompiling `spawn`/`tell`.
    let path = workspace_root().join("tests/fixtures/valid/actors/counter_logger.lby");
    let output = lullaby()
        .args(["native", path.to_str().expect("fixture path")])
        .output()
        .expect("run cli");
    let stderr = stderr(&output);
    assert!(
        !output.status.success(),
        "native should skip an actor program. stderr: {stderr}"
    );
    assert!(
        stderr.contains("L0339"),
        "native skip should report L0339. stderr: {stderr}"
    );
}

#[test]
fn actor_wasm_skips_cleanly() {
    // A program using actors is not WASM-eligible: `lullaby wasm` skips it with
    // `L0338` (no eligible scalar function).
    let path = workspace_root().join("tests/fixtures/valid/actors/counter_logger.lby");
    let output = lullaby()
        .args(["wasm", path.to_str().expect("fixture path")])
        .output()
        .expect("run cli");
    let stderr = stderr(&output);
    assert!(
        !output.status.success(),
        "wasm should skip an actor program. stderr: {stderr}"
    );
    assert!(
        stderr.contains("L0338"),
        "wasm skip should report L0338. stderr: {stderr}"
    );
}

#[test]
fn actor_tell_to_reply_handler_is_rejected() {
    // `tell` may only target a fire-and-forget handler; a reply (`-> T`) handler
    // is an `ask` handler (a later stage).
    assert_check_rejects(
        "tests/fixtures/invalid/actors/tell_reply_handler.lby",
        "L0352",
    );
}

#[test]
fn actor_tell_to_unknown_handler_is_rejected() {
    assert_check_rejects("tests/fixtures/invalid/actors/unknown_handler.lby", "L0352");
}

#[test]
fn actor_non_sendable_message_arg_is_rejected() {
    // A non-atomic `rc<T>` must not cross an actor boundary.
    assert_check_rejects(
        "tests/fixtures/invalid/actors/non_sendable_arg.lby",
        "L0353",
    );
}

#[test]
fn actor_external_state_read_is_rejected() {
    // An actor's `state` is private: no external field read through the handle.
    assert_check_rejects(
        "tests/fixtures/invalid/actors/external_state_read.lby",
        "L0354",
    );
}

#[test]
fn actor_external_state_write_is_rejected() {
    // An actor's `state` is private: no external field write through the handle.
    assert_check_rejects(
        "tests/fixtures/invalid/actors/external_state_write.lby",
        "L0354",
    );
}

#[test]
fn actor_program_formats_idempotently() {
    // `lullaby fmt` renders an actor program canonically, and re-formatting the
    // output is a fixed point (the formatter round-trips the new construct).
    let path = workspace_root().join("tests/fixtures/valid/actors/counter_logger.lby");
    let first = lullaby()
        .args(["fmt", path.to_str().expect("fixture path")])
        .output()
        .expect("run fmt");
    assert!(first.status.success(), "fmt failed: {}", stderr(&first));
    let formatted = stdout(&first);

    // Write the formatted text to a temp file and format it again; the result
    // must be byte-identical (idempotent).
    let temp = std::env::temp_dir().join("lullaby_actor_fmt_idempotent.lby");
    std::fs::write(&temp, &formatted).expect("write temp");
    let second = lullaby()
        .args(["fmt", temp.to_str().expect("temp path")])
        .output()
        .expect("run fmt again");
    assert!(
        second.status.success(),
        "second fmt failed: {}",
        stderr(&second)
    );
    assert_eq!(stdout(&second), formatted, "fmt must be idempotent");
}
