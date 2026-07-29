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

## Wave status (2026-07-24, weekly-limit stop) — RESUME HERE

Six lanes were killed by a **weekly** quota limit. All work preserved as WIP commits on
their branches; nothing lost. main is `473a10d`, clean, and every merged change is
integrated-gate-verified. **Nothing below has merged — all six need finishing + review.**

| Branch (`agent-…`) | Task | Base | WIP | State when killed |
|---|---|---|---|---|
| `ad2d699021451a1d6` | loader tier attribution — **fixing review FAIL** | `5c62173` | 27f/+1607 | **Blocker fix WORKED** — last words: *"The no-CRT binary is no longer produced."* Was starting the full gate pass. Still owes: the 2 doc/comment accuracy fixes + teeth proof. |
| `a9c31bd061eba8075` | `interrupt`/`naked` — **fixing review FAIL** | `5c62173` | 52f/+3016 | Was rewriting the vacuous promotion test with a real control + fixing the tautological guard pins. Unclear whether the **L0446 fn-value bypass blocker** was fixed yet — verify first. |
| `ad7ac0ea16b3f6ac7` | actor back-pressure — **fixing review FAIL** | `0e66e2b` | 23f/+1429 | Was running the CLI suite detached. Verify both blockers (reply-slot teeth fixture; `bound` literal forms) actually landed. |
| `a2e9b7e1bc51809f1` | HOF pass-onward — **under review** | `0e66e2b` | 18f/+1332 | ⚠️ **The REVIEWER's last words were "Found a trap. Let me narrow it down."** — an unresolved potential finding. Re-dispatch the reviewer and have it re-derive; do NOT merge on the implementer's account. |
| `a86120db5f2ae68d0` | optimizer assign-path miscompile (sweep #3 F1) | `5c62173` | 23f/+1069 | Was running the CLI test after a fixture sweep. |
| `a92f1c48587129cea` | semantic assign-path gates (sweep #3 F3/F4) | `5c62173` | 13f/+535 | Implementation complete on disk; reviewer was dispatched and died immediately. Needs review from scratch (implementer filed no report). |

**Resume order:** (1) re-dispatch the HOF reviewer — it found something; (2) finish the three
FAIL round-trips; (3) review the two sweep-#3 fix lanes; (4) merge in D1 order, rebasing the
four `5c62173`-based branches onto whatever main has become; (5) integrated gate.

## Wave status (2026-07-23) — earlier stop

**Merged + integrated-gate-verified on main `0e66e2b`** (`cargo test --all` exit 0: CLI
609, `lullaby_ir` 531, semantics 308, 53 actor tests, 0 failures; clippy `-D warnings` 0;
fmt 0): the asm-operand class fix (`cefff8a`), the promotion flatness gate (`5a37349`),
and the closure-loop leak fixes (`b3af6c1`).

**Five lanes were killed mid-flight by a session limit.** All work is preserved as a WIP
commit on each branch; NOTHING is lost. Each needs *resuming*, then its adversarial
review, before merge. Resume by sending the branch's agent a message (its transcript is
intact) or re-dispatching against the committed WIP.

| Branch (worktree) | Task | Base | WIP size | State when killed |
|---|---|---|---|---|
| `agent-ad2d699021451a1d6` | `no-runtime` import contagion + cross-module span misattribution | `cefff8a` | 20 files, +1197 | waiting on its final suite; new `crates/lullaby_parser/src/origins.rs` |
| `agent-a9c31bd061eba8075` | `interrupt fn` / `naked fn` (§6 design) | `cefff8a` | 50 files, +2756 | strengthening a CLI assertion to a positive `compiled <name>` check |
| `agent-ad7ac0ea16b3f6ac7` | actor back-pressure + `try_tell` + `spawn bound N` | `0e66e2b` | 17 files, +1148 | writing docs §2.1/§2.2/§2.5 |
| `agent-a2e9b7e1bc51809f1` | multi-level HOF pass-onward | `0e66e2b` | 11 files, +470 | writing the escape probes |
| (sweep #3, no worktree) | silent-skip construct×walker hunt | — | none | barely started; re-dispatch fresh |

**Two lanes are based on `cefff8a`** (pre-Phase-0) — rebase them onto current main before
review, as was done for Phase 0 (expect a `repository_map.md` conflict; resolve by
grafting both sides' entries, then verify both survive).

## Sweep #3 (2026-07-24) — SIX FINDINGS, one new root-cause shape

Aimed at the *pattern* rather than a subsystem, and it paid: after sweep #2 hardened
`Stmt::Asm`, **every AST/IR/bytecode enum VARIANT now descends correctly** — the
productive axis was a different shape entirely.

**The new shape: a child-bearing struct FIELD silently dropped by `..`.**
`Stmt::Assign`'s `path: Vec<Place>`, where `Place::Index(Expr)` carries a whole
expression tree. Walkers destructuring `Stmt::Assign { name, value, .. }` never see it.
Four passes drop it (`walk_lifetimes`, both `semantics_array_extent` walkers,
`loader::collect_stmt_references`, `ir_optimizer_copyprop`/`_cse`); four correctly walk it
(`semantics_no_runtime`, `semantics_actor_ownership`, `semantics_consts`,
`semantics_checker_calls`). A second family: `semantics_aliases.rs`'s
`other => other.clone()`, which never enters expressions at all.

| # | Sev | Finding | Root cause | Status |
|---|---|---|---|---|
| 1 | **HIGH** | `--optimize full` returns a **wrong value** (99 vs 1) on ir+bytecode: a call in an assign-target index never clears copyprop's aliases, so a read is rewritten to a stale source. CSE has the byte-identical shape (latent). | `ir_optimizer_copyprop.rs:88`, `ir_optimizer_cse.rs:113` | fix lane dispatched |
| 2 | **HIGH** | **`L0392` cross-package privacy evaded** — a private cross-package fn called from an assign-target index compiles (`check` ok) and really executes. | `loader.rs:637` | queued behind loader-branch merge (file collision) |
| 3 | MED-HIGH | **`L0350` UAF not detected** through an assign-path index (the hoisted-variable form IS caught). | `semantics/lib.rs:2821` | fix lane dispatched |
| 4 | MED | **`L0463` skipped + real AST-vs-all divergence**: `a[len([0; n])] = 99` → AST runs and prints 99; ir/bytecode/native/wasm hit `L0501`. Also falsifies the "not reached in practice" comment at `bytecode_vm.rs:1868`. | `semantics_array_extent.rs:618` | fix lane dispatched |
| 5 | MED-LOW | Type alias never resolved in a **closure parameter** — valid program falsely rejected (`L0327`/`L0301`). `semantics_array_extent.rs:437` descends for exactly this reason; the two passes disagree. | `semantics_aliases.rs:316` | queued behind loader-branch merge |
| 6 | MED-LOW | Alias resolution reaches a `match` only in `Stmt::Expr` position; as a `let` RHS the same code is rejected `L0303`. | `semantics_aliases.rs:414` | queued behind loader-branch merge |

**Swept clean:** all 13 `Stmt::Asm` operand walkers (no regression of sweep #2), IR-optimizer
`IrStmt` *variant* coverage (exhaustive, no wildcards), `native_object_confine.rs`
(exhaustive, default-deny holds), native array-length inference (attacked two ways; caught
downstream by a hard layout check — fail-safe, though the fail-safety is defence-in-depth
rather than the inference being right), the `ExprKind`-exhaustive semantic passes, and OOB
array *store* parity. **Observation (not a finding):** `lullaby_lsp/analysis.rs:275` omits
`ForEach`/`If`/`Try`-catch bodies — editor-feature gap, no correctness impact.

**Next sweep starts here:** the variant axis is exhausted; hunt *fields* — every
child-bearing field reachable behind a `..`, and every `other => other.clone()` catch-all.
**Add a third axis (found by the loader review, 2026-07-24): KEY UNIQUENESS.** A
`(function, span)` lookup key in `semantics_no_runtime.rs:541` collides because impl-method
names are NOT unique across modules — `L0398` forbids method-vs-free-function collisions but
permits `Card::label` and `Coin::label`. Two same-named methods at the same line/column in
different files collide; `find` returns the first, so a heap type is never seen and **a
`to_string` executes inside a `--freestanding` no-CRT binary** (verified: exit 9). Pre-existing,
now being fixed. Generalize the lesson: **any map keyed by a display name in a flat merged
program is a candidate hole** — audit every such key for a uniqueness guarantee that actually
holds, rather than one that looks plausible.

## Sweep #4 (2026-07-29) — a CRITICAL multi-module miscompile

Aimed at the two axes prior sweeps *discovered* (fields behind `..`; key uniqueness).
**Axis 2 came back essentially clean** — 53 struct-like variants enumerated mechanically
across every AST/IR/bytecode enum, all 149k lines of `crates/` scanned for
`Enum::Variant { … .. }` patterns, named-vs-declared fields diffed: every remaining `..`
is legitimately partial. `Stmt::Try.catch_body` and `Stmt::For.step` — the two shapes most
expected to be wrong — were already correct everywhere. **Axis 3 is where the defects were.**

| # | Sev | Finding | Status |
|---|---|---|---|
| 1 | **CRITICAL** | **Closure ids are per-FILE and the loader merges without renumbering.** The simplest two-module program with one closure each gives **four-way disagreement**: expected 11020, interpreters 20020 (last-wins table), native exe 11011 (first-wins symbol), `check` exit 0. Also breaks **type soundness** — a `fn(i64) -> string` closure executes an `i64` body (`L0417` at runtime), and a 1-param closure executes a 2-param body (`L0402`). The emitted `.lbc` is itself malformed (two entries with `"id": 0`). Never caught because **not one multi-module fixture contains a closure literal**. | fix lane dispatched |
| 2 | MED (latent) | Actor use-after-send keys its type map on `(line, column)` alone (`semantics_actor_ownership.rs:220`) — the exact shape of the already-fixed `(function, span)` defect, and it contradicts the documented invariant on `ExpressionType::module`. **Could not be made to fire**: `check_message_ownership` runs at the end of `validate_function`, so current-function entries are always most-recently-pushed and win. Goes live the moment anything reorders or re-appends `expression_types`. | queued (cheap hardening) |
| 3 | LOW-MED (latent) | `mangle_method` sanitization is not injective — `Wrap<i64>` and a struct named `Wrap_i64_` both mangle to `$mth$Wrap_i64_$get`, and the dedup `HashSet` then synthesizes one body for both call sites. Today the layout check catches it (`L0339` clean refusal, **not** a miscompile); becomes one as soon as two colliding receivers share a native layout. The module doc calls the mangled name "unique, collision-free" — it isn't. | queued |
| 4 | LOW | `L0357` attributed to the entry file when two modules declare the same impl-method name (`build_origins` keys on the ambiguous display name). | queued |

**Sweep #5 starts here:** axis 2 is exhausted (scanner scripts kept, re-runnable); axis 3 is
productive and only partly audited. The generative question that found #1 and #2: *what else
is assigned or keyed per-file and then merged?* — closure ids were the answer; verify actor
ids, region ids, generic instantiation ids, and `$mth$` names.

## Found in passing by the optimizer review (2026-07-29) — NOT that branch's defect

- **HIGH — native shadow name-resolution crashes.** A module function called *textually
  before* a same-named `fn`-typed local is declared produces an **access violation
  (`0xC0000005`)** in `lullaby native` output. With a non-inlinable callee — the optimizer
  entirely uninvolved — **main and the fixed branch crash identically**; both interpreters
  return 82 correctly. Main only appeared to "work" on the inlinable variant because the
  shadowing bug replaced both calls with arithmetic, yielding a *wrong* value (440) instead
  of a crash — i.e. one defect was masking another. A no-shadow control compiles and returns
  82, so this is specific to native's shadow name-resolution, not to shadowing generally.
  **Wrong-value-or-crash in shipped native output; needs its own increment.**
- **The pre-existing `assert_parity` WASM exec cases were blind to optimizer miscompiles** —
  proven: with an inliner substitution bug injected, 39 exec tests passed while only the new
  `assert_parity_after_inlining` case failed (63 vs 42). Any future WASM value-semantics
  test should go through the optimizer-aware helper.

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
