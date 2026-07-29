//! Semantic tests for **which positions alias resolution reaches**.
//!
//! `resolve_program_aliases` rewrites a program so nothing downstream ever sees a
//! user `alias`. Its statement walker used to rebuild a handful of `Stmt` variants
//! and fall through to `other => other.clone()` for everything else, which made it
//! blind in two ways at once:
//!
//! - it **never entered expressions**, so a closure parameter spelled with an
//!   alias (`list_map(base, fn x Num -> x + x)`) kept the unresolved spelling and
//!   the checker falsely **rejected a valid program** — while the identical
//!   closure spelled `fn x i64` compiled; and
//! - it reached a `match` only through a hand-written
//!   `Stmt::Expr(ExprKind::Match { .. })` special case, so identical arm bodies
//!   containing `let x Num = v` were **accepted** as a bare statement and
//!   **rejected** (`L0303 binding 'x' declares 'Num' but initializer has 'i64'`)
//!   as a `let` RHS, an assignment RHS, a `return` operand, or when nested.
//!
//! Both are the same root cause, so the fix is one traversal that is exhaustive by
//! construction — every `Stmt` variant and every `ExprKind` named, no catch-all —
//! rather than two more special cases. These tests pin each position the two
//! annotation carriers can occupy. A closure literal can appear in **any**
//! expression position; a `match` is a value-position form only — the grammar
//! admits it as a bare statement, and through `parse_value_expr`'s four callers as
//! a `let` RHS, an assignment RHS, a `return` operand and a `const` initializer,
//! plus `parse_arm_body`'s inline nested form — so those are all of its positions.
//!
//! What this pass does **not** fix is which type *spellings* `resolve_alias_type`
//! understands: it descends into `array`/`ptr`/`ref`/`rc` only, so `list<Num>`,
//! `map<K, Num>`, `option<Num>` and `fn(Num) -> Num` still leave the alias
//! unresolved and are falsely rejected. That is a separate open gap — orthogonal
//! to position — and it is why the sources below spell their collections
//! `list<i64>` rather than `list<Num>`.
//!
//! The negative controls at the end matter as much as the positives: widening a
//! rewrite is exactly the kind of change that can turn a gate into a rubber stamp,
//! so an unknown type name, a genuine type mismatch, and the alias-definition
//! gates must all still fire.
//!
//! Cross-tier agreement for the same program is pinned by
//! `tests/fixtures/valid/run_alias_positions.lby`.

use lullaby_lexer::lex;
use lullaby_parser::{ExprKind, Place, Stmt, TypeRef, parse};

use super::*;

fn diags(source: &str) -> Vec<SemanticDiagnostic> {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    validate(&program).err().unwrap_or_default()
}

fn has(source: &str, code: &str) -> bool {
    diags(source).iter().any(|d| d.code == code)
}

/// Assert `source` type-checks cleanly, reporting what came back if it does not.
fn assert_clean(source: &str) {
    let d = diags(source);
    assert!(d.is_empty(), "expected no diagnostics, got {d:?}");
}

/// The post-pass program, for asserting on what alias resolution actually
/// rewrote — not merely that the program was accepted.
fn checked(source: &str) -> CheckedProgram {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    validate(&program).expect("expected a clean program")
}

fn main_body(program: &Program) -> &[Stmt] {
    &program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main")
        .body
}

/// A `list<i64>` built by `push`, as a source prefix. `[1, 2]` would be an
/// `array<i64>`, which `list_map` does not take.
const LIST_PREFIX: &str = concat!(
    "    let base list<i64> = list_new()\n",
    "    base = push(base, 1)\n",
    "    base = push(base, 2)\n",
);

// -- Defect 2: an alias in a closure parameter --------------------------------

/// The defect: a closure parameter spelled with an alias was never resolved, so
/// the checker compared `Num` against the element type `i64` and rejected a valid
/// program. This is a **false rejection**, not a missing diagnostic.
#[test]
fn closure_parameter_alias_is_resolved() {
    let source = format!(
        concat!(
            "alias Num = i64\n\n",
            "fn main -> i64\n",
            "{}",
            "    let doubled list<i64> = list_map(base, fn x Num -> x + x)\n",
            "    list_reduce(doubled, 0, fn acc i64 x i64 -> acc + x)\n",
        ),
        LIST_PREFIX
    );
    assert_clean(&source);
}

/// The control that was already passing when the defect above was open: the same
/// closure with the alias spelled out. It must keep compiling, so a regression
/// cannot be masked by this case.
#[test]
fn closure_parameter_control_spelled_directly_is_clean() {
    let source = format!(
        concat!(
            "fn main -> i64\n",
            "{}",
            "    let doubled list<i64> = list_map(base, fn x i64 -> x + x)\n",
            "    list_reduce(doubled, 0, fn acc i64 x i64 -> acc + x)\n",
        ),
        LIST_PREFIX
    );
    assert_clean(&source);
}

