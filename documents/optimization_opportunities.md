# Optimization Opportunities — Tokens and Performance

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

This document is the standing analysis of where Lullaby can close its two
marketed gaps against the cross-language benchmark
([benchmarks/crosslang](../benchmarks/crosslang/README.md)): **fewer tokens than
Python** and **native performance at or below C**. Rerun the benchmark and
refresh this analysis whenever the corpus, the language, or the optimizer
changes.

## Current standing (o200k_base tokens; same-box native timings)

| Metric | Lullaby | Target | Gap |
|---|---|---|---|
| Corpus tokens | 20,321 | < 19,120 (Python) | **+6.3% over Python** |
| `count_primes_below` native | 29.3 ms | ≤ 28.3 ms (C) | **1.04× C** |
| `fib(40)` native | 1.53 ns/call | ≤ 1.28 ns (C) | **1.20× C** |

Lullaby is now **terser than every language except Python**: ~22% ahead of C,
~24% ahead of C++, ~18% ahead of Rust, ~8% ahead of JavaScript. Python is the
only language still ahead on tokens (by +6.3%, down from +16.9% after return-type
inference and the four inline-syntax features shipped and were adopted). On
speed, only C/Rust beat it on native — both by a hair.

**The last token lever to go under Python is parameter-type inference** (~1,841
tokens, the only remaining structural cost now that return types infer) — a
genuine design decision, not a desugar. Two other candidates were **measured and
ruled out**: string interpolation saves ~0 *counted* tokens (the tokenizer strips
the `main` driver, and there are zero `to_string` calls in counted functions),
and dropping redundant array-length params (`n`→`len(a)`) is net-neutral to worse
(`len(a)` is more tokens than `n`). See the artifact's Open-gaps tab.

### Shipped: the four token gaps (2026-07-12)

All four language gaps below are **implemented end-to-end** (parser → semantics →
IR desugar → AST interpreter, with the IR/bytecode/native/WASM backends covered
by the desugar) and adopted in the corpus:

- **Inline conditional** `A if C else B` — the broad win, replacing 1/0 (and
  small-value) block `if`/`else` across most categories.
- **`string + char` / `+= char`** — drops the `to_string(char_from(...))` wrapper.
- **Membership `x in collection`** — `c in "aeiou"`, `sub in s`, list membership.
- **String slicing `s[i:j]`** (and `s[i:]`, `s[:j]`, `s[:]`).

Adopting them (plus `array_fill(n, 0)` for the DP-buffer literals) moved the
corpus from 22,356 → 21,535 tokens (+16.9% → +12.6% over Python) with every
function's output byte-identical.

The remaining Python gap is now **structural, not ergonomic**: the corpus is
mostly array/numeric algorithms, so the string features have limited reach. What
still separates Lullaby from Python is (a) **mandatory type annotations** on
every parameter and return, and (b) **explicit array-length parameters** (`n`,
`la`, `lb`) carried for cross-language algorithm parity. Closing the rest means
optional **return-type inference** and dropping redundant length params in favor
of `len(a)` — see below.

## Token gap: where it lives

Per-category `Lullaby − Python` deltas. The **top 7 string/text categories are
67% (+2,166) of the entire +3,236 deficit**:

| Category | Lby | Py | Δ | ratio |
|---|---|---|---|---|
| string_algos | 1996 | 1488 | +508 | 1.34× |
| text_processing | 703 | 333 | +370 | 2.11× |
| parsing | 1267 | 950 | +317 | 1.33× |
| validation | 1035 | 765 | +270 | 1.35× |
| combinatorics | 1151 | 921 | +230 | 1.25× |
| services | 708 | 520 | +188 | 1.36× |
| collections | 1417 | 1234 | +183 | 1.15× |
| …18 more, each < +170 | | | | |
| statistics | 678 | 724 | −46 | 0.94× |
| state_machines | 649 | 697 | −48 | 0.93× |

Lullaby already **beats** Python on `statistics` and `state_machines` — the
computation-heavy categories — confirming the deficit is almost entirely a
**string/text-ergonomics** problem, not a fundamental verbosity problem.

## Root causes (empirically verified against the current compiler)

1. **No inline conditional (ternary) expression.** `let x = 1 if c else 0`
   → `L0207`. The block form `if C` / `X` / `else` / `Y` costs ~4 lines where
   Python spends one. This idiom recurs in nearly every `validation`,
   `services`, `text_processing`, and `games` function that returns `1/0`.
   *Highest-ROI single feature.* Proposed surface (indentation-friendly, mirrors
   Python and the corpus): `value if cond else other` as an expression.
2. **No string slicing.** `s[0:3]` → `L0207`, no reverse. Python leans on
   `s[i:j]`, `s[:n]`, `s[::-1]`. Lullaby only has verbose `substring(s, i, j)`
   and hand-rolled reverse loops. Proposed: `s[i:j]` slice (and array slices),
   optionally negative indices.
3. **No `in` membership operator.** `c in "aeiou"` / `sub in s` → `L0207`.
   Lullaby writes `contains(s, sub)` or `c == 'a' or c == 'e' or …`. Proposed:
   `x in collection` for `string`/`array`/`list`/`map`.
