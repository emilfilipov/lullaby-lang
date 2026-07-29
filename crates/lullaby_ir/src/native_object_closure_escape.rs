//! The closure **escape analysis** and the module-wide **higher-order sink index** —
//! the default-deny rules that decide which closure and `fn`-typed-parameter uses the
//! native backend may lower. Split out of `native_object_closure.rs` (which the
//! stage-3c sweep pushed past the size-backlog cap), mirroring how
//! `native_object_retain.rs` was split out of `native_object_eligibility.rs`; sees the
//! parent`s items via `use super::*`.
//!
//! Two cooperating pieces live here, and only together do they make the closure ABI
//! safe:
//!
//! * **The use walker** ([`body_closure_use_ok`] and its statement/expression arms) —
//!   one default-deny traversal answering "is every use of this callable name a
//!   SUPPORTED use?". Three entry points specialize it: [`closure_local_ok`] (a
//!   closure literal local, which MAY escape by return — stage 4a),
//!   [`call_returned_callable_ok`] (a factory-produced closure, strictly call-only),
//!   and [`closure_local_confined_to_iteration`] (a loop-body literal, which must not
//!   outlive the iteration edge where its drop fires).
//! * **The sink index** ([`build_hof_index`]) — the whole-module sweep deciding which
//!   `fn(...)`-typed PARAMETERS are non-escaping sinks, which is what lets the walker
//!   admit a closure handed to `apply(f, x)` (stage 3a) or relayed onward through a
//!   chain (stage 3c).
//!
//! They are mutually dependent — the sweep's per-parameter local check IS the walker,
//! and the walker consults the sweep's result — which is exactly why they belong in one
//! file: the onward-pass edges the sweep resolves are RECORDED BY the walk that
//! accepted them, so the dependency graph and the check can never disagree about which
//! uses are onward passes.

use super::*;

/// The call signature of an INDIRECT callable that is NOT a locally-created
/// closure literal but a **fn-typed parameter** holding a closure env pointer
/// passed in by a caller (the callee side of a non-escaping higher-order call).
/// It carries only what the *call site* needs — the parameter scalar classes and
/// the return class — because the captures are read by the closure body (which is
/// synthesized from its own `BytecodeClosureDef`), never by the callee. The ABI to
/// call through it is byte-identical to a closure-local call: env pointer in `rcx`,
/// visible arguments shifted to effective positions 1.., `call [env]`.
#[derive(Debug, Clone)]
pub(crate) struct ClosureCallSig {
    /// The parameter scalar classes in order (env pointer is the hidden position 0).
    pub(crate) params: Vec<NativeType>,
    /// The return scalar class (integer cell in `rax`, float in `xmm0`).
    pub(crate) ret: NativeType,
}

/// A **higher-order sink parameter**: a `fn(...)`-typed parameter of a function whose
/// fn-signature is entirely native scalars and whose every use is non-escaping —
/// either the callee of a direct call `param(args)`, or a bare argument handed to
/// another function's sink position (`inner(param, v)`, the multi-level stage-3c
/// pass-onward). This is the callee side of a non-escaping higher-order call: the
/// parameter receives a closure's `[code_ptr][captures…]` block pointer and either
/// calls through it or hands it to another call-only-or-onward sink, never letting it
/// escape — so the creating caller's capture environment stays valid for the whole
/// dynamic extent of the chain.
#[derive(Debug, Clone)]
pub(crate) struct HofParam {
    /// The parameter's position in the function's parameter list. This is what a
    /// caller's escape check matches against: passing a closure as argument `index`
    /// of this function is a sanctioned non-escaping sink.
    pub(crate) index: usize,
    /// The parameter's name (its frame-slot key inside the callee).
    pub(crate) name: String,
    /// Its native call signature (parameter + return scalar classes).
    pub(crate) sig: ClosureCallSig,
}

