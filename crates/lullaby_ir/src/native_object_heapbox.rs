//! Native backend: the **interim heap-box** builtins `alloc` / `dealloc`. Sees the
//! parent's items via `use super::*`.
//!
//! # What `alloc` actually is (it is NOT a byte allocator)
//!
//! The name misleads, and the roadmap/diagnostic text around it has been written as
//! though `alloc(n)` reserved `n` bytes. It does not. The type checker types
//! `alloc(v)` as `ptr_{typeof v}` (`semantics_checker_calls.rs`), and the
//! interpreters implement it as
//!
//! ```text
//! self.heap.push(Some(value));        // one cell, holding `value`
//! Ok(Value::Ptr(self.heap.len() - 1)) // the cell's INDEX
//! ```
//!
//! So `alloc(8)` is a **box holding the value 8** — one cell — and `ptr_read` of it
//! yields `8`, not uninitialized 8-byte storage. It is `box(v)`, not `malloc(n)`.
//!
//! Native lowering therefore allocates **one 8-byte cell** through the shared
//! bump/RC allocator (`__lullaby_alloc`, which carries the heap-exhaustion `ud2`
//! guard) and stores the initializer into it, returning the cell's real machine
//! address. `ptr_read`/`ptr_write` through that address reuse the existing
//! raw-pointer surface unchanged: `raw_pointee_name` already strips the legacy
//! `ptr_T` spelling alongside `ptr<T>`, so an `alloc`-derived pointer flows through
//! `native_object_rawptr.rs` with no new load/store path.
//!
//! # Default-deny scope
//!
//! Only an **8-byte cell** pointee is lowered (`is_addressable_word_type`:
//! `i64`/`u64`/`isize`/`usize`/`ptr<T>`), where the Lullaby normalized cell and the
//! C width coincide, so the store `alloc` emits and the load `ptr_read` emits agree
//! bit-for-bit. `alloc("s")` (`ptr_string`), `alloc(true)` (`ptr_bool`),
//! `alloc(1.5)` (`ptr_f64`) and a narrow `alloc(to_i32(x))` (`ptr_i32`) each skip
//! cleanly (`L0339`) rather than guess a representation.
//!
//! # Why `dealloc` is NOT lowered (it skips cleanly)
//!
//! `dealloc` deliberately has **no** native lowering. Every candidate is a
//! correctness regression against the interpreters:
//!
//! * **`rc_free` (return the block to the free list).** The interpreters' `dealloc`
//!   sets the cell to `None`, so a later read is a *detected* error (`L0406`
//!   "invalid pointer"). Natively the block would be readable free-list memory —
//!   a **detected error would become a silent wrong answer** — and a double free
//!   would push the same block onto the LIFO free list twice, making the list
//!   cyclic so two later allocations alias the same cell (silent corruption) where
//!   the interpreters cleanly raise `L0406`.
//! * **A no-op.** Then `ptr_read` after `dealloc` returns the value natively while
//!   the interpreters raise `L0406` — the same divergence class.
//!
//! The `L0350` "used after it was freed" check does NOT make `rc_free` safe: it is
//! **name-based and does not survive aliasing**. Both of these compile and reach
//! the backend, failing only at interpreter run time with `L0406`:
//!
//! ```text
//! let p = alloc(5)   let q = p   dealloc(p)   ptr_read(q)   # use-after-free
//! let p = alloc(5)   let q = p   dealloc(p)   dealloc(q)    # double free
//! ```
//!
//! A faithful native `dealloc` needs a per-block validity tombstone plus a check on
//! every raw read — and that check is not implementable on the shared raw-pointer
//! read path, which also serves `addr_of` and `int_to_ptr` (MMIO) pointers that
//! have no allocator header at all. So a function using `dealloc` skips cleanly and
//! runs on the interpreters, where `L0406` is raised correctly. Skipping loses
//! nothing measurable: the interpreters never reuse a freed cell either
//! (`builtin_alloc` always pushes), so `dealloc` reclaims no interpreter memory.
//!
//! # Reclamation: what actually happens to an `alloc`'d block natively
//!
//! Nothing frees it. An `alloc`'d block is manually managed, so no `rc_dec` drop
//! glue is emitted for it (drop glue covers `string`/`list`/`map` records only),
//! and `alloc_defeats_arena` (below) keeps every `alloc`-using function off the
//! arena path, so no bump rewind reclaims it either. The block lives until the
//! process exits — bounded by the 1 MiB region, whose exhaustion is the allocator's
//! defined `ud2` trap, never a silent overrun. That **matches the interpreters**,
//! whose `heap: Vec<Option<Value>>` also grows monotonically and never reuses a
//! cell.
//!
//! # The arena hazard this module defuses
//!
//! `alloc` is invisible to the arena escape analysis: `type_is_directly_heap` does
//! not include `ptr_*`, and `expr_touches_heap` on a `Call { name: "alloc" }` only
//! inspects the arguments — so `let p = alloc(0)` registers as *not touching the
//! heap*. Without a gate, this is a real **use-after-free**:
//!
//! ```text
//! fn f -> i64
//!     unsafe
//!         let mut q = alloc(0)
//!         for i from 0 to 10
//!             let s string = to_string(i)   # a heap touch -> the loop is "heap"
//!             q = alloc(i)                  # `ptr_i64` is not a heap type ->
//!                                           # not counted as an escape
//!         ptr_read(q)                       # reads a rewound block
//! ```
//!
//! The loop looks heap-touching (the `string`) AND confined (the only store is a
//! `ptr_i64`, which `type_is_heap` says is not heap), so stage 2 gives it a
//! per-iteration sub-region and rewinds the bump pointer at the iteration edge —
//! reclaiming `q`'s cell while `q` still points at it.
//!
//! Rather than teach the escape analysis to track raw-pointer provenance (which
//! would have to see through `ptr_cast`/`int_to_ptr` and is not soundly decidable
//! here), this takes the **conservative exclusion** the arena rules are already
//! built around: a function whose body contains any `alloc` is never arena-eligible
//! (see [`alloc_defeats_arena`], applied in `arena_eligible_functions`). It then
//! stays on the RC / free-list path, where nothing reclaims its cells, so both the
//! loop-edge rewind and the return-edge rewind are impossible. This costs only the
//! arena optimization for such a function; correctness and codegen are otherwise
//! unchanged.

