//! Loop-invariant code motion (LICM) pass. Split out of `ir_optimizer.rs`;
//! behavior-preserving move. The shared `ExprSignature` type and
//! `combine_signatures` helper live in `ir_optimizer.rs` (also used by CSE). See
//! `ir_optimizer.rs` for the pass pipeline that drives it.

use super::*;

#[derive(Default)]
pub(crate) struct LoopInvariantMover {
    pub(crate) hoisted_loop_invariants: usize,
    next_temp: usize,
    reserved_names: HashSet<String>,
    /// Bindings of the function being rewritten whose address is taken with
    /// `addr_of`. See [`collect_address_taken_names`].
    address_taken: HashSet<String>,
}

impl LoopInvariantMover {
    pub(crate) fn move_module(&mut self, module: &IrModule) -> IrModule {
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
                .map(|function| self.move_function(function))
                .collect(),
        }
    }

    fn move_function(&mut self, function: &IrFunction) -> IrFunction {
        self.next_temp = 0;
        self.reserved_names = collect_function_binding_names(function);
        self.address_taken = collect_address_taken_names(function);
        let mut available = function
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<HashSet<_>>();

        IrFunction {
            name: function.name.clone(),
            params: function.params.clone(),
            return_type: function.return_type.clone(),
            body: self.move_block(&function.body, &mut available),
            span: function.span,
        }
    }

    fn move_block(
        &mut self,
        statements: &[IrStmt],
        available: &mut HashSet<String>,
    ) -> Vec<IrStmt> {
        let mut moved = Vec::new();

        for statement in statements {
            let statements = self.move_statement(statement, available);
            for statement in statements {
                add_available_declaration(&statement, available);
                moved.push(statement);
            }
        }

        moved
    }

    fn move_statement(&mut self, statement: &IrStmt, available: &HashSet<String>) -> Vec<IrStmt> {
        match statement {
            IrStmt::If {
                branches,
                else_body,
                span,
            } => {
                let branches = branches
                    .iter()
                    .map(|branch| {
                        let mut branch_available = available.clone();
                        IrIfBranch {
                            condition: branch.condition.clone(),
                            body: self.move_block(&branch.body, &mut branch_available),
                        }
                    })
                    .collect();
                let mut else_available = available.clone();
                vec![IrStmt::If {
                    branches,
                    else_body: self.move_block(else_body, &mut else_available),
                    span: *span,
                }]
            }
            IrStmt::While {
                condition,
                body,
                span,
            } => {
                let mut body_available = available.clone();
                let body = self.move_block(body, &mut body_available);
                let (mut hoisted, body) = self.hoist_loop_body(body, available);
                hoisted.push(IrStmt::While {
                    condition: condition.clone(),
                    body,
                    span: *span,
                });
                hoisted
            }
            IrStmt::For {
                name,
                start,
                end,
                step,
                body,
                span,
            } => {
                let mut body_available = available.clone();
                body_available.insert(name.clone());
                let body = self.move_block(body, &mut body_available);
                let (mut hoisted, body) = self.hoist_loop_body(body, available);
                hoisted.push(IrStmt::For {
                    name: name.clone(),
                    start: start.clone(),
                    end: end.clone(),
                    step: step.clone(),
                    body,
                    span: *span,
                });
                hoisted
            }
            IrStmt::Loop { body, span } => {
                let mut body_available = available.clone();
                let body = self.move_block(body, &mut body_available);
                let (mut hoisted, body) = self.hoist_loop_body(body, available);
                hoisted.push(IrStmt::Loop { body, span: *span });
                hoisted
            }
            // A region block is treated as an opaque passthrough, exactly like
            // `if`/`try`/`match`: LICM does not hoist across or into it (conservative,
            // and it preserves the region's scope boundary intact for slot planning).
            //
            // `asm` is in the same bucket, and DELIBERATELY does not descend into
            // its operands: an operand's evaluation is ordered against the
            // machine-code bytes, so hoisting a subexpression out of one would
            // change when it runs relative to the assembly. Keeping the statement
            // verbatim is the only correct treatment. (Its `out` writes ARE
            // reported to the invariance analysis by `collect_mutated_names`.)
            //
            // `IrStmt::Assign`'s target-path index expressions are likewise passed
            // through verbatim, and safely so: this pass only ever hoists whole
            // `IrStmt::Let` statements out of a loop — it never rewrites an
            // expression in place — so there is nothing to miss inside a path.
            // What the path DOES contribute is reads and (through calls) writes,
            // and both reach the invariance analysis below.
            IrStmt::Let { .. }
            | IrStmt::Assign { .. }
            | IrStmt::Return(_)
            | IrStmt::Break(_)
            | IrStmt::Continue(_)
            | IrStmt::Throw { .. }
            | IrStmt::Try { .. }
            | IrStmt::Match { .. }
            | IrStmt::RegionBlock { .. }
            | IrStmt::Asm { .. }
            | IrStmt::Expr(_) => vec![statement.clone()],
        }
    }

    fn hoist_loop_body(
        &mut self,
        body: Vec<IrStmt>,
        pre_loop_available: &HashSet<String>,
    ) -> (Vec<IrStmt>, Vec<IrStmt>) {
        let mut loop_declared = HashSet::new();
        collect_declared_names(&body, &mut loop_declared);
        let mut loop_mutated = HashSet::new();
        collect_mutated_names(&body, &mut loop_mutated);
        // `collect_mutated_names` sees only writes this pass can name: an
        // `IrStmt::Assign` target and an `asm` `out` clause. A CALL is opaque —
        // the callee can write through a raw pointer straight into one of this
        // function's bindings, with nothing in the loop body naming it. Without
        // this, `while … { let v = x + 1; … poke(p) … }` (where `p = addr_of(x)`)
        // hoisted `x + 1` out of the loop and froze it at its first-iteration
        // value — a wrong-value miscompile at `--optimize full` on the ir and
        // bytecode tiers, while `--optimize none` and the AST tier were correct.
        //
        // `addr_of` is the only way to obtain a pointer at a named local (it
        // requires an addressable place; everything else points into the heap or
        // an arena), so the address-taken set is exactly what an opaque call can
        // reach. Deriving a pointer to a *different* local with `ptr_offset` is
        // already outside the raw-pointer contract and is not defended against.
        // Functions that never use `addr_of` pay nothing: the set is empty and
        // the call scan is skipped.
        if !self.address_taken.is_empty() && block_contains_call(&body) {
            loop_mutated.extend(self.address_taken.iter().cloned());
        }

        let mut hoisted = Vec::new();
        let mut rewritten_body = Vec::new();

        for statement in body {
            let IrStmt::Let {
                name,
                ty,
                value,
                span,
            } = statement
            else {
                rewritten_body.push(statement);
                continue;
            };

            let Some(signature) = loop_invariant_expr_signature(&value) else {
                rewritten_body.push(IrStmt::Let {
                    name,
                    ty,
                    value,
                    span,
                });
                continue;
            };

            if !is_hoist_worthwhile(&value)
                || !signature
                    .dependencies
                    .iter()
                    .all(|name| pre_loop_available.contains(name))
                || signature
                    .dependencies
                    .iter()
                    .any(|name| loop_declared.contains(name) || loop_mutated.contains(name))
            {
                rewritten_body.push(IrStmt::Let {
                    name,
                    ty,
                    value,
                    span,
                });
                continue;
            }

            let temp = self.next_temp_name();
            let temp_expr_span = value.span;
            hoisted.push(IrStmt::Let {
                name: temp.clone(),
                ty: ty.clone(),
                value,
                span,
            });
            rewritten_body.push(IrStmt::Let {
                name,
                ty: ty.clone(),
                value: IrExpr {
                    kind: IrExprKind::Variable(temp),
                    ty,
                    span: temp_expr_span,
                },
                span,
            });
            self.hoisted_loop_invariants += 1;
        }

        (hoisted, rewritten_body)
    }

    fn next_temp_name(&mut self) -> String {
        loop {
            let name = format!("__lullaby_loop_invariant_{}", self.next_temp);
            self.next_temp += 1;
            if self.reserved_names.insert(name.clone()) {
                return name;
            }
        }
    }
}

