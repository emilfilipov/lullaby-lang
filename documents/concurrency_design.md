# Concurrency Design (Threads and Channels)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
Supersedes the deferral in [concurrency_semantics.md](concurrency_semantics.md)
for the first, message-passing increment.

Lullaby's interpreters are real Rust programs, so concurrency ships as runtime
builtins backed by `std::thread` and `std::sync::mpsc` — no native codegen
needed. The model is **message passing** (share by communicating), which fits
the value-semantics runtime cleanly and sidesteps shared-mutable-state hazards.

## The capture constraint shapes the API

First-class functions currently do **not** capture their environment. So a
spawned thread cannot close over outer variables — it must receive everything it
needs as an explicit argument. The API therefore passes a single argument to the
thread function (typically a channel):

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

## Builtins

- `chan_new() -> Chan` — create a channel carrying `i64` values (a single
  channel element type for the first increment; a generic `Chan<T>` follows once
  it is proven). A channel is a **shared** handle: cloning the value shares the
  same underlying queue (reference semantics, like `rc<T>`).
- `send(ch Chan, v i64) -> void` — enqueue a value (never blocks; unbounded).
- `recv(ch Chan) -> i64` — dequeue, blocking until a value is available.
- `try_recv(ch Chan) -> option<i64>` — non-blocking; `some(v)` or `none`.
- `spawn(f fn(Chan, i64) -> void, ch Chan, v i64) -> Task` — run `f(ch, v)` on a
  new OS thread. (The signature is fixed to `(Chan, i64)` in the first
  increment; a more general `spawn` over arbitrary argument tuples follows with
  capturing closures.)
- `join(t Task) -> void` — wait for a spawned thread to finish.

## Runtime representation

- `Value::Chan(Arc<Mutex<mpsc::Receiver<Value>>>, mpsc::Sender<Value>)` (or an
  equivalent shared duplex handle) — `Send` and shareable; a `Chan` cloned into
  a thread and used in `main` refer to the same queue.
- `Value::Task(JoinHandle<...>)` behind a handle so it can be `join`ed once.
- `Value` is already `Send` (it holds `String`/`Vec`/numbers, no `Rc`), so values
  cross threads safely.

## Running Lullaby on a thread

A spawned thread needs to execute a Lullaby function independently:

- The `Program` becomes shareable — hold it as `Arc<Program>` so each thread owns
  a clone and builds its own interpreter over it. This is the one structural
  change (the interpreters currently borrow `&Program`; they move to
  `Arc<Program>` or an equivalent `'static`-capable share).
- The thread builds a fresh interpreter (fresh locals/heap) over the shared
  program and invokes the target function with the passed arguments. Heaps are
  per-thread (no shared heap in this increment); cross-thread communication is
  only through channels.

## Determinism and testing

Threads are non-deterministic in *scheduling*, so tests must be
*order-independent*: e.g. spawn N workers that each `send` one value and have
`main` `recv` N values and sum them — the total is deterministic regardless of
completion order, so it is safe for the cross-backend parity harness and for
CLI tests. Never assert on interleaving or per-message order.

## Backends and parity

All three backends run the same builtins over the same shared representation; a
sum-of-worker-results parity fixture returns the same deterministic total on the
AST, IR, and bytecode backends (and optimized variants).

## Scope and sequencing

First increment: `spawn`/`join`, and `i64` channels with `send`/`recv`/
`try_recv` — message passing with per-thread heaps. Deferred: generic `Chan<T>`,
shared mutable state via a `Mutex`/`rc` across threads, `select` over multiple
channels, thread-pools, and `async`/`await` with an executor (a separate,
larger effort that reuses this channel layer).

## Why these choices

- **Message passing first**: matches value semantics; no data races by
  construction; the smallest safe concurrency surface.
- **Explicit thread argument**: works today without capturing closures; when
  capture lands, `spawn(f)` over a captured environment becomes the ergonomic
  form and this stays as the primitive.
- **`Arc<Program>` share**: the minimal change that lets a thread run Lullaby
  independently, keeping heaps per-thread so there is nothing to lock.
