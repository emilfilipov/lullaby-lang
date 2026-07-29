# Lullaby — the finish-line plan to 1.0-stable (2026-07-20)

The execution plan from here to technical 1.0-stable. Grounded in
`1_0_stable_assessment.md` (the judgment) and `road_to_1_0_stable.md` (the
tracking). Stops **before** branding/packaging (Phase 8), per standing owner
instruction.

## The core insight that orders everything

There are **no known open miscompiles** — every defect discovered this cycle was
fixed. So "fix all existing and discovered issues" resolves to three buckets, in
priority order:

1. **Stability debt (the real 1.0-stable gate):** undiscovered miscompiles. The
   review defect-rate stayed near 1-in-2/1-in-3 and *every* FAIL was real, so
   more surely exist. **Attacking this is the highest-leverage work and comes
   first.** Success = an adversarial sweep of the whole surface comes up empty.
2. **Completion debt (the "100%"/spanning-set axis):** the narrow remaining
   features. All are additive; none is a spanning-set blocker.
3. **Coverage debt (native-codegen deferrals):** dozens of shapes that *skip
   cleanly to the interpreters* (correct-or-refuse). These are perf/parity, not
   correctness — lowest urgency, and most are explicitly post-1.0.

The trap to avoid: treating the long feature/coverage list as "the work" and
declaring stable on a green feature checklist. **Feature-complete ≠ stable.**

## Phase 0 — Hygiene (small, parallel, no decisions) — START NOW

Clears the known infrastructure debt so later phases run clean.
- **P0.1 — Split `native_object_eligibility.rs`** (6 lines over the ~1500 cap;
  already chipped). Behavior-preserving.
