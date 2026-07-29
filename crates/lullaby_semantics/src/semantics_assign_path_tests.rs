//! Semantic tests for the **assignment-path** expression class.
//!
//! `Stmt::Assign`'s `path` is a child-bearing field: a `Place::Index` step holds
//! a full `Expr` tree, so `a[f(x)] = v` puts an arbitrary expression inside the
//! assignment *target*. A pass that destructures `Stmt::Assign { name, value,
//! .. }` and drops `path` never sees it — the same class of blind spot as an
//! empty `Stmt::Asm { .. } => {}` arm, and it produced two real defects:
//!
//! - the lifetime walk missed `dealloc(p)` then `a[ptr_read(p)] = 99`, so
//!   `L0350` never fired even though the hoisted `let i = ptr_read(p)` form was
//!   correctly rejected; and
//! - the const-sized-array pass neither validated nor expanded a fill literal in
//!   an index, so `a[len([0; n])] = 99` passed `check` and then ran only under
//!   the AST interpreter (every other tier rejected the un-expanded `ArrayFill`),
//!   while a legal `array<T, N>` spelling in an index stayed un-erased and was
//!   falsely rejected by the type checker.
//!
//! These tests pin both gates to the assignment path specifically, and keep the
//! ordinary-position controls beside them so a future regression cannot pass by
//! only proving the ordinary case still works. The shared accessor the fix
//! introduced is `lullaby_parser::assign_path_exprs` (see `Place::index_expr`).

use lullaby_lexer::lex;
use lullaby_parser::{ExprKind, Place, Stmt, parse};

use super::*;

fn diags(source: &str) -> Vec<SemanticDiagnostic> {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    validate(&program).err().unwrap_or_default()
}

fn has(source: &str, code: &str) -> bool {
    diags(source).iter().any(|d| d.code == code)
}

/// The post-pass program, for asserting on what the semantic passes rewrote.
fn checked(source: &str) -> CheckedProgram {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    validate(&program).expect("expected a clean program")
}

// -- L0350: use-after-free through an assignment-target index -----------------

/// The defect: a freed pointer read inside the assignment *target* is a
/// use-after-free exactly as it is anywhere else.
#[test]
fn use_after_free_in_assign_path_index_is_l0350() {
    let source = concat!(
        "fn main -> i64\n",
        "    let p ptr_i64 = alloc(1)\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    dealloc(p)\n",
        "    unsafe\n",
        "        a[ptr_read(p)] = 99\n",
        "    a[0]\n",
    );
    assert!(
        has(source, "L0350"),
        "a freed pointer read in an assignment-target index must be L0350, got {:?}",
        diags(source)
    );
}

/// The control that was already passing when the defect above was open: hoisting
/// the same read into a `let` moves it out of the path and into ordinary
/// position. It must keep firing, so a regression cannot be masked by this case.
#[test]
fn hoisted_use_after_free_control_is_still_l0350() {
    let source = concat!(
        "fn main -> i64\n",
        "    let p ptr_i64 = alloc(1)\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    dealloc(p)\n",
        "    unsafe\n",
        "        let i i64 = ptr_read(p)\n",
        "        a[i] = 99\n",
        "    a[0]\n",
    );
    assert!(
        has(source, "L0350"),
        "the hoisted-variable control must still be L0350, got {:?}",
        diags(source)
    );
}

/// No false positive: reading a still-live pointer in an assignment-target index
/// is fine. Walking the path must not turn every indexed store into a report.
#[test]
fn live_pointer_in_assign_path_index_is_clean() {
    let source = concat!(
        "fn main -> i64\n",
        "    let p ptr_i64 = alloc(1)\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    unsafe\n",
        "        a[ptr_read(p)] = 99\n",
        "    dealloc(p)\n",
        "    a[0]\n",
    );
    let d = diags(source);
    assert!(d.is_empty(), "expected no diagnostics, got {d:?}");
}