fn add_available_declaration(statement: &IrStmt, available: &mut HashSet<String>) {
    if let IrStmt::Let { name, .. } = statement {
        available.insert(name.clone());
    }
}

fn collect_mutated_names(statements: &[IrStmt], names: &mut HashSet<String>) {
    for statement in statements {
        match statement {
            IrStmt::Assign { name, .. } => {
                names.insert(name.clone());
            }
            IrStmt::If {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    collect_mutated_names(&branch.body, names);
                }
                collect_mutated_names(else_body, names);
            }
            IrStmt::While { body, .. }
            | IrStmt::Loop { body, .. }
            | IrStmt::RegionBlock { body, .. } => {
                collect_mutated_names(body, names);
            }
            IrStmt::Try {
                body, catch_body, ..
            } => {
                collect_mutated_names(body, names);
                collect_mutated_names(catch_body, names);
            }
            IrStmt::Match { arms, .. } => {
                for arm in arms {
                    collect_mutated_names(&arm.body, names);
                }
            }
            IrStmt::For { name, body, .. } => {
                names.insert(name.clone());
                collect_mutated_names(body, names);
            }
            // An `asm` statement's `out <reg> = <lvalue>` clause WRITES that
            // lvalue's root binding after the machine code runs, so the binding is
            // mutated by the loop body. Missing it would let an expression reading
            // that binding look loop-invariant and be hoisted out of the loop — a
            // miscompile. Registers/clobbers name no IR binding, and an `in`
            // clause only reads.
            IrStmt::Asm { operands, .. } => {
                for operand in operands {
                    if let IrAsmOperand::Out { place, .. } = operand
                        && let Some(root) = expr_root_name(place)
                    {
                        names.insert(root);
                    }
                }
            }
            IrStmt::Let { .. }
            | IrStmt::Return(_)
            | IrStmt::Break(_)
            | IrStmt::Continue(_)
            | IrStmt::Throw { .. }
            | IrStmt::Expr(_) => {}
        }
    }
}

