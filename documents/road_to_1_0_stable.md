# Road to 1.0-stable — Decisions and Gaps

**Purpose:** the single tracking doc for what still needs deciding or building before
Lullaby can be stamped **1.0-stable** (a real API-stability promise, not preview).
Complements the architecture docs — it does not restate them:
[execution_tiers_and_1_0_scope.md](execution_tiers_and_1_0_scope.md) (identity,
memory model, two tiers, kernel scope), [concurrency_model_design.md](concurrency_model_design.md),
[freestanding_tier_design.md](freestanding_tier_design.md), and the split/opt
backlogs ([large_file_split_plan.md](large_file_split_plan.md),
[optimization_opportunities.md](optimization_opportunities.md)).

Status values: **OPEN** (needs owner decision) · **PLANNED** (decided, needs
building) · **CONFIRMED GAP** (verified missing in the current compiler).

## Already decided AND built (context — not open)
- **Module/package system** — file-as-module, `import` + `pub` exports, multi-file
  projects via a `lullaby.json` manifest with local path dependencies.
- **Numeric type set** — `i8…i64`, `u8…u64`, `isize`/`usize`, `f32`/`f64`, `bool`,
  `char`, `byte`.
- **FFI (base)** — `extern`/`export fn` over the C scalar + `ptr<T>` + `cstr` set
  (L0423/L0424/L0426).
- **Traits + bounded generics on functions** — `trait`/`impl` with receiver dispatch,
  `<T: Trait>` / `<T: A + B>` bounds.
- **Memory model** (arena-first + RC + unsafe), **concurrency** (actors) and
  **freestanding** surface syntax — all decided (see the architecture docs).

## Already decided, NOT yet built (the engineering bulk — tracked, not open)
Actors, the freestanding/kernel tier, arena stages 3–5 (explicit `region` blocks,
escape/promotion, static-buffer arenas), native-aggregate expansion + the native
optimization backlog (O(n²) `for c in s`, array-by-ref), and Linux tier-1 /
direct-ELF. These need no new decision — they're in flight or queued.

---

## A. Open decisions — need owner sign-off

### A1. Generic user types — **OPEN / CONFIRMED GAP**
User-defined generic types (`struct Stack<T>`, `enum Opt<T>`) do not parse today
(`enum Opt<T>` → **L0205**, `struct Stack<T>` → **L0205**). Bounded generic
*functions* work; generic *types* do not.
- **Decision:** in 1.0, or post-1.0?
- **Recommendation: 1.0.** You cannot write reusable containers or a real
  data-structure library without them; that is core to a "spanning set." This is
  the single largest open language-feature decision.

### A2. Const / compile-time evaluation — **OPEN / CONFIRMED GAP**
There is no `const` keyword or compile-time-constant story (`const N i64 = 5` →
**L0201**; no `const` token in lexer/parser). Kernel/embedded code needs it
(MMIO addresses, fixed buffer sizes, `const`-sized arrays).
- **Decision:** minimal const-eval in 1.0?
- **Recommendation: yes, a minimal story in 1.0** — named compile-time constants
  and const-sized arrays. Full const-fn evaluation can be post-1.0.

### A3. FFI completeness — **OPEN**
Base FFI ships; deferred today (L0424): **callbacks (fn pointers), struct-by-value,
and `string`/`list`/`map` marshalling**.
- **Decision:** which of these are 1.0?
- **Recommendation: callbacks (fn pointers across the C ABI) in 1.0** — real C
  interop, drivers, and registering handlers need them. Deep struct/collection
  marshalling can be post-1.0.

### A4. Integer overflow semantics — **OPEN**
Arithmetic is **wrapping** everywhere today (the native path relies on it).
- **Decision:** make wrapping the conscious 1.0 default, or add checked/trapping?
- **Recommendation: keep wrapping as the default** (matches the fast native path and
  kernel expectations) and provide explicit `checked_*` / `saturating_*` operations
  in the stdlib. The point is to *decide* it, not inherit it by accident.

### A5. Safe-tier failure semantics — **OPEN**
What a bounds-check failure / `option` unwrap-on-`none` / divide-by-zero does in the
**safe** tier (the freestanding tier already routes to a user panic handler).
- **Decision:** abort-only, or unwinding/catchable?
- **Recommendation: abort-with-diagnostic, no unwinding.** It's deterministic and
  GC-free-friendly; recoverable errors already flow through `result`/`?`/throw-catch,
  so panics are reserved for bugs.

---

## B. Planned but unscheduled — no decision needed, needs building

### B1. Closures native codegen — **PLANNED**
Closures run on the interpreters only today; native-AOT-completeness (the
"no silent native fallback" decision) requires native codegen for them. Sizeable
native-backend work; schedule after the arena/native-aggregate line settles.

### B2. Concrete stdlib contents — **PLANNED**
The API-stability *posture* is decided (freeze a small core, version the rest) but
the *contents* are not enumerated. Define the 1.0 stdlib surface — strings,
collections, math, fs, io, time, os, (maybe net) — and mark each item **stable** vs
**extended/experimental**. Do this near the finish line, informed by dogfooding.

### B3. "Stable"-grade toolchain — **PLANNED**
Before stamping "stable": a built-in **test runner**, **debug info on Linux/macOS**
(DWARF — CodeView is Windows-only today), and **LSP + package-manager maturity**.
These are toolchain-completeness items, not language decisions.

---

## Decision log
| # | Item | Status | Recommendation | Owner decision |
|---|------|--------|----------------|----------------|
| A1 | Generic user types | OPEN (confirmed gap) | 1.0 | — |
| A2 | Const / compile-time eval | OPEN (confirmed gap) | minimal in 1.0 | — |
| A3 | FFI completeness | OPEN | callbacks in 1.0 | — |
| A4 | Integer overflow semantics | OPEN | wrapping default + checked ops | — |
| A5 | Safe-tier failure semantics | OPEN | abort + diagnostic, no unwind | — |
| B1 | Closures native codegen | PLANNED | schedule post-arena | n/a |
| B2 | Concrete stdlib contents | PLANNED | enumerate near finish | n/a |
| B3 | Stable-grade toolchain | PLANNED | test runner + DWARF + LSP/pkg | n/a |

Fill the **Owner decision** column as calls are made, then move each into the
relevant architecture doc / implementation backlog.
