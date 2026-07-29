//! Native codegen for closures — Stage 2 (**scalar completeness**: integer AND
//! float captures/parameters/returns, any parameter count, direct non-escaping
//! call) plus **Stage 3a/3c** (**non-escaping higher-order functions**: a closure
//! passed as an argument to another function that calls it, or relays it onward).
//! Split out of native_object.rs; sees the parent's items via `use super::*`.
//!
//! This file owns **layout and lowering** — the scalar subset, the capture analysis,
//! the `[code_ptr][captures…]` block, the indirect-call ABI, and the synthesized
//! `__closure_{id}` body. The **escape rules** that decide what may be lowered at all
//! — the default-deny use walker ([`closure_local_ok`], [`call_returned_callable_ok`],
//! [`closure_local_confined_to_iteration`]) and the module-wide higher-order sink
//! index ([`build_hof_index`]) — live in the sibling `native_object_closure_escape.rs`,
//! split out when the stage-3c sweep pushed this file past the size-backlog cap.
//!
//! A closure literal `fn PARAMS -> EXPR` lowers (in the interpreters) to a
//! `Value::Closure { id, captured }` whose body lives in `BytecodeModule::closures`
//! keyed by the parse-order `id`. The native backend compiles the **provably-sound
//! scalar slice** of that model:
//!
//! - The closure is created by a direct literal (`let f fn(...) = fn x -> ...`),
//!   captures only **native scalars** — an integer cell (`i64`/fixed-width/`bool`/
//!   `char`/`byte`) or a float (`f64`/`f32`) — takes any number of native-scalar
//!   parameters, returns a native scalar, and its single-expression body neither
//!   touches the heap nor calls a user/`extern` function.
//! - The closure local is used as the callee of a direct call (`f(args)`), as a bare
//!   argument passed to a **non-escaping higher-order sink** — a `fn(...)`-typed
//!   parameter of a known function that is itself call-only or passes the value onward
//!   only to further sinks (`apply(f, x)`, stage 3a; `outer(f, x)` → `inner(f, x)`,
//!   stage 3c; see [`build_hof_index`] and [`ClosureCallSig`]) — or **RETURNED** when it is a factory's
//!   own fresh literal-bound closure (stage 4a; see [`closure_local_ok`]'s return
//!   relaxation and [`returns_only_local_closure_literals`]). It is never reassigned,
//!   stored, read as a bare value, or passed to a non-sink position.
//!
//! A **higher-order callee** side is symmetric: a `fn(...)` parameter that is a
//! **sink** — never stored, returned, reassigned, or read as a value — is a single
//! pointer word holding a closure env block a caller passed in, and a call
//! `param(args)` through it reuses the identical indirect-call ABI as a
//! closure-local call ([`NativeCtx::indirect_callable_sig`] resolves either). The
//! parameter is non-escaping precisely so the closure never escapes the callee, which
//! is what keeps a caller's capture environment valid for the whole call.
//!
//! **Stage 3c — multi-level pass-onward.** A sink parameter may also be handed
//! ONWARD as a bare argument at another function's sink position (`outer(g, v)` calls
//! `inner(g, v)` which calls `g(v)`), to arbitrary depth. "Is a sink" is therefore a
//! mutually recursive whole-module property, resolved once per module by
//! [`build_hof_index`] as a memoized DFS with SCCs pre-poisoned to NOT-a-sink —
//! structurally the same sweep as the arena's `retaining_summary`, with the deny
//! default inverted. Every other use of a `fn` parameter (stored, returned,
//! reassigned, bare-read, passed to a non-sink position, or passed to an
//! `extern`/indirect/unknown callee) keeps it out of the index, so the holding
//! function skips cleanly (`L0339`).
//!
//! A **returned closure** (stage 4a) is the first *escaping* shape admitted: a factory
//! whose every return edge is a locally-created literal closure lowers, its `fn(...)`
//! return admitted as an `I64` block pointer; a caller binds the result to a
//! call-returned callable local ([`call_returned_callable_ok`]) and invokes it through
//! the same indirect ABI. The factory stays OFF the arena (a `fn`/heap return is
//! arena-ineligible), so the returned block is never reclaimed — sound without any
//! promotion (that is increment b). A returned fn PARAMETER (it aliases a caller's env)
//! and a returned call-returned closure stay refused.
//!
//! Everything else — a `string`/`list`/`map`/aggregate capture, a closure stored or
//! passed to a **non-sink** position, a mutable capture, a returned fn parameter, a
//! returned call-returned closure, a call-returned closure passed onward — makes the
//! enclosing function **skip cleanly to the interpreters** (`L0339`), never
//! miscompiled.
//!
//! A closure body is a **single expression** in the surface grammar (`expr_parser`
//! parses it with `parse_conditional`), so there is no block-bodied closure for
//! this backend to lower — the shape does not exist in the language. Its *lowering*
//! is not a single expression, though: the one intra-body control form the grammar
//! admits, the inline conditional `A if C else B`, desugars to a hoisted temporary
//! plus an `if`, carried in `BytecodeClosureDef::prelude` and run in the closure's
//! own frame by the interpreters. **This backend does not lower a prelude**, so a
//! ternary-bodied closure must skip: its body is a bare `#cond_N` reference, which
//! `ctx.local` cannot resolve, and the enclosing function falls back to the
//! interpreters (`L0339`). The `?` desugar cannot reach here at all — semantics
//! refuses `?` in a closure body (`L0462`).
//!
//! That skip is load-bearing, and it is **only** sound because the desugar temps
//! are unspellable. They are prefixed `#` (see `bytecode_vm::TEMP`), which the
//! lexer cannot produce, so no user binding can ever satisfy the `ctx.local`
//! lookup for one. When the temps were spelled `__cond_N` this guard was
//! *accidental* and a user could defeat it by declaring the name:
//!
//! ```text
//! let __cond_0 i64 = 555
//! let f fn(i64) -> i64 = fn x i64 -> 1 if x > 0 else 2
//! f(1) + __cond_0        # interpreters 556; native compiled the body,
//!                        # ignored the prelude, and exited 1110
//! ```
//!
//! The body's `__cond_0` resolved to the *user's* local, so this backend compiled a
//! closure it had always refused — while ignoring the prelude that defines the
//! value — and answered differently from all three interpreters. Compiling a body
//! whose prelude is dropped is exactly the outcome correct-or-refuse forbids; the
//! unspellable prefix makes that unreachable by construction rather than by luck.
//!
//! ## Object layout and call ABI
//!
//! A closure value is a heap block `[code_ptr][capture0][capture1]…]` allocated by
//! the shared bump/RC allocator (`__lullaby_alloc`), so it is reclaimed by the
//! arena rewind (a per-iteration sub-region for a non-escaping loop closure) or the
//! RC/free-list path exactly like any other heap block. Word 0 holds the address of
//! the synthesized closure-body function `__closure_{id}` (materialized with a
//! `lea rax,[rip+__closure_{id}]` + REL32 relocation); words 1.. hold the captured
//! scalar values in capture order — one raw 8-byte word each, whatever the class
//! (a float capture stores its raw IEEE-754 bits; an `f32` occupies the low four
//! bytes of its word, and every reader loads it with `movss`, so the undefined high
//! four bytes are never observed).
//!
//! A direct call `f(args)` puts the **env pointer** (the block base) in `rcx`,
//! then places each argument at its Win64 **effective position** `i + 1` — the env
//! pointer is the hidden first argument, exactly as an aggregate return's hidden
//! `sret` pointer is elsewhere in this backend, so the two share one staging and
//! distribution path ([`emit_native_call_args_with`]). Positions 1..3 land in
//! `rdx`/`r8`/`r9` (integer) or `xmm1`/`xmm2`/`xmm3` (float) — **positionally**, so
//! a float at position 2 takes `xmm2`, never "the next unused XMM"; positions 4+
//! spill to the outgoing stack area. The call then loads word 0 (the code pointer)
//! into `rax` and issues an indirect `call rax`. The synthesized body seats `rcx`
//! (env) and its parameters into frame slots by the mirror-image rule; a captured
//! name resolves to `[env + 8*(k+1)]`.

