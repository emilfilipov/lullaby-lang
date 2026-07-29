//! CLI integration tests, part 29 — the two expression positions a pass can
//! silently skip: an **assignment-target index** and an **alias-annotated
//! expression**.
//!
//! Both are the same shape of defect: a walker that names only the positions its
//! author happened to think of, and quietly does nothing everywhere else.
//!
//! * **Loader visibility (`L0392`).** `Stmt::Assign`'s `path` carries a full
//!   expression tree, so `a[secret()] = 99` puts a call inside the assignment
//!   *target*. The loader's module-reference collector destructured
//!   `Stmt::Assign { value, .. }` and dropped `path`, so a call to a **private**
//!   function in another package hid there and evaded the cross-package
//!   visibility check entirely: `check` exited **0** and the private function
//!   really ran. The hoisted control (`let i = secret()` then `a[i] = 99`) was
//!   always rejected, which is why the gap survived — proving the ordinary
//!   position still worked proved nothing about this one. Both forms are pinned
//!   below, against fixtures that differ by exactly that hoist.
//!
//! * **Alias resolution positions.** The alias pass never entered expressions, so
//!   a closure parameter spelled with an alias was falsely rejected, and a
//!   `match` was reached only in bare-statement position — the identical arm
//!   bodies were accepted as a statement and rejected (`L0303`) as a `let` RHS,
//!   an assignment RHS, a `return` operand, or when nested. The fixture here
//!   exercises every position both annotation carriers can occupy and must
//!   evaluate identically on all three interpreters.
//!
//! The semantic halves live in `crates/lullaby_semantics/src/`
//! (`semantics_assign_path_tests.rs`, `semantics_alias_position_tests.rs`); the
//! `asm`-operand sibling of the visibility defect is in `suite27.rs`.

use crate::*;

/// Run `lullaby check` on a project directory and return the captured output.
fn check_project(relative: &str) -> std::process::Output {
    let project = workspace_root().join(relative);
    lullaby()
        .args(["check", project.to_str().expect("project path")])
        .output()
        .expect("run cli")
}

/// VISIBILITY THROUGH AN ASSIGNMENT-TARGET INDEX: `a[hidden_helper(0)] = 99`,
/// where `hidden_helper` is private to another package. This is the defect — it
/// passed `check` with exit 0 and executed the private function.
#[test]
pub(crate) fn rejects_cross_package_private_use_from_an_assign_index_with_l0392() {
    let output = check_project("tests/fixtures/invalid/project_private_cross_index/app");
    assert!(
        !output.status.success(),
        "a private cross-package call inside an assignment-target index must be REJECTED: {output:?}"
    );
    let stderr = stderr(&output);
    assert!(stderr.contains("L0392 [loader error]"), "{stderr}");
    assert!(stderr.contains("hidden_helper"), "{stderr}");
}

/// The hoisted control: the identical call moved out of the index into a `let`.
/// It was always rejected and must stay rejected, so a regression in the index
/// case cannot be masked by proving only that ordinary position works.
#[test]
pub(crate) fn rejects_hoisted_cross_package_private_use_control_with_l0392() {
    let output = check_project("tests/fixtures/invalid/project_private_cross_index/app_hoisted");
    assert!(
        !output.status.success(),
        "the hoisted control must still be REJECTED: {output:?}"
    );
    let stderr = stderr(&output);
    assert!(stderr.contains("L0392 [loader error]"), "{stderr}");
    assert!(stderr.contains("hidden_helper"), "{stderr}");
}

/// ALIAS POSITIONS, end to end: a closure parameter spelled with an alias (in
/// call-argument, loop-body, and assignment-target-index positions) plus a
/// `match` whose arm bodies bind alias-typed locals, in every position a `match`
/// can occupy — bare statement, `let` RHS, assignment RHS, `return` operand, and
/// nested in another arm.
///
/// The whole-directory harness in `crates/lullaby_ir/src/ir_lib_tests.rs` also
/// runs this fixture across all five backend variants, but it *skips* any fixture
/// that fails to validate, so a regression there would be silent. Naming the
/// fixture and its exact output here is what gives it teeth.
#[test]
pub(crate) fn alias_positions_run_identically_on_all_interpreters() {
    let path = workspace_root().join("tests/fixtures/valid/run_alias_positions.lby");
    let expected = "12\n13\n7\n2\n100\n2000\n20\n2\n12\n2168\n";
    let mut results = Vec::new();
    for backend in ["ast", "ir", "bytecode"] {
        let output = lullaby()
            .args([
                "run",
                "--backend",
                backend,
                path.to_str().expect("program path"),
            ])
            .output()
            .expect("run lullaby");
        assert!(output.status.success(), "{backend}: {output:?}");
        results.push(stdout(&output));
    }
    assert_eq!(results[0].replace("\r\n", "\n"), expected, "ast result");
    assert_eq!(results[1], results[0], "ir output differs from ast");
    assert_eq!(results[2], results[0], "bytecode output differs from ast");
}
