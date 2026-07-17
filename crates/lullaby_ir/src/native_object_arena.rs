//! Native backend: the freestanding-tier **static-buffer arena** (§5 of
//! `documents/freestanding_tier_design.md`) — `region <name> in <buffer>` and the
//! `arena_alloc(region, count)` bump builtin. A sibling of
//! `native_object_rawptr.rs`; sees the parent's items via `use super::*`.
//!
//! Native is the tier §5 ultimately targets — a kernel runs on bare metal — but it
//! is **not** the only tier that defines the arena. The AST, IR, and bytecode
//! interpreters run it too, and agree with this backend on every program: an arena
//! cell is an ordinary `array<i64>` element, so `arena_alloc(r, n)` is exactly
//! `addr_of(buf[cursor])` plus an integer cursor, which every tier already models.
//! `L0460` is the arena **overflow** diagnostic, not a refusal.
//!
//! Because the tiers share one model, they must share one **scoping** model too, and
//! that is the subtle part. A `region <name> in <buffer>` is scoped to its
//! **enclosing block**, and its buffer is pinned at the **declaration**:
//!
//! * the checker block-scopes region names (`Scope::arena_regions`);
//! * the interpreters hold the arena in a single env binding with block lifetime,
//!   carrying the buffer's `RootSlot`;
//! * this backend re-zeroes the cursor **at the declaration site** (not the
//!   prologue) and binds the buffer to a frame slot.
//!
//! Each of those three was independently wrong at first, and each produced a silent
//! wrong answer rather than a diagnostic — see `emit_arena_cursor_init` for the
//! prologue-zeroing case (native 123 vs 300 on every interpreter).
//!
//! # What an arena is, natively
//!
//! Three things, and no runtime whatsoever:
//!
//! * **the buffer** — an ordinary `array<i64>` local the author declared, living in
//!   its own frame slots. The arena adds no storage of its own.
//! * **the cursor** — one dedicated frame word per region (reserved by
//!   `NativeCtx::plan`, sized in `native_object_frame.rs`), holding a **cell
//!   count**, zeroed at its `region` declaration so it resets on block re-entry.
//! * **the bump** — `arena_alloc(a, n)` reads the cursor, range-checks `cursor + n`
//!   against the buffer's length, stores the new cursor, and returns
//!   `&buffer[old_cursor]`.
//!
//! No host allocator, no growth, no hidden control flow beyond the one visible
//! overflow branch. That is exactly what makes it legal in `no-runtime`.
//!
//! # Cells, not bytes — and why the buffer is `array<i64>`
//!
//! §5 sketches the backing buffer as `array<byte>` with byte-granular bumping. The
//! delivered native value model forbids that: **every Lullaby scalar is a
//! normalized 8-byte cell** (an `i32` local occupies a full sign-extended word),
//! which is precisely why `addr_of` lowers for 8-byte scalars only (see
//! `native_object_rawptr.rs`). An `array<byte>` is therefore an array of 8-byte
//! cells, not packed bytes, so a byte-granular arena over one would hand back
//! pointers whose reads and writes corrupt the cell invariant every other native
//! path depends on. The arena bumps in **cells** and the buffer is `array<i64>`,
//! which makes every pointer it returns exactly as well-formed as an
//! `addr_of(buf[i])` — the delivered, tested kernel idiom this reuses wholesale.
//!
//! # Overflow: a real, deterministic edge — `ud2`
//!
//! A bump that would exceed the buffer **traps with `ud2`**, the same instruction
//! and the same reasoning as the delivered native bounds check for an out-of-range
//! `buf[i]` (see `emit_dynamic_addr_into_rcx`). It is deterministic, immediate, and
//! — the point — it can never hand back a pointer past the buffer's end. Returning
//! null or a truncated pointer would be a silent wrong answer that corrupts memory
//! at some later, unrelated point; trapping is the honest edge.
//!
//! This is the **§8 seam**. §8 makes the panic edge pluggable: a `no-runtime`
//! program will declare `panic fn on_panic info ptr<PanicInfo> -> never`, and the
//! trap here becomes a call to that symbol with `kind = arena_overflow`. That is a
//! localized change to [`emit_arena_overflow_trap`] and nothing else — the range
//! check, the cursor, and the bump are already in their final form. Until then the
//! edge is `ud2`: honest, observable, and never silent. It does **not** unwind
//! (decision A5), and neither will the §8 handler.
//!
//! # Composition with the arena-FIRST escape analysis: disjoint by construction
//!
//! `native_object_eligibility.rs` carries the arena-first escape analysis
//! (`arena_eligible_functions`, function regions, confined-loop sub-regions), where
//! a per-iteration sub-region rewind freeing a still-live cell is a demonstrated
//! miscompile class. **A static-buffer arena cannot interact with it**, and this is
//! a structural fact rather than a judgement call:
//!
//! * That analysis governs the **host heap bump pointer** (`__lullaby_heap_next`),
//!   which backs `list`/`string`/`map`/`rc`/`alloc` values. A static-buffer arena
//!   never reads or writes that pointer — its memory is the author's `array<i64>`
//!   local and its cursor is a private frame word.
//! * Its rewinds (`emit_arena_reset`, `arena_loop_reset_mark`) restore
//!   `__lullaby_heap_next`. They cannot rewind an arena cursor, which they do not
//!   know about; the cursor is only ever written by [`lower_arena_alloc`] and
//!   zeroed by the prologue.
//! * The two are gated on disjoint programs anyway: `arena_eligible_functions`
//!   requires a function that *touches the heap*, while every heap-touching
//!   construct is `L0441`-rejected in the `no-runtime` modules where arenas live.
//!
//! So there is no exclusion to make and nothing to be conservative about: the
//! feature adds a frame word and a bump, and touches no state the escape analysis
//! observes. The escape-analysis fuzzers stay green because arena functions produce
//! byte-identical code to before (`arena_buffers` is empty for them).