use super::*;

/// The `.text` symbol name of the synthesized body for closure `id`.
pub(crate) fn closure_symbol(id: usize) -> String {
    format!("__closure_{id}")
}

/// Whether a source type is an integer-cell native scalar — `i64`, a fixed-width
/// integer (`i8`…`usize`, stored as a normalized `i64` cell), or `bool`/`char`/
/// `byte`.
pub(crate) fn is_i64_cell_scalar(ty: &TypeRef) -> bool {
    let name = ty.name.as_str();
    name == "i64" || fixed_int_kind(name).is_some() || matches!(name, "bool" | "char" | "byte")
}

/// Classify a source type as a **native closure scalar** — the capture, parameter,
/// and return types this backend can lower — or `None` for anything else (a heap
/// value `string`/`list`/`map`, an aggregate, a nested `fn(...)`), which makes the
/// enclosing function skip cleanly (`L0339`).
///
/// Exactly one 8-byte word each: an integer cell normalizes into an `i64` word; a
/// float keeps its raw IEEE-754 bits (an `f32` in the low four bytes). This is the
/// single place the scalar subset is defined, so the layout, the literal store, the
/// call ABI, and the body prologue can never disagree about a type's class.
pub(crate) fn native_closure_scalar(ty: &TypeRef) -> Option<NativeType> {
    if is_i64_cell_scalar(ty) {
        return Some(NativeType::I64);
    }
    match FloatWidth::from_type_name(ty.name.as_str()) {
        Some(FloatWidth::F64) => Some(NativeType::F64),
        Some(FloatWidth::F32) => Some(NativeType::F32),
        None => None,
    }
}

/// The [`FloatWidth`] of a native closure scalar, or `None` when it is an integer
/// cell. Drives the register class (GPR vs XMM) at every ABI boundary.
pub(crate) fn closure_scalar_float_width(ty: &NativeType) -> Option<FloatWidth> {
    match ty {
        NativeType::F64 => Some(FloatWidth::F64),
        NativeType::F32 => Some(FloatWidth::F32),
        _ => None,
    }
}

/// The resolved native layout of a closure: its captured free variables (name +
/// scalar class, in capture order) and its parameter/return layouts. A closure with
/// any non-supported piece has no layout (`compute_closure_layout` returns `None`),
/// so an enclosing function referencing it skips gracefully.
#[derive(Debug, Clone)]
pub(crate) struct ClosureLayout {
    /// Captured free variables in capture (first-seen) order: `(name, class)`.
    /// Each is a single raw word stored at env offset `8*(index+1)`.
    pub(crate) captures: Vec<(String, NativeType)>,
    /// The closure's parameters in order: `(name, class)`. Any count — positions
    /// past the three register slots left by the env pointer spill to the stack.
    pub(crate) params: Vec<(String, NativeType)>,
    /// The closure's return class (a native scalar: integer cell in `rax`, or a
    /// float in `xmm0`).
    pub(crate) ret: NativeType,
}

/// A closure body's env binding while it is being lowered: the frame slot holding
/// the env pointer (block base; word 0 is the code pointer, captures follow) and
/// each captured name's byte offset within the env block plus its scalar class
/// (the class picks the GPR vs XMM load, so a float capture can never be read
/// through the integer path).
pub(crate) struct ClosureEnv {
    pub(crate) env_slot: i32,
    pub(crate) captures: HashMap<String, (i32, NativeType)>,
}

