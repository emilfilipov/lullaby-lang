# Cross-language benchmark

A living, six-language benchmark that measures the two things Lullaby markets:
**token efficiency** (does the terse, indentation-only syntax cost an LLM fewer
tokens for equivalent logic?) and **native performance** (does the `native`
backend run close to C/C++/Rust and far ahead of Python?).

The corpus is real-world function families — arrays, finance, geometry, parsing,
state machines, services (rate limiters, caches, pagination), string algorithms,
hashing, sorting, and more — each implemented in **Lullaby, C, C++, Rust, Python,
and JavaScript** with identical function names and the same algorithm, so the
only differences are syntax and codegen.

This is a **standing practice**: rerun it whenever the language, the optimizer,
or the corpus changes, regenerate the artifact, and publish it. See
[Regular cadence](#regular-cadence).

## Layout

```
benchmarks/crosslang/
  corpus/<category>/{lullaby.lby,c.c,cpp.cpp,rust.rs,python.py,js.js}
      one function family per category; same function names across all six files.
      Every lullaby.lby must pass `lullaby check`.
  {c,cpp,rust,python,lullaby,js}/scalar.*   legacy scalar suite (category "numeric_basics")
  {c,cpp,rust,python,lullaby}/bench_primes.* the perf workload (count_primes_below)

  corpus_tokens.py     tokenize the corpus (o200k_base) -> corpus_data.json
  run_perf.ps1         compile + time the perf workload   -> perf_data.json
  report_template.html tabbed artifact shell (source of truth for the HTML)
  assemble_report.py   inject font + JSON into the template -> report.html
  run_benchmark.ps1    ONE-COMMAND runner: tokenize [+ perf] + assemble

  corpus_data.json     generated: per-function tokens + source, per-lang totals
  perf_data.json       generated: perf numbers + optimization history
  LULLABY_CHEATSHEET.md Lullaby syntax + gotchas for corpus authors
  SPEC.md              original method rationale + the first 35-function spec
```

## Run it

```powershell
# tokens + assemble (fast; use after adding/editing corpus functions)
pwsh benchmarks/crosslang/run_benchmark.ps1

# also re-run the performance harness (needs MSVC, rustc, node, release lullaby)
pwsh benchmarks/crosslang/run_benchmark.ps1 -Perf
```

`run_benchmark.ps1` calls the standalone Python 3.14 at
`C:\Users\emil\AppData\Local\Programs\Python\Python314\python.exe` because that
interpreter has `tiktoken`; the shell venv does not. It writes `report.html`,
the self-contained artifact (font + data inlined, no network needed).

To publish: open `report.html`, confirm the tabs render, then publish it with the
Artifact tool (keep the same file path to preserve the artifact URL).

## Tokenizer rules (how a function is counted)

`corpus_tokens.py` strips comments, imports/`use`/`#include`/`using`, and the
verification driver (everything from the language's `main`), then splits the
remainder into functions and counts `o200k_base` tokens per function. A function
counts toward the totals only if it exists in **Lullaby plus at least one other
language** (real comparisons only). Cross-language matching is by function name,
so names must be identical across the six files.

## Adding to the corpus

1. Pick a real-world family (a REST endpoint's logic, a parser, a scheduler...).
   Prefer useful code over leetcode puzzles; whole-service *logic* is welcome as
   pure functions (see `corpus/services/`).
2. Create `corpus/<category>/` with all six files, identical function names.
3. Write idiomatic-but-minimal code in each language — not golfed, not padded, so
   the token comparison is fair. Use `Math.trunc(a/b)` in JS for C-style integer
   division; keep the `(arr, n)` array arity uniform.
4. `lullaby check benchmarks/crosslang/corpus/<category>/lullaby.lby` must pass.
5. Verify output parity: the runnable trio (Lullaby `run`, `python`, `node`)
   should print byte-identical results; C/C++/Rust match by construction.
6. Rerun `run_benchmark.ps1` and republish.

Consult `LULLABY_CHEATSHEET.md` for current Lullaby capabilities and gaps (no `%`
operator yet, string ergonomics limits, etc.) — those gaps are exactly what the
benchmark is meant to surface.

## Regular cadence

Treat the benchmark as part of the language's feedback loop:

- **After an optimizer or codegen change** — rerun with `-Perf`; the artifact's
  Performance tab records the before/after so wins are documented (e.g. the
  register-promotion loop win).
- **After a syntax/semantics change that affects ergonomics** — rerun tokens;
  watch whether categories that lose on tokens today (string-heavy parsing/text)
  improve as gaps close.
- **When growing the corpus** — add categories in parallel, rerun, republish.
- **Each rerun** — update the "Language gaps" findings if new gaps surface, and
  keep `documents/repository_map.md` pointing here.

The honest headline today: Lullaby is terser than C/C++, on par with Rust, and
Python stays tersest (dynamic typing); native runs ~1.2-1.6x C and far ahead of
CPython. The benchmark exists to keep those claims truthful as the language grows.