/// The native call signature of a `fn(param types) -> R` type when every parameter
/// and the return are native scalars, or `None` when any piece is outside the
/// scalar slice (a heap/aggregate parameter or return, or a nested `fn(...)`). The
/// scalar subset is exactly [`native_closure_scalar`] — the same classes a closure
/// literal supports — so a closure and the parameter it is passed to always agree
/// on register classes by construction.
pub(crate) fn native_fn_call_sig(fn_ty: &TypeRef) -> Option<ClosureCallSig> {
    let (param_types, ret_ty) = fn_ty.function_signature()?;
    let mut params = Vec::with_capacity(param_types.len());
    for ty in &param_types {
        params.push(native_closure_scalar(ty)?);
    }
    let ret = native_closure_scalar(&ret_ty)?;
    Some(ClosureCallSig { params, ret })
}

/// The **candidate** higher-order parameters of `function`: every `fn(...)`-typed
/// parameter whose fn-signature is entirely native scalars ([`native_fn_call_sig`]).
///
/// This is only the TYPE half of the sink test and says nothing about how the
/// parameter is USED. Whether a candidate is an actual **sink** is a whole-module
/// property — "passed onward to a sink" is mutually recursive across functions — and
/// is resolved by [`build_hof_index`]. A `fn` parameter that is not even a candidate
/// can never be a sink, so a function holding one is refused (`L0339`).
fn candidate_hof_params(function: &BytecodeFunction) -> Vec<HofParam> {
    let mut out = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        if !param.ty.is_function() {
            continue;
        }
        let Some(sig) = native_fn_call_sig(&param.ty) else {
            continue;
        };
        out.push(HofParam {
            index,
            name: param.name.clone(),
            sig,
        });
    }
    out
}

/// A candidate higher-order sink position: `(function name, parameter index)`. The
/// node type of the onward-pass graph [`build_hof_index`] sweeps.
type SinkNode<'a> = (&'a str, usize);

