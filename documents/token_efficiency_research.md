# Token-Efficiency Research: Closing the Gap With Python/JS

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

## Purpose

Lullaby markets two promises: **compiled, near-C speed** and **lowest possible
tokens to write code**, so it is optimal for LLM/agentic generation. This
document audits the *token* promise against the standing six-language benchmark
(`benchmarks/crosslang/`), pinpoints where Lullaby spends tokens it does not have
to, and proposes a prioritized set of changes to reduce token count further.

It is a research/recommendation document, not a spec change. Nothing here is
implemented until it is turned into a ClickUp task, designed against the real
grammar, and shipped at backend parity per the Production Quality Standard.

## Current standing (measured, not aspirational)

From `benchmarks/crosslang/corpus_data.json` (o200k_base tokens, 417 functions,
25 real-world categories, comments/imports/drivers stripped):

| Language   | Tokens | vs Lullaby |
|------------|-------:|-----------:|
| Python     | 19,120 | **−21.0%** |
| JavaScript | 22,146 | **−8.5%**  |
| **Lullaby**| 24,183 | —          |
| Rust       | 24,867 | +2.8%      |
| C          | 26,030 | +7.6%      |
| C++        | 26,894 | +11.2%     |

Lullaby is terser than C/C++/Rust but **loses to Python by ~26.5%** and **to JS
by ~9.2%** (measured the other direction). The design guide's stated goal is
"40–60% reduction vs traditional languages." Against the *systems* languages we
are inside that band; against the *dynamic* languages we are behind. Since the
whole pitch is "fewest tokens for an LLM," Python and JS are the bars that
matter, and today we do not clear them.

### Where the gap lives

Per-category Lullaby-minus-Python token delta (worst first):

| Category          | Lullaby | Python | Δ vs Py |
|-------------------|--------:|-------:|--------:|
| parsing           |   1,811 |    950 |   +861  |
| text_processing   |     989 |    333 |   +656  |
| string_algos      |   2,065 |  1,488 |   +577  |
| validation        |   1,194 |    765 |   +429  |
| combinatorics     |   1,205 |    921 |   +284  |
| collections       |   1,482 |  1,234 |   +248  |
| sorting           |   1,875 |  1,629 |   +246  |
| services          |     716 |    520 |   +196  |
| … (state_machines is the only category Lullaby *wins*: −42) |

The gap is overwhelmingly concentrated in **string / text / parsing / validation**
work — exactly the code an agent writes most (request handling, parsing,
formatting, glue). Numeric-only categories (bitwise, physics, units, geometry,
numeric_basics) are already at or near parity with Python and beat JS. **So the
token promise is won or lost on string-shaped code.**

## Two distinct problems

The gap splits cleanly into two causes. Treat them separately.

1. **Measurement honesty** — the benchmark corpus is written in *stale idioms*
   that predate features we have already shipped, so it over-reports Lullaby's
   token count. Fixing the corpus costs no language change and makes the number
   truthful.
2. **Real language gaps** — genuine missing terseness features that force a
   Lullaby author to write more tokens than a Python/JS author for the same
   logic. Closing these reduces tokens for *every* real user, not just the
   benchmark.

---

## Part A — Measurement honesty (do first; no language change)

The corpus is penalizing Lullaby with idioms that are no longer necessary. These
are pure wins: rewrite the corpus to current idioms, rerun `run_benchmark.ps1`,
republish. Estimated combined effect: **~700–1,000 tokens off Lullaby's total
(~3–4%)**, moving Lullaby from +26.5% to roughly +21–22% vs Python before any
feature work.

1. **`0 - N` → `-N` (unary minus).** Unary minus on any numeric operand shipped
   (`documents/alpha1_language_surface.md`), but the corpus still writes
   `return 0 - 1`, `0 - t`, `0 - val`, `[3, 4, 0 - 1, ...]`. **36 occurrences**,
   ~2 tokens each ≈ 70+ tokens. (Note: internally `-N` still lowers to `0 - N`
   in the AST, but the *source token count* — which is what the benchmark
   measures — drops.)