/// Acceptance is not proof of resolution — a checker that merely tolerated `Num`
/// would also pass. Assert on the post-pass AST that the closure parameter now
/// spells the canonical type, so no backend can meet an alias.
#[test]
fn closure_parameter_alias_is_erased_from_the_checked_program() {
    let source = format!(
        concat!(
            "alias Num = i64\n\n",
            "fn main -> i64\n",
            "{}",
            "    let doubled list<i64> = list_map(base, fn x Num -> x + x)\n",
            "    list_reduce(doubled, 0, fn acc i64 x i64 -> acc + x)\n",
        ),
        LIST_PREFIX
    );
    let program = checked(&source).program;
    let Stmt::Let { value, .. } = &main_body(&program)[3] else {
        panic!("expected the `doubled` binding");
    };
    let ExprKind::Call { args, .. } = &value.kind else {
        panic!("expected the `list_map` call, got {:?}", value.kind);
    };
    let ExprKind::Closure { params, .. } = &args[1].kind else {
        panic!("expected the closure argument, got {:?}", args[1].kind);
    };
    assert_eq!(
        params[0].ty,
        TypeRef::new("i64"),
        "the closure parameter kept an unresolved alias spelling"
    );
}

/// A closure parameter alias inside a **nested block** (a loop body) resolves
/// too: the walk must descend through statement bodies into their expressions,
/// not stop at the first block boundary.
#[test]
fn closure_parameter_alias_in_a_nested_block_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "fn twice f fn(i64) -> i64 v i64 -> i64\n",
        "    f(f(v))\n\n",
        "fn main -> i64\n",
        "    let total i64 = 0\n",
        "    for i from 0 to 2\n",
        "        if i > 0\n",
        "            total = total + twice(fn x Num -> x + 1, i)\n",
        "    total\n",
    );
    assert_clean(source);
}

/// A closure parameter alias inside an **assignment-target index** resolves: the
/// index is an ordinary expression carried by `Stmt::Assign`'s `path`, the field
/// this walker used to drop entirely. See `Place::index_expr`.
#[test]
fn closure_parameter_alias_in_assign_path_index_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "fn apply f fn(i64) -> i64 v i64 -> i64\n",
        "    f(v)\n\n",
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    a[apply(fn x Num -> x + 1, 0)] = 99\n",
        "    a[1]\n",
    );
    assert_clean(source);
}

/// ...and it is really erased there, not merely tolerated.
#[test]
fn closure_parameter_alias_in_assign_path_index_is_erased() {
    let source = concat!(
        "alias Num = i64\n\n",
        "fn apply f fn(i64) -> i64 v i64 -> i64\n",
        "    f(v)\n\n",
        "fn main -> i64\n",
        "    let a array<i64> = [10, 20, 30]\n",
        "    a[apply(fn x Num -> x + 1, 0)] = 99\n",
        "    a[1]\n",
    );
    let program = checked(source).program;
    let Stmt::Assign { path, .. } = &main_body(&program)[1] else {
        panic!("expected the indexed assignment");
    };
    let [Place::Index(index)] = &path[..] else {
        panic!("expected a single index step, got {path:?}");
    };
    let ExprKind::Call { args, .. } = &index.kind else {
        panic!("expected the `apply` call, got {:?}", index.kind);
    };
    let ExprKind::Closure { params, .. } = &args[0].kind else {
        panic!("expected the closure argument, got {:?}", args[0].kind);
    };
    assert_eq!(
        params[0].ty,
        TypeRef::new("i64"),
        "the alias survived in an assignment-target index"
    );
}

/// A closure parameter alias inside an inline-`asm` **operand clause** resolves.
/// An operand is an ordinary expression, so it must face the same rewrite as any
/// other — see `AsmOperand::expr`.
#[test]
fn closure_parameter_alias_in_an_asm_operand_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "fn apply f fn(i64) -> i64 v i64 -> i64\n",
        "    f(v)\n\n",
        "fn main -> i64\n",
        "    let out_val i64 = 0\n",
        "    unsafe\n",
        "        asm 144\n",
        "            in rcx = apply(fn x Num -> x + 1, 0)\n",
        "            out rax = out_val\n",
        "    out_val\n",
    );
    assert_clean(source);
}

// -- Defect 3: an alias in a `match` arm body, in every position a `match` has -

/// The control that always worked: a bare-statement `match` whose arm bodies
/// declare alias-typed locals. It was reached by the old hand-written
/// `Stmt::Expr(Match)` special case, and must keep working.
#[test]
fn match_arm_alias_in_statement_position_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let seen i64 = 0\n",
        "    match Odd\n",
        "        Even ->\n            let a Num = 3\n            seen = a\n",
        "        Odd ->\n            let b Num = 7\n            seen = b\n",
        "    seen\n",
    );
    assert_clean(source);
}

/// The defect: the identical arm bodies as a `let` RHS were rejected `L0303`,
/// because the special case matched only `Stmt::Expr(Match)`.
#[test]
fn match_arm_alias_in_let_rhs_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let r i64 = match Odd\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd ->\n            let b Num = 7\n            b\n",
        "    r\n",
    );
    assert_clean(source);
}