/// Build the module's **higher-order sink index**: for every function, the
/// `fn(...)`-typed parameters proven to be non-escaping **sinks**, keyed by function
/// name. A caller's closure-escape check ([`closure_local_ok`]) consults it to decide
/// whether passing a closure at a given function+position is sanctioned, and a callee's
/// `plan` reads its own entry to lower `param(args)` as an indirect call. A function
/// with no sink parameter gets no entry, so its codegen is byte-identical.
///
/// # The rule
///
/// A `fn` parameter is a **sink** iff it is a candidate ([`candidate_hof_params`]) and
/// **every** use of it is either
///
/// * **(a)** the callee of a direct call `param(args)`, or
/// * **(b)** a bare argument at another function's **sink** position
///   (`outer(g, v)` → `inner(g, v)` — the multi-level pass-onward, closures stage 3c).
///
/// Anything else makes it NOT a sink: stored (`let saved = g`), returned, reassigned,
/// read as a bare value, passed to a non-sink position, or passed to an `extern` /
/// indirect / unknown callee. **Default-deny** on every axis.
///
/// # Structure — mirrors [`retaining_summary`]
///
/// Rule (b) is mutually recursive, so this is computed exactly the way the arena's
/// cross-call retention summary is, with the polarity inverted (`retaining` is a
/// deny-lattice defaulting to `true`; `sink` is an admit-lattice defaulting to
/// `false`):
///
/// 1. **Local property first.** Each candidate is checked once under the
///    **optimistic** index — every candidate position in the module assumed to be a
///    sink. That is the most permissive assumption possible, so a parameter whose uses
///    fail under it can never be a sink under any fixpoint. This is the analogue of
///    `function_is_locally_retaining`, and it also folds in the unknown-callee denial:
///    the optimistic index contains only module-function candidate positions, so a bare
///    pass to an `extern`/builtin/indirect callee, or to a non-scalar `fn` parameter,
///    fails here.
/// 2. **Edges.** The same walk records the onward-pass positions the parameter depends
///    on — the exact `(callee, argument index)` pairs rule (b) admitted — so the
///    dependency graph can never disagree with the check that produced it.
/// 3. **One memoized DFS, no fixpoint.** `sink(node) = local_ok(node) AND every onward
///    edge is a sink`. **SCCs are pre-poisoned**: reaching a node already on the DFS
///    stack (a back-edge — self-recursion, mutual recursion, any cycle) yields `false`
///    without inspecting it, and that poison propagates outward through rule (b). The
///    lattice is monotone and the cycles are pre-poisoned, so a single
///    `O(nodes + edges)` sweep suffices. Without it a mutually recursive pair of
///    "sinks" would admit each other and an unbounded chain would be accepted.
/// 4. **Unknown nodes deny.** A recorded edge whose callee is not a module function, or
///    whose position is not a candidate, resolves to `false` — the same default
///    inversion `classify_call` applies for `extern`/indirect callees.
///
/// # Why the admitted shape is sound
///
/// The `[code_ptr][captures…]` block is allocated in the CREATING function's frame
/// region and owned by it. Every function in the chain receives it only as a `fn(...)`
/// parameter and may only call through it or hand it to another sink, so every use is
/// dynamically nested inside the creating call and the owner frame is live throughout.
/// Any creator whose closure is actually **invoked** through the chain is off the
/// arena: `fn_typed_binding_names` puts every `fn`-typed parameter into the
/// indirect-denied set, so the chain function that CALLS through its parameter is
/// R4-retaining, the poison propagates back up the chain through R4, and
/// [`all_callees_non_retaining`] fails for the creator — it gets neither a function
/// region nor a loop sub-region, so no rewind can fall between block creation and last
/// use. This is the identical argument that makes the shipped single-level stage-3a
/// sound; stage 3c only lengthens the chain.
///
/// The invocation qualifier is load-bearing and deliberately not stated as "by
/// construction": a sink parameter with **zero** uses is admitted vacuously (the walker
/// is an `all` over its uses), and such a sink calls nothing indirect, so it does not
/// become R4-retaining and a creator that only feeds it CAN be arena-eligible. That is
/// harmless — a closure that is never invoked has no capture read to dangle, and the
/// block itself is reclaimed by the creator's own rewind exactly like any other
/// per-iteration allocation — but the weaker claim is the true one.
pub(crate) fn build_hof_index(module: &BytecodeModule) -> HashMap<String, Vec<HofParam>> {
    // Type-level candidates: a `fn(...)` parameter with an all-native-scalar signature.
    // Nothing else can ever be a sink.
    let mut candidates: HashMap<&str, Vec<HofParam>> = HashMap::new();
    for function in &module.functions {
        let params = candidate_hof_params(function);
        if !params.is_empty() {
            candidates.insert(function.name.as_str(), params);
        }
    }
    if candidates.is_empty() {
        return HashMap::new();
    }
    // Module-lifetime function names, so a recorded edge's callee can be interned back
    // into a `&'module str` node (mirrors `classify_call`'s `module_fns.get`).
    let fn_names: std::collections::HashSet<&str> =
        module.functions.iter().map(|f| f.name.as_str()).collect();

    // The OPTIMISTIC index — every candidate position assumed to be a sink.
    let optimistic: HashMap<String, Vec<HofParam>> = candidates
        .iter()
        .map(|(name, params)| ((*name).to_string(), params.clone()))
        .collect();

    // Per-candidate LOCAL verdict plus the onward-pass edges it depends on, from ONE
    // walk each so the two can never disagree. `allow_return_escape = false`: a
    // returned fn parameter aliases a caller/grandcaller value and is never a
    // sanctioned escape (only a factory returning its OWN fresh literal is — see
    // [`closure_local_ok`]).
    let mut local_ok: HashMap<SinkNode, bool> = HashMap::new();
    let mut edges: HashMap<SinkNode, Vec<(String, usize)>> = HashMap::new();
    for function in &module.functions {
        let Some(params) = candidates.get(function.name.as_str()) else {
            continue;
        };
        for param in params {
            let mut onward = Vec::new();
            let ok = body_closure_use_ok(
                &function.instructions,
                &param.name,
                &optimistic,
                false,
                &mut onward,
            );
            let node = (function.name.as_str(), param.index);
            local_ok.insert(node, ok);
            // Edges matter only for a locally-ok parameter; a locally-denied one is
            // already `false` and its partially-walked edges are meaningless.
            edges.insert(node, if ok { onward } else { Vec::new() });
        }
    }

    // The sweep: memoized DFS over the onward-pass graph with cycles pre-poisoned.
    let mut memo: HashMap<SinkNode, bool> = HashMap::new();
    let mut on_stack: std::collections::HashSet<SinkNode> = std::collections::HashSet::new();
    for function in &module.functions {
        let Some(params) = candidates.get(function.name.as_str()) else {
            continue;
        };
        for param in params {
            resolve_hof_sink(
                (function.name.as_str(), param.index),
                &fn_names,
                &local_ok,
                &edges,
                &mut memo,
                &mut on_stack,
            );
        }
    }

    // Keep only the positions the sweep proved to be sinks.
    let mut index: HashMap<String, Vec<HofParam>> = HashMap::new();
    for function in &module.functions {
        let Some(params) = candidates.get(function.name.as_str()) else {
            continue;
        };
        let sinks: Vec<HofParam> = params
            .iter()
            .filter(|p| {
                memo.get(&(function.name.as_str(), p.index))
                    .copied()
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if !sinks.is_empty() {
            index.insert(function.name.clone(), sinks);
        }
    }
    index
}

/// Memoized DFS over the onward-pass graph. Returns whether `node` is a **sink**.
///
/// Cycle handling (default-deny, the mirror image of `resolve_retaining`'s poison):
/// reaching a `node` already on the DFS stack means it sits on a cycle, so it yields
/// `false` WITHOUT being memoized — its own frame computes the same `false` as the
/// stack unwinds (one of its edges reached it as a gray node), and every dependent
/// inherits the poison through rule (b). A `true` verdict is only ever recorded once
/// every edge of a node has been fully resolved to a sink, so no node is memoized as a
/// sink while still on a cycle.
fn resolve_hof_sink<'a>(
    node: SinkNode<'a>,
    fn_names: &std::collections::HashSet<&'a str>,
    local_ok: &HashMap<SinkNode<'a>, bool>,
    edges: &HashMap<SinkNode<'a>, Vec<(String, usize)>>,
    memo: &mut HashMap<SinkNode<'a>, bool>,
    on_stack: &mut std::collections::HashSet<SinkNode<'a>>,
) -> bool {
    if let Some(v) = memo.get(&node) {
        return *v;
    }
    // Back-edge to a node already being resolved ⇒ `node` is on a cycle ⇒ poison.
    if on_stack.contains(&node) {
        return false;
    }
    // A node with no local entry is not a candidate sink position at all (a non-`fn`
    // parameter, a non-native-scalar `fn` signature, or a callee this module does not
    // define): default-deny.
    let Some(&local) = local_ok.get(&node) else {
        return false;
    };
    if !local {
        memo.insert(node, false);
        return false;
    }
    on_stack.insert(node);
    let mut sink = true;
    if let Some(deps) = edges.get(&node) {
        for (callee, index) in deps {
            // Intern the callee against the module's function names so the resolved
            // node carries the module lifetime; an unknown name denies.
            let Some(&callee) = fn_names.get(callee.as_str()) else {
                sink = false;
                break;
            };
            if !resolve_hof_sink((callee, *index), fn_names, local_ok, edges, memo, on_stack) {
                sink = false;
                break;
            }
        }
    }
    on_stack.remove(&node);
    memo.insert(node, sink);
    sink
}

/// The higher-order sink parameters of `name` in `hof_index`, or an empty slice for a
/// function with no sink parameter. The single read accessor, so no call site has to
/// re-derive the "absent ⇒ none" convention.
pub(crate) fn hof_params_of<'a>(
    hof_index: &'a HashMap<String, Vec<HofParam>>,
    name: &str,
) -> &'a [HofParam] {
    hof_index.get(name).map_or(&[], Vec::as_slice)
}