2. **`chars(s)` + `get(cs, i)` → `s[i]`.** Direct string indexing `s[i]`
   (returns `char`, bounds-checked) shipped, yet the text/parsing corpus opens
   nearly every function with `let cs list<char> = chars(s)` then reads
   `get(cs, i)`. That preamble line is ~8 tokens and appears **32 times**; the
   per-access `get(cs, i)` (6 tokens) vs `s[i]` (4 tokens) recurs **27+ times**.
   Combined ≈ 300+ tokens, and it makes the code read worse than it needs to.

3. **`array_fill(n, 0)` → drop the 64-element zero literal.** `array_fill(n i64,
   value T)` shipped specifically to kill the fixed-capacity workaround — the
   corpus comment in `string_algos/lullaby.lby` even names it as the fix — but
   **7 giant `[0, 0, …, 0]` literals** (~64 tokens each ≈ 450 tokens) remain.
   Only 1 category uses `array_fill`. This single fix is the largest honesty
   win.

4. **Drop inferable `let` type annotations.** `let n i64 = len(cs)`,
   `let c char = get(...)`, `let code i64 = char_code(...)` all carry a type the
   checker already infers from the initializer. **71+** such len/char_code/get
   annotations alone; across all **531** `let` bindings a large fraction are
   redundant. Each drop saves ~1 token. Recommend the corpus (and
   `LULLABY_CHEATSHEET.md`) prefer `let n = len(cs)` when the type is obvious.
   *Caveat:* explicit annotations aid LLM reliability and reviewer clarity, so
   this is a style call — but for a pure token comparison the annotations are
   free tokens we are handing away.

> These are corpus/tooling fixes. They do not change the language; they stop the
> benchmark from lying about it. The current `report.html` headline
> ("terser than C/C++, on par with Rust, Python tersest") is *true but
> pessimistic* for Lullaby until the corpus is modernized.

---

## Part B — Real language gaps (ranked by token impact)

These require design + implementation at full backend parity. Ranked by
estimated token reduction on the categories where we bleed.

### B1. `for x in collection` — a value-iterating loop  ★ highest impact

Today the only loops are `while`, indexed `for i from a to b`, and `loop`. Every
traversal of a string/array/list therefore costs a length binding + an index
loop + an indexed read:

```lullaby
let cs list<char> = chars(s)     # or: let n = len(s)
let n i64 = len(cs)
for i from 0 to n - 1
    let c char = get(cs, i)      # or s[i]
    ...
```

Python is:

```python
for c in s:
    ...
```

The indexed-loop pattern `for i from 0 to n - 1` appears **200 times** in the
corpus. A `for x in <string|array|list>` form collapses the length binding, the
`- 1`, and the per-iteration index read into one line and one name. This is the
single biggest structural lever, because it fires on *every* string/array/
collection category — precisely the ones with the largest gap. Estimated impact:
**a large fraction of the parsing/text/validation/collections deltas**, easily
the biggest available reduction.

Design notes: keep it indentation-only (`for c in s` NEWLINE block); iterate
`char` over `string`, `T` over `array<T>`/`list<T>`; a `for i, x in …` enumerated
form is a natural follow-up but optional. This also unblocks native/WASM string
iteration only where the element type is a scalar.

### B2. A conditional expression (inline `if`/ternary)  ★ high impact

`if` is already an *expression*, but it requires a newline + indented block, so a
one-value choice costs 4 lines:

```lullaby
if starts_with(s, p)
    1
else
    0
```

Python: `return 1 if s.startswith(p) else 0`. The corpus has **91 trivial `0`/`1`
body lines** driven largely by this pattern (bool-as-int returns, clamps,
sign choices like `-val if neg else val`). An inline conditional — either
one-line `if cond then a else b` as an expression, or a dedicated ternary —
removes 2–3 lines each time. Estimated impact: **150–300 tokens** across
validation/parsing/business/games, plus much more natural code.

