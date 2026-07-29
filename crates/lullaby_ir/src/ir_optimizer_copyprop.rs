//! Copy-propagation pass. Split out of `ir_optimizer.rs`; behavior-preserving
//! move. The shared `expr_requires_optimizer_barrier` predicate lives in
//! `ir_optimizer.rs` (also used by CSE). See `ir_optimizer.rs` for the pass
//! pipeline that drives it.
//!
//! # The barrier is owned by the expression walk
//!
//! An alias (`b` -> `x`, recorded for `let b = x`) is valid only until something
//! mutates `x`. A call can do that invisibly — `poke(p)` writing through a
//! pointer taken with `addr_of(x)` mutates `x` without any `IrStmt::Assign` to
//! `x` — so **every call, `await`, and indexing operation is a barrier that
//! clears the alias map**.
//!
//! That barrier is applied inside [`CopyPropagator::propagate_expr`], at the
//! point the barrier-bearing node is reached in evaluation order, *not* after
//! the statement has already been rewritten. Two wrong-value miscompiles came
//! from getting this wrong, both fixed here:
//!
//! * `out = pick(poke(p), b)` — the old code rewrote the whole RHS first and
//!   cleared afterwards, so `b` was rewritten to `x` even though `poke` had
//!   already changed `x` by the time `b` is read. Walking in evaluation order
//!   clears at `poke(p)` and leaves the later `b` alone, while still propagating
//!   into arguments that precede any call.
//! * `arr[poke(p)] = 5` — the old code evaluated the barrier over the assigned
//!   value only and cloned the target path verbatim, so the call in the index
//!   never cleared anything. The path's expressions now go through the same walk
//!   (see [`ir_assign_path_exprs`]).
//!
//! Both returned a stale value at `--optimize full` on the ir and bytecode tiers
//! while `--optimize none` and the AST tier were correct.

use super::*;

#[derive(Default)]
pub(crate) struct CopyPropagator {
    pub(crate) propagated_copies: usize,
}

impl CopyPropagator {
    pub(crate) fn propagate_module(&mut self, module: &IrModule) -> IrModule {
        IrModule {
            structs: module.structs.clone(),
            enums: module.enums.clone(),
            impls: module.impls.clone(),
            trait_methods: module.trait_methods.clone(),
            async_functions: module.async_functions.clone(),
            extern_functions: module.extern_functions.clone(),
            extern_signatures: module.extern_signatures.clone(),
            export_functions: module.export_functions.clone(),
            interrupt_functions: module.interrupt_functions.clone(),
            naked_functions: module.naked_functions.clone(),
            // Closure bodies are carried through unchanged; this pass only
            // rewrites top-level function bodies. (Closures run on the
            // interpreters, so optimizing their bodies is a separate concern.)
            closures: module.closures.clone(),
            functions: module
                .functions
                .iter()
                .map(|function| self.propagate_function(function))
                .collect(),
        }
    }

    fn propagate_function(&mut self, function: &IrFunction) -> IrFunction {
        IrFunction {
            name: function.name.clone(),
            params: function.params.clone(),
            return_type: function.return_type.clone(),
            body: self.propagate_block(&function.body, &mut HashMap::new()),
            span: function.span,
        }
    }

    fn propagate_block(
        &mut self,
        statements: &[IrStmt],
        aliases: &mut HashMap<String, String>,
    ) -> Vec<IrStmt> {
        statements
            .iter()
            .map(|statement| self.propagate_statement(statement, aliases))
            .collect()
    }