/// Whether the call `callee` has a **higher-order sink** at argument position
/// `index` — a `fn(...)`-typed parameter of `callee` that the module-wide sweep
/// proved non-escaping (see [`build_hof_index`]). Passing a closure into such a
/// position is a sanctioned non-escaping use: the callee only *calls* the closure, or
/// hands it to a further sink, and never lets it escape, so the creator's capture
/// environment stays valid for the whole chain. `hof_index` maps each function name to
/// its sink parameters (absent for a function with none).
fn is_hof_sink(hof_index: &HashMap<String, Vec<HofParam>>, callee: &str, index: usize) -> bool {
    hof_index
        .get(callee)
        .is_some_and(|params| params.iter().any(|p| p.index == index))
}

/// Whether every use of a closure-bound local `name` in `function` is a
/// **supported** use: the value initializer of its own `let` (a direct closure
/// literal), the callee of a direct call `name(args)`, or a bare argument passed to
/// a **higher-order sink** — argument position `i` of a call whose callee has a
/// call-only fn parameter there (per `hof_index`). Default-deny — a bare value read,
/// a reassignment, a field/index, or being passed to any NON-sink position makes the
/// closure escape, so the enclosing function must skip. A bare `return name` IS
/// admitted here (`allow_return_escape = true`): a factory returning its OWN fresh,
/// flat, scalar-capture literal-bound closure is the sanctioned escape this increment
/// unblocks — the block is a heap value the factory does not reclaim (the factory is
/// kept off the arena, `native_object_eligibility.rs`), so it stays valid for the
/// caller to invoke. Returning a fn PARAMETER (which aliases a caller value) or a
/// call-returned closure stays refused, checked with `allow_return_escape = false`
/// (see [`build_hof_index`] and [`call_returned_callable_ok`]). This keeps a stored closure
/// out of the native slice while admitting a non-escaping higher-order argument
/// (`apply(f, x)`) and a returned local literal.
pub(crate) fn closure_local_ok(
    function: &BytecodeFunction,
    name: &str,
    hof_index: &HashMap<String, Vec<HofParam>>,
) -> bool {
    // A closure LOCAL reaches here only if its `let` bound a native-layout closure
    // LITERAL (`collect_native_locals` gates that), so a returned one is a fresh flat
    // scalar-capture block — safe to escape by return.
    body_closure_use_ok(
        &function.instructions,
        name,
        hof_index,
        true,
        &mut Vec::new(),
    )
}

