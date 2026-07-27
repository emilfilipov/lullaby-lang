//! Native codegen: the **arena stage-4b promotion gate** — the single predicate
//! deciding whether a closure factory may have its returned capture block RELOCATED
//! to the caller's region mark by the promoting arena reset. Split out of
//! `native_object_closure.rs`; sees the parent's items via `use super::*`.
//!
//! # The invariant this module makes explicit
//!
//! The promoting reset (`emit_arena_reset`, stage 4b) relocates the survivor block
//! with a **flat unrolled word memmove** to the region mark `markF` and then sets
//! `heap_next = markF + size`. That is sound only if the block is **self-contained**:
//! every word is an immediate scalar value, so moving the words moves the whole
//! object and the reclaimed range `[markF + size, old_next)` is provably unreachable.
//!
//! A capture word that instead HOLDS A POINTER breaks both halves at once. The
//! memmove copies the pointer verbatim (it does not relocate the pointee), and the
//! `heap_next = markF + size` reclaim then frees the pointee the copied word still
//! addresses — a textbook use-after-free, silent, with no diagnostic anywhere.
//!
//! Historically this invariant was **implicit**: the gate admitted a factory purely
//! on `layout.captures.len()`, and flatness was delegated to
//! [`native_closure_scalar`] happening to refuse every heap type. That coupling is
//! invisible from the widening side — the first commit teaching
//! `native_closure_scalar` about `string` (or `list`, or a multi-word struct) would
//! have silently converted the promoting reset into a use-after-free, with nothing in
//! `emit_arena_reset`, `arena_eligible_functions`, or `function_is_locally_retaining`
//! to notice. The R2 carve-out in `function_is_locally_retaining` compounds it: it
//! stops treating a promotable factory's closure literals as a capture channel on the
//! stated ground that "a fresh flat closure has no way to hand a heap pointer to a
//! third party" — which stops being true at the same instant.
//!
//! So the flatness requirement is now an **explicit, default-deny gate** here:
//! [`promoted_capture_words`] matches the capture class exhaustively and admits only
//! the 8-byte non-pointer scalars, and [`promoted_survivor_words`] sums those words
//! (never counts captures) against [`MAX_PROMOTED_CLOSURE_WORDS`]. Widening
//! `native_closure_scalar` now yields a clean `L0339` deferral to the interpreters,
//! never a UAF; admitting a widened class for promotion is a deliberate edit here,
//! next to the reasoning it has to satisfy.

use super::*;

/// The maximum survivor size (in 8-byte words, `1 + Σ capture words`) a returned
/// closure may have to be **promoted** by the mark-advance reset (arena stage-4b). At
/// or below this cap the block is compacted by a short unrolled memmove on the return
/// edge; above it the factory stays OFF the arena (stage-4a: the returned block is
/// simply never reclaimed, which is sound with no promotion). Owner decision D4.
pub(crate) const MAX_PROMOTED_CLOSURE_WORDS: usize = 8;

/// The number of 8-byte words a capture of class `class` contributes to a **promotable**
/// survivor block, or `None` when the class may not appear in one at all.
///
/// Admitted (each exactly one word, and that word is an immediate value — never an
/// address): [`NativeType::I64`] (an integer cell — `i64`, a fixed-width integer,
/// `bool`/`char`/`byte`), [`NativeType::F64`], and [`NativeType::F32`].
///
/// Everything else is **refused**, and the match is written out variant-by-variant with
/// no `_` arm precisely so the refusal is a decision rather than a default: a class
/// added to [`NativeType`] tomorrow fails to compile here and must be classified
/// deliberately. Two distinct reasons to refuse, both fatal to the memmove:
///
/// * **pointer-carrying** — [`NativeType::String`], [`NativeType::HeapStruct`],
///   [`NativeType::List`], [`NativeType::Map`], [`NativeType::FatArray`]. One word
///   wide, but the word is an ADDRESS into the same region. The memmove would copy the
///   address and the `heap_next` reclaim would free its pointee.
/// * **not a single 8-byte value word** — [`NativeType::Void`] (no value at all),
///   [`NativeType::Narrow`] (sub-word stride), [`NativeType::Struct`],
///   [`NativeType::Array`], [`NativeType::Enum`] (multi-word, and possibly
///   pointer-carrying inside). The env layout stores one raw word per capture, so
///   these have no promotable representation even before aliasing is considered.
pub(crate) fn promoted_capture_words(class: &NativeType) -> Option<usize> {
    match class {
        // Immediate 8-byte scalars — the entire admitted set.
        NativeType::I64 | NativeType::F64 | NativeType::F32 => Some(1),
        // Pointer-carrying: relocating the word does not relocate the pointee.
        NativeType::String
        | NativeType::HeapStruct { .. }
        | NativeType::List { .. }
        | NativeType::Map { .. }
        | NativeType::FatArray { .. } => None,
        // Not a single 8-byte value word.
        NativeType::Void
        | NativeType::Narrow { .. }
        | NativeType::Struct { .. }
        | NativeType::Array { .. }
        | NativeType::Enum { .. } => None,
    }
}

/// The size in 8-byte words of the survivor block a promoting reset would relocate for
/// `layout` — `1` (the code pointer) plus the **summed** words of every capture — or
/// `None` when the block is not promotable at all: some capture is not an 8-byte
/// non-pointer scalar ([`promoted_capture_words`]), or the total exceeds
/// [`MAX_PROMOTED_CLOSURE_WORDS`].
///
/// This is the ONE definition of "promotable, and this big". The admission gate
/// ([`returns_promotable_closure`]) and the emitted size
/// (`NativeCtx::promoted_survivor_bytes`) both call it, so the reset can never move a
/// number of words the gate did not sanction.
pub(crate) fn promoted_survivor_words(layout: &ClosureLayout) -> Option<usize> {
    // Word 0 is the code pointer; captures follow at `8*(k+1)`.
    let mut words = 1usize;
    for (_, class) in &layout.captures {
        words = words.checked_add(promoted_capture_words(class)?)?;
    }
    (words <= MAX_PROMOTED_CLOSURE_WORDS).then_some(words)
}