/// Collect a closure body's **free variables** — the `Variable` reads that are not
/// one of the closure's own parameters — in first-seen (deterministic pre-order)
/// order, each paired with its static type. The body is a single expression with
/// no inner bindings, so every non-parameter variable is a captured enclosing
/// local. Order fixes both the heap block layout and the env offsets, and is shared
/// by the literal lowering (which stores captures) and the body synthesis (which
/// reads them), so they always agree.
fn free_variables(body: &BytecodeExpr, params: &[String]) -> Vec<(String, TypeRef)> {
    fn visit(
        expr: &BytecodeExpr,
        params: &[String],
        seen: &mut std::collections::HashSet<String>,
        out: &mut Vec<(String, TypeRef)>,
    ) {
        if let BytecodeExprKind::Variable(name) = &expr.kind
            && !params.contains(name)
            && seen.insert(name.clone())
        {
            out.push((name.clone(), expr.ty.clone()));
        }
        // Recursing over `expr_children` never descends into a `Closure` node (it
        // carries only an id), so a body containing a nested closure contributes no
        // free variables here and is rejected later by the body-lowering trial.
        for child in expr_children(expr) {
            visit(child, params, seen, out);
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    visit(body, params, &mut seen, &mut out);
    out
}

/// Compute the native layout of a closure definition, or `None` if any piece is
/// outside the supported scalar slice (a heap/aggregate capture, parameter, or
/// return). The body's lowerability (heap-free, no user calls, actually compilable)
/// is verified separately by [`synthesize_closure_body`]; a failure there demotes
/// the enclosing function, so a layout existing here does not on its own promise
/// the body compiles.
pub(crate) fn compute_closure_layout(
    def: &BytecodeClosureDef,
    fn_signature: &(Vec<TypeRef>, TypeRef),
) -> Option<ClosureLayout> {
    let (param_types, ret_ty) = fn_signature;
    // The literal's static `fn(...)` type must agree with the def's parameter
    // names; a mismatch means the two were built from different nodes, so refuse.
    if def.params.len() != param_types.len() {
        return None;
    }
    // Parameters: every one a native scalar. Any COUNT is fine — the env pointer
    // takes the first integer register slot and positions past the fourth spill to
    // the outgoing stack area, exactly as an ordinary call's 5th+ argument does.
    let mut params = Vec::new();
    for (name, ty) in def.params.iter().zip(param_types.iter()) {
        params.push((name.clone(), native_closure_scalar(ty)?));
    }
    // Return: a native scalar (an integer cell in `rax`, a float in `xmm0`).
    let ret = native_closure_scalar(ret_ty)?;
    // Captures: every free variable must be a native scalar.
    let mut captures = Vec::new();
    for (name, ty) in free_variables(&def.body, &def.params) {
        captures.push((name, native_closure_scalar(&ty)?));
    }
    Some(ClosureLayout {
        captures,
        params,
        ret,
    })
}

/// Whether **every** value `function` returns is a locally-created closure literal —
/// a direct `fn ... ` literal, or a local bound to one. This is the tight admission
/// gate for treating a `fn(...)` RETURN type as native (a scalar block-pointer word):
/// a returned closure literal born in this function is a fresh heap block the factory
/// owns and does not reclaim (the factory is kept off the arena), so the caller can
/// invoke it safely. A returned fn PARAMETER (it aliases a caller's env) or a returned
/// call-returned closure (increment b's promotion territory) is NOT a fresh local
/// literal, so this returns `false` and the function is refused — it skips cleanly to
/// the interpreters rather than risking a dangling or aliased env.
///
/// Default-deny: a function with no discernible return edge, or any return edge that
/// is not a literal / literal-bound local, fails. The narrower "flat, scalar-capture"
/// property is enforced downstream (a non-scalar-capture literal has no
/// [`ClosureLayout`], so `collect_native_locals` / `lower_closure_literal` /
/// `synthesize_closure_body` demote the function) — so admitting the shape here never
/// risks a miscompile, only a later skip.
pub(crate) fn returns_only_local_closure_literals(function: &BytecodeFunction) -> bool {
    let mut literal_locals = std::collections::HashSet::new();
    collect_closure_literal_lets(&function.instructions, &mut literal_locals);
    let mut edges: Vec<&BytecodeExpr> = Vec::new();
    collect_return_value_exprs(&function.instructions, &mut edges);
    if edges.is_empty() {
        return false;
    }
    edges.iter().all(|e| match &e.kind {
        BytecodeExprKind::Closure { .. } => true,
        BytecodeExprKind::Variable(n) => literal_locals.contains(n),
        _ => false,
    })
}

/// The eligibility of admitting `function`'s `fn(...)` RETURN as a native `I64`
/// block-pointer word (a factory returning a closure), or an `Err` reason to refuse
/// (skip cleanly, `L0339`). Tight gate: the returned fn signature is all native scalars
/// AND every return edge is a locally-created closure literal — a returned fn PARAMETER
/// (aliases a caller's env) or a call-returned closure (increment b) is refused. Only
/// call this when `function.return_type.is_function()`.
pub(crate) fn native_fn_return_eligibility(function: &BytecodeFunction) -> Result<(), String> {
    if native_fn_call_sig(&function.return_type).is_none() {
        return Err(format!(
            "returned fn type `{}` has a non-native-scalar signature",
            function.return_type.name
        ));
    }
    if !returns_only_local_closure_literals(function) {
        return Err(
            "a `fn(...)` return is native only when every return edge is a locally-created \
             closure literal; a returned fn parameter or call-returned closure is deferred"
                .to_string(),
        );
    }
    Ok(())
}

/// Collect every local `name` bound anywhere in `body` by `let name = <closure
/// literal>`, so [`returns_only_local_closure_literals`] can tell a returned
/// literal-bound local from a returned fn parameter or factory result.
fn collect_closure_literal_lets(
    body: &[BytecodeInstruction],
    out: &mut std::collections::HashSet<String>,
) {
    for stmt in body {
        if let BytecodeInstruction::Let { name, value, .. } = stmt
            && matches!(value.kind, BytecodeExprKind::Closure { .. })
        {
            out.insert(name.clone());
        }
        for nested in stmt_nested_bodies(stmt) {
            collect_closure_literal_lets(nested, out);
        }
    }
}

/// Collect the expressions in the function's **return-value** positions: every
/// explicit `return <e>`, plus a tail expression statement whose value is a `fn(...)`
/// (the implicit return of a factory whose body ends in a closure). A tail expression
/// that is not fn-typed (e.g. a loop body's tail) is ignored, so a factory with a
/// loop is not spuriously refused.
pub(crate) fn collect_return_value_exprs<'a>(
    body: &'a [BytecodeInstruction],
    out: &mut Vec<&'a BytecodeExpr>,
) {
    for stmt in body {
        if let BytecodeInstruction::Return(Some(e)) = stmt {
            out.push(e);
        }
        for nested in stmt_nested_bodies(stmt) {
            collect_return_value_exprs(nested, out);
        }
    }
    if let Some(BytecodeInstruction::Expr(e)) = body.last()
        && e.ty.is_function()
    {
        out.push(e);
    }
}

/// The nested statement bodies of a statement (branch/loop/match-arm/try bodies),
/// for the two return-edge collectors above. Carries no expression children.
pub(crate) fn stmt_nested_bodies(stmt: &BytecodeInstruction) -> Vec<&[BytecodeInstruction]> {
    match stmt {
        BytecodeInstruction::If {
            branches,
            else_body,
            ..
        } => {
            let mut v: Vec<&[BytecodeInstruction]> = branches.iter().map(|b| &b.body[..]).collect();
            v.push(else_body);
            v
        }
        BytecodeInstruction::While { body, .. }
        | BytecodeInstruction::For { body, .. }
        | BytecodeInstruction::Loop { body, .. }
        | BytecodeInstruction::RegionBlock { body, .. } => vec![body],
        BytecodeInstruction::Match { arms, .. } => arms.iter().map(|a| &a.body[..]).collect(),
        BytecodeInstruction::Try {
            body, catch_body, ..
        } => vec![body, catch_body],
        _ => Vec::new(),
    }
}

/// Collect every closure `id` referenced by a `Closure { id }` literal anywhere in
/// a function body, in deterministic order. Used after the main lowering fixpoint
/// to synthesize exactly the closure bodies the compiled functions reference.
pub(crate) fn referenced_closure_ids(function: &BytecodeFunction) -> Vec<usize> {
    let mut ids = Vec::new();
    fn visit_expr(expr: &BytecodeExpr, ids: &mut Vec<usize>) {
        if let BytecodeExprKind::Closure { id } = &expr.kind
            && !ids.contains(id)
        {
            ids.push(*id);
        }
        for child in expr_children(expr) {
            visit_expr(child, ids);
        }
    }
    fn visit_body(body: &[BytecodeInstruction], ids: &mut Vec<usize>) {
        for stmt in body {
            match stmt {
                BytecodeInstruction::Let { value, .. }
                | BytecodeInstruction::Assign { value, .. }
                | BytecodeInstruction::Return(Some(value))
                | BytecodeInstruction::Expr(value)
                | BytecodeInstruction::Throw { value, .. } => visit_expr(value, ids),
                BytecodeInstruction::If {
                    branches,
                    else_body,
                    ..
                } => {
                    for b in branches {
                        visit_expr(&b.condition, ids);
                        visit_body(&b.body, ids);
                    }
                    visit_body(else_body, ids);
                }
                BytecodeInstruction::While {
                    condition, body, ..
                } => {
                    visit_expr(condition, ids);
                    visit_body(body, ids);
                }
                BytecodeInstruction::For {
                    start,
                    end,
                    step,
                    body,
                    ..
                } => {
                    visit_expr(start, ids);
                    visit_expr(end, ids);
                    if let Some(s) = step {
                        visit_expr(s, ids);
                    }
                    visit_body(body, ids);
                }
                BytecodeInstruction::Loop { body, .. } => visit_body(body, ids),
                BytecodeInstruction::Match {
                    scrutinee, arms, ..
                } => {
                    visit_expr(scrutinee, ids);
                    for arm in arms {
                        visit_body(&arm.body, ids);
                    }
                }
                BytecodeInstruction::Try {
                    body, catch_body, ..
                } => {
                    visit_body(body, ids);
                    visit_body(catch_body, ids);
                }
                _ => {}
            }
        }
    }
    visit_body(&function.instructions, &mut ids);
    ids
}

/// Compute the native layout of every Stage-1-supported closure in the module,
/// keyed by parse-order `id`. Each closure literal node carries its full
/// `fn(param types) -> R` static type, so the parameter/return types are read from
/// the literal (the `BytecodeClosureDef` stores only parameter *names* and the
/// body). A closure whose literal type is missing/malformed or whose layout is
/// outside the Stage-1 subset simply gets no entry, so any function binding it
/// skips cleanly.
pub(crate) fn compute_module_closure_layouts(
    module: &BytecodeModule,
) -> HashMap<usize, ClosureLayout> {
    // id -> (param types, return type), read from each `Closure { id }` literal's
    // static function type across every function body.
    let mut types: HashMap<usize, (Vec<TypeRef>, TypeRef)> = HashMap::new();
    for function in &module.functions {
        collect_closure_types_in_body(&function.instructions, &mut types);
    }
    let mut layouts = HashMap::new();
    for def in &module.closures {
        let Some(signature) = types.get(&def.id) else {
            continue;
        };
        if let Some(layout) = compute_closure_layout(def, signature) {
            layouts.insert(def.id, layout);
        }
    }
    layouts
}

fn collect_closure_types_in_body(
    body: &[BytecodeInstruction],
    out: &mut HashMap<usize, (Vec<TypeRef>, TypeRef)>,
) {
    for stmt in body {
        match stmt {
            BytecodeInstruction::Let { value, .. }
            | BytecodeInstruction::Assign { value, .. }
            | BytecodeInstruction::Return(Some(value))
            | BytecodeInstruction::Expr(value)
            | BytecodeInstruction::Throw { value, .. } => collect_closure_types_in_expr(value, out),
            BytecodeInstruction::If {
                branches,
                else_body,
                ..
            } => {
                for b in branches {
                    collect_closure_types_in_expr(&b.condition, out);
                    collect_closure_types_in_body(&b.body, out);
                }
                collect_closure_types_in_body(else_body, out);
            }
            BytecodeInstruction::While {
                condition, body, ..
            } => {
                collect_closure_types_in_expr(condition, out);
                collect_closure_types_in_body(body, out);
            }
            BytecodeInstruction::For {
                start,
                end,
                step,
                body,
                ..
            } => {
                collect_closure_types_in_expr(start, out);
                collect_closure_types_in_expr(end, out);
                if let Some(s) = step {
                    collect_closure_types_in_expr(s, out);
                }
                collect_closure_types_in_body(body, out);
            }
            BytecodeInstruction::Loop { body, .. } => collect_closure_types_in_body(body, out),
            BytecodeInstruction::Match {
                scrutinee, arms, ..
            } => {
                collect_closure_types_in_expr(scrutinee, out);
                for arm in arms {
                    collect_closure_types_in_body(&arm.body, out);
                }
            }
            BytecodeInstruction::Try {
                body, catch_body, ..
            } => {
                collect_closure_types_in_body(body, out);
                collect_closure_types_in_body(catch_body, out);
            }
            _ => {}
        }
    }
}

fn collect_closure_types_in_expr(
    expr: &BytecodeExpr,
    out: &mut HashMap<usize, (Vec<TypeRef>, TypeRef)>,
) {
    if let BytecodeExprKind::Closure { id } = &expr.kind
        && let Some(signature) = expr.ty.function_signature()
    {
        out.entry(*id).or_insert(signature);
    }
    for child in expr_children(expr) {
        collect_closure_types_in_expr(child, out);
    }
}

// -- Closure literal + call lowering (inside an enclosing function) -----------

/// Lower a closure literal `Closure { id }` in binding position: allocate the
/// `[code_ptr][captures…]` heap block, store the code pointer (a
/// `lea rax,[rip+__closure_{id}]` REL32 relocation) and each captured scalar, and
/// leave the block pointer in `rax`. The captures are read from the enclosing
/// function's frame locals of the same name.
///
/// Each capture is copied as one **raw 8-byte word** through `rax`, whatever its
/// class: an integer cell is its own word, an `f64` its full bit pattern, an `f32`
/// its meaningful low four bytes (the high four are undefined but never read — the
/// body loads an `f32` capture with `movss`). So one GPR copy serves every scalar
/// class and no XMM round-trip is needed here.
///
/// The layout's class for each capture came from the IR `Variable` node's static
/// type; this re-derives the class from the enclosing frame local's resolved
/// `NativeType` and refuses on any disagreement, so a capture can never be stored
/// under one class and read back under another.
pub(crate) fn lower_closure_literal(
    ctx: &mut NativeCtx,
    id: usize,
    code: &mut Vec<u8>,
) -> Result<(), String> {
    let layout = ctx
        .closure_layouts
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("closure #{id} is not in the native closure subset"))?;
    let word_count = 1 + layout.captures.len();

    // Allocate the block: `mov rcx, word_count*8 ; call __lullaby_alloc` → rax = block.
    let byte_size = (word_count as i64) * 8;
    emit_mov_rax_imm(code, byte_size);
    code.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
    emit_call_symbol(ctx, HEAP_ALLOC_SYMBOL, code);

    // Save the block pointer into a scratch slot so it survives the capture loads.
    let saved_scratch = ctx.scratch_next;
    let blk_slot = ctx.alloc_scratch(1);
    store_local(code, blk_slot); // mov [rbp - blk_slot], rax

    // Word 0 = code pointer: `lea rax,[rip+__closure_{id}]` (REL32), then store.
    code.extend_from_slice(&[0x48, 0x8D, 0x05]); // lea rax, [rip + disp32]
    let site = code.len();
    code.extend_from_slice(&[0, 0, 0, 0]);
    ctx.relocations.push(CodeRelocation {
        offset: site as u32,
        symbol: closure_symbol(id),
    });
    emit_mov_rcx_from_slot(code, blk_slot); // mov rcx, [rbp - blk_slot] (block base)
    code.extend_from_slice(&[0x48, 0x89, 0x01]); // mov [rcx], rax  (word 0 = code ptr)

    // Words 1.. = captured scalars, in capture order.
    for (index, (cap_name, cap_class)) in layout.captures.iter().enumerate() {
        let local = ctx
            .local(cap_name)
            .map_err(|_| {
                format!("closure #{id} captures `{cap_name}`, which is not a native local")
            })?
            .clone();
        // Cross-check the layout's class (derived from the IR node's static type)
        // against the enclosing local's resolved layout. They must agree, or the
        // env word would be written as one class and read as another — a silent
        // reinterpretation of the bits. Refuse instead, so the function skips.
        if &local.ty != cap_class {
            ctx.scratch_next = saved_scratch;
            return Err(format!(
                "closure #{id} captures `{cap_name}` typed {:?} in the closure layout but \
                 {:?} in the enclosing frame; refusing to reinterpret the capture",
                cap_class, local.ty
            ));
        }
        let cap_slot = local.slot;
        // A closure-using function is excluded from register promotion, so the
        // capture always lives in its frame slot. Guard against a stray promotion.
        if ctx.promoted_reg(cap_slot).is_some() {
            ctx.scratch_next = saved_scratch;
            return Err(format!(
                "closure #{id} captures promoted local `{cap_name}`; unsupported"
            ));
        }
        load_local(code, cap_slot); // mov rax, [rbp - cap_slot] (captured value)
        emit_mov_rcx_from_slot(code, blk_slot); // mov rcx, [rbp - blk_slot]
        let offset = ((index + 1) * 8) as i32;
        // mov [rcx + offset], rax
        code.extend_from_slice(&[0x48, 0x89, 0x81]);
        code.extend_from_slice(&offset.to_le_bytes());
    }

    // Result: the block pointer.
    load_local(code, blk_slot); // mov rax, [rbp - blk_slot]
    ctx.scratch_next = saved_scratch;
    Ok(())
}