/// Whether every use of a **call-returned callable** local `name` (a `fn`-typed local
/// bound to a factory call, not a literal) is supported: its own defining `let`, or
/// the callee of a direct call `name(args)`. Strict — `allow_return_escape = false`,
/// so a RETURN of `name` is refused (re-escaping a caller-region block is increment
/// b's promotion job, not this one), as are a store/reassign/alias/bare read. Its
/// defining `let` binds a `Call`, which the shared walker's `Let` arm now accepts.
pub(crate) fn call_returned_callable_ok(function: &BytecodeFunction, name: &str) -> bool {
    body_closure_use_ok(
        &function.instructions,
        name,
        &HashMap::new(),
        false,
        &mut Vec::new(),
    )
}

/// Whether every use of the closure-literal local `name` in `rest` — the statements
/// that FOLLOW its `let` inside the same loop body — keeps it confined to the
/// iteration: it is only the callee of a direct call `name(args)` or a bare argument
/// at a **higher-order sink** (`apply(name, x)`). Checked with
/// `allow_return_escape = false`, so a bare `return name` / bare `name` value read is
/// refused — those would let the block outlive the iteration edge at which the
/// loop-body drop frees it (see `collect_loop_body_drops`). Default-deny: anything
/// else (a store, a reassignment, a non-sink argument position) fails.
pub(crate) fn closure_local_confined_to_iteration(
    rest: &[BytecodeInstruction],
    name: &str,
    hof_index: &HashMap<String, Vec<HofParam>>,
) -> bool {
    body_closure_use_ok(rest, name, hof_index, false, &mut Vec::new())
}