/// Every binding of `function` whose address is taken with `addr_of(<place>)`,
/// anywhere in the body — including outside the loop being considered, which is
/// where the pointer is usually made.
///
/// `addr_of` is the only builtin that yields a pointer at a *named local*: it
/// requires an addressable place (`L0458`), while every other pointer source
/// points into the heap, an arena, or another pointer. So this set is exactly
/// the set of bindings an opaque call can mutate without any statement in the
/// loop body naming them.
///
/// The `addr_of` match is by name and deliberately does not consult
/// [`collect_function_binding_names`]: a local shadowing the builtin would make
/// this record a binding whose address was never taken, which only *widens* the
/// set and so can only cost a hoist. That is the safe direction — unlike the
/// inliner, where the same unchecked name lookup changed emitted semantics.
fn collect_address_taken_names(function: &IrFunction) -> HashSet<String> {
    let mut names = HashSet::new();
    walk_block_exprs(&function.body, &mut |expr| {
        if let IrExprKind::Call { name, args } = &expr.kind
            && name == "addr_of"
            && let Some(place) = args.first()
            && let Some(root) = expr_root_name(place)
        {
            names.insert(root);
        }
    });
    names
}

/// Whether a block evaluates any call or `await` — the operations whose effects
/// on this function's bindings the pass cannot see.
fn block_contains_call(statements: &[IrStmt]) -> bool {
    let mut found = false;
    walk_block_exprs(statements, &mut |expr| {
        if matches!(
            expr.kind,
            IrExprKind::Call { .. } | IrExprKind::Await { .. }
        ) {
            found = true;
        }
    });
    found
}

/// Visit every expression evaluated by `statements`, nested blocks included.
///
/// Every child-bearing field is walked: an assignment's target path (via
/// [`ir_assign_path_exprs`]) and an `asm` statement's operand clauses (via
/// [`ir_asm_operand_exprs`]) carry ordinary expressions and are as capable of
/// holding a call as any RHS.
fn walk_block_exprs<F: FnMut(&IrExpr)>(statements: &[IrStmt], visit: &mut F) {
    for statement in statements {
        match statement {
            IrStmt::Let { value, .. } | IrStmt::Expr(value) | IrStmt::Throw { value, .. } => {
                walk_expr(value, visit);
            }
            IrStmt::Assign { path, value, .. } => {
                walk_expr(value, visit);
                for index in ir_assign_path_exprs(path) {
                    walk_expr(index, visit);
                }
            }
            IrStmt::Return(expr) => {
                if let Some(expr) = expr {
                    walk_expr(expr, visit);
                }
            }
            IrStmt::Break(_) | IrStmt::Continue(_) => {}
            IrStmt::If {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    walk_expr(&branch.condition, visit);
                    walk_block_exprs(&branch.body, visit);
                }
                walk_block_exprs(else_body, visit);
            }
            IrStmt::While {
                condition, body, ..
            } => {
                walk_expr(condition, visit);
                walk_block_exprs(body, visit);
            }
            IrStmt::For {
                start,
                end,
                step,
                body,
                ..
            } => {
                walk_expr(start, visit);
                walk_expr(end, visit);
                if let Some(step) = step {
                    walk_expr(step, visit);
                }
                walk_block_exprs(body, visit);
            }
            IrStmt::Loop { body, .. } | IrStmt::RegionBlock { body, .. } => {
                walk_block_exprs(body, visit);
            }
            IrStmt::Try {
                body, catch_body, ..
            } => {
                walk_block_exprs(body, visit);
                walk_block_exprs(catch_body, visit);
            }
            IrStmt::Match {
                scrutinee, arms, ..
            } => {
                walk_expr(scrutinee, visit);
                for arm in arms {
                    walk_block_exprs(&arm.body, visit);
                }
            }
            IrStmt::Asm { operands, .. } => {
                for expr in ir_asm_operand_exprs(operands) {
                    walk_expr(expr, visit);
                }
            }
        }
    }
}