Recommendation: prefer a **one-line `if C then A else B` expression** (reuses the
existing `if` keyword and expression semantics, no new operator, stays readable
for a small model) over a punctuation ternary (`?:`), which fights the
"no punctuation noise" principle. `then` already appears in the design guide.

### B3. Richer string/char builtins mapped 1:1 to the common cases  ★ high impact

Python wins string categories partly through a dense builtin vocabulary. Lullaby
has a good set (`substring`, `find`, `contains`, `split`, `join`, `trim`,
`replace`, `upper`, `lower`, `starts_with`, `ends_with`) but is missing the
high-frequency primitives, forcing hand-rolled loops:

- **Char predicates**: have `is_whitespace`; add `is_digit`, `is_alpha`,
  `is_alnum`, `is_upper`, `is_lower`. The parsing corpus reimplements these as
  `code >= 48 and code <= 57` etc. dozens of times.
- **`count(s, sub)`** — `count_char`/`count_fields`/`count_lines` are all
  hand-rolled loops; Python is `s.count(x)`.
- **`repeat(s, n)`** (or `s * n`) — `repeat_str`/`left_pad` hand-roll a loop.
- **`reverse(s)`** — `reverse_str` hand-rolls; Python is `s[::-1]`.
- **`parse_int`/`to_int`** — the entire `parse_uint`/`parse_int_signed`/
  `hex_to_int`/`bin_to_int` family exists only because there is no numeric parse
  builtin. A `to_int(s) -> option<i64>` (and radix variant) would delete whole
  functions of hand-written digit loops.
- **Char/string bridging**: allow `string + char` (and/or `push` onto a string)
  so building a string in a loop drops the `to_string(c)` wrapper that recurs in
  `to_upper_ascii`/`reverse_str`/`initials`.

Each of these turns a multi-line loop into a call. Estimated impact:
**meaningful chunks of the parsing (+861) and text (+656) deltas** — the two
worst categories. Prioritize `is_digit`/`is_alpha`/predicates and `to_int`, which
fire the most.

### B4. Comprehension / map–filter–reduce over collections  ◆ medium-high impact

Python's `"".join(chr(ord(c) - 32) if … else c for c in s)` collapses an entire
build-a-string loop into one expression; `sum(…)`, `[f(x) for x in xs]`,
`any(…)`, `all(…)` do the same for numeric/collection work. Lullaby has
first-class functions but no comprehension and no `map`/`filter`/`reduce`/`sum`/
`any`/`all` over collections, so these become explicit accumulator loops.

Two paths, not mutually exclusive:
- **Builtin higher-order functions** over `array`/`list` (`map`, `filter`,
  `reduce`, `sum`, `any`, `all`, `count_if`). Cheapest to add; needs closures
  (lambda literals) to be truly terse, which are already on the roadmap
  (`closures_capture_design.md`).
- **A comprehension form** (`[f(x) for x in xs if pred(x)]`). Terser still but a
  bigger grammar change.

Estimated impact: large on collections/statistics/combinatorics, but gated on
lambda literals landing first. Sequence it *after* B1–B3.

### B5. Optional `let` / lighter binding  ◆ medium impact, higher risk

Every new local costs the `let` keyword (`let val i64 = 0` vs Python `val = 0`).
With **531** bindings in the corpus, dropping mandatory `let` for a first
assignment would save ~1 token each (~500 tokens) — but `let` is what makes
declaration-vs-mutation unambiguous and enforces the no-shadowing rule, which is
also an LLM-reliability feature. This is an identity-level decision for the owner,
not a mechanical win. Options, least invasive first:
- Keep `let` mandatory (status quo; clearest for small models).
- Allow bare `name = expr` to *introduce* a binding when the name is new in
  scope (Python-like), keeping `let` optional. Risk: reintroduces the
  declaration/assignment ambiguity `let` was chosen to remove.

Recommendation: **do not** chase this until B1–B4 are done; it trades a core
clarity property for a modest token win and should be decided deliberately.

---

## Part C — Micro-optimizations (small but free-ish)

### C1. Tokenizer-aware keyword and type names