/// The shared default-deny walker behind [`closure_local_ok`],
/// [`call_returned_callable_ok`], [`closure_local_confined_to_iteration`], and
/// [`build_hof_index`]'s local check.
///
/// `onward` is the **onward-pass recorder**: every bare `Variable(name)` argument the
/// walk ACCEPTS at a higher-order sink position is pushed as `(callee, argument
/// index)`. [`build_hof_index`] needs those positions as its dependency edges, and
/// deriving them from the very walk that accepted them is what guarantees the graph
/// and the check agree about which uses are onward passes. Short-circuiting is
/// harmless: a `false` verdict discards the partial record, and a `true` verdict means
/// every use was visited. The three escape-check entry points pass a throwaway `Vec`.
fn body_closure_use_ok(
    body: &[BytecodeInstruction],
    name: &str,
    hof_index: &HashMap<String, Vec<HofParam>>,
    allow_return_escape: bool,
    onward: &mut Vec<(String, usize)>,
) -> bool {
    body.iter()
        .all(|stmt| stmt_closure_use_ok(stmt, name, hof_index, allow_return_escape, onward))
}

fn stmt_closure_use_ok(
    stmt: &BytecodeInstruction,
    name: &str,
    hof_index: &HashMap<String, Vec<HofParam>>,
    allow_return_escape: bool,
    onward: &mut Vec<(String, usize)>,
) -> bool {
    match stmt {
        BytecodeInstruction::Let {
            name: bound, value, ..
        } => {
            // The callable local's own declaration binds it to a closure LITERAL (a
            // closure-literal local) or a factory CALL (a call-returned callable) —
            // that is its one allowed defining occurrence. A factory call's own args
            // still cannot leak `name` (impossible before the binding, but checked
            // defensively). Any OTHER `let` must not reference `name` except as a
            // direct call callee inside its value.
            if bound == name {
                match &value.kind {
                    BytecodeExprKind::Closure { .. } => true,
                    BytecodeExprKind::Call { .. } => {
                        expr_closure_use_ok(value, name, hof_index, onward)
                    }
                    _ => false,
                }
            } else {
                expr_closure_use_ok(value, name, hof_index, onward)
            }
        }
        // A reassignment of the callable local is a mutable-closure rebind (deferred);
        // any other assignment must use `name` only as a call callee in its value/
        // path indices.
        BytecodeInstruction::Assign {
            name: target,
            path,
            value,
            ..
        } => {
            if target == name {
                return false;
            }
            let path_ok = path.iter().all(|p| match p {
                BytecodePlace::Index(i) => expr_closure_use_ok(i, name, hof_index, onward),
                BytecodePlace::Field(_) => true,
            });
            path_ok && expr_closure_use_ok(value, name, hof_index, onward)
        }
        // A bare `return name` is the sanctioned escape only when `allow_return_escape`
        // (a factory returning its own literal-bound closure local). Otherwise — and
        // for any non-bare return expression — the returned value must not leak `name`.
        //
        // A bare `Expr(Variable(name))` is treated identically: the language's implicit
        // tail return of a factory whose body ends in `f` is an `Expr`, not a `Return`,
        // so the sanctioned tail escape reaches here as an `Expr`. Admitting a bare
        // `name` in ANY `Expr` position (not just the tail) under `allow_return_escape`
        // is still sound: a discarded closure read stores the pointer nowhere, so it
        // never escapes — only the tail one actually leaves the function.
        BytecodeInstruction::Return(Some(e)) | BytecodeInstruction::Expr(e) => {
            if allow_return_escape && matches!(&e.kind, BytecodeExprKind::Variable(n) if n == name)
            {
                true
            } else {
                expr_closure_use_ok(e, name, hof_index, onward)
            }
        }
        BytecodeInstruction::Throw { value: e, .. } => {
            expr_closure_use_ok(e, name, hof_index, onward)
        }
        BytecodeInstruction::Return(None)
        | BytecodeInstruction::Break(_)
        | BytecodeInstruction::Continue(_)
        | BytecodeInstruction::Asm { .. } => true,
        BytecodeInstruction::If {
            branches,
            else_body,
            ..
        } => {
            let branches_ok = branches.iter().all(|b| {
                expr_closure_use_ok(&b.condition, name, hof_index, onward)
                    && body_closure_use_ok(&b.body, name, hof_index, allow_return_escape, onward)
            });
            branches_ok
                && body_closure_use_ok(else_body, name, hof_index, allow_return_escape, onward)
        }
        BytecodeInstruction::While {
            condition, body, ..
        } => {
            expr_closure_use_ok(condition, name, hof_index, onward)
                && body_closure_use_ok(body, name, hof_index, allow_return_escape, onward)
        }
        BytecodeInstruction::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            if !expr_closure_use_ok(start, name, hof_index, onward)
                || !expr_closure_use_ok(end, name, hof_index, onward)
            {
                return false;
            }
            let step_ok = step
                .as_ref()
                .is_none_or(|s| expr_closure_use_ok(s, name, hof_index, onward));
            step_ok && body_closure_use_ok(body, name, hof_index, allow_return_escape, onward)
        }
        BytecodeInstruction::Loop { body, .. } | BytecodeInstruction::RegionBlock { body, .. } => {
            body_closure_use_ok(body, name, hof_index, allow_return_escape, onward)
        }
        BytecodeInstruction::Try {
            body, catch_body, ..
        } => {
            body_closure_use_ok(body, name, hof_index, allow_return_escape, onward)
                && body_closure_use_ok(catch_body, name, hof_index, allow_return_escape, onward)
        }
        BytecodeInstruction::Match {
            scrutinee, arms, ..
        } => {
            expr_closure_use_ok(scrutinee, name, hof_index, onward)
                && arms.iter().all(|arm| {
                    body_closure_use_ok(&arm.body, name, hof_index, allow_return_escape, onward)
                })
        }
    }
}

