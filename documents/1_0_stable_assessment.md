# Lullaby 1.0-stable — honest assessment (2026-07-20)

An assessment of where 1.0-stable actually stands, written after the arena
memory model and the kernel-capability tripod both landed. Companion to
`road_to_1_0_stable.md` (the tracking doc); this is the *judgment* layer.

## Headline

**The spanning-primitive set is substantially complete. The gate to 1.0-*stable*
is not feature coverage — it is the defect-discovery rate, which is still
non-trivial and has not visibly decayed.** Lullaby can express the programs 1.0
promised, including a kernel. Whether it can be *trusted* to compile them
correctly is a separate, less-finished question, and it is the real work left.

Two axes, held apart deliberately:

- **Can it express any program? — ~90%.** High and climbing; the remaining gaps
  are narrow.
- **Will it miscompile them? — unmeasured-but-improving.** The honest number is
  not a percentage; it is a *rate* (below).

## 1. Spanning-primitive coverage (the "express any program" axis)

**Shipped and reviewed** (a representative, not exhaustive, list):
- Scalars + wrapping/checked/saturating integer ops; f32/f64; strings, `list`,
  `map`; user structs/enums; **generics** (structs, enums, methods, multi-param,
  bounds) across interpreters + native + WASM.
- **Const-sized arrays** `array<T, N>` with `[v; count]` fill — frontend + all
  tiers; **native inline by-value** fixed-array struct fields + whole-field ops.
- **Closures**: scalar captures, HOF (non-escaping), and **returned/escaping
  closures** (stage 3b) natively.
- **Actors** stages 1–5 (spawn/tell, ask/await/Future, ownership, supervision,
  `join_all`/`select`) on the AST tier; clean tier deferral elsewhere.
- **FFI**: base C ABI + **callbacks** (fn-pointer params).
- **The freestanding / kernel tier** (see §3).
- Five execution backends: AST, IR, and bytecode interpreters; native x86-64
  (COFF + direct-PE + direct-ELF); WASM.

**Genuinely remaining (narrow, none a spanning-set blocker):**
- Closures stage 3c: heap/aggregate captures, mutable-capture rebind, multi-level
  HOF chains.
- FFI: fn-pointer **returns**, struct-by-value, deep collection marshalling
  (marshalling is post-1.0 by decision).
- Actors: back-pressure (bounded mailboxes), and **stage-6 native/WASM actor
  codegen** (actors are AST-tier-only today).
- Kernel niceties: **interrupt/naked** function attributes (for the IDT), a
  **mnemonic-template `asm`** (a clean superset of the shipped byte form; needs an
  encoder).
- Full const-fn evaluation (post-1.0 by decision).

None of these prevent expressing a program; they are coverage/ergonomics.

## 2. The safe-tier memory model (COMPLETE)