    fn propagate_statement(
        &mut self,
        statement: &IrStmt,
        aliases: &mut HashMap<String, String>,
    ) -> IrStmt {
        match statement {
            IrStmt::Let {
                name,
                ty,
                value,
                span,
            } => {
                // `propagate_expr` clears `aliases` at every barrier it walks
                // past, so by the time it returns the map already reflects
                // whatever the initializer's calls may have mutated.
                let value = self.propagate_expr(value, aliases);
                invalidate_alias(name, aliases);
                if let IrExprKind::Variable(source) = &value.kind {
                    let source = resolve_alias(source, aliases);
                    if source != *name {
                        aliases.insert(name.clone(), source);
                    }
                }
                IrStmt::Let {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                    span: *span,
                }
            }
            IrStmt::Assign {
                name,
                path,
                op,
                value,
                span,
            } => {
                // An assignment evaluates its RHS *and* every index expression in
                // its target path. The tiers do not agree on which runs first (the
                // IR interpreter evaluates the RHS then resolves the path), so if
                // the path can trip the barrier, drop the alias state before
                // either side is rewritten rather than betting on an order.
                if ir_assign_path_exprs(path).any(expr_requires_optimizer_barrier) {
                    aliases.clear();
                }
                let value = self.propagate_expr(value, aliases);
                let mut rewritten_path = Vec::with_capacity(path.len());
                for place in path {
                    rewritten_path
                        .push(place.map_index_expr(|index| self.propagate_expr(index, aliases)));
                }
                invalidate_alias(name, aliases);
                IrStmt::Assign {
                    name: name.clone(),
                    path: rewritten_path,
                    op: *op,
                    value,
                    span: *span,
                }
            }
            IrStmt::Return(expr) => {
                IrStmt::Return(expr.as_ref().map(|expr| self.propagate_expr(expr, aliases)))
            }
            IrStmt::Break(span) => IrStmt::Break(*span),
            IrStmt::Continue(span) => IrStmt::Continue(*span),
            IrStmt::Expr(expr) => IrStmt::Expr(self.propagate_expr(expr, aliases)),
            IrStmt::If {
                branches,
                else_body,
                span,
            } => {
                let branches = branches
                    .iter()
                    .map(|branch| IrIfBranch {
                        condition: self.propagate_expr(&branch.condition, aliases),
                        body: self.propagate_block(&branch.body, &mut HashMap::new()),
                    })
                    .collect();
                let else_body = self.propagate_block(else_body, &mut HashMap::new());
                aliases.clear();
                IrStmt::If {
                    branches,
                    else_body,
                    span: *span,
                }
            }
            IrStmt::While {
                condition,
                body,
                span,
            } => {
                let condition = self.propagate_expr(condition, aliases);
                let body = self.propagate_block(body, &mut HashMap::new());
                aliases.clear();
                IrStmt::While {
                    condition,
                    body,
                    span: *span,
                }
            }
            IrStmt::For {
                name,
                start,
                end,
                step,
                body,
                span,
            } => {
                let start = self.propagate_expr(start, aliases);
                let end = self.propagate_expr(end, aliases);
                let step = step.as_ref().map(|step| self.propagate_expr(step, aliases));
                let body = self.propagate_block(body, &mut HashMap::new());
                aliases.clear();
                IrStmt::For {
                    name: name.clone(),
                    start,
                    end,
                    step,
                    body,
                    span: *span,
                }
            }
            IrStmt::Loop { body, span } => {
                let body = self.propagate_block(body, &mut HashMap::new());
                aliases.clear();
                IrStmt::Loop { body, span: *span }
            }
            // A region block is a run-once nested scope preserved as its own node.
            // Propagate its body conservatively (fresh alias map, then clear), exactly
            // like a loop body, so a block-local rebind never leaks a stale alias.
            IrStmt::RegionBlock { body, span } => {
                let body = self.propagate_block(body, &mut HashMap::new());
                aliases.clear();
                IrStmt::RegionBlock { body, span: *span }
            }
            // Inline assembly is opaque: clear aliases (it may write registers
            // backing outputs) and pass the bytes/operands/clobbers through. The
            // operand input expressions are left verbatim rather than propagated
            // into — never removing the original binding an asm operand reads.
            IrStmt::Asm {
                bytes,
                operands,
                clobbers,
                span,
            } => {
                aliases.clear();
                IrStmt::Asm {
                    bytes: bytes.clone(),
                    operands: operands.clone(),
                    clobbers: clobbers.clone(),
                    span: *span,
                }
            }
            IrStmt::Throw { value, span } => {
                let value = self.propagate_expr(value, aliases);
                aliases.clear();
                IrStmt::Throw { value, span: *span }
            }
            IrStmt::Try {
                body,
                catch_name,
                catch_body,
                span,
            } => {
                let body = self.propagate_block(body, &mut HashMap::new());
                let catch_body = self.propagate_block(catch_body, &mut HashMap::new());
                aliases.clear();
                IrStmt::Try {
                    body,
                    catch_name: catch_name.clone(),
                    catch_body,
                    span: *span,
                }
            }
            IrStmt::Match {
                scrutinee,
                arms,
                span,
            } => {
                let scrutinee = self.propagate_expr(scrutinee, aliases);
                let arms = arms
                    .iter()
                    .map(|arm| IrMatchArm {
                        pattern: arm.pattern.clone(),
                        body: self.propagate_block(&arm.body, &mut HashMap::new()),
                    })
                    .collect();
                aliases.clear();
                IrStmt::Match {
                    scrutinee,
                    arms,
                    span: *span,
                }
            }
        }
    }