use super::*;

/// The `arena_alloc` builtin name — `arena_alloc(region, count) -> ptr<T>`.
pub(crate) const ARENA_ALLOC_BUILTIN: &str = "arena_alloc";

/// The `arena_region` declaration marker the AST->IR lowering emits for
/// `region <name> in <buffer>`. It is a compile-time binding, not an operation: it
/// carries the region and buffer names to this backend and emits no code.
pub(crate) const ARENA_REGION_MARKER: &str = "arena_region";

/// A static-buffer arena's native binding: the backing buffer's local name and the
/// frame word holding its bump cursor (a **cell count**, not a byte offset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArenaBinding {
    pub(crate) buffer: String,
    pub(crate) cursor_slot: i32,
}

/// Lower a static-buffer arena builtin, leaving its result in `rax`. Returns `None`
/// when `name` is not an arena builtin, so the caller falls through to its other
/// dispatch arms and an unhandled name still reaches the ordinary
/// unknown-function error (skipping cleanly with `L0339`) rather than being
/// treated as a user call.
pub(crate) fn lower_arena_call(
    ctx: &mut NativeCtx,
    name: &str,
    args: &[BytecodeExpr],
    expr_ty: &TypeRef,
    code: &mut Vec<u8>,
) -> Option<Result<(), String>> {
    match name {
        // The declaration marker: `plan` already reserved this region's cursor word;
        // the declaration ZEROES it. See `emit_arena_cursor_init`.
        ARENA_REGION_MARKER => Some(emit_arena_cursor_init(ctx, args, code)),
        ARENA_ALLOC_BUILTIN => Some(lower_arena_alloc(ctx, args, expr_ty, code)),
        _ => None,
    }
}