/// Lower a direct call `name(args)` where `name` is a closure-bound local, leaving
/// the result in `rax` (an integer-cell return) or `xmm0` (a float return) and
/// reporting the return class.
///
/// The **env pointer is the hidden first argument**, structurally identical to the
/// hidden `sret` pointer an aggregate-returning call passes: it takes effective
/// register position 0 (`rcx`) and shifts every visible argument to position
/// `i + 1`. That is why this routes through [`emit_native_call_args_with`] — the
/// same staging and distribution the whole backend uses — rather than a parallel
/// closure-only sequence. Consequences that fall out of sharing it, all of which
/// this backend previously got wrong or could not express:
///
/// - **Float register positions are positional, not sequential.** Argument `i` at
///   effective position `i + 1` takes `xmm{i+1}` when it is a float, so the first
///   parameter of `fn a i64 b f64 -> …` is `rdx` (position 1) and `b` is **`xmm2`**
///   (position 2) — *not* `xmm0`/`xmm1`. Win64 pairs each XMM with the GPR of the
///   same index and consumes both, so "the next unused XMM" is the classic
///   silent-corruption bug here.
/// - **A 4th parameter is the 5th argument**, so it spills to the outgoing stack
///   area at `[rsp + 32 + 8*(pos-4)]` (above the 32-byte shadow space) — reserved
///   by the caller's frame via `max_outgoing_stack_words`, which counts the hidden
///   env word for exactly this reason.
///
/// The env pointer is placed into `rcx` last (after the staging words are
/// discarded), so `mov rax,[rcx]` then reads the code pointer at word 0 with the
/// env still live in `rcx` — where the callee expects it — and `call rax` clobbers
/// no argument register.
pub(crate) fn lower_closure_call(
    ctx: &mut NativeCtx,
    name: &str,
    args: &[BytecodeExpr],
    code: &mut Vec<u8>,
) -> Result<NativeType, String> {
    let (params, ret) = ctx
        .indirect_callable_sig(name)
        .ok_or_else(|| format!("`{name}` is not an indirect callable (closure/fn pointer)"))?;
    if args.len() != params.len() {
        return Err(format!(
            "indirect callable `{name}` expects {} argument(s) but got {}",
            params.len(),
            args.len()
        ));
    }
    // The env pointer lives in the callable's own frame slot — for a closure local
    // the block the literal allocated, for a fn-typed parameter the block pointer a
    // caller passed in. Both cases exclude the enclosing function from register
    // promotion, so the slot is always addressable.
    let env_slot = ctx.local_slot(name)?;
    if ctx.promoted_reg(env_slot).is_some() {
        return Err(format!(
            "indirect callable `{name}` was register-promoted; the env pointer must stay \
             addressable in its frame slot"
        ));
    }
    let param_tys: Vec<Option<NativeType>> = params.iter().map(|t| Some(t.clone())).collect();
    emit_native_call_args_with(
        ctx,
        &param_tys,
        args,
        Some(HiddenArg::ClosureEnv(env_slot)),
        code,
    )?;
    // rax = [rcx] (code pointer at word 0), then `call rax`. `rcx` holds the env
    // pointer, which is also what the callee reads as its hidden first argument.
    code.extend_from_slice(&[0x48, 0x8B, 0x01]); // mov rax, [rcx]
    code.extend_from_slice(&[0xFF, 0xD0]); // call rax
    Ok(ret)
}