use super::*;

/// The heap-box builtin names this module owns.
pub(crate) const ALLOC_BUILTIN: &str = "alloc";
pub(crate) const DEALLOC_BUILTIN: &str = "dealloc";

/// Whether `name` is an interim heap-box builtin. Used by the call dispatcher so
/// these names never fall through to the unknown-function arm as if they were user
/// calls.
pub(crate) fn is_heap_box_builtin(name: &str) -> bool {
    matches!(name, ALLOC_BUILTIN | DEALLOC_BUILTIN)
}

/// Lower an interim heap-box builtin call. Returns `None` when `name` is not one,
/// so the caller falls through to its other dispatch arms.
///
/// `expr_ty` is the *call's* own type — `ptr_T` for `alloc`, `void` for `dealloc`.
pub(crate) fn lower_heap_box_call(
    ctx: &mut NativeCtx,
    name: &str,
    args: &[BytecodeExpr],
    expr_ty: &TypeRef,
    code: &mut Vec<u8>,
) -> Option<Result<(), String>> {
    if !is_heap_box_builtin(name) {
        return None;
    }
    Some(match name {
        ALLOC_BUILTIN => lower_alloc(ctx, args, expr_ty, code),
        // Skips cleanly — see the module docs. This is a deliberate, permanent-for-now
        // refusal, not an unimplemented case: every lowering of `dealloc` available on
        // this heap turns an interpreter-detected error into a silent wrong answer or
        // silent heap corruption.
        _ => Err(
            "`dealloc` is not lowered natively: the interpreters invalidate the freed cell \
             and DETECT a later use or a double free (`L0406`), which the native bump/RC \
             heap cannot reproduce — returning the block to the free list would make a \
             use-after-free read free-list memory silently and a double free alias two \
             live allocations, and a no-op would make a use-after-free succeed. A \
             function using `dealloc` runs on the interpreters, where the error is \
             raised correctly"
                .to_string(),
        ),
    })
}

/// `alloc(v) -> ptr_T`: allocate one 8-byte cell through the shared bump/RC
/// allocator and store `v` into it, leaving the cell's real address in `rax`.
///
/// The initializer is evaluated **before** the allocation, matching the
/// interpreters (`eval` the argument, then `heap.push`), so a call or a nested
/// `alloc` inside `v` is staged in the same order.
fn lower_alloc(
    ctx: &mut NativeCtx,
    args: &[BytecodeExpr],
    expr_ty: &TypeRef,
    code: &mut Vec<u8>,
) -> Result<(), String> {
    let [value] = args else {
        return Err("`alloc` takes exactly one argument".to_string());
    };
    // The call must type as `ptr_T`/`ptr<T>` (the checker produces the legacy
    // `ptr_T` spelling); `T` is the boxed value's own type.
    let pointee = raw_pointee_name(&expr_ty.name).ok_or_else(|| {
        format!(
            "`alloc` must type as a pointer on the native backend, found `{}`",
            expr_ty.name
        )
    })?;
    // Default-deny: only an 8-byte cell, where the Lullaby normalized cell width and
    // the C width coincide, so the store here and the `ptr_read` load agree exactly.
    // A `string`/`bool`/`char`/float/narrow-integer box skips cleanly.
    if !is_addressable_word_type(pointee) {
        return Err(format!(
            "`alloc` of a `{pointee}` value is not lowered natively: the native heap box \
             holds an 8-byte cell (`i64`/`u64`/`isize`/`usize`/`ptr<T>`) only, where the \
             normalized cell width matches the C width a `ptr_read`/`ptr_write` through \
             it uses"
        ));
    }
    // Defensive agreement between the boxed value's type and the checker-inferred
    // pointee: `alloc(v)` must type as `ptr_{typeof v}`. A mismatch means an
    // assumption here no longer holds, so skip rather than store a word whose reader
    // would use a different width.
    if pointee != value.ty.name {
        return Err(format!(
            "`alloc` of a `{}` value typed as `{}` is not lowered natively (the pointee \
             must be the boxed value's own type)",
            value.ty.name, expr_ty.name
        ));
    }

    // Evaluate the initializer first (interpreter order), and park it in a scratch
    // slot rather than on the stack: `__lullaby_alloc` is a call, and the `push`/`pop`
    // idiom the binary ops use would leave `rsp` misaligned at the call site.
    let saved_scratch = ctx.scratch_next;
    let value_slot = ctx.alloc_scratch(1);
    lower_native_expr(ctx, value, code)?; // v -> rax
    store_local(code, value_slot);

    // rcx = 8 (one cell) ; call __lullaby_alloc -> rax = the cell's address.
    emit_mov_rcx_imm(code, 8);
    emit_call_symbol(ctx, HEAP_ALLOC_SYMBOL, code);

    // [rax] = v. The cell is a fresh 8-byte block, so this initializing store is the
    // whole box; `rax` already holds the address the call returns.
    emit_mov_rcx_from_slot(code, value_slot); // rcx = v
    code.extend_from_slice(&[0x48, 0x89, 0x08]); // mov [rax], rcx

    ctx.scratch_next = saved_scratch;
    Ok(())
}

