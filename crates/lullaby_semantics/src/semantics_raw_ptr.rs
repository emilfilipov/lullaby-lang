//! Typing for the freestanding-tier raw-pointer *addressing* surface (stage 2):
//! `addr_of` / `ptr_offset` / `ptr_cast`. Kept out of `semantics_checker_calls.rs`
//! (and `lib.rs`, already over the size cap) as a cohesive `impl Checker` block for
//! the raw-pointer builtins. See `documents/freestanding_tier_design.md` §2.2.
//!
//! All three are `unsafe`-gated exactly like the delivered raw-pointer builtins
//! (`L0330` outside `unsafe`) and require a *sized* pointee so element-scaled
//! arithmetic (`ptr_offset`) is well-defined. They are available in both tiers
//! under `unsafe`, and the `no-runtime` gate allows them (they yield an allowed
//! `ptr<T>` and are not host-allocator builtins).

use super::*;

impl Checker<'_> {
    /// `addr_of(place) -> ptr<T>`: the address of an addressable place — a local
    /// (`Variable`), an array element (`Index`), or a struct field (`Field`) — whose
    /// type `T` has a defined C-natural layout. A whole-array place decays to a
    /// pointer to its element type (so `ptr_offset` walks it), matching C array
    /// decay and the interpreters' region model. Taking the address of a temporary
    /// (a literal, a call result, arithmetic) is rejected with `L0458`.
    pub(crate) fn check_addr_of(
        &mut self,
        args: &[Expr],
        call_span: Span,
        scope: &Scope,
        function: &Function,
    ) -> Option<TypeRef> {
        self.expect_arg_count("addr_of", args, 1, function)?;
        let place = &args[0];
        if !matches!(
            place.kind,
            ExprKind::Variable(_) | ExprKind::Index { .. } | ExprKind::Field { .. }
        ) {
            self.diagnostics.push(SemanticDiagnostic::at(
                "L0458",
                "addr_of requires an addressable place (a local, an array element, or a \
                 struct field); the address of a temporary cannot be taken"
                    .to_string(),
                Some(function.name.clone()),
                place.span,
            ));
            return None;
        }
        let place_ty = self.check_expr(place, scope, function)?;
        if !self.type_has_layout(&place_ty) {
            self.diagnostics.push(SemanticDiagnostic::at(
                "L0431",
                format!(
                    "addr_of requires a place whose type has a defined memory layout but got `{}`",
                    place_ty.name
                ),
                Some(function.name.clone()),
                place.span,
            ));
            return None;
        }
        self.require_unsafe("addr_of", call_span, function)?;
        // A whole-array place decays to a pointer to its element type.
        let pointee = place_ty.array_element().unwrap_or(place_ty);
        Some(TypeRef::new(format!("ptr<{}>", pointee.name)))
    }

    /// `ptr_offset(p: ptr<T>, n: isize) -> ptr<T>`: element-scaled pointer
    /// arithmetic (`p + n*size_of(T)`). The pointee `T` must be a *sized* type so
    /// the scale factor is defined; an unsized `T` is rejected with `L0431`. The
    /// count `n` is an `isize`/`i64` signed element count. The result keeps the
    /// input pointer type.
    pub(crate) fn check_ptr_offset(
        &mut self,
        args: &[Expr],
        call_span: Span,
        scope: &Scope,
        function: &Function,
    ) -> Option<TypeRef> {
        self.expect_arg_count("ptr_offset", args, 2, function)?;
        let ptr_ty = self.check_expr(&args[0], scope, function)?;
        let pointee = self.expect_raw_pointer("ptr_offset", &ptr_ty, args[0].span, function)?;
        if !self.type_has_layout(&pointee) {
            self.diagnostics.push(SemanticDiagnostic::at(
                "L0431",
                format!(
                    "ptr_offset scales by size_of<T>, so its pointer's pointee `T` must be a \
                     sized type, but `{}` has no defined layout",
                    pointee.name
                ),
                Some(function.name.clone()),
                args[0].span,
            ));
            return None;
        }
        let count_ty = self.check_expr(&args[1], scope, function)?;
        if count_ty.name != "i64" && count_ty.name != "isize" {
            self.diagnostics.push(SemanticDiagnostic::at(
                "L0331",
                format!(
                    "ptr_offset expects an `isize`/`i64` element count but got `{}`",
                    count_ty.name
                ),
                Some(function.name.clone()),
                args[1].span,
            ));
            return None;
        }
        self.require_unsafe("ptr_offset", call_span, function)?;
        Some(ptr_ty)
    }

    /// `ptr_cast(p: ptr<T>) -> ptr<U>`: reinterpret a raw pointer's pointee type
    /// with no value conversion. The target `U` comes from the caller's expected
    /// annotation when it is a raw pointer (mirroring `int_to_ptr`), defaulting to
    /// `ptr<i64>` when there is no annotation. This is the minimal-consistent
    /// spelling: the delivered raw-pointer builtins take no turbofish, so the target
    /// element type is supplied by the `let bp ptr<byte> = ptr_cast(base)` context.
    ///
    /// # `ptr_cast` preserves the pointer MODEL
    ///
    /// Lullaby has two pointer models and they are **not convertible**: the legacy
    /// `ptr_T` heap box that only `alloc` produces (the interpreters model it as a
    /// heap-SLOT INDEX over a one-cell `Vec<Option<Value>>`, not an address), and the
    /// modern `ptr<T>` raw address from `addr_of`/`int_to_ptr`. `let`/parameter
    /// binding already enforces that (`L0303`/`L0313`).
    ///
    /// `ptr_cast` used to be the one hole in that wall, because it derived its result
    /// **purely from the caller's annotation and never from the operand**. That
    /// laundered a pointer across models in *both* directions:
    ///
    /// * `let q ptr<i64> = ptr_cast(alloc(8))` rewrote a box into an address, after
    ///   which `ptr_offset(q, 1)` type-checked — natively striding 8 bytes off a
    ///   one-cell payload into the next heap block's `[size]` header, the word
    ///   `__lullaby_alloc`'s free-list scan reads. Real allocator corruption.
    /// * `let fake ptr_i64 = ptr_cast(addr_of(buf[0]))` rewrote an address into a
    ///   box, falsifying the invariant that a `ptr_T`-typed expression is always
    ///   `alloc`-derived — which `native_object_rawptr.rs`'s `is_legacy_box_pointer`
    ///   spelling test relies on.
    ///
    /// The result model is therefore taken from the **operand**, and only the pointee
    /// within that model is retargetable:
    ///
    /// * A `ptr_T` operand yields exactly `ptr_T` — a box is one opaque cell, so
    ///   there is no pointee to reinterpret; the cast is an identity. A `ptr<U>` or
    ///   `ptr_U` annotation over it now correctly collides at the `let` (`L0303`).
    /// * A `ptr<T>` operand yields `ptr<U>` from a **modern** annotation only,
    ///   defaulting to `ptr<i64>`. A legacy `ptr_U` annotation no longer captures it.
    ///
    /// The native backend keeps its own `refuse_legacy_box_pointer` gate on the
    /// operand as defense in depth.
    ///
    /// # What is, and is not, a whole-program property
    ///
    /// This check was originally documented as making the backend's spelling test
    /// "sound as a whole-program property". **That over-claimed, and it still would.**
    /// `ptr_cast` was not the only annotation-governed pointer producer: `int_to_ptr`
    /// and `arena_alloc` carried the identical
    /// `expected.filter(|ty| ty.is_raw_pointer())` pattern. `arena_alloc` was a real
    /// third door and is now closed ([`annotated_address_type`]). `int_to_ptr` is
    /// **deliberately left open**, so the strong property is false and cannot be
    /// recovered. What actually holds:
    ///
    /// > **Every builtin whose pointer model is derivable now derives it.** `alloc` is
    /// > the only producer of the legacy spelling; `ptr_cast` takes its model from the
    /// > operand; `ptr_offset` preserves its operand's type; `addr_of` derives `ptr<T>`
    /// > from the place; `arena_alloc` yields only `ptr<T>`. `let`/parameter binding
    /// > (`L0303`/`L0313`) keeps the two spellings from meeting anywhere else.
    ///
    /// > **`int_to_ptr` is the sole remaining exception, and it is irreducible.** Its
    /// > operand is an `i64`, and **an integer carries no provenance** — so neither
    /// > model is derivable from it, by construction rather than by omission. On the
    /// > interpreters an integer genuinely may be either (a heap-slot handle below
    /// > `RAW_POINTER_BASE`, a byte address above it), and both round trips are
    /// > delivered and fixture-pinned: `run_ptr_cast.lby` reconstructs a real box from
    /// > `ptr_to_int(box)` as `ptr_i64`, and `freestanding_mmio_vga.lby` names
    /// > `0xB8000` as `ptr<i64>`. Its annotation is therefore an **`unsafe`
    /// > assertion**, not an inference — and a false one (`let fake ptr_i64 =
    /// > int_to_ptr(ptr_to_int(addr_of(buf[0])))`) still compiles.
    ///
    /// Three ways to close it were designed and attacked; all three failed:
    ///
    /// * **Track provenance into the `i64`.** Defeated by arithmetic, arrays, and
    ///   function boundaries — the integer is an ordinary value.
    /// * **Split the builtin** (`int_to_ptr` for addresses, `int_to_box` for handles).
    ///   Disproven empirically: `int_to_ptr(753664)` — a *pure constant* — already
    ///   yields a `ptr_i64` under a legacy annotation, so `int_to_box` would launder
    ///   identically. Renaming relocates the assertion; it does not remove it.
    /// * **Refuse the `addr_of`-derived shape.** `run_ptr_cast.lby` launders through a
    ///   temp var, which is indistinguishable from the legitimate round trip.
    ///
    /// Only removing `ptr_to_int(box)` from the language closes it, and that is an
    /// owner-level surface decision, not a checker fix.
    ///
    /// So the spelling test is **not** sound whole-program, and this doc makes no
    /// claim that anything downstream compensates for that. In particular, do **not**
    /// read the backend's `refuse_legacy_box_pointer` gate as containment: it is a
    /// prefix test on the *outer* type name, so it does not see a box model nested in a
    /// pointee (`ptr<ptr_i64>`), and the model mismatch is reachable in shapes whose
    /// operand is never spelled `ptr_T` at all. The gate guards the cases it names and
    /// nothing more.
    ///
    /// What is **not** implied either way: this is a spelling property, not provenance
    /// analysis. It says a `ptr_T` came from `alloc`; it says nothing about *which*
    /// box, whether it is live, or whether two `ptr_T`s alias. Lifetime is `L0350`'s
    /// job, and its limits are documented in `semantics_lifetime_alias.rs`.
    pub(crate) fn check_ptr_cast(
        &mut self,
        args: &[Expr],
        call_span: Span,
        expected: Option<&TypeRef>,
        scope: &Scope,
        function: &Function,
    ) -> Option<TypeRef> {
        self.expect_arg_count("ptr_cast", args, 1, function)?;
        let ptr_ty = self.check_expr(&args[0], scope, function)?;
        self.expect_raw_pointer("ptr_cast", &ptr_ty, args[0].span, function)?;
        self.require_unsafe("ptr_cast", call_span, function)?;
        // A legacy `alloc` box casts to itself: one opaque cell, nothing to retarget.
        if is_legacy_box_spelling(&ptr_ty) {
            return Some(ptr_ty);
        }
        // A modern `ptr<T>` retargets its pointee from a modern annotation only, so a
        // legacy `ptr_U` annotation cannot relabel an address as a box.
        Some(
            expected
                .filter(|ty| is_modern_raw_pointer(ty))
                .cloned()
                .unwrap_or_else(|| TypeRef::new("ptr<i64>")),
        )
    }
}