impl NativeCtx<'_> {
    /// The `(parameter classes, return class)` of an **indirect callable** local
    /// `name` — either a closure-bound local (a closure literal created in this
    /// function) or a call-only `fn(...)`-typed parameter (a closure passed in by a
    /// caller) — or `None` when `name` is neither. Both hold a closure
    /// `[code_ptr][captures…]` block pointer in their frame slot and are invoked
    /// through it with the identical ABI, so this single accessor lets one call
    /// lowering serve both. The classes drive the register file at every ABI
    /// boundary (a float return in `xmm0`, an integer cell in `rax`).
    pub(crate) fn indirect_callable_sig(
        &self,
        name: &str,
    ) -> Option<(Vec<NativeType>, NativeType)> {
        if let Some(id) = self.closure_locals.get(name) {
            let layout = self.closure_layouts.get(id)?;
            let params = layout.params.iter().map(|(_, t)| t.clone()).collect();
            return Some((params, layout.ret.clone()));
        }
        if let Some(sig) = self.fn_param_callables.get(name) {
            return Some((sig.params.clone(), sig.ret.clone()));
        }
        // A call-returned callable local (`let g fn(...) = factory(...)`) holds a
        // closure block pointer a factory produced. It is invoked through the identical
        // ABI as a closure local / fn parameter — env pointer in `rcx`, code pointer at
        // word 0 — so one call lowering serves all three; its signature comes from the
        // binding's declared `fn(...)` type.
        if let Some(sig) = self.call_returned_callables.get(name) {
            return Some((sig.params.clone(), sig.ret.clone()));
        }
        None
    }

    /// Whether `name` is an indirect callable (a closure local, a call-only fn
    /// parameter, or a call-returned callable local). A `Call` whose callee name is one
    /// lowers as an indirect call through the block's code pointer, not a
    /// direct/builtin/extern call.
    pub(crate) fn is_indirect_callable(&self, name: &str) -> bool {
        self.closure_locals.contains_key(name)
            || self.fn_param_callables.contains_key(name)
            || self.call_returned_callables.contains_key(name)
    }
}