/// Visit `expr` and every sub-expression it contains, outermost first.
fn walk_expr<F: FnMut(&IrExpr)>(expr: &IrExpr, visit: &mut F) {
    visit(expr);
    match &expr.kind {
        IrExprKind::Array(values) => {
            for value in values {
                walk_expr(value, visit);
            }
        }
        IrExprKind::Index { target, index } => {
            walk_expr(target, visit);
            walk_expr(index, visit);
        }
        IrExprKind::Field { target, .. } => walk_expr(target, visit),
        IrExprKind::Unary { expr: inner, .. } | IrExprKind::Await { expr: inner } => {
            walk_expr(inner, visit);
        }
        IrExprKind::Binary { left, right, .. } => {
            walk_expr(left, visit);
            walk_expr(right, visit);
        }
        IrExprKind::Call { args, .. } => {
            for arg in args {
                walk_expr(arg, visit);
            }
        }
        // A closure literal carries only an id; its body lives in the module's
        // closure table and is walked (if at all) when that function is rewritten.
        IrExprKind::Closure { .. }
        | IrExprKind::Integer(_)
        | IrExprKind::Float(_)
        | IrExprKind::Bool(_)
        | IrExprKind::String(_)
        | IrExprKind::Char(_)
        | IrExprKind::Variable(_)
        | IrExprKind::Local { .. } => {}
    }
}

/// The root binding name an lvalue expression writes through (`y`, `s.f`, `a[i]`
/// all root at their leftmost variable). `None` for anything that is not rooted
/// at a named binding.
fn expr_root_name(expr: &IrExpr) -> Option<String> {
    match &expr.kind {
        IrExprKind::Variable(name) => Some(name.clone()),
        IrExprKind::Local { name, .. } => Some(name.clone()),
        IrExprKind::Field { target, .. } => expr_root_name(target),
        IrExprKind::Index { target, .. } => expr_root_name(target),
        _ => None,
    }
}

fn loop_invariant_expr_signature(expr: &IrExpr) -> Option<ExprSignature> {
    let (key, dependencies) = match &expr.kind {
        IrExprKind::Integer(value) => (format!("i64:{value}:{}", expr.ty.name), HashSet::new()),
        IrExprKind::Float(value) => (
            format!("f64:{}:{}", value.to_bits(), expr.ty.name),
            HashSet::new(),
        ),
        IrExprKind::Bool(value) => (format!("bool:{value}:{}", expr.ty.name), HashSet::new()),
        IrExprKind::String(value) => (format!("string:{value:?}:{}", expr.ty.name), HashSet::new()),
        IrExprKind::Char(value) => (format!("char:{value}:{}", expr.ty.name), HashSet::new()),
        IrExprKind::Variable(name) | IrExprKind::Local { name, .. } => {
            let mut dependencies = HashSet::new();
            dependencies.insert(name.clone());
            (format!("var:{name}:{}", expr.ty.name), dependencies)
        }
        IrExprKind::Unary { op, expr: inner } => {
            let inner = loop_invariant_expr_signature(inner)?;
            combine_signatures(&format!("unary:{op:?}"), &expr.ty.name, vec![inner])
        }
        IrExprKind::Binary { left, op, right } => {
            if matches!(op, BinaryOp::Divide | BinaryOp::Remainder) {
                return None;
            }
            let left = loop_invariant_expr_signature(left)?;
            let right = loop_invariant_expr_signature(right)?;
            combine_signatures(&format!("binary:{op:?}"), &expr.ty.name, vec![left, right])
        }
        IrExprKind::Array(_)
        | IrExprKind::Index { .. }
        | IrExprKind::Field { .. }
        | IrExprKind::Call { .. }
        | IrExprKind::Await { .. }
        // A closure captures the live environment at evaluation time, so it is
        // never loop-invariant (its captured values may change per iteration).
        | IrExprKind::Closure { .. } => return None,
    };

    Some(ExprSignature { key, dependencies })
}

fn is_hoist_worthwhile(expr: &IrExpr) -> bool {
    matches!(
        expr.kind,
        IrExprKind::Unary { .. } | IrExprKind::Binary { .. }
    )
}