/// Whether `function` is a factory the arena may make **promoting** (stage-4b): every
/// return edge yields a FRESH, FLAT, scalar-capture closure literal small enough to
/// relocate to the region mark. This is the single shared predicate gating both the
/// eligibility side (criterion 1b admitting the factory onto the arena — see
/// `arena_eligible_functions`) and the retention side (the local R1 carve-out in
/// `function_is_locally_retaining`, treating the factory as non-retaining so an arena
/// caller may rewind over it).
///
/// **PURELY LOCAL** — a function of `function`'s own body and the module closure
/// layouts ONLY, never of whether `function` is itself arena-eligible. That is
/// mandatory: the retention summary is computed BEFORE `arena_eligible_functions` and
/// criterion 3 reads the summary, so gating this on arena status would close a
/// summary→eligibility→summary cycle the single-sweep DFS cannot express. Locality is
/// sound because a fresh flat closure return lands in the caller's region
/// (`markF ≥ markC`) whether the factory promotes (arena) or merely bump-allocates into
/// the caller's region (off-arena) — either way it dies at the caller's rewind (see
/// `documents/lullaby_memory_management.md`).
///
/// Requires, in order: (1) a `fn(...)` return whose every edge is a locally-created
/// closure literal with a native-scalar signature ([`native_fn_return_eligibility`]);
/// (2) the tail is not a value-`if`/`match`/`asm` — those route their value through a
/// single post-convergence reset with no per-edge survivor expression, so the per-site
/// promoting size cannot be resolved and the factory stays off-arena; and (3) every
/// returned closure has a resolved [`ClosureLayout`] whose survivor block is promotable
/// — **every capture an 8-byte non-pointer scalar** and the summed size at or below
/// [`MAX_PROMOTED_CLOSURE_WORDS`] ([`promoted_survivor_words`]). Requirement (3) is the
/// flat-memmove invariant, checked explicitly rather than inherited from whatever
/// [`native_closure_scalar`] currently admits — see the module docs. Default-deny: any
/// unresolved edge ⇒ not promotable.
pub(crate) fn returns_promotable_closure(
    function: &BytecodeFunction,
    closure_layouts: &HashMap<usize, ClosureLayout>,
) -> bool {
    if !function.return_type.is_function() {
        return false;
    }
    if native_fn_return_eligibility(function).is_err() {
        return false;
    }
    // A value tail `if`/`match` (its branches route through one post-convergence
    // reset) or a tail `asm` (leaves `rax` by contract, no closure expression) has no
    // per-edge survivor expression the promoting reset can size, so refuse promotion —
    // the factory stays off-arena (stage-4a, still sound). A fn-returning factory with
    // such a tail is in fact already native-INeligible today (its tail-branch closures
    // are not collected as return edges, so `returns_only_local_closure_literals`
    // fails), but this keeps the promotion gate self-contained and default-deny.
    if matches!(
        function.instructions.last(),
        Some(
            BytecodeInstruction::If { .. }
                | BytecodeInstruction::Match { .. }
                | BytecodeInstruction::Asm { .. }
        )
    ) {
        return false;
    }
    // Every return edge must resolve to a closure id whose flat scalar-capture layout
    // exists, carries no pointer word, and fits the promotion cap.
    let Some(ids) = return_edge_closure_ids(function) else {
        return false;
    };
    ids.iter().all(|id| {
        closure_layouts
            .get(id)
            .and_then(promoted_survivor_words)
            .is_some()
    })
}

/// The closure `id` of every return-edge value of `function`, or `None` if any edge is
/// not a resolvable closure literal (a direct `fn …` literal, or a local bound to
/// one). Mirrors [`returns_only_local_closure_literals`]'s edge collection but yields
/// the ids (so the promoting reset can size each survivor per-site), resolving a
/// returned literal-bound local through its `let`.
fn return_edge_closure_ids(function: &BytecodeFunction) -> Option<Vec<usize>> {
    let mut literal_ids: HashMap<String, usize> = HashMap::new();
    collect_closure_literal_local_ids(&function.instructions, &mut literal_ids);
    let mut edges: Vec<&BytecodeExpr> = Vec::new();
    collect_return_value_exprs(&function.instructions, &mut edges);
    if edges.is_empty() {
        return None;
    }
    let mut ids = Vec::with_capacity(edges.len());
    for e in edges {
        match &e.kind {
            BytecodeExprKind::Closure { id } => ids.push(*id),
            BytecodeExprKind::Variable(n) => ids.push(*literal_ids.get(n)?),
            _ => return None,
        }
    }
    Some(ids)
}

/// Collect every local `name` bound by `let name = <closure literal>` to its closure
/// `id` — the name→id analogue of `collect_closure_literal_lets`, so a returned
/// literal-bound local can be sized for the promoting reset.
fn collect_closure_literal_local_ids(
    body: &[BytecodeInstruction],
    out: &mut HashMap<String, usize>,
) {
    for stmt in body {
        if let BytecodeInstruction::Let { name, value, .. } = stmt
            && let BytecodeExprKind::Closure { id } = &value.kind
        {
            out.insert(name.clone(), *id);
        }
        for nested in stmt_nested_bodies(stmt) {
            collect_closure_literal_local_ids(nested, out);
        }
    }
}
