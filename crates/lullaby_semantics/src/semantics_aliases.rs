//! Type-alias resolution split out of `lib.rs`: expands user `alias`
//! declarations to their canonical types across a whole `Program` before any
//! checking, so the rest of the pipeline (and IR/runtime) never sees an alias.
//!
//! This is a behavior-preserving move. `resolve_program_aliases` is re-exported
//! at the crate root for `validate`; the remaining helpers stay module-private.

use std::collections::{HashMap, HashSet};

use lullaby_parser::{
    ConstDecl, EnumDecl, EnumVariant, Expr, ExprKind, Function, Param, Program, Stmt, StructDecl,
    StructField, TypeRef, asm_operand_exprs_mut, assign_path_exprs_mut,
};

use super::SemanticDiagnostic;

/// Resolve all type aliases in a program to canonical types, returning the
/// rewritten program plus any alias-definition diagnostics (duplicate `L0360`,
/// cyclic `L0361`).
pub(crate) fn resolve_program_aliases(program: &Program) -> (Program, Vec<SemanticDiagnostic>) {
    let mut diagnostics = Vec::new();
    let mut map: HashMap<String, TypeRef> = HashMap::new();
    for alias in &program.aliases {
        if map.contains_key(&alias.name) {
            diagnostics.push(SemanticDiagnostic::at(
                "L0360",
                format!("duplicate type alias `{}`", alias.name),
                None,
                alias.span,
            ));
            continue;
        }
        map.insert(alias.name.clone(), alias.target.clone());
    }

    // Detect cyclic alias chains (e.g. `alias A = B` / `alias B = A`).
    for alias in &program.aliases {
        if chain_is_cyclic(&alias.name, &map) {
            diagnostics.push(SemanticDiagnostic::at(
                "L0361",
                format!("type alias `{}` is defined in terms of itself", alias.name),
                None,
                alias.span,
            ));
        }
    }

    let functions = program
        .functions
        .iter()
        .map(|function| Function {
            name: function.name.clone(),
            type_params: function.type_params.clone(),
            params: function
                .params
                .iter()
                .map(|param| Param {
                    name: param.name.clone(),
                    ty: resolve_alias_type(&param.ty, &map),
                })
                .collect(),
            return_type: resolve_alias_type(&function.return_type, &map),
            body: rewritten_block(&function.body, &map),
            span: function.span,
            is_public: function.is_public,
            is_async: function.is_async,
            is_extern: function.is_extern,
            is_export: function.is_export,
            is_interrupt: function.is_interrupt,
            is_naked: function.is_naked,
            // Alias resolution rewrites type spellings only; it never moves a
            // declaration between files, so its origin carries over.
            module: function.module.clone(),
        })
        .collect();

    let structs = program
        .structs
        .iter()
        .map(|declaration| StructDecl {
            name: declaration.name.clone(),
            type_params: declaration.type_params.clone(),
            fields: declaration
                .fields
                .iter()
                .map(|field| StructField {
                    name: field.name.clone(),
                    ty: resolve_alias_type(&field.ty, &map),
                })
                .collect(),
            span: declaration.span,
            is_public: declaration.is_public,
        })
        .collect();

    let enums = program
        .enums
        .iter()
        .map(|declaration| EnumDecl {
            name: declaration.name.clone(),
            type_params: declaration.type_params.clone(),
            variants: declaration
                .variants
                .iter()
                .map(|variant| EnumVariant {
                    name: variant.name.clone(),
                    payload: variant
                        .payload
                        .iter()
                        .map(|ty| resolve_alias_type(ty, &map))
                        .collect(),
                })
                .collect(),
            span: declaration.span,
            is_public: declaration.is_public,
        })
        .collect();

    (
        Program {
            functions,
            aliases: program.aliases.clone(),
            structs,
            enums,
            imports: program.imports.clone(),
            // Trait/impl method types are resolved against aliases below via the
            // same `resolve_alias_type` mapping so the checker never sees an alias.
            traits: program
                .traits
                .iter()
                .map(|decl| lullaby_parser::TraitDecl {
                    name: decl.name.clone(),
                    methods: decl
                        .methods
                        .iter()
                        .map(|method| lullaby_parser::MethodSig {
                            name: method.name.clone(),
                            params: method
                                .params
                                .iter()
                                .map(|param| Param {
                                    name: param.name.clone(),
                                    ty: resolve_alias_type(&param.ty, &map),
                                })
                                .collect(),
                            return_type: resolve_alias_type(&method.return_type, &map),
                            span: method.span,
                        })
                        .collect(),
                    span: decl.span,
                    is_public: decl.is_public,
                })
                .collect(),
            impls: program
                .impls
                .iter()
                .map(|decl| lullaby_parser::ImplDecl {
                    trait_name: decl.trait_name.clone(),
                    type_name: decl.type_name.clone(),
                    type_params: decl.type_params.clone(),
                    methods: decl
                        .methods
                        .iter()
                        .map(|function| Function {
                            name: function.name.clone(),
                            type_params: function.type_params.clone(),
                            params: function
                                .params
                                .iter()
                                .map(|param| Param {
                                    name: param.name.clone(),
                                    ty: resolve_alias_type(&param.ty, &map),
                                })
                                .collect(),
                            return_type: resolve_alias_type(&function.return_type, &map),
                            body: rewritten_block(&function.body, &map),
                            span: function.span,
                            is_public: function.is_public,
                            is_async: function.is_async,
                            is_extern: function.is_extern,
                            is_export: function.is_export,
                            is_interrupt: function.is_interrupt,
                            is_naked: function.is_naked,
                            module: function.module.clone(),
                        })
                        .collect(),
                    span: decl.span,
                })
                .collect(),
            // A constant's declared type may name an alias (`const N Count = 5`);
            // resolve it to the canonical type so the checker/const-evaluator
            // never sees an alias. The initializer is an expression position like
            // any other — the const folder descends into a closure literal there
            // — so it goes through the same expression walk rather than a bare
            // clone.
            consts: program
                .consts
                .iter()
                .map(|decl| ConstDecl {
                    name: decl.name.clone(),
                    ty: resolve_alias_type(&decl.ty, &map),
                    value: rewritten_expr(&decl.value, &map),
                    span: decl.span,
                    is_public: decl.is_public,
                })
                .collect(),
            // An actor's state-field, init-parameter, handler-parameter, and
            // reply types may name aliases; resolve them all so the checker and
            // interpreter never see an alias. Handler/init bodies are rewritten
            // through the same `rewrite_stmt_types` mapping as function bodies.
            actors: program
                .actors
                .iter()
                .map(|decl| lullaby_parser::ActorDecl {
                    name: decl.name.clone(),
                    state: decl
                        .state
                        .iter()
                        .map(|field| StructField {
                            name: field.name.clone(),
                            ty: resolve_alias_type(&field.ty, &map),
                        })
                        .collect(),
                    init: decl.init.as_ref().map(|init| lullaby_parser::ActorInit {
                        params: init
                            .params
                            .iter()
                            .map(|param| Param {
                                name: param.name.clone(),
                                ty: resolve_alias_type(&param.ty, &map),
                            })
                            .collect(),
                        body: rewritten_block(&init.body, &map),
                        span: init.span,
                    }),
                    handlers: decl
                        .handlers
                        .iter()
                        .map(|handler| lullaby_parser::ActorHandler {
                            name: handler.name.clone(),
                            params: handler
                                .params
                                .iter()
                                .map(|param| Param {
                                    name: param.name.clone(),
                                    ty: resolve_alias_type(&param.ty, &map),
                                })
                                .collect(),
                            reply_type: handler
                                .reply_type
                                .as_ref()
                                .map(|ty| resolve_alias_type(ty, &map)),
                            body: rewritten_block(&handler.body, &map),
                            span: handler.span,
                        })
                        .collect(),
                    span: decl.span,
                    is_public: decl.is_public,
                })
                .collect(),
            // The freestanding-tier directive is a whole-module property; alias
            // resolution preserves it so the tier gate still fires downstream.
            is_no_runtime: program.is_no_runtime,
            // Alias resolution rewrites type spellings only — it neither renames
            // nor moves a declaration — so the per-declaration origin/tier table
            // carries over verbatim. Dropping it here would silently disable the
            // per-module tier gate and cross-module diagnostic attribution.
            origins: program.origins.clone(),
        },
        diagnostics,
    )
}