4. **`string += char` is rejected.** `out += char_from(65)` → `L0315`
   ("requires a string operand"), forcing `out += to_string(char_from(…))`.
   Every ASCII case-fold / char-building loop pays the `to_string(…)` wrapper.
   Proposed: implicit `char → string` in `+`/`+=` (and `string + char`).
5. **Mandatory type annotations on every parameter and return.** Python spends
   zero tokens on types. This is a deliberate Lullaby value proposition and
   should stay for parameters, but **return types are inferable** from the body
   in the common case and could become optional — ~2–3 tokens saved per
   function across 200+ functions.
6. **Corpus does not always use Lullaby's own builtins.** `repeat(s, n)` and
   `split(s, sep)` **exist and pass `check`**, yet `text_processing` hand-rolls
   `repeat_str`/`word_count` with loops. This is a *corpus-hygiene* deficit that
   inflates Lullaby's count and understates the language. Fixing it is a pure,
   immediate win that also makes the benchmark honest.
7. **`graph_algos` is incomplete.** Only `lullaby.lby` exists (no C/C++/Rust/
   Python/JS siblings), so the category scores 0/0 and contributes nothing. It
   should be completed to all six languages (adds coverage; likely a Lullaby win
   given `statistics`/`state_machines`).

## Recommended sequencing (fastest path to "under Python")

- **Phase A — corpus hygiene (low risk, no compiler change).** Sweep every
  category for un-idiomatic verbosity: use `repeat`/`split`/`join`, real bitwise
  operators in `bitwise` (they ship now — the corpus still models bits with
  arithmetic), drop redundant temporaries. Complete `graph_algos`. Expected to
  remove several hundred tokens immediately.
- **Phase B — inline conditional + char→string coercion (root causes 1, 4).**
  The two highest structural wins; each touches parser + semantics + all five
  backends but is well-scoped. Together these should erase most of the top-7
  string/text deficit.
- **Phase C — slicing + `in` (root causes 2, 3).** Further string wins and a
  large ergonomics jump; slicing also benefits `parsing`/`string_algos`.
- **Phase D — optional return-type inference (root cause 5).** Broad, small
  per-function savings; ergonomics improvement.

After Phases A–C, Lullaby should cross under Python's 19,120.

## Performance gap: closing 1.04× / 1.20× to ≤ 1.0× C

The native backend is already essentially at C on the arithmetic-and-loops
workload — `count_primes_below` is at **1.01× C** after adopting the native `%`
operator. The remaining margin is per-call overhead in deep recursion:

Two levers guessed here earlier were checked against the emitted machine code and
**ruled out**:

- *Overflow-check elision* is a non-issue: the native backend already emits plain
  `add`/`imul` for `i64` arithmetic (bit-for-bit with the interpreters by the
  two's-complement semantics, not a runtime check) — there is nothing to elide.
- *Leaf-frame omission* already happens: the prologue self-omits the frame and
  shadow space when a function makes no calls and needs no stack (`stack_size == 0`).

The real remaining levers are:

1. **Recursive-call codegen.** `fib(35)` runs 1.26× C: the residual is per-call
   overhead (C keeps the argument in a callee-saved register across the two
   recursive calls and tightens the prologue). Needs self-tail / argument-in-register
   handling and tighter register allocation for the recursive case.
2. **Broaden SIMD auto-vectorization.** Three phases are shipped, all emitting an
   SSE2 packed loop (two `i64` lanes per iteration, scalar tail, verified bit-for-bit
   identical to every interpreter): (1) `i64` sum reduction (3.3× the scalar loop),
   (2) element-wise map `c[i] = a[i] ± b[i]` (3.36×), (3) bitwise reductions
   `acc = acc <op> a[i]` and bitwise maps `c[i] = a[i] <op> b[i]` for `& | ^`
   (2.89×), via `pand`/`por`/`pxor` seeded with the operator identity. Native
   scalar `i64` bitwise operators (`& | ^ << >>`) shipped alongside, which the
   bitwise reductions build on.

   The remaining requested patterns are **blocked by the x86-64 baseline ISA, not
   by effort** — shipping an emulation would be a measured regression:
   - *Dot product / product reduction* need a 64-bit packed multiply. SSE2 has
     none; the `pmuludq` schoolbook emulation is ~6 SSE ops/element versus one
     scalar `imul`, so it measures slower than the scalar loop.
   - *Min/max reductions* need a 64-bit packed compare (`pcmpgtq`), which is SSE4.2,
     not part of the guaranteed x86-64 baseline.
   - *`f64` accumulation* would break bit-exact interpreter parity: floating-point
     addition is not associative, so a two-lane packed sum rounds differently from
     the scalar left-to-right fold.

   Unlocking these needs runtime CPU-feature detection (CPUID) selecting a widened
   SSE4.2/AVX2 code path, or an opt-in fast-math mode that relaxes the `f64`
   parity contract. Both are separate multi-session epics.

## How to refresh this analysis

```
pwsh benchmarks/crosslang/run_benchmark.ps1        # tokens + reassemble
pwsh benchmarks/crosslang/run_benchmark.ps1 -Perf  # also re-time the workloads
```

Then update the standing table above and republish the artifact
(`benchmarks/crosslang/report.html`).