// -- Per-register-file closure hooks ------------------------------------------
//
// The integer lowerer (`native_object_expr.rs`) and the float lowerer
// (`native_object_lowering.rs`) resolve a value through different register files, so
// each needs its own env-capture and closure-call branch. All four live HERE, next to
// the layout they read, and the two lowerers call in — one closure ABI and one place
// that knows the env block's shape, with no placement logic duplicated per file.
//
// Each pair is deliberately symmetric: the capture hook returns `Ok(None)` when the
// name is not a capture of ITS class (so the lowerer falls through to its ordinary
// frame-slot path), and the call hook refuses a return of the other class rather than
// reading the wrong register — that refusal is what stops a float's bits from being
// silently reinterpreted as an integer, or vice versa.

/// Load a captured **integer cell** `name` into `rax` from the env block, or
/// `Ok(false)` when `name` is not a capture, so the integer lowerer falls through to
/// its ordinary frame-slot path.
///
/// `mov rax, [rbp - env_slot]` (env pointer) then `mov rax, [rax + offset]`. A FLOAT
/// capture reaching here is refused: handing back its raw IEEE-754 bits as an integer
/// would be a silent reinterpretation. (A float capture in a float context is resolved
/// by [`lower_closure_float_capture`].)
pub(crate) fn lower_closure_int_capture(
    ctx: &mut NativeCtx,
    name: &str,
    code: &mut Vec<u8>,
) -> Result<bool, String> {
    let Some(env) = ctx.closure_env.as_ref() else {
        return Ok(false);
    };
    let Some((offset, class)) = env.captures.get(name) else {
        return Ok(false);
    };
    if !matches!(class, NativeType::I64) {
        return Err(format!(
            "captured `{name}` is a float; it cannot be read as an integer cell"
        ));
    }
    let (offset, env_slot) = (*offset, env.env_slot);
    load_local(code, env_slot); // mov rax, [rbp - env_slot] (env ptr)
    code.extend_from_slice(&[0x48, 0x8B, 0x80]); // mov rax, [rax + offset]
    code.extend_from_slice(&offset.to_le_bytes());
    Ok(true)
}