/// True if following the alias chain from `name` revisits `name` (a cycle).
fn chain_is_cyclic(name: &str, map: &HashMap<String, TypeRef>) -> bool {
    let mut seen = HashSet::new();
    let mut current = name.to_string();
    while let Some(target) = map.get(&current) {
        if !map.contains_key(&target.name) {
            return false;
        }
        current = target.name.clone();
        if current == name {
            return true;
        }
        if !seen.insert(current.clone()) {
            return false;
        }
    }
    false
}

/// Expand alias names inside a type, including generic arguments, to canonical
/// form. Bounded by a depth guard so cyclic aliases cannot loop forever.
fn resolve_alias_type(ty: &TypeRef, map: &HashMap<String, TypeRef>) -> TypeRef {
    resolve_alias_type_depth(ty, map, 0)
}

fn resolve_alias_type_depth(ty: &TypeRef, map: &HashMap<String, TypeRef>, depth: usize) -> TypeRef {
    if depth > 32 {
        return ty.clone();
    }
    for ctor in ["array", "ptr", "ref", "rc"] {
        if let Some(inner) = ty.generic_arg(ctor) {
            let resolved = resolve_alias_type_depth(&inner, map, depth + 1);
            return TypeRef::new(format!("{ctor}<{}>", resolved.name));
        }
    }
    if let Some(target) = map.get(&ty.name) {
        return resolve_alias_type_depth(target, map, depth + 1);
    }
    ty.clone()
}

