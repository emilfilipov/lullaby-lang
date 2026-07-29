//! IR-to-IR optimization passes (inlining, constant folding, common
//! subexpression elimination, loop-invariant code motion, copy propagation, dead
//! code elimination). Each pass transforms an `IrModule` and is driven, in order,
//! by the crate's `optimize` fn (in `lib.rs`).
//!
//! Each pass lives in its own submodule (`ir_optimizer_<pass>.rs`); this file is
//! the parent that wires them together and holds the few items shared across
//! passes: the `ExprSignature` type and `combine_signatures` helper (used by both
//! CSE and LICM) and the `expr_requires_optimizer_barrier` predicate (used by
//! both CSE and copy propagation). Uses the crate's IR types via `use super::*`.

use super::*;

#[path = "ir_optimizer_constfold.rs"]
mod ir_optimizer_constfold;
#[path = "ir_optimizer_copyprop.rs"]
mod ir_optimizer_copyprop;
#[path = "ir_optimizer_cse.rs"]
mod ir_optimizer_cse;
#[path = "ir_optimizer_dce.rs"]
mod ir_optimizer_dce;
#[path = "ir_optimizer_inline.rs"]
mod ir_optimizer_inline;
#[path = "ir_optimizer_licm.rs"]
mod ir_optimizer_licm;

pub(crate) use ir_optimizer_constfold::ConstantFolder;
pub(crate) use ir_optimizer_copyprop::CopyPropagator;
pub(crate) use ir_optimizer_cse::CommonSubexpressionEliminator;
pub(crate) use ir_optimizer_dce::DeadCodeEliminator;
pub(crate) use ir_optimizer_inline::Inliner;
pub(crate) use ir_optimizer_licm::LoopInvariantMover;

/// The structural fingerprint of a pure expression: a canonical `key` string
/// (equal keys denote structurally identical pure expressions) plus the set of
/// variable names the expression depends on (`dependencies`). Shared by CSE
/// (which reuses an available binding when keys match) and LICM (which hoists a
/// binding when none of its dependencies are declared or mutated in the loop).
#[derive(Debug, Clone)]
struct ExprSignature {
    key: String,
    dependencies: HashSet<String>,
}

fn combine_signatures(
    prefix: &str,
    ty: &str,
    signatures: Vec<ExprSignature>,
) -> (String, HashSet<String>) {
    let mut dependencies = HashSet::new();
    let mut parts = Vec::new();
    for signature in signatures {
        dependencies.extend(signature.dependencies);
        parts.push(signature.key);
    }
    (format!("{prefix}:{ty}({})", parts.join(",")), dependencies)
}

/// Every name bound **locally** inside `function`: its parameters plus every
/// binding the body introduces (`let`, a `for` loop variable, a `catch` name, a
/// `match` arm's payload bindings), at any nesting depth.
///
/// # A call whose callee name is in this set is NOT the module function
///
/// Lullaby lets a `fn`-typed local or parameter shadow a module function, and a
/// call then targets the local value:
///
/// ```text
/// fn inner v i64 -> i64
///     v * 10
///
/// fn main -> i64
///     let n i64 = 2
///     let inner fn(i64) -> i64 = fn x i64 -> x + n
///     inner(40)          # 42 — the closure, NOT the module `inner`
/// ```
///
/// The inliner used to look `inner` up in its module-function table with no
/// scope check and substitute `40 * 10`, so `--optimize full` answered **400**
/// while `none`/`constant-fold`/`dead-code` on every tier answered **42** —
/// and `lullaby native`, which runs the `full` pipeline, shipped the wrong
/// answer in a binary. **Any pass that maps a call name to a module function
/// must consult this set first.**
///
/// The set is deliberately function-wide rather than scope-precise. Declining to
/// resolve a name is always semantics-preserving, so over-approximating costs at
/// most a missed inline, while a lexical scope walk is exactly the kind of
/// reasoning that produced the bug. LICM uses the same set for a second purpose:
/// reserving names so a hoisted temp cannot collide with a user binding.
fn collect_function_binding_names(function: &IrFunction) -> HashSet<String> {
    let mut names = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    collect_declared_names(&function.body, &mut names);
    names
}

/// Every name declared by `statements` and the blocks nested in them. The
/// recursive worker behind [`collect_function_binding_names`]; LICM also calls it
/// directly for a loop body, to know which bindings a candidate expression's
/// dependencies were (re)declared inside the loop.
fn collect_declared_names(statements: &[IrStmt], names: &mut HashSet<String>) {
    for statement in statements {
        match statement {
            IrStmt::Let { name, .. } => {
                names.insert(name.clone());
            }
            IrStmt::If {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_declared_names(&branch.body, names);
                }
                collect_declared_names(else_body, names);
            }
            IrStmt::While { body, .. }
            | IrStmt::Loop { body, .. }
            | IrStmt::RegionBlock { body, .. } => {
                collect_declared_names(body, names);
            }
            IrStmt::Try {
                body,
                catch_name,
                catch_body,
                ..
            } => {
                names.insert(catch_name.clone());
                collect_declared_names(body, names);
                collect_declared_names(catch_body, names);
            }
            IrStmt::Match { arms, .. } => {
                for arm in arms {
                    if let IrMatchPattern::Variant { bindings, .. } = &arm.pattern {
                        for binding in bindings {
                            names.insert(binding.clone());
                        }
                    }
                    collect_declared_names(&arm.body, names);
                }
            }
            IrStmt::For { name, body, .. } => {
                names.insert(name.clone());
                collect_declared_names(body, names);
            }
            // DELIBERATELY does not descend into `asm` operands, or into an
            // assignment's target path: this collects names DECLARED here, and
            // neither an operand clause nor an `[i]` index expression can
            // introduce a binding — both only read or write bindings that already
            // exist. (Writes are LICM's `collect_mutated_names`.)
            IrStmt::Assign { .. }
            | IrStmt::Return(_)
            | IrStmt::Break(_)
            | IrStmt::Continue(_)
            | IrStmt::Throw { .. }
            | IrStmt::Asm { .. }
            | IrStmt::Expr(_) => {}
        }
    }
}

fn expr_requires_optimizer_barrier(expr: &IrExpr) -> bool {
    match &expr.kind {
        IrExprKind::Call { .. } => true,
        // `await` spawns/joins a thread, so it is never removable dead code.
        IrExprKind::Await { .. } => true,
        IrExprKind::Array(values) => values.iter().any(expr_requires_optimizer_barrier),
        IrExprKind::Index { .. } => true,
        // Field access is pure; only its target can require a barrier.
        IrExprKind::Field { target, .. } => expr_requires_optimizer_barrier(target),
        IrExprKind::Unary { expr, .. } => expr_requires_optimizer_barrier(expr),
        IrExprKind::Binary { left, right, .. } => {
            expr_requires_optimizer_barrier(left) || expr_requires_optimizer_barrier(right)
        }
        // Constructing a closure value only snapshots locals (no side effect), so
        // it is not an optimizer barrier — an unused closure binding is removable.
        IrExprKind::Closure { .. }
        | IrExprKind::Integer(_)
        | IrExprKind::Float(_)
        | IrExprKind::Bool(_)
        | IrExprKind::String(_)
        | IrExprKind::Char(_)
        | IrExprKind::Variable(_)
        | IrExprKind::Local { .. } => false,
    }
}