Because the metric is *BPE* tokens (o200k_base), not characters, the token cost
of a keyword/identifier depends on whether it is a single merge in the vocab.
snake_case builtins tend to split (`char_code` → ~`char` + `_code`, `to_string`
→ ~`to` + `_string`, `println` → ~`print` + `ln`), costing 2–3 tokens per use.
Short common words (`get`, `len`, `map`, `int`) are usually one token; `i64`/
`f64` are likely **2** tokens each (`i`/`f` + `64`) versus a hypothetical `int`
at 1.

Action: **measure** the per-token cost of every Lullaby keyword, type name, and
builtin under o200k_base (and cl100k for older models) on the bench box that has
`tiktoken`, then, where it does not hurt clarity, prefer single-token spellings
for the *highest-frequency* names. Candidates worth pricing: `char_code`/
`char_from` (vs `ord`/`chr` — both likely 1 token and far more familiar to an
LLM from its training data), `to_string` (vs `str`), `println` (vs `print` +
explicit newline), and whether an `int` alias for `i64` would pay for itself.
This was not measurable in this environment (the o200k BPE blob is proxy-blocked
here); run it where the benchmark already runs.

> Note the tension: names an LLM has seen a billion times (`ord`, `chr`, `str`,
> `int`, `print`) are both fewer tokens *and* higher-probability to emit
> correctly. Token cost and generation reliability point the same way here.

### C2. Comparison ergonomics

- **Chained comparison** (`0 <= x <= 9`, `'0' <= c <= '9'`) replaces
  `x >= 0 and x <= 9` (saves ~2 tokens; recurs heavily in parsing/validation).
- These are small but land in the exact categories where we bleed.

---

## Recommended sequence

1. **Part A (corpus honesty)** — mechanical, no language change, immediate ~3–4%
   truthful improvement. Do this first so every later measurement is honest.
   Update `LULLABY_CHEATSHEET.md` to teach the current idioms (`-N`, `s[i]`,
   `array_fill`, drop inferable annotations).
2. **B1 `for x in`** — the single biggest structural lever; fires on every
   string/array/collection category.
3. **B3 string/char builtins** (`is_digit`/`is_alpha`/predicates, `to_int`,
   `count`, `repeat`, `reverse`, `string + char`) — deletes hand-rolled loops in
   the two worst categories.
4. **B2 inline conditional** — cheap, high-frequency, improves readability.
5. **C1 tokenizer audit** + **C2 chained comparison** — measure, then apply the
   safe renames and add chaining.
6. **B4 comprehension / higher-order builtins** — after lambda literals
   (`closures_capture_design.md`) land.
7. **B5 optional `let`** — owner decision, deliberately, last.

After each step, rerun `benchmarks/crosslang/run_benchmark.ps1`, update the
"Language gaps" findings, and republish `report.html`, per the benchmark's
standing cadence.

## Guardrails

- Every proposal must keep the invariants in `core_language_rules.md`:
  indentation-only scope, no curly-brace blocks, no semicolon terminators.
- Terseness must not cost small-model parseability. Prefer keyword-based forms
  (`for x in`, `if C then A else B`) over new punctuation; the design principle
  is "eliminate noise," not "add sigils."
- Each feature ships at full AST/IR/bytecode parity with negative tests and
  updated Markdown + offline docs before it counts as done. Heap-shaped features
  (string iteration, collection builtins) may be interpreter-first where the
  native/WASM scalar backends cannot yet lower them, matching current practice.
- Re-benchmark on every change; the point is a *truthful* token claim that keeps
  improving, not a one-time headline.

## Open questions for the owner

1. Is beating **Python** on tokens a hard 1.0 goal, or is "tersest compiled +
   memory-safe language" (already true vs C/C++/Rust) the real target? The
   feature list above is sized to overtake Python; a narrower goal changes the
   priority cutoff.
2. Appetite for the `let`/mandatory-annotation identity questions (B5, A4) —
   token savings vs the explicitness that aids small-model reliability.
3. Should the offline docs and `report.html` headline be updated to the
   *modernized-corpus* number as soon as Part A lands, before any feature work?