/// `arena_alloc(region, count) -> ptr<T>`: bump `count` 8-byte cells out of
/// `region`'s backing buffer and return a pointer to the first.
///
/// Emits, with `cur` = the region's cursor word and `N` = the buffer's cell length:
///
/// ```text
///     mov  rcx, [rbp - cur]        ; old cursor (cell index)
///     mov  rdx, count              ; requested cells
///     ...  (count may be a full expression, evaluated into rax first)
///     mov  r8, rcx
///     add  r8, rdx                 ; new cursor = old + count
///     jc   overflow                ; unsigned wrap => overflow
///     cmp  r8, N
///     ja   overflow                ; new cursor past the buffer => overflow
///     mov  [rbp - cur], r8         ; commit the bump
///     lea  rax, [rbp - buf_slot]   ; &buffer[0]
///     lea  rax, [rax + rcx*8]      ; &buffer[old cursor]
///     jmp  done
/// overflow:
///     ud2
/// done:
/// ```
///
/// The `jc` is not belt-and-braces: `count` is a signed `i64` the author supplies,
/// so a huge or negative value could wrap `old + count` back into range and hand
/// out a pointer *before* the buffer. Both the carry and the `ja` are unsigned, so
/// a negative `count` reads as an enormous unsigned value and is caught by one or
/// the other. A zero-cell request is well-defined: it commits nothing and returns
/// the current position, exactly as a zero-size bump should.
fn lower_arena_alloc(
    ctx: &mut NativeCtx,
    args: &[BytecodeExpr],
    expr_ty: &TypeRef,
    code: &mut Vec<u8>,
) -> Result<(), String> {
    let [region, count] = args else {
        return Err(format!(
            "`{ARENA_ALLOC_BUILTIN}` takes exactly two arguments"
        ));
    };
    // The call must type as `ptr<P>`; the pointee must be an 8-byte cell so a
    // `ptr_read`/`ptr_write` through the result is width-exact, for the same
    // normalized-cell reason `addr_of` is 8-byte-only.
    let pointee = raw_pointee_name(&expr_ty.name).ok_or_else(|| {
        format!(
            "`{ARENA_ALLOC_BUILTIN}` must type as a `ptr<T>` on the native backend, found `{}`",
            expr_ty.name
        )
    })?;
    if !is_addressable_word_type(pointee) {
        return Err(format!(
            "`{ARENA_ALLOC_BUILTIN}` yields an 8-byte pointee (`i64`/`u64`/`isize`/`usize`/\
             `ptr<T>`) only, found `ptr<{pointee}>`: a narrower pointee is stored as a \
             normalized 8-byte cell, so a width-correct store through the returned pointer \
             would corrupt the cell's upper bits"
        ));
    }

    // Resolve the region name to its buffer + cursor. The checker (`L0445`) already
    // guarantees the operand names a declared region; this is the backend's own
    // total check, so a mismatch skips cleanly rather than miscompiling.
    let BytecodeExprKind::Variable(region_name) = &region.kind else {
        return Err(format!(
            "`{ARENA_ALLOC_BUILTIN}` requires a static-buffer region name as its first operand"
        ));
    };
    let binding = ctx.arena_buffers.get(region_name).cloned().ok_or_else(|| {
        format!("`{ARENA_ALLOC_BUILTIN}` names region `{region_name}`, which is not declared")
    })?;

    // The buffer must be a real fixed `array<i64>` in this frame. A fat-pointer
    // (runtime-length) array parameter is refused for the same reason `addr_of` is:
    // it shares the CALLER's storage read-only, and an arena hands out writable
    // pointers into its buffer.
    let local = ctx.local(&binding.buffer)?;
    let (elem_words, len) = match &local.ty {
        NativeType::Array { elem, len } => (elem.words(), *len),
        NativeType::FatArray { .. } => {
            return Err(format!(
                "static-buffer arena `{region_name}` is backed by the fat-pointer \
                 (runtime-length) array parameter `{}`, which is not lowered natively: the \
                 descriptor shares the caller's storage read-only, so arena pointers into it \
                 could be used to mutate the caller's array",
                binding.buffer
            ));
        }
        other => {
            return Err(format!(
                "static-buffer arena `{region_name}` must be backed by a fixed `array<i64>` \
                 local, but `{}` is `{other:?}`",
                binding.buffer
            ));
        }
    };
    if elem_words != 1 {
        return Err(format!(
            "static-buffer arena `{region_name}` must be backed by a fixed `array<i64>` (a \
             one-word element type), but `{}`'s element occupies {elem_words} words",
            binding.buffer
        ));
    }
    let buffer_slot = local.slot;
    // A register-promoted local has no address. Promotion only picks `i64` scalars,
    // never arrays, so this cannot fire — but gate it rather than trust that, so a
    // future change to the promotion rule becomes a clean skip, not a `lea` of an
    // unwritten frame slot.
    if ctx.promoted_reg(buffer_slot).is_some() {
        return Err(format!(
            "static-buffer arena `{region_name}`'s buffer `{}` is register-promoted and has no \
             address",
            binding.buffer
        ));
    }

    // Evaluate the cell count into rax (it is an arbitrary `i64` expression).
    lower_native_expr(ctx, count, code)?;
    code.extend_from_slice(&[0x48, 0x89, 0xC2]); // mov rdx, rax   (count)
    // rcx = old cursor
    code.extend_from_slice(&[0x48, 0x8B, 0x8D]); // mov rcx, [rbp + disp32]
    code.extend_from_slice(&(-binding.cursor_slot).to_le_bytes());
    // r8 = old + count, carry set on unsigned wrap.
    code.extend_from_slice(&[0x4C, 0x8B, 0xC1]); // mov r8, rcx
    code.extend_from_slice(&[0x49, 0x01, 0xD0]); // add r8, rdx

    let mut overflow_jumps: Vec<usize> = Vec::new();
    code.extend_from_slice(&[0x0F, 0x82]); // jc rel32
    overflow_jumps.push(code.len());
    code.extend_from_slice(&0i32.to_le_bytes());

    // cmp r8, N  — unsigned. `len` is a real array length, well inside i32.
    let limit = i32::try_from(len).map_err(|_| {
        format!("static-buffer arena `{region_name}`'s buffer length {len} does not fit an i32")
    })?;
    code.extend_from_slice(&[0x49, 0x81, 0xF8]); // cmp r8, imm32
    code.extend_from_slice(&limit.to_le_bytes());
    code.extend_from_slice(&[0x0F, 0x87]); // ja rel32
    overflow_jumps.push(code.len());
    code.extend_from_slice(&0i32.to_le_bytes());

    // Commit: [rbp - cur] = r8
    code.extend_from_slice(&[0x4C, 0x89, 0x85]); // mov [rbp + disp32], r8
    code.extend_from_slice(&(-binding.cursor_slot).to_le_bytes());
    // rax = &buffer[0]; rax += old_cursor * 8
    emit_lea_rax_local(code, buffer_slot);
    code.extend_from_slice(&[0x48, 0x8D, 0x04, 0xC8]); // lea rax, [rax + rcx*8]

    // jmp done
    code.push(0xE9);
    let done_jump = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());

    // overflow: the deterministic edge (see the module docs; §8's seam).
    let overflow_target = code.len();
    for site in overflow_jumps {
        let rel = (overflow_target as i64 - (site as i64 + 4)) as i32;
        code[site..site + 4].copy_from_slice(&rel.to_le_bytes());
    }
    emit_arena_overflow_trap(code);

    let done_target = code.len();
    let rel = (done_target as i64 - (done_jump as i64 + 4)) as i32;
    code[done_jump..done_jump + 4].copy_from_slice(&rel.to_le_bytes());
    Ok(())
}