// -- The arena gate ----------------------------------------------------------

/// Whether `instrs` contain any `alloc` call. A function that boxes a value on the
/// heap is excluded from arena eligibility: an `alloc`'d cell is manually managed
/// and invisible to the escape analysis, so an arena rewind at a loop edge or a
/// return edge could reclaim a cell a live pointer still names (see the module docs
/// for the exact use-after-free shape).
///
/// The gate is deliberately whole-function and coarse rather than per-allocation,
/// exactly like [`body_takes_address`]'s register-promotion gate: it cannot be
/// defeated by an aliasing pattern a finer analysis failed to see through, and it
/// only ever DENIES an optimization — it never changes emitted semantics.
pub(crate) fn alloc_defeats_arena(instrs: &[BytecodeInstruction]) -> bool {
    instrs.iter().any(instr_allocates_box)
}

fn instr_allocates_box(instr: &BytecodeInstruction) -> bool {
    match instr {
        BytecodeInstruction::Let { value, .. } | BytecodeInstruction::Assign { value, .. } => {
            expr_allocates_box(value)
        }
        BytecodeInstruction::Return(Some(expr))
        | BytecodeInstruction::Expr(expr)
        | BytecodeInstruction::Throw { value: expr, .. } => expr_allocates_box(expr),
        BytecodeInstruction::Return(None)
        | BytecodeInstruction::Break(_)
        | BytecodeInstruction::Continue(_)
        | BytecodeInstruction::Asm { .. } => false,
        BytecodeInstruction::If {
            branches,
            else_body,
            ..
        } => {
            branches
                .iter()
                .any(|b| expr_allocates_box(&b.condition) || alloc_defeats_arena(&b.body))
                || alloc_defeats_arena(else_body)
        }
        BytecodeInstruction::While {
            condition, body, ..
        } => expr_allocates_box(condition) || alloc_defeats_arena(body),
        BytecodeInstruction::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            expr_allocates_box(start)
                || expr_allocates_box(end)
                || step.as_ref().is_some_and(expr_allocates_box)
                || alloc_defeats_arena(body)
        }
        BytecodeInstruction::Loop { body, .. } => alloc_defeats_arena(body),
        BytecodeInstruction::Match {
            scrutinee, arms, ..
        } => expr_allocates_box(scrutinee) || arms.iter().any(|a| alloc_defeats_arena(&a.body)),
        BytecodeInstruction::Try {
            body, catch_body, ..
        } => alloc_defeats_arena(body) || alloc_defeats_arena(catch_body),
    }
}

fn expr_allocates_box(expr: &BytecodeExpr) -> bool {
    match &expr.kind {
        BytecodeExprKind::Call { name, args } => {
            name == ALLOC_BUILTIN || args.iter().any(expr_allocates_box)
        }
        BytecodeExprKind::Unary { expr: inner, .. } | BytecodeExprKind::Await { expr: inner } => {
            expr_allocates_box(inner)
        }
        BytecodeExprKind::Binary { left, right, .. } => {
            expr_allocates_box(left) || expr_allocates_box(right)
        }
        BytecodeExprKind::Array(values) => values.iter().any(expr_allocates_box),
        BytecodeExprKind::Index { target, index } => {
            expr_allocates_box(target) || expr_allocates_box(index)
        }
        BytecodeExprKind::Field { target, .. } => expr_allocates_box(target),
        BytecodeExprKind::Integer(_)
        | BytecodeExprKind::Float(_)
        | BytecodeExprKind::Bool(_)
        | BytecodeExprKind::Char(_)
        | BytecodeExprKind::String(_)
        | BytecodeExprKind::Variable(_)
        | BytecodeExprKind::Closure { .. } => false,
    }
}