/// Lower an **integer-cell-returning** closure call `name(args)`, leaving the result
/// in `rax`. Placement is the shared closure ABI ([`lower_closure_call`]); this only
/// asserts the return really is an integer cell, so a float-returning closure (whose
/// value is in `xmm0`) cannot have `rax`'s garbage read out of it.
pub(crate) fn lower_closure_int_call(
    ctx: &mut NativeCtx,
    name: &str,
    args: &[BytecodeExpr],
    code: &mut Vec<u8>,
) -> Result<(), String> {
    let ret = lower_closure_call(ctx, name, args, code)?;
    if !matches!(ret, NativeType::I64) {
        return Err(format!(
            "closure `{name}` returns a float (in xmm0); it cannot be used \
             where an integer cell is expected"
        ));
    }
    Ok(())
}

/// `movs{d,s} xmm0, [rcx + disp32]` — load a float at a constant byte offset from the
/// pointer in `rcx`. Reads a captured float out of a closure's env block
/// (`[env + 8*(k+1)]`). ModRM 0x81 = `[rcx + disp32]`, reg 0. `movss` reads only the
/// low four bytes, so an `f32` capture never observes its word's undefined high half.
fn load_float_from_rcx_disp(code: &mut Vec<u8>, width: FloatWidth, disp: i32) {
    let prefix = match width {
        FloatWidth::F64 => 0xF2,
        FloatWidth::F32 => 0xF3,
    };
    code.extend_from_slice(&[prefix, 0x0F, 0x10, 0x81]);
    code.extend_from_slice(&disp.to_le_bytes());
}

/// The [`FloatWidth`] of a captured float `name` inside a synthesized closure body,
/// or `None` when `name` is not a capture or is an integer cell. Used by
/// `float_width_of_expr` to classify a capture before any code is emitted.
pub(crate) fn closure_env_float_width(ctx: &NativeCtx, name: &str) -> Option<FloatWidth> {
    let (_, class) = ctx.closure_env.as_ref()?.captures.get(name)?;
    closure_scalar_float_width(class)
}

/// The [`FloatWidth`] an indirect callable `name` (a closure local or a call-only
/// fn parameter) returns, or `None` when it returns an integer cell (or is not an
/// indirect callable).
pub(crate) fn closure_call_float_width(ctx: &NativeCtx, name: &str) -> Option<FloatWidth> {
    let (_, ret) = ctx.indirect_callable_sig(name)?;
    closure_scalar_float_width(&ret)
}

/// Load a captured **float** `name` into `xmm0` from the env block, reporting its
/// width — or `Ok(None)` when `name` is not a float capture, so the float lowerer
/// falls through to its ordinary frame-slot path.
///
/// `mov rcx, [rbp - env_slot]` (env pointer) then `movs{d,s} xmm0, [rcx + offset]`.
/// An `f32` is loaded with `movss`, reading only the meaningful low four bytes of
/// its env word, so the undefined high four bytes the literal's raw 8-byte capture
/// store may have written are never observed.
pub(crate) fn lower_closure_float_capture(
    ctx: &mut NativeCtx,
    name: &str,
    code: &mut Vec<u8>,
) -> Result<Option<FloatWidth>, String> {
    let Some(env) = ctx.closure_env.as_ref() else {
        return Ok(None);
    };
    let Some((offset, class)) = env.captures.get(name) else {
        return Ok(None);
    };
    let Some(width) = closure_scalar_float_width(class) else {
        // An integer-cell capture in a float context: not a float value. Report
        // "not a float" rather than reinterpreting the word's bits.
        return Ok(None);
    };
    let (offset, env_slot) = (*offset, env.env_slot);
    emit_mov_rcx_from_slot(code, env_slot); // rcx = env pointer (block base)
    load_float_from_rcx_disp(code, width, offset); // xmm0 = [rcx + offset]
    Ok(Some(width))
}

/// Lower a **float-returning** closure call `name(args)` in float position, leaving
/// the result in `xmm0`. Placement is the shared closure ABI ([`lower_closure_call`]);
/// this only asserts the return really is a float, so an integer-returning closure
/// used in a float context is refused rather than having `rax`'s bits read out of
/// `xmm0`.
pub(crate) fn lower_closure_float_call(
    ctx: &mut NativeCtx,
    name: &str,
    args: &[BytecodeExpr],
    code: &mut Vec<u8>,
) -> Result<FloatWidth, String> {
    let ret = lower_closure_call(ctx, name, args, code)?;
    closure_scalar_float_width(&ret).ok_or_else(|| {
        format!("closure `{name}` returns an integer cell; it cannot be used as a float")
    })
}

// -- Closure body synthesis ---------------------------------------------------