/// A double free spelled through an assignment-target index is also caught: the
/// path is walked for freeing calls' arguments, not only for plain reads.
#[test]
fn use_after_free_through_a_field_and_index_path_is_l0350() {
    let source = concat!(
        "struct Grid\n",
        "    cells array<i64>\n\n",
        "fn main -> i64\n",
        "    let p ptr_i64 = alloc(1)\n",
        "    let g Grid = Grid(cells: [10, 20, 30])\n",
        "    dealloc(p)\n",
        "    unsafe\n",
        "        g.cells[ptr_read(p)] = 99\n",
        "    g.cells[0]\n",
    );
    assert!(
        has(source, "L0350"),
        "a freed read in a `.field[index]` path must be L0350, got {:?}",
        diags(source)
    );
}

// -- L0463 / erasure: const-sized arrays in an assignment-target index --------

/// The defect: a non-constant fill count in an assignment-target index was
/// neither rejected nor expanded, so `check` passed and the tiers diverged.
#[test]
fn non_constant_fill_count_in_assign_path_index_is_l0463() {
    let source = concat!(
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    let n = 2\n",
        "    a[len([0; n])] = 99\n",
        "    a[2]\n",
    );
    assert!(
        has(source, "L0463"),
        "a non-constant fill count in an assignment-target index must be L0463, got {:?}",
        diags(source)
    );
}

/// The ordinary-position control: the very same fill in a `let` initializer was
/// always rejected. It must keep firing.
#[test]
fn non_constant_fill_count_control_in_let_is_l0463() {
    let source = concat!(
        "fn main -> i64\n",
        "    let n = 2\n",
        "    let b array<i64> = [0; n]\n",
        "    len(b)\n",
    );
    assert!(
        has(source, "L0463"),
        "the ordinary-position control must still be L0463, got {:?}",
        diags(source)
    );
}

/// The latent half of the same defect: a *constant*-count fill in an
/// assignment-target index agreed across the tiers but survived the pass
/// **un-expanded**, leaving an `ArrayFill` node in a program every backend is
/// promised never sees one. Assert on the post-pass AST that it is gone and that
/// the expansion produced the right element count.
#[test]
fn constant_fill_in_assign_path_index_is_expanded() {
    let source = concat!(
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    a[len([0; 2])] = 99\n",
        "    a[2]\n",
    );
    let program = checked(source).program;
    let main = program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main");
    let Stmt::Assign { path, .. } = &main.body[1] else {
        panic!("expected the indexed assignment, got {:?}", main.body[1]);
    };
    let [Place::Index(index)] = &path[..] else {
        panic!("expected a single index step, got {path:?}");
    };
    let ExprKind::Call { name, args } = &index.kind else {
        panic!("expected the `len(...)` call, got {:?}", index.kind);
    };
    assert_eq!(name, "len");
    match &args[0].kind {
        ExprKind::Array(items) => assert_eq!(
            items.len(),
            2,
            "the fill must expand to exactly 2 elements, got {items:?}"
        ),
        ExprKind::ArrayFill { .. } => {
            panic!("the fill literal survived the pass un-expanded in the assignment path")
        }
        other => panic!("expected an expanded array literal, got {other:?}"),
    }
}

/// A legal `array<T, N>` spelling inside an assignment-target index (a closure
/// parameter annotation) must be resolved and erased like any other. While the
/// path was dropped it stayed un-erased and the checker rejected the program
/// with `L0313` — a false rejection, not a missing diagnostic.
#[test]
fn fixed_extent_spelling_in_assign_path_index_is_erased() {
    let source = concat!(
        "fn apply f fn(array<i64>) -> i64 v array<i64> -> i64\n",
        "    f(v)\n\n",
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    a[apply(fn x array<i64, 2> -> 1, [1, 2])] = 99\n",
        "    a[1]\n",
    );
    let d = diags(source);
    assert!(d.is_empty(), "expected no diagnostics, got {d:?}");
}

/// ...and an illegal one there is still rejected: a non-constant extent in an
/// assignment-target index is `L0463`, exactly as in ordinary position.
#[test]
fn non_constant_extent_in_assign_path_index_is_l0463() {
    let source = concat!(
        "fn apply f fn(array<i64>) -> i64 v array<i64> -> i64\n",
        "    f(v)\n\n",
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    let n = 2\n",
        "    a[apply(fn x array<i64, n> -> 1, [1, 2])] = 99\n",
        "    a[1]\n",
    );
    assert!(
        has(source, "L0463"),
        "a non-constant extent in an assignment-target index must be L0463, got {:?}",
        diags(source)
    );
}