/// Whether every occurrence of the closure local `name` inside `expr` is a
/// supported position: the callee of a direct call `name(args)`, or a bare
/// `Variable(name)` argument at a **higher-order sink** (argument position `i` of a
/// call whose callee `callee` has a call-only fn parameter there — `is_hof_sink`).
/// A bare `Variable(name)` anywhere else is an escaping/value use and is rejected.
fn expr_closure_use_ok(
    expr: &BytecodeExpr,
    name: &str,
    hof_index: &HashMap<String, Vec<HofParam>>,
    onward: &mut Vec<(String, usize)>,
) -> bool {
    match &expr.kind {
        // A bare read of the closure local (as a value) is an escape unless it is
        // the callee position handled by the `Call` arm below.
        BytecodeExprKind::Variable(n) => n != name,
        BytecodeExprKind::Call { name: callee, args } => {
            // `callee(args)` — the callee name itself is not an argument, so the
            // closure local named as a direct callee never leaks as a value. Each
            // argument must not leak `name`, EXCEPT a bare `name` handed to a
            // higher-order sink (a sink fn parameter of `callee`), which is a
            // sanctioned non-escaping use: `callee` only calls the closure through
            // the pointer, or hands it to a further sink, and never lets it escape.
            args.iter().enumerate().all(|(i, a)| {
                if matches!(&a.kind, BytecodeExprKind::Variable(n) if n == name)
                    && is_hof_sink(hof_index, callee, i)
                {
                    // Record the accepted onward pass so `build_hof_index` can make
                    // this position an edge of its sweep.
                    onward.push((callee.clone(), i));
                    true
                } else {
                    expr_closure_use_ok(a, name, hof_index, onward)
                }
            })
        }
        BytecodeExprKind::Integer(_)
        | BytecodeExprKind::Float(_)
        | BytecodeExprKind::Bool(_)
        | BytecodeExprKind::String(_)
        | BytecodeExprKind::Char(_)
        | BytecodeExprKind::Closure { .. } => true,
        BytecodeExprKind::Array(elems) => elems
            .iter()
            .all(|e| expr_closure_use_ok(e, name, hof_index, onward)),
        BytecodeExprKind::Unary { expr, .. } | BytecodeExprKind::Await { expr } => {
            expr_closure_use_ok(expr, name, hof_index, onward)
        }
        BytecodeExprKind::Binary { left, right, .. } => {
            expr_closure_use_ok(left, name, hof_index, onward)
                && expr_closure_use_ok(right, name, hof_index, onward)
        }
        BytecodeExprKind::Field { target, .. } => {
            expr_closure_use_ok(target, name, hof_index, onward)
        }
        BytecodeExprKind::Index { target, index } => {
            expr_closure_use_ok(target, name, hof_index, onward)
                && expr_closure_use_ok(index, name, hof_index, onward)
        }
    }
}