- **P0.2 — TCP fixture TOCTOU: verified real but LOW-SEVERITY, deprioritized**
  (`cli.rs:261`/`:637` probe-and-release; comment already acknowledges "a small race
  window but adequate"; never fired across 1536 executions). The fix needs a
  streamed-stdout fixture redesign (a server can't use `.output()` without
  deadlocking) — disproportionate effort for a non-firing test race. Left as a known
  low-priority item; hardening the compiler outranks polishing a test race.
- **P0.3 — Test-runner robustness: ALREADY DELIVERED** (`60eff81` + follow-ups;
  verified 2026-07-20 — the plan mis-listed it from a stale roadmap read). `lullaby
  test` already runs each test in an isolated child with a 60s default `--timeout`;
  a crash → reported FAIL, a hang → reported timeout, the run continues with a clean
  summary. No work remains. **Lesson: verify each plan item against the repo before
  dispatching — the roadmap/assessment can lag the code.**

## Phase 1 — Hardening (THE 1.0-stable gate) — the priority

Drive the defect-discovery rate toward zero. Expect this phase to *find real
bugs* — that is success, not failure; each found bug is fixed and its shape folded
into the permanent fuzzers.

- **P1.1 — Broaden the differential fuzzers over the newest/thinnest subsystems.**
  One fuzzer-strengthening lane per area, each generating random programs and
  asserting all tiers agree (native == interpreters == WASM where applicable):
  arena call-graphs (deeper graphs, mixed retain/non-retain, recursion); asm
  operands (random reg/clobber/width combos vs a golden oracle); the value-copy
  class (WASM + native, all aggregate shapes); escaping/returned closures; native
  string/map/enum/generic heap matrix; const-sized arrays + fixed-array struct
  fields.
- **P1.2 — Fold every reviewer-hand-invented shape into the permanent fuzzers.**
  This cycle's reviews repeatedly caught bugs by constructing a shape the fuzzer
  didn't generate. Sweep the review transcripts / test suites and ensure each such
  shape is now *generated*, not just pinned once. Makes the fuzzers monotonically
  stronger.
- **P1.3 — Close the execution-verification gaps** (retire "verified by reading"):
  a Linux execution job that runs the full native-ELF suite under `linux/amd64`
  Docker (already used for direct-ELF + the syscall — extend to the whole suite),
  and AArch64 under QEMU. The `wasmi` gate already covers WASM; extend its coverage.
- **P1.4 — The cross-subsystem adversarial sweep (the exit test).** Dispatch
  reviewer-agents to hunt cross-tier divergences / UAFs across the *whole* surface,
  untied to any new feature — every backend pair, the memory model, the freestanding
  tier, FFI, generics, actors. **Exit criterion for Phase 1 (and the stable gate):
  N consecutive sweep lanes find nothing real.**

## Phase 1 — progress log

- **Sweep #1 (2026-07-20, base `2b52b96`) — the codegen surface came up CLEAN.**
  ~45 aggressive probes across the newest subsystems (arena cross-call + promotion,
  asm operands, escaping closures, the native heap-aggregate value-semantics matrix,
  const/narrow/fixed arrays, and the cross-feature seams) on all interpreters +
  native + WASM: **zero native/WASM miscompiles, zero use-after-frees/segfaults.**
  The first empty-with-real-breadth result — the first data point toward the defect
  rate decaying. Two items surfaced, neither a codegen miscompile:
  - **Finding 1 (MEDIUM, oracle-integrity — FIXED `9268169`+`ab7c272`):** the three
    interpreters stack-overflowed the *host process* on deep recursion at *different*
    depths, blinding the differential fuzzers past ~200 frames. Fixed: all three
    interpreters now evaluate on a 2 GiB-stack thread and share a uniform
    `INTERPRETER_RECURSION_LIMIT = 20000` → a clean **non-catchable** `L0466`
    (a fault like a bounds/div-by-zero, not a `try`/`catch`-recoverable error — a
    settled semantic, consistent with A5). Verified transparent (900 fixture×backend
    comparisons, 0 value changes). Bonus: fixed an O(depth²) traceback rebuild
    (a 50k-deep error took 87s → O(depth)). Oracle **permanently hardened**:
    `fuzz_recursion.rs` now generates deep recursion (agreeing at 9000 frames), so
    the blind spot cannot reopen.
  - **Finding 2 (LOW, documented limitation — noted, not a defect):** the fixed
    ~1 MiB bump heap. Non-arena code that leaks (e.g. a promoting factory in a loop
    under a *non-arena* caller) traps cleanly with `ud2` at ~100k iterations where
    the interpreters (unbounded host heap) complete. Correct-but-conservative (native
    traps deterministically, never corrupts); the arena-eligible counterpart runs
    bounded and correct. **Owner-visible limitation:** a growable/larger native heap
    is a post-sweep design option if larger non-arena programs need it — not a
    miscompile, not a stable blocker.
  Continue: further sweeps after the oracle fix (which lets them go deeper), each
  folding its shapes into the permanent fuzzers, until a sweep + a fuzzer run over
  the whole surface finds nothing real.
- **Sweep #2 (2026-07-21, base `7174ca7`) — three disjoint hunter lanes; ONE REAL
  MISCOMPILE found and fixed.** Ran against the now-deep-recursion-valid oracle.
  - **Lane A (deep recursion × arena) — CLEAN.** ~45 programs; every native outcome
    was exact agreement with all three interpreters, a clean `L0339` refusal, or a
    clean output-free trap at a documented capacity limit (the OS-stack ceiling
    ~13k frames; the ~1 MiB bump-heap under *arena-denied* recursion). No wrong
    value, no corruption, no interpreter divergence. Positively confirmed arena
    reclamation: `confined_200k` (200k allocs in a confined loop) runs correct on
    native because per-iteration sub-regions reclaim.
  - **Lane B (heap-aggregate value semantics under generics/nesting) — FOUND A
    MISCOMPILE (fixed `3ec6495`).** The historical `let g = f` aliasing class swept
    clean on all four tiers, but the hunt surfaced a *different*, high-severity hole:
    **WASM array/list element access had no bounds check** — an OOB index computed a
    raw linear-memory offset and silently read/**wrote** a neighboring heap object,
    where native traps (`ud2`) and the interpreters raise `L0413`. Green across the
    entire existing suite; found only by asking "what if the index is out of range?"
    Fixed by an unsigned-compare + `unreachable` trap on every checked element path
    (array read/write incl. struct-array-field, `list` get/set, empty-`list` pop),
    with 8 OOB exec-parity tests under `wasmi` **proven to fail pre-fix**. Reviewed
    PASS (teeth reproduced independently; whole-class completeness audited — no other
    unchecked element path survives; string `s[i]` is unsupported on WASM so emits no
    offset; maps are key-hashed). One reviewer FAIL round-tripped: a stale
    "relies on linear-memory trapping" claim in `wasm.rs`'s overview, corrected
    before merge. **This is the permanent-fuzzer gap to close next (P1.2): the
    differential fuzzers generate no OOB indices — fold OOB index generation in so
    the class is generated, not just pinned.**
  - **Lane C (freestanding/pointer/asm/FFI) — FOUND A HIGH-SEVERITY SOUNDNESS HOLE.**
    34 programs; FFI, both pointer models, arena region tracking, byte reinterpretation
    and the asm register-promotion exclusion all swept **clean** (every documented
    deferral refuses exactly as specified — no silent miscompile on those surfaces).
    But **inline-`asm` operand expressions are invisible to semantic passes**:
    `Stmt::Asm { .. } => {}` is grouped with genuinely-childless statements
    (`Break`/`Continue`/`Return(None)`) and returns without walking the operand block,
    so every check reachable only from `check_expr` never runs on `in <reg> = <expr>`.
    Consequence: **a real heap allocation compiles and RUNS inside a `no-runtime`
    freestanding binary** (`len(to_string(addr))` from a runtime stack address →
    exe returns 707; `check` exits 0 where `L0441` is required) — violating the tier's
    hard rule #1. The same blind spot **defeats the use-after-free analysis** (`L0350`
    disappears when the `load(p)` moves into an operand). Traced to **six more passes**
    with the identical skip-without-descending pattern (array-extent ×2, actor-ownership,
    consts, semantics lib, loader). Fix in flight — the whole class, with a shape that
    makes reintroduction a compile error. **Cause: the arm predates the operand block
    and was written by analogy with childless statements; no pass that matched it that
    way was revisited when operands were added.**
  - **Lane C findings 2–3 (queued — they touch `loader.rs`, so they sequence AFTER the
    Finding-1 fix merges).** (2) MEDIUM: `no-runtime` is **contagious across imports** —
    a hosted program that merely imports a freestanding module is forced into the
    freestanding tier and wrongly rejected, with a diagnostic naming a directive the
    file does not contain. Known in a code comment as "conservative default-deny", but
    it contradicts `freestanding_tier_design.md` §1's stated goal (unit-test a
    `no-runtime` driver in a hosted harness), appears in no doc, and is unactionable
    from the message. Over-rejection only — the soundness-relevant direction is correct.
    (3) LOW: cross-module `L0441` reports the *importing* file's path with the
    *imported* file's line numbers (flat module merge loses per-module span attribution).
  - **Lane C note (accepted, not a defect):** an **undeclared** `asm` clobber of a
    callee-saved register silently corrupts a caller's promoted local (returns 222 vs 3).
    Correctly the author's contract — the body is opaque bytes, exactly as in C and
    Rust's `asm!` — but it is the one place a `no-runtime` author gets silent corruption
    with no diagnostic and no interpreter cross-check, and only the *declared* case is
    pinned. **Add a negative fixture documenting it as intended** so the suite records
    the accepted edge.

## Phase 1 — queued residual (found by review, proven pre-existing)

- **Closure-loop reclamation is NARROWED, NOT CLOSED** (Phase-0 reviewer, 2026-07-23).
  Two shapes still compile natively and trap `0xC000001D` while all three interpreters
  answer correctly — both proven pre-existing (identical on the emulated base), so not
  regressions, but the correct-or-refuse violation class must NOT be marked closed:
  1. a **factory-returned** closure local (`let g = make(i)`) in a loop of an
     arena-**denied** function — `call_returned_callables` has no drop path at all;
  2. a closure literal declared one level deeper (**inside an `if` under the loop**) —
     correctly refused by the default-deny, therefore never reclaimed.
  Next increment after the current wave; same teeth discipline (≥100k iterations so the
  fixture provably traps without the fix).

- **A HOF chain whose TERMINAL SINK allocates** (stage-3c reviewer, 2026-07-28). A third
  shape in the same class, and the one the multi-level widening made newly *reachable*:

  ```
  fn inner f fn(i64) -> i64 v i64 -> i64
      let s string = "abcd"          # a per-call fresh heap temporary
      f(v) + len(s)
  fn outer f fn(i64) -> i64 v i64 -> i64
      inner(f, v)
  fn main -> i64
      let n i64 = 2
      let total i64 = 0
      for i from 0 to 100000
          let c fn(i64) -> i64 = fn x i64 -> x + n
          total = (total + outer(c, 1)) % 251
      total
  ```

  Interpreters answer **219**; native traps `0xC000001D`. **Proven pre-existing** — the
  single-level stage-3a form traps identically, and 101 iterations passes — so it is NOT
  a stage-3c regression and was deliberately not fixed in that change. But stage 3c
  **widens its reach**: at chain depth ≥ 2 this program previously refused with `L0339`
  and ran correctly on the interpreters; now it compiles, so the pre-existing trap
  becomes observable at a depth where it was not before.

  Root cause is the documented **cross-call fresh-temporary** class, not the closure
  block: a sink calls through its `fn` parameter, which is an indirect callee, so
  criterion 3 `all_callees_non_retaining` fails and the sink is held **off the arena** —
  and its per-call fresh heap `string` consequently has no drop path. The closure block
  itself IS reclaimed (the per-iteration drop covers it); what leaks is the callee's own
  heap temporary. Fixing it means giving an off-arena callee a drop path for its fresh
  temporaries — the same underlying work items 1 and 2 need, which is why all three
  belong together in one increment rather than being patched per-shape.

## Phase 2 — Completions (the spanning-set "100%")

Each design→build→adversarial-review, serialized where files collide.
- **P2.1 — Closures stage 3c:** heap/aggregate captures and mutable-capture rebind.
  (Heap captures now have a home — the arena/RC model is complete — so this is
  unblocked.) **Multi-level HOF chains SHIPPED** (2026-07-28): a `fn` parameter may be
  passed onward at a proven sink position to any depth, resolved by `build_hof_index`'s
  module-wide memoized DFS with SCCs pre-poisoned; recursive chains stay refused by
  design. See `documents/native_backend_contract.md`. Note the residual it made newly
  reachable, logged in Phase 1 above.
- **P2.2 — FFI fn-pointer returns** (completes A3's 1.0 scope; struct-by-value +
  deep marshalling stay post-1.0 by decision).
- **P2.3 — Actor back-pressure** (bounded mailboxes; scheduler change) and
  **P2.4 — actor stage-6 native/WASM codegen** (actors are AST-tier-only today —
  the larger piece; assess scope first).
- **P2.5 — Interrupt/naked function attributes** (kernel IDT; small syntax — flag
  to owner).
- **P2.6 — Native-codegen coverage worth closing for real programs:** prioritize
  only shapes that block *expressing* a program natively (most deferrals just run
  on the interpreter). Candidates: `parse_f64`/`to_string(f64)` (dtoa), deeper heap
  nesting if a real program needs it. Defer the rest as documented perf gaps.
- **Post-1.0 (do NOT pursue for 1.0):** full const-fn eval, deep FFI marshalling,
  mnemonic-template `asm` (the byte form is the escape hatch), a `volatile`/`repr`
  pointer qualifier.

## Phase 3 — Parked owner decisions (need the owner; surfaced, not decided by me)

- **P3.1 — `test_*` vs `test "name"` block syntax.** A user-facing surface choice.
- **P3.2 — Unify `ptr_i64` with `ptr<T>`.** The P0 (nested box laundering)
  strengthened this: the two-model split is where several laundering routes lived.
  Unifying could retire an entire bug class — worth a design pass to scope, then an
  owner decision. Highest-value of the parked items for *stability*.

## Phase 4 — Declare 1.0-stable

Gate (all four): Phase 1 exit met (defect rate demonstrably decayed — the empty
sweep); Phase 2 spanning-set complete; Phase 3 decisions resolved; docs/roadmap
reconciled and `1_0_stable_assessment.md` refreshed to "stable demonstrated." Then
**stop** for the owner to open the branding/packaging phase — do not proceed into
Phase 8.

## Sequencing & cadence

- Phase 0 + Phase 1 start immediately and run in parallel (hygiene is disjoint;
  hardening is the priority). Phase 2 completions interleave as review capacity
  and file-ownership allow, but **hardening is weighted over features** — a found
  miscompile preempts a new primitive.
- Every lane keeps the session's discipline: design-before-build in
  correctness-critical code, adversarial review with proven teeth, real exit codes,
  no merge on an unverified claim.
- The parked decisions (Phase 3) are surfaced to the owner early so they don't
  block the endgame.

**Bottom line:** the distance to 1.0-stable is mostly Phase 1 (prove it doesn't
miscompile), not Phase 2 (add the last features). Weight the effort accordingly.