/// Whether `ty` uses the legacy `ptr_T` spelling that only `alloc` produces, as
/// opposed to the modern `ptr<T>` address spelling. Mirrors
/// `native_object_rawptr.rs`'s `is_legacy_box_pointer`; keeping `ptr_cast` from
/// crossing the two spellings is what keeps that backend test sound.
fn is_legacy_box_spelling(ty: &TypeRef) -> bool {
    ty.name.starts_with("ptr_")
}

/// Whether `ty` is the modern `ptr<T>` raw-pointer spelling (an address), excluding
/// the legacy `ptr_T` box spelling that `TypeRef::is_raw_pointer` also admits.
///
/// This is the predicate an annotation-driven pointer producer must filter its
/// expected type through **when the builtin's own model is fixed**.
/// `TypeRef::is_raw_pointer` admits both spellings, so using it to capture a caller's
/// annotation lets that annotation *choose the pointer model*. That is a bug for a
/// producer whose model is known (`arena_alloc`), and correct-by-design for the one
/// producer whose model genuinely is not (`int_to_ptr`). See
/// [`annotated_address_type`].
pub(crate) fn is_modern_raw_pointer(ty: &TypeRef) -> bool {
    ty.generic_arg("ptr").is_some()
}

/// The result type of a builtin that **mints a fresh machine address** whose model is
/// therefore known, and takes only its *pointee* from the caller's annotation. Today
/// that is `arena_alloc`.
///
/// # Why `arena_alloc` is annotation-governed, but only over the pointee
///
/// `check_ptr_cast` takes its result *model* from the operand, because its operand is
/// a pointer and so carries a model to preserve. `arena_alloc(region, count)` has no
/// such operand — a region name is a compile-time entity, not a value — so the
/// annotation must supply the pointee. But the **model** is fixed by what the builtin
/// *is*: an arena cell is a real address bumped out of a caller-owned `array<i64>`,
/// the host allocator is never involved, and **only `alloc` produces a box**. So the
/// result is always the modern `ptr<T>` spelling.
///
/// # The laundering this closes
///
/// `arena_alloc` used to filter the annotation through `TypeRef::is_raw_pointer`,
/// which admits the legacy `ptr_T` box spelling too, so the annotation could mint a
/// lie:
///
/// ```text
/// let fake ptr_i64 = arena_alloc(pool, 1)
/// ```
///
/// — a value spelled "I am an `alloc` box" over an arena cell, falsifying the
/// invariant `native_object_rawptr.rs`'s `is_legacy_box_pointer` spelling test rests
/// on. Filtering through [`is_modern_raw_pointer`] means a legacy annotation no longer
/// captures the result; the builtin yields its natural `ptr<T>` and the `let` then
/// collides at the existing `L0303` wall, exactly as the `ptr_cast` fix does.
///
/// Refusing the legacy spelling here costs nothing real: there was never a legitimate
/// meaning for it. A `ptr_T` from `arena_alloc` was *always* a false claim.
///
/// # Why `int_to_ptr` is NOT routed through here
///
/// `int_to_ptr` looks like the same pattern and is deliberately left alone. Its
/// operand is an `i64`, and on the interpreters an integer may be *either* model — a
/// heap-slot handle below `RAW_POINTER_BASE`, or a byte address above it. Both round
/// trips are delivered and fixture-pinned:
///
/// ```text
/// let back ptr_i64  = int_to_ptr(ptr_to_int(box))  # run_ptr_cast.lby — TRUTHFUL
/// let base ptr<i64> = int_to_ptr(753664)           # freestanding_mmio_vga.lby
/// ```
///
/// So `int_to_ptr` has **no derivable model** — an `i64` carries no provenance, and
/// restricting it to `ptr<T>` breaks the first fixture, which rebuilds a genuine box.
/// Its annotation is an `unsafe` assertion, not a hole. See `check_ptr_cast`'s
/// "What is, and is not, a whole-program property" for the closure designs that were
/// attacked and why each failed.
pub(crate) fn annotated_address_type(expected: Option<&TypeRef>) -> TypeRef {
    expected
        .filter(|ty| is_modern_raw_pointer(ty))
        .cloned()
        .unwrap_or_else(|| TypeRef::new("ptr<i64>"))
}