/// Synthesize the native `.text` body of a closure definition: a function
/// `__closure_{id}` receiving the env pointer in `rcx` and its parameters at
/// effective Win64 positions 1.., seating them into frame slots, resolving each
/// captured name to `[env + 8*(k+1)]`, and returning the single-expression body's
/// scalar value in `rax` (integer cell) or `xmm0` (float).
///
/// The prologue is the exact mirror of [`lower_closure_call`]'s placement, and of
/// the ordinary function prologue in `native_object_stmt.rs`: parameter `i` sits at
/// effective position `i + 1` (the env pointer is the hidden position 0), so
/// positions 1..3 arrive in `rdx`/`r8`/`r9` or `xmm1`/`xmm2`/`xmm3` **by position
/// and class**, and positions 4+ arrive on the caller's stack.
///
/// The stack-argument displacement is fixed by the entry sequence: on entry the
/// return address is at `[rsp]`; after `push rbp; mov rbp,rsp` the saved `rbp` is
/// at `[rbp]`, the return address at `[rbp+8]`, the caller's 32-byte shadow at
/// `[rbp+16..rbp+48]`, and the first stack argument at `[rbp+48]`. So effective
/// position `pos >= 4` reads `[rbp + 48 + 8*(pos-4)]` — which is exactly the word
/// the caller wrote to `[rsp + 32 + 8*(pos-4)]` before the `call`.
///
/// The body must be heap-free and free of user/`extern` calls (so the closure
/// allocates nothing and retains no pointer — keeping the enclosing arena reasoning
/// a true leaf); a violation returns `Err`, demoting the enclosing function.
#[allow(clippy::too_many_arguments)]
pub(crate) fn synthesize_closure_body(
    def: &BytecodeClosureDef,
    layout: &ClosureLayout,
    callable: &std::collections::HashSet<&str>,
    extern_sigs: &HashMap<&str, &crate::IrExternSignature>,
    structs: &[IrStructDef],
    enums: &[IrEnumDef],
    strings: &mut StringPool,
    signatures: &HashMap<String, NativeSignature>,
    closure_layouts: &HashMap<usize, ClosureLayout>,
) -> Result<LoweredNativeFunction, String> {
    // Default-deny body shape: heap-free (no allocation, so nothing to reclaim
    // inside the closure) and free of user/`extern` calls (a leaf w.r.t. user code,
    // so the enclosing arena function stays a true leaf). Inline scalar builtins are
    // fine (they emit no `call`).
    let empty_aggs = std::collections::HashSet::new();
    if expr_touches_heap(&def.body, &empty_aggs) {
        return Err(format!(
            "closure #{} body touches the heap; native closures are scalar-only",
            def.id
        ));
    }
    let user_names: std::collections::HashSet<&str> = callable.iter().copied().collect();
    if expr_calls_user(&def.body, &user_names) {
        return Err(format!(
            "closure #{} body calls a user/extern function; deferred",
            def.id
        ));
    }

    // -- Plan the frame: env pointer slot + parameter slots + a small scratch. ----
    let mut locals: HashMap<String, NativeLocal> = HashMap::new();
    let mut next_slot: i32 = 0;
    // Env pointer (word 0 = code ptr; captures at 8*(k+1)).
    next_slot += 8;
    let env_slot = next_slot;
    // Parameters (native scalars), one word each, in order. The slot's `NativeType`
    // is the parameter's real class, so the body's float path finds an `F64`/`F32`
    // local and the integer path an `I64` one — the two can never cross.
    for (pname, pclass) in &layout.params {
        next_slot += 8;
        locals.insert(
            pname.clone(),
            NativeLocal {
                slot: next_slot,
                ty: pclass.clone(),
            },
        );
    }
    // A small scratch region for any inline builtin that spills, plus the guard
    // word the scratch cursor reserves. The body has no aggregate/`match` scratch
    // needs (scalar-only), so a few words are ample.
    let scratch_base = next_slot;
    next_slot += (4 + 1) * 8;
    // 32 bytes of shadow space so `rsp` stays 16-byte aligned across any inline
    // builtin's internal stack use; the body issues no real `call`, but reserving
    // the shadow is harmless and keeps alignment robust.
    let raw = next_slot + 32;
    let frame_size = ((raw + 15) / 16) * 16;

    // Env capture offsets: capture `k` at byte offset `8*(k+1)` in the env block,
    // paired with its scalar class so the body loads it through the right register
    // file.
    let mut captures = HashMap::new();
    for (index, (cap_name, cap_class)) in layout.captures.iter().enumerate() {
        captures.insert(
            cap_name.clone(),
            (((index + 1) * 8) as i32, cap_class.clone()),
        );
    }

    let mut ctx = NativeCtx::for_closure_body(
        locals,
        frame_size,
        scratch_base,
        layout.ret.clone(),
        ClosureEnv { env_slot, captures },
        callable,
        extern_sigs,
        structs,
        enums,
        strings,
        signatures,
        closure_layouts,
    );

    let mut code = Vec::new();
    // Prologue: push rbp; mov rbp, rsp; sub rsp, frame.
    code.extend_from_slice(&[0x55, 0x48, 0x89, 0xE5]);
    emit_sub_rsp(&mut code, frame_size);
    // Seat the env pointer (rcx) into its slot: `mov [rbp - env_slot], rcx`.
    code.extend_from_slice(&[0x48, 0x89, 0x8D]);
    code.extend_from_slice(&(-env_slot).to_le_bytes());
    // Seat each parameter from its effective Win64 position `i + 1` (the env
    // pointer is the hidden position 0). Integer positions 1..3 arrive in
    // rdx/r8/r9; float positions 1..3 arrive in the POSITIONALLY matching
    // xmm1/xmm2/xmm3; positions 4+ arrive on the caller's stack at
    // `[rbp + 48 + 8*(pos-4)]`.
    const PARAM_STORE: [&[u8]; 4] = [
        &[0x48, 0x89, 0x8D], // mov [rbp+disp32], rcx (position 0 — the env pointer)
        &[0x48, 0x89, 0x95], // mov [rbp+disp32], rdx (position 1)
        &[0x4C, 0x89, 0x85], // mov [rbp+disp32], r8  (position 2)
        &[0x4C, 0x89, 0x8D], // mov [rbp+disp32], r9  (position 3)
    ];
    for (index, (pname, pclass)) in layout.params.iter().enumerate() {
        let slot = ctx.local_slot(pname)?;
        let pos = index + 1; // the env pointer consumes effective position 0
        if pos >= 4 {
            // A stack argument is already a raw 8-byte word for every scalar class
            // (an integer cell, or raw float bits): copy it bit-for-bit into the
            // parameter's slot. A float needs no XMM round-trip — the slot holds
            // raw bits and every float reader loads it with movsd/movss.
            let stack_disp = 48 + (pos as i32 - 4) * 8;
            emit_mov_rax_from_rbp_pos(&mut code, stack_disp);
            store_local(&mut code, slot);
        } else if let Some(width) = closure_scalar_float_width(pclass) {
            // `xmm{pos}` — chosen by POSITION, not by how many floats came before.
            emit_store_xmm_to_slot(&mut code, pos as u8, slot, width);
        } else {
            code.extend_from_slice(PARAM_STORE[pos]);
            code.extend_from_slice(&(-slot).to_le_bytes());
        }
    }

    // Body: a single native-scalar expression. Routed through `lower_return_value`
    // — the backend's ONE value-position routing point — so an integer-cell return
    // lands in `rax` and a float return in `xmm0` by exactly the same rule every
    // other function return follows. `ctx.return_ty` (set above) is what it reads;
    // a closure return is never an aggregate, so `sret_slot` stays `None`.
    lower_return_value(&mut ctx, &def.body, &mut code)?;

    // Epilogue: add rsp, frame; pop rbp; ret.
    emit_native_epilogue(&mut code, frame_size, &[]);

    Ok(LoweredNativeFunction::new_closure(
        closure_symbol(def.id),
        code,
        ctx.take_relocations(),
    ))
}