/// The static-buffer arena **overflow edge**: a bump that would leave the buffer.
///
/// `ud2` — an architecturally guaranteed invalid-opcode trap. This is the same
/// edge, and the same instruction, the delivered native bounds check uses for an
/// out-of-range `buf[i]`, so arena overflow behaves exactly like the safety failure
/// it is: immediate, deterministic, and impossible to mistake for success.
///
/// **This is the single seam §8 replaces.** When the pluggable panic handler lands,
/// this emits a call to the program's `panic fn` symbol with a `PanicInfo` whose
/// `kind` is `arena_overflow`, instead of trapping. Everything else about the arena
/// is already in its final shape. Per decision A5 the handler still terminates —
/// it does not unwind — so the shape of this edge (a divergent tail, no return
/// path) is unchanged by that work.
pub(crate) fn emit_arena_overflow_trap(code: &mut Vec<u8>) {
    code.extend_from_slice(&[0x0F, 0x0B]); // ud2
}

/// Zero a static-buffer arena's bump cursor — **at its `region` declaration**, which
/// is what makes "reset at dedent" true.
///
/// # Why the declaration site and not the prologue
///
/// A frame slot holds whatever the previous call left there, so the cursor must be
/// zeroed *somewhere*. Doing it once in the prologue is not enough, and the gap was a
/// real divergence: a `region` declared inside a loop body is re-entered every
/// iteration, and the interpreters re-create the arena each time (its env binding
/// dies at dedent), so its cursor restarts at zero. A prologue-only zeroing never
/// re-zeroes, so native kept bumping across iterations and handed out different
/// cells — measured 123 natively against 300 on all three interpreters, with no
/// diagnostic. Native contradicted this feature's own documented semantics.
///
/// Emitting the zeroing at the declaration fixes that by construction: the
/// declaration is *inside* the loop body, so the cursor is re-zeroed exactly when the
/// region is re-entered, and exactly once when it is not. A region declared but never
/// reached zeroes nothing and allocates nothing, which is also correct.
fn emit_arena_cursor_init(
    ctx: &mut NativeCtx,
    args: &[BytecodeExpr],
    code: &mut Vec<u8>,
) -> Result<(), String> {
    let [region, _buffer] = args else {
        return Err(format!(
            "`{ARENA_REGION_MARKER}` takes the region and buffer names"
        ));
    };
    let BytecodeExprKind::String(region) = &region.kind else {
        return Err(format!(
            "`{ARENA_REGION_MARKER}` takes the region name as a string literal"
        ));
    };
    let binding = ctx.arena_buffers.get(region).cloned().ok_or_else(|| {
        format!("`{ARENA_REGION_MARKER}` names region `{region}`, which was not planned")
    })?;
    // mov qword ptr [rbp + disp32], 0
    code.extend_from_slice(&[0x48, 0xC7, 0x85]);
    code.extend_from_slice(&(-binding.cursor_slot).to_le_bytes());
    code.extend_from_slice(&0i32.to_le_bytes());
    Ok(())
}