The arena-first model is fully realized and adversarially verified end to end:
implicit function + per-iteration loop regions, **cross-call reclamation** (an
inferred per-function retention summary, default-deny on recursion/indirect),
explicit `region` blocks, and **escape promotion** (mark-advance of a returned
closure into the caller's region). `ref`/RC is the opt-in secondary tool;
raw-pointer `unsafe` is the freestanding subset. Native reclaims; the interpreters
never reclaim, so `native == interpreters` is a value-neutral safety oracle that a
premature free would break — this is the backbone of the arena's correctness story.

## 3. Kernel capability (the tripod is COMPLETE)

The owner's 1.0 vision required Lullaby to write a kernel. That needs three things,
now all shipped:
- **Output** — direct-PE (Windows) and **direct-ELF** (Linux) linker-free
  executable emitters; execution-verified under Docker with no toolchain.
- **Memory** — the safe-tier arena reclaims with no runtime (§2); static-buffer
  arenas in the freestanding tier.
- **Control** — **inline `asm` with operand binding** (syscalls, control
  registers, MSRs over program values), on top of already-shipped raw pointers,
  MMIO, and port I/O.
A Linux `write`+`exit` syscall driven entirely through operand `asm` runs under
Docker. "Lullaby can express a kernel" is now a checklist item, not a slogan.

## 4. The stability axis (the REAL 1.0-stable gate)

This is where "stable" is won or lost, and it is the honest weak point.

**The signal:** across this session's adversarial review lanes, roughly **one in
two to one in three reviews found a genuine defect** — and *every* FAIL was a real
bug, never noise. The defects were not cosmetic: a live memory-safety
use-after-free reachable from `check`-passing code (nested box-model laundering); a
whole backend (WASM) that had *never been executed in tests*, hiding a value-copy
miscompile class; a shadowing slot-aliasing UAF that shipped on `main` independent
of this session; cross-tier divergences in ternary-closure lowering; the
`alloc_mode` nesting bug that would have been a UAF the moment arenas nested.

**Why the rate stayed high:** each new feature *surfaced latent bugs in its
interaction with existing code* — widening the arena exposed segfaults unreachable
in leaf functions; a new `region` scope reintroduced a just-closed UAF through a
flatten; every layer of verification turned out to need its own teeth-check
(tests that couldn't fail, fuzzers that missed code paths, `readelf`/`ld`/Docker
all blind to `sh_info`, fail-fast truncating a run). This is *healthy* churn —
it means the review loop is catching things — but a sustained real-defect rate is
**definitionally incompatible with "stable."**

**What "stable" requires:** the defect-discovery rate must decay toward zero. It
has not visibly done so, because feature work kept adding surface. Stability is
therefore **not yet demonstrated**, regardless of how complete the feature set is.

**What's still verified by reading, not execution** (the residual coverage gaps):
- POSIX runtime behavior on some paths (though Docker now covers ELF + the syscall,
  and `wasmi` now gives WASM an execution gate — both closed this session).
- AArch64 native (compile-checked, not run here).
- The `cfg(unix)` branches on a Windows host (Linux cross-lint compiles them; a
  real Linux execution job would retire "verified by reading").

## 5. Recommended finish sequence

1. **A dedicated hardening pass — the highest-leverage next step.** Broaden the
   differential fuzzers over the newest subsystems (arena call-graphs, asm
   operands, the value-copy class, escaping closures); add the fuzzer shapes each
   review had to invent by hand; close the execution-verification gaps (a real
   Linux/WSL runner to retire verified-by-reading for POSIX + AArch64). This
   attacks the *rate* directly, which is the actual gate. **Do this before
   declaring stable, ideally before more features.**
2. **The narrow completions** (interrupt/naked, FFI fn-ptr returns, back-pressure,
   closure 3c) — each design→build→review; a few carry small syntax decisions for
   the owner. Feature-completeness, lower urgency than (1).
3. **Declare 1.0-stable when the defect rate demonstrably decays** — e.g. a run of
   N consecutive adversarial review lanes across the whole surface finding nothing
   real. That empirical decay, not the feature checklist, is the stable bar.

## 6. Known open items (tracked, non-blocking)

- **Parked owner decisions:** `test_*` vs `test "name"` block syntax; unifying
  `ptr_i64` with `ptr<T>` (the P0 strengthened the case — the two-model split is
  where several laundering routes lived).
- **File-size backlog:** several `native_object_*.rs` / `lib.rs` files are at/over
  the ~1500 cap (a behavior-preserving split of `native_object_eligibility.rs` is
  chipped).
- **Branding/packaging (Phase 8):** deliberately out of scope until technical 1.0
  is done — logo, installers, the website.

## Bottom line

Feature-complete enough to call the spanning set essentially done; **not yet
demonstrably stable.** The most valuable thing to do next is not another primitive
— it is to stop the defect-discovery rate from finding real bugs, by hardening the
newest subsystems until an adversarial sweep comes up empty. That is the honest
distance between here and 1.0-*stable*.