/// Clone a block and resolve every alias spelling reachable from it.
///
/// The `Program`-rebuilding code above is expression-shaped, so it needs a
/// by-value form; the walk itself is in place. See [`rewrite_stmt_types`].
fn rewritten_block(body: &[Stmt], map: &HashMap<String, TypeRef>) -> Vec<Stmt> {
    let mut body = body.to_vec();
    rewrite_block_types(&mut body, map);
    body
}

/// Clone an expression and resolve every alias spelling reachable from it.
/// The by-value companion to [`rewrite_expr_types`], for the same reason.
fn rewritten_expr(expr: &Expr, map: &HashMap<String, TypeRef>) -> Expr {
    let mut expr = expr.clone();
    rewrite_expr_types(&mut expr, map);
    expr
}

fn rewrite_block_types(body: &mut [Stmt], map: &HashMap<String, TypeRef>) {
    for stmt in body {
        rewrite_stmt_types(stmt, map);
    }
}

/// Resolve alias spellings in every type annotation a statement can reach,
/// descending through nested blocks **and** through expressions.
///
/// The match names every [`Stmt`] variant and every child-bearing field, with no
/// catch-all: exhaustiveness is what makes the pass correct, so a new variant (or
/// a new field on an existing one) must be a compile error rather than a silent
/// skip. The previous shape — rebuild a handful of variants, `other =>
/// other.clone()` for the rest — was such a silent skip twice over. It never
/// entered expressions, so `list_map(base, fn x Num -> x + x)` kept an unresolved
/// alias in the closure parameter and the checker falsely rejected a valid
/// program; and it reached a `match` only through a hand-written
/// `Stmt::Expr(ExprKind::Match { .. })` special case, so identical arm bodies
/// behaved differently in statement position (accepted) and in `let`/`return`/
/// assignment-RHS/call-argument position (rejected `L0303`).
fn rewrite_stmt_types(stmt: &mut Stmt, map: &HashMap<String, TypeRef>) {
    match stmt {
        Stmt::Let {
            name: _,
            ty,
            value,
            span: _,
        } => {
            if let Some(ty) = ty {
                *ty = resolve_alias_type(ty, map);
            }
            rewrite_expr_types(value, map);
        }
        // An assignment target's index is an ordinary expression and can spell an
        // alias (in a closure parameter) exactly like the value can — see
        // `Place::index_expr`.
        Stmt::Assign {
            name: _,
            path,
            op: _,
            value,
            span: _,
        } => {
            for index in assign_path_exprs_mut(path) {
                rewrite_expr_types(index, map);
            }
            rewrite_expr_types(value, map);
        }
        Stmt::Return(value) => {
            if let Some(value) = value {
                rewrite_expr_types(value, map);
            }
        }
        Stmt::Expr(expr) => rewrite_expr_types(expr, map),
        Stmt::Throw { value, span: _ } => rewrite_expr_types(value, map),
        Stmt::If {
            branches,
            else_body,
            span: _,
        } => {
            for branch in branches {
                rewrite_expr_types(&mut branch.condition, map);
                rewrite_block_types(&mut branch.body, map);
            }
            rewrite_block_types(else_body, map);
        }
        Stmt::While {
            condition,
            body,
            span: _,
        } => {
            rewrite_expr_types(condition, map);
            rewrite_block_types(body, map);
        }
        Stmt::For {
            name: _,
            start,
            end,
            step,
            body,
            span: _,
        } => {
            rewrite_expr_types(start, map);
            rewrite_expr_types(end, map);
            if let Some(step) = step {
                rewrite_expr_types(step, map);
            }
            rewrite_block_types(body, map);
        }
        Stmt::ForEach {
            name: _,
            iterable,
            body,
            span: _,
        } => {
            rewrite_expr_types(iterable, map);
            rewrite_block_types(body, map);
        }
        Stmt::Loop { body, span: _ }
        | Stmt::Unsafe { body, span: _ }
        | Stmt::RegionBlock { body, span: _ } => rewrite_block_types(body, map),
        Stmt::Try {
            body,
            catch_name: _,
            catch_body,
            span: _,
        } => {
            rewrite_block_types(body, map);
            rewrite_block_types(catch_body, map);
        }
        // The machine-code bytes are opaque, but an `asm` operand clause carries
        // an ordinary expression, which may spell an alias like any other — see
        // `AsmOperand::expr`.
        Stmt::Asm {
            bytes: _,
            operands,
            clobbers: _,
            span: _,
        } => {
            for expr in asm_operand_exprs_mut(operands) {
                rewrite_expr_types(expr, map);
            }
        }
        // Genuinely childless: no type annotation, no sub-expression, no nested
        // block. `RegionDecl` carries only a name, integer size/align, and kind.
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Region(_) => {}
    }
}