/// The same arm bodies as an **assignment RHS**.
#[test]
fn match_arm_alias_in_assignment_rhs_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let r i64 = 0\n",
        "    r = match Odd\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd ->\n            let b Num = 7\n            b\n",
        "    r\n",
    );
    assert_clean(source);
}

/// The same arm bodies as a **`return` operand**.
#[test]
fn match_arm_alias_in_return_position_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn pick p P -> i64\n",
        "    return match p\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd ->\n            let b Num = 7\n            b\n\n",
        "fn main -> i64\n",
        "    pick(Odd)\n",
    );
    assert_clean(source);
}

/// The same arm bodies **nested** inside another `match` arm — the position
/// reached through `parse_arm_body`'s inline-`match` form, two expression levels
/// below the enclosing statement.
#[test]
fn match_arm_alias_nested_in_a_match_arm_is_resolved() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let r i64 = match Odd\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd -> match Even\n",
        "            Even ->\n                let b Num = 7\n                b\n",
        "            Odd ->\n                let c Num = 9\n                c\n",
        "    r\n",
    );
    assert_clean(source);
}

/// And the arm-body annotation is really erased in the `let`-RHS position, not
/// merely tolerated: the arm's inner `let` must carry the canonical type.
#[test]
fn match_arm_alias_in_let_rhs_is_erased() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let r i64 = match Odd\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd ->\n            let b Num = 7\n            b\n",
        "    r\n",
    );
    let program = checked(source).program;
    let Stmt::Let { value, .. } = &main_body(&program)[0] else {
        panic!("expected the `r` binding");
    };
    let ExprKind::Match { arms, .. } = &value.kind else {
        panic!("expected a match RHS, got {:?}", value.kind);
    };
    let Stmt::Let { ty, .. } = &arms[0].body[0] else {
        panic!("expected the arm's inner `let`, got {:?}", arms[0].body[0]);
    };
    assert_eq!(
        ty.as_ref(),
        Some(&TypeRef::new("i64")),
        "the alias survived inside a `let`-RHS match arm body"
    );
}

// -- Negative controls: widening the rewrite must not disarm any gate ----------

/// An **unknown** type name in a closure parameter is still rejected. Resolving
/// alias spellings must not turn the pass into a rubber stamp that accepts any
/// identifier in a type position.
#[test]
fn unknown_type_in_a_closure_parameter_is_still_rejected() {
    let source = format!(
        concat!(
            "fn main -> i64\n",
            "{}",
            "    let doubled list<i64> = list_map(base, fn x Nope -> x + x)\n",
            "    list_reduce(doubled, 0, fn acc i64 x i64 -> acc + x)\n",
        ),
        LIST_PREFIX
    );
    assert!(
        !diags(&source).is_empty(),
        "an unknown closure parameter type must still be rejected"
    );
}

/// A **genuine** mismatch inside a `let`-RHS match arm is still `L0303`. The
/// alias-position fix removed a false rejection; it must not remove the real one
/// that shares the diagnostic.
///
/// NOTE on what this does and does not prove: it is a rubber-stamp control, not a
/// resolver-insensitive one. Neutering `resolve_alias_type` leaves this test green
/// — but only because the source's other arm (`let a Num = 3`) then emits `L0303`
/// too, and the assertion is `has(source, "L0303")`, which cannot tell the two
/// apart. It still catches the failure mode it exists for (a rewrite so wide that
/// `let b bool = 7` stops being an error); do not read its survival in the
/// resolver-neutered run as evidence of independence.
#[test]
fn genuine_mismatch_in_a_let_rhs_match_arm_is_still_l0303() {
    let source = concat!(
        "alias Num = i64\n\n",
        "enum P\n    Even\n    Odd\n\n",
        "fn main -> i64\n",
        "    let r i64 = match Odd\n",
        "        Even ->\n            let a Num = 3\n            a\n",
        "        Odd ->\n            let b bool = 7\n            0\n",
        "    r\n",
    );
    assert!(
        has(source, "L0303"),
        "a real arm-body type mismatch must still be L0303, got {:?}",
        diags(source)
    );
}

/// The alias-definition gates are unaffected: a duplicate alias is still `L0360`
/// and a cyclic one still `L0361`, now that the rewrite reaches further.
#[test]
fn alias_definition_gates_still_fire() {
    let duplicate = concat!(
        "alias A = i64\n",
        "alias A = bool\n\n",
        "fn main -> i64\n    0\n",
    );
    assert!(
        has(duplicate, "L0360"),
        "duplicate alias must still be L0360, got {:?}",
        diags(duplicate)
    );

    let cyclic = concat!(
        "alias A = B\n",
        "alias B = A\n\n",
        "fn main -> i64\n    0\n"
    );
    assert!(
        has(cyclic, "L0361"),
        "cyclic alias must still be L0361, got {:?}",
        diags(cyclic)
    );
}