    /// Rewrite `expr`, walking its children in evaluation order and clearing
    /// `aliases` at each barrier the walk passes.
    ///
    /// Threading the alias map mutably through the walk is what makes the barrier
    /// positional: reads that happen *before* the statement's first call still
    /// get propagated, and reads after it do not. See the module docs.
    fn propagate_expr(&mut self, expr: &IrExpr, aliases: &mut HashMap<String, String>) -> IrExpr {
        match &expr.kind {
            IrExprKind::Variable(name) => {
                let replacement = resolve_alias(name, aliases);
                if replacement != *name {
                    self.propagated_copies += 1;
                    IrExpr {
                        kind: IrExprKind::Variable(replacement),
                        ty: expr.ty.clone(),
                        span: expr.span,
                    }
                } else {
                    expr.clone()
                }
            }
            // Array elements evaluate left to right.
            IrExprKind::Array(values) => {
                let mut rewritten = Vec::with_capacity(values.len());
                for value in values {
                    rewritten.push(self.propagate_expr(value, aliases));
                }
                IrExpr {
                    kind: IrExprKind::Array(rewritten),
                    ty: expr.ty.clone(),
                    span: expr.span,
                }
            }
            IrExprKind::Index { target, index } => {
                // The two children's evaluation order is shape-dependent: the IR
                // interpreter's bare-variable fast path evaluates the index before
                // borrowing the target. So if either child can trip the barrier,
                // clear first rather than betting on an order. (A bare-variable
                // target is a leaf and cannot trip it, which is why the fast path
                // is safe at all.)
                if expr_requires_optimizer_barrier(target) || expr_requires_optimizer_barrier(index)
                {
                    aliases.clear();
                }
                let target = Box::new(self.propagate_expr(target, aliases));
                let index = Box::new(self.propagate_expr(index, aliases));
                // Indexing is itself a barrier (`expr_requires_optimizer_barrier`
                // treats it as one), so nothing after it may reuse an alias.
                aliases.clear();
                IrExpr {
                    kind: IrExprKind::Index { target, index },
                    ty: expr.ty.clone(),
                    span: expr.span,
                }
            }
            // Field access is pure; only its target can trip the barrier, and the
            // recursion handles that.
            IrExprKind::Field { target, field } => IrExpr {
                kind: IrExprKind::Field {
                    target: Box::new(self.propagate_expr(target, aliases)),
                    field: field.clone(),
                },
                ty: expr.ty.clone(),
                span: expr.span,
            },
            IrExprKind::Unary { op, expr: inner } => IrExpr {
                kind: IrExprKind::Unary {
                    op: *op,
                    expr: Box::new(self.propagate_expr(inner, aliases)),
                },
                ty: expr.ty.clone(),
                span: expr.span,
            },
            // Left evaluates before right on every tier — including the
            // short-circuiting `and`/`or`, where the right side may not evaluate
            // at all (clearing for a barrier that never ran is conservative, and
            // conservative is sound).
            IrExprKind::Binary { left, op, right } => {
                let left = Box::new(self.propagate_expr(left, aliases));
                let right = Box::new(self.propagate_expr(right, aliases));
                IrExpr {
                    kind: IrExprKind::Binary {
                        left,
                        op: *op,
                        right,
                    },
                    ty: expr.ty.clone(),
                    span: expr.span,
                }
            }
            // Arguments evaluate left to right, then the call runs. The callee may
            // mutate any binding — including through a raw pointer — so every
            // alias dies at the call, but arguments to its left are still read
            // before it and may be propagated.
            IrExprKind::Call { name, args } => {
                let mut rewritten = Vec::with_capacity(args.len());
                for arg in args {
                    rewritten.push(self.propagate_expr(arg, aliases));
                }
                aliases.clear();
                IrExpr {
                    kind: IrExprKind::Call {
                        name: name.clone(),
                        args: rewritten,
                    },
                    ty: expr.ty.clone(),
                    span: expr.span,
                }
            }
            // `await` joins a thread that may have mutated anything.
            IrExprKind::Await { expr: inner } => {
                let inner = Box::new(self.propagate_expr(inner, aliases));
                aliases.clear();
                IrExpr {
                    kind: IrExprKind::Await { expr: inner },
                    ty: expr.ty.clone(),
                    span: expr.span,
                }
            }
            // A closure literal node carries only an id; its captured values are
            // materialized at runtime and its body lives in the module table, so
            // copy propagation has nothing to rewrite here.
            // A `Local` is only introduced after every optimization pass (at
            // interpretation time), so it never reaches copy propagation; copy it
            // through unchanged for match completeness.
            IrExprKind::Closure { .. }
            | IrExprKind::Integer(_)
            | IrExprKind::Float(_)
            | IrExprKind::Bool(_)
            | IrExprKind::String(_)
            | IrExprKind::Char(_)
            | IrExprKind::Local { .. } => expr.clone(),
        }
    }
}

fn resolve_alias(name: &str, aliases: &HashMap<String, String>) -> String {
    let mut current = name;
    let mut seen = HashSet::new();
    while let Some(next) = aliases.get(current).map(String::as_str) {
        if !seen.insert(current) {
            break;
        }
        current = next;
    }
    current.to_string()
}

fn invalidate_alias(name: &str, aliases: &mut HashMap<String, String>) {
    let stale = aliases
        .keys()
        .filter(|alias| alias.as_str() == name || resolve_alias(alias, aliases) == name)
        .cloned()
        .collect::<Vec<_>>();
    for alias in stale {
        aliases.remove(&alias);
    }
}
