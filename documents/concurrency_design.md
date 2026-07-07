# Concurrency Design (Data Parallelism, then Threads and Channels)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
Supersedes the deferral in [concurrency_semantics.md](concurrency_semantics.md).

Lullaby's interpreters are real Rust programs, so concurrency ships as runtime
builtins backed by the standard library — no native codegen needed. The first
delivered increment is **data parallelism** via `parallel_map`, which runs on
real OS threads while keeping the interpreter's `&Program` borrow (no
`Arc<Program>` refactor) and producing fully deterministic, ordered results.

## First increment (delivered): `parallel_map`

`parallel_map(f fn(i64) -> i64, args list<i64>) -> list<i64>` evaluates `f(arg)`
for every element of `args` concurrently on separate OS threads and returns the
results in the **same order as `args`**, regardless of thread scheduling.

```lby
fn sq x i64 -> i64
    x * x

fn main -> i64
    let base list<i64> = list_new()
    base = push(base, 1)
    base = push(base, 2)
    base = push(base, 3)
    base = push(base, 4)
    let out list<i64> = parallel_map(sq, base)
    let total i64 = 0
    for i from 0 to 3
        total += get(out, i)
    total                     # 1 + 4 + 9 + 16 = 30
```

### Why this is safe without an `Arc<Program>` refactor

`std::thread::scope` lets spawned threads borrow non-`'static` data. The AST and
IR interpreters keep a `&Program`/`&IrModule` borrow; the builtin spawns a scoped
thread per argument, and each thread:

- borrows the same shared `&Program`/`&IrModule` (no clone, no `Arc`),
- builds a **fresh sibling interpreter** (fresh locals and heap), and
- calls the target function by name with the single argument value.

Heaps are per-thread, so there is **no shared mutable state and no locking**.
`Value` is already `Send` (it holds `String`/`Vec`/numbers, no `Rc`), so the
argument values and results cross threads safely. Results are joined and
collected in **input order**, so output is fully deterministic — which is what
makes `parallel_map` safe for the cross-backend parity harness.

### Constraints (first increment)

- `f` must be an ordinary top-level function value (`fn(i64) -> i64`); its name
  must be resolvable in a fresh interpreter. First-class functions do not capture
  their environment, so nothing is closed over.
- The element type is fixed to `i64` (both the argument list and the result).
- Semantics rejects a wrong arity, a non-`fn(i64) -> i64` first argument, or a
  non-`list<i64>` second argument with diagnostic **L0334**.

### Backends and parity

All three backends run `parallel_map` over the same shared program: the AST
interpreter, the IR interpreter (`--backend ir`), and the bytecode VM
(`--backend bytecode`, which round-trips through the IR interpreter) each build
fresh per-thread interpreters. The `run_parallel.lby` fixture returns the same
deterministic `30` on the AST, IR, and bytecode backends and their optimized
variants.

## Second increment (deferred): `spawn`/`join` and channels

The next increment is **message passing** (share by communicating) with explicit
threads and channels. It is deferred because it requires making the program
shareable as `Arc<Program>` so a detached thread can run Lullaby independently —
a larger structural change than the scoped-thread borrow above.

```lby
fn worker ch Chan v i64 -> void
    send(ch, v * v)

fn main -> i64
    let ch Chan = chan_new()
    let t1 Task = spawn(worker, ch, 2)
    let t2 Task = spawn(worker, ch, 3)
    join(t1)
    join(t2)
    recv(ch) + recv(ch)      # 4 + 9 in some order → 13
```

### Deferred builtins

- `chan_new() -> Chan` — create a channel carrying `i64` values (a single
  channel element type for this increment; a generic `Chan<T>` follows once it is
  proven). A channel is a **shared** handle: cloning the value shares the same
  underlying queue (reference semantics, like `rc<T>`).
- `send(ch Chan, v i64) -> void` — enqueue a value (never blocks; unbounded).
- `recv(ch Chan) -> i64` — dequeue, blocking until a value is available.
- `try_recv(ch Chan) -> option<i64>` — non-blocking; `some(v)` or `none`.
- `spawn(f fn(Chan, i64) -> void, ch Chan, v i64) -> Task` — run `f(ch, v)` on a
  new OS thread. (The signature is fixed to `(Chan, i64)` in this increment; a
  more general `spawn` over arbitrary argument tuples follows with capturing
  closures.)
- `join(t Task) -> void` — wait for a spawned thread to finish.

### Why an `Arc<Program>` share is needed here

A detached (non-scoped) thread outlives the `spawn` call, so it cannot borrow the
caller's stack. The `Program`/`IrModule` becomes shareable — held as
`Arc<Program>` so each thread owns a clone and builds its own interpreter over
it. This is the one structural change that lets a *detached* thread run Lullaby
independently, keeping heaps per-thread so there is nothing to lock.

### Determinism and testing (deferred increment)

Detached threads are non-deterministic in *scheduling*, so tests must be
*order-independent*: e.g. spawn N workers that each `send` one value and have
`main` `recv` N values and sum them — the total is deterministic regardless of
completion order. Never assert on interleaving or per-message order. (In
contrast, `parallel_map` is already order-deterministic by construction.)

### Further deferred work

Generic `Chan<T>`, shared mutable state via a `Mutex`/`rc` across threads,
`select` over multiple channels, thread-pools, and `async`/`await` with an
executor (a separate, larger effort that reuses this channel layer).

## Why these choices

- **Data parallelism first**: `parallel_map` is the smallest safe concurrency
  surface — real OS-thread parallelism, deterministic output, and no interpreter
  refactor, so it lands without risk to the parity harness.
- **Message passing next**: matches value semantics; no data races by
  construction. Explicit thread arguments work today without capturing closures;
  when capture lands, `spawn(f)` over a captured environment becomes the
  ergonomic form and this stays as the primitive.
- **Scoped threads before `Arc<Program>`**: borrow the program for the duration
  of a `parallel_map` call; only pay for the `Arc` share when detached threads
  actually need it.