/// Resolve alias spellings reachable through an expression.
///
/// A closure literal is the only expression that carries a **type annotation**
/// (its parameter types) and a `match` the only one that carries **nested
/// statements** (its arm bodies); every other kind simply recurses into its
/// sub-expressions. Named exhaustively for the same reason as
/// [`rewrite_stmt_types`]: a new [`ExprKind`] must not be able to hide an
/// annotation from this pass.
fn rewrite_expr_types(expr: &mut Expr, map: &HashMap<String, TypeRef>) {
    match &mut expr.kind {
        ExprKind::Closure {
            id: _,
            params,
            body,
        } => {
            for param in params {
                param.ty = resolve_alias_type(&param.ty, map);
            }
            rewrite_expr_types(body, map);
        }
        ExprKind::Match { scrutinee, arms } => {
            rewrite_expr_types(scrutinee, map);
            for arm in arms {
                rewrite_block_types(&mut arm.body, map);
            }
        }
        ExprKind::Array(items) => {
            for item in items {
                rewrite_expr_types(item, map);
            }
        }
        ExprKind::ArrayFill { value, count } => {
            rewrite_expr_types(value, map);
            rewrite_expr_types(count, map);
        }
        ExprKind::Index { target, index } => {
            rewrite_expr_types(target, map);
            rewrite_expr_types(index, map);
        }
        ExprKind::Unary { op: _, expr } | ExprKind::Await { expr } | ExprKind::Try(expr) => {
            rewrite_expr_types(expr, map)
        }
        ExprKind::Binary { left, op: _, right }
        | ExprKind::In {
            value: left,
            collection: right,
        } => {
            rewrite_expr_types(left, map);
            rewrite_expr_types(right, map);
        }
        ExprKind::Call { name: _, args }
        | ExprKind::Spawn {
            actor: _,
            args,
            supervise: _,
            bound: _,
        } => {
            for arg in args {
                rewrite_expr_types(arg, map);
            }
        }
        ExprKind::Tell {
            target,
            handler: _,
            args,
            kind: _,
        } => {
            rewrite_expr_types(target, map);
            for arg in args {
                rewrite_expr_types(arg, map);
            }
        }
        ExprKind::StructLiteral { name: _, fields } => {
            for (_, value) in fields {
                rewrite_expr_types(value, map);
            }
        }
        ExprKind::Field { target, field: _ } => rewrite_expr_types(target, map),
        ExprKind::Conditional {
            cond,
            then_branch,
            else_branch,
        } => {
            rewrite_expr_types(cond, map);
            rewrite_expr_types(then_branch, map);
            rewrite_expr_types(else_branch, map);
        }
        ExprKind::Slice { target, start, end } => {
            rewrite_expr_types(target, map);
            if let Some(start) = start {
                rewrite_expr_types(start, map);
            }
            if let Some(end) = end {
                rewrite_expr_types(end, map);
            }
        }
        ExprKind::Combinator { op: _, operand } => rewrite_expr_types(operand, map),
        // Leaves: no sub-expression and no type annotation.
        ExprKind::Integer(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::String(_)
        | ExprKind::Char(_)
        | ExprKind::Variable(_) => {}
    }
}
