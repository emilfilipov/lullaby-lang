//! Concurrency runtime values (channels, tasks, futures, mutexes, atomics), the
//! `MemoryOrder`-taking atomic builtins, and the join/await/channel helpers
//! shared by every interpreter backend. Split out of `lib.rs` as a
//! behavior-preserving code move; `Value`, `RuntimeError`, and `expect_i64` (in
//! sibling modules) are reached through `crate::` paths.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{RuntimeError, Value, expect_i64};

/// An unbounded `i64` message-passing channel handle. Built over `std::sync::mpsc`:
/// a cloneable `Sender` and a `Receiver` shared behind an `Arc<Mutex<_>>` so the
/// value's `Clone` shares the same underlying queue (reference semantics, like a
/// socket handle). `Send` because every field is `Send`.
#[derive(Debug, Clone)]
pub struct Chan {
    pub sender: Sender<Value>,
    pub receiver: Arc<Mutex<Receiver<Value>>>,
}

impl PartialEq for Chan {
    /// Two channel handles are equal when they refer to the same queue (pointer
    /// identity of the shared receiver), mirroring how sockets compare by handle.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.receiver, &other.receiver)
    }
}

/// The shared, take-once join slot behind a `Task`. `JoinHandle` is not `Clone`,
/// so it lives behind an `Arc<Mutex<Option<_>>>`: `task_join` takes the handle
/// out (leaving `None`) so a second `task_join` is a harmless no-op.
pub type TaskHandle = Arc<Mutex<Option<JoinHandle<Result<Value, RuntimeError>>>>>;

/// A spawned-thread handle. `Send` because a `JoinHandle` is `Send`.
#[derive(Debug, Clone)]
pub struct Task {
    pub handle: TaskHandle,
}

impl PartialEq for Task {
    /// Task handles compare by identity of the shared join slot.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.handle, &other.handle)
    }
}

/// The shared, take-once join slot behind a `Future`. Structurally identical to a
/// `TaskHandle` (an `Arc<Mutex<Option<JoinHandle<...>>>>`), but the joined thread
/// PRODUCES a `Value`: `await` takes the handle out and returns that value,
/// whereas `task_join` discards it. Behind an `Arc<Mutex<Option<_>>>` so a
/// future can be moved/cloned as a value and `await`ed exactly once (a second
/// `await` on the same handle finds `None`).
pub type FutureHandle = Arc<Mutex<Option<JoinHandle<Result<Value, RuntimeError>>>>>;

/// A handle to an `async fn` call running on a spawned OS thread that will
/// produce a `T`. `await`ing it blocks until the thread completes and yields its
/// `T`. `Send` because a `JoinHandle` is `Send`; shared on clone.
#[derive(Debug, Clone)]
pub struct Future {
    pub handle: FutureHandle,
}

impl PartialEq for Future {
    /// Future handles compare by identity of the shared join slot.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.handle, &other.handle)
    }
}

/// A shared mutex over one `i64`. `Arc<Mutex<i64>>`, so the value's `Clone`
/// shares the same lock and cell across threads (reference semantics). `Send`.
#[derive(Debug, Clone)]
pub struct SharedMutex {
    pub cell: Arc<Mutex<i64>>,
}

impl PartialEq for SharedMutex {
    /// Mutex handles compare by identity of the shared cell.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cell, &other.cell)
    }
}

/// A shared atomic `i64` cell. `Arc<AtomicI64>`, so the value's `Clone` shares
/// the same lock-free cell across threads (reference semantics, exactly like
/// [`SharedMutex`], but backed by `std::sync::atomic` for wait-free access).
/// `Send + Sync`, so the handle crosses thread boundaries safely. Every
/// operation uses `Ordering::SeqCst` in this increment; weaker orderings are a
/// documented future optimization.
#[derive(Debug, Clone)]
pub struct SharedAtomic {
    pub cell: Arc<AtomicI64>,
}

impl PartialEq for SharedAtomic {
    /// Atomic handles compare by identity of the shared cell.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cell, &other.cell)
    }
}

/// Unwrap a runtime `Value` expected to be a channel handle, reporting `L0417`
/// otherwise.
pub fn expect_chan(name: &str, value: Value) -> Result<Chan, RuntimeError> {
    match value {
        Value::Chan(chan) => Ok(chan),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a Chan but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be a task handle, reporting `L0417`
/// otherwise.
pub fn expect_task(name: &str, value: Value) -> Result<Task, RuntimeError> {
    match value {
        Value::Task(task) => Ok(task),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a Task but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be a future handle, reporting `L0417`
/// otherwise. The semantic checker (`L0344`) normally prevents awaiting a
/// non-future, so this is a defensive runtime guard.
pub fn expect_future(name: &str, value: Value) -> Result<Future, RuntimeError> {
    match value {
        Value::Future(future) => Ok(future),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a Future but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be a mutex handle, reporting `L0417`
/// otherwise.
pub fn expect_mutex(name: &str, value: Value) -> Result<SharedMutex, RuntimeError> {
    match value {
        Value::Mutex(mutex) => Ok(mutex),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a Mutex but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be an `atomic_i64` handle, reporting
/// `L0417` otherwise.
pub fn expect_atomic(name: &str, value: Value) -> Result<SharedAtomic, RuntimeError> {
    match value {
        Value::Atomic(atomic) => Ok(atomic),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects an atomic_i64 but got `{other}`"),
        )),
    }
}

/// The five `MemoryOrder` enum variant names, in strengthening order. Registered
/// as a compiler-provided nominal enum so `relaxed`/`acquire`/`release`/
/// `acq_rel`/`seq_cst` construct `MemoryOrder` unit-variant values that the
/// ordering-taking atomic builtins and `fence` decode into real
/// `std::sync::atomic::Ordering` values.
pub const MEMORY_ORDER_VARIANTS: [&str; 5] =
    ["relaxed", "acquire", "release", "acq_rel", "seq_cst"];

/// Decode a `MemoryOrder` unit-variant runtime value into the corresponding
/// `std::sync::atomic::Ordering`. Semantics guarantees the argument type, so a
/// non-`MemoryOrder` value here indicates an interpreter bug and reports
/// `L0432`.
pub fn expect_memory_order(name: &str, value: Value) -> Result<Ordering, RuntimeError> {
    match value {
        Value::Enum(e) if e.enum_name == "MemoryOrder" => match e.variant.as_str() {
            "relaxed" => Ok(Ordering::Relaxed),
            "acquire" => Ok(Ordering::Acquire),
            "release" => Ok(Ordering::Release),
            "acq_rel" => Ok(Ordering::AcqRel),
            "seq_cst" => Ok(Ordering::SeqCst),
            other => Err(RuntimeError::new(
                "L0432",
                format!("{name} received an unknown MemoryOrder variant `{other}`"),
            )),
        },
        other => Err(RuntimeError::new(
            "L0432",
            format!("{name} expects a MemoryOrder but got `{other}`"),
        )),
    }
}

/// Guard that `order` is a legal ordering for an atomic *load* (or a CAS failure
/// ordering): `relaxed`, `acquire`, or `seq_cst`. `release`/`acq_rel` would
/// panic inside `std`, so they are rejected with `L0432` first.
fn load_ordering(name: &str, order: Ordering) -> Result<Ordering, RuntimeError> {
    match order {
        Ordering::Relaxed | Ordering::Acquire | Ordering::SeqCst => Ok(order),
        _ => Err(RuntimeError::new(
            "L0432",
            format!("{name} cannot use a `release`/`acq_rel` ordering for a load"),
        )),
    }
}

/// Guard that `order` is a legal ordering for an atomic *store*: `relaxed`,
/// `release`, or `seq_cst`. `acquire`/`acq_rel` would panic inside `std`.
fn store_ordering(name: &str, order: Ordering) -> Result<Ordering, RuntimeError> {
    match order {
        Ordering::Relaxed | Ordering::Release | Ordering::SeqCst => Ok(order),
        _ => Err(RuntimeError::new(
            "L0432",
            format!("{name} cannot use an `acquire`/`acq_rel` ordering for a store"),
        )),
    }
}

/// Guard that `order` is a legal ordering for a `fence`: `acquire`, `release`,
/// `acq_rel`, or `seq_cst`. `relaxed` would panic inside `std`.
fn fence_ordering(name: &str, order: Ordering) -> Result<Ordering, RuntimeError> {
    match order {
        Ordering::Acquire | Ordering::Release | Ordering::AcqRel | Ordering::SeqCst => Ok(order),
        _ => Err(RuntimeError::new(
            "L0432",
            format!("{name} cannot use a `relaxed` ordering"),
        )),
    }
}

/// Build the `L0405` arity error for a free-standing ordering builtin.
fn ordering_arity(name: &str, expected: usize, actual: usize) -> RuntimeError {
    RuntimeError::new(
        "L0405",
        format!("function `{name}` expects {expected} arguments but got {actual}"),
    )
}

/// `atomic_load_ordered(a atomic_i64, order MemoryOrder) -> i64`: read the cell
/// under `order` (`relaxed`/`acquire`/`seq_cst`), mapping to the real
/// `std::sync::atomic::Ordering`.
pub fn builtin_atomic_load_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = "atomic_load_ordered";
    let [atomic, order]: [Value; 2] = args
        .try_into()
        .map_err(|args: Vec<Value>| ordering_arity(name, 2, args.len()))?;
    let atomic = expect_atomic(name, atomic)?;
    let order = load_ordering(name, expect_memory_order(name, order)?)?;
    Ok(Value::I64(atomic.cell.load(order)))
}

/// `atomic_store_ordered(a atomic_i64, v i64, order MemoryOrder) -> void`: write
/// the cell under `order` (`relaxed`/`release`/`seq_cst`).
pub fn builtin_atomic_store_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = "atomic_store_ordered";
    let [atomic, value, order]: [Value; 3] = args
        .try_into()
        .map_err(|args: Vec<Value>| ordering_arity(name, 3, args.len()))?;
    let atomic = expect_atomic(name, atomic)?;
    let value = expect_i64(name, value)?;
    let order = store_ordering(name, expect_memory_order(name, order)?)?;
    atomic.cell.store(value, order);
    Ok(Value::Void)
}

/// Shared decode for the ordered read-modify-write family: an atomic handle, an
/// `i64` operand, and any of the five orderings (all are valid for an RMW).
fn atomic_rmw_ordered_args(
    name: &str,
    args: Vec<Value>,
) -> Result<(SharedAtomic, i64, Ordering), RuntimeError> {
    let [atomic, value, order]: [Value; 3] = args
        .try_into()
        .map_err(|args: Vec<Value>| ordering_arity(name, 3, args.len()))?;
    let atomic = expect_atomic(name, atomic)?;
    let value = expect_i64(name, value)?;
    let order = expect_memory_order(name, order)?;
    Ok((atomic, value, order))
}

/// `atomic_swap_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`: store
/// `v`, return the previous value, under any of the five orderings.
pub fn builtin_atomic_swap_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_swap_ordered", args)?;
    Ok(Value::I64(atomic.cell.swap(value, order)))
}

/// `atomic_add_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`:
/// fetch-and-add returning the previous value, under any of the five orderings.
pub fn builtin_atomic_add_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_add_ordered", args)?;
    Ok(Value::I64(atomic.cell.fetch_add(value, order)))
}

/// `atomic_sub_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`:
/// fetch-and-sub returning the previous value, under any of the five orderings.
pub fn builtin_atomic_sub_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_sub_ordered", args)?;
    Ok(Value::I64(atomic.cell.fetch_sub(value, order)))
}

/// `atomic_and_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`:
/// fetch-and-and returning the previous value, under any of the five orderings.
pub fn builtin_atomic_and_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_and_ordered", args)?;
    Ok(Value::I64(atomic.cell.fetch_and(value, order)))
}

/// `atomic_or_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`:
/// fetch-and-or returning the previous value, under any of the five orderings.
pub fn builtin_atomic_or_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_or_ordered", args)?;
    Ok(Value::I64(atomic.cell.fetch_or(value, order)))
}

/// `atomic_xor_ordered(a atomic_i64, v i64, order MemoryOrder) -> i64`:
/// fetch-and-xor returning the previous value, under any of the five orderings.
pub fn builtin_atomic_xor_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let (atomic, value, order) = atomic_rmw_ordered_args("atomic_xor_ordered", args)?;
    Ok(Value::I64(atomic.cell.fetch_xor(value, order)))
}

/// `atomic_cas_ordered(a atomic_i64, expected i64, new i64, success MemoryOrder,
/// failure MemoryOrder) -> i64`: strong compare-and-swap returning the observed
/// value. `success` may be any of the five orderings; `failure` must be a valid
/// load ordering (`relaxed`/`acquire`/`seq_cst`), exactly as `std` requires.
pub fn builtin_atomic_cas_ordered(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = "atomic_cas_ordered";
    let [atomic, expected, new, success, failure]: [Value; 5] = args
        .try_into()
        .map_err(|args: Vec<Value>| ordering_arity(name, 5, args.len()))?;
    let atomic = expect_atomic(name, atomic)?;
    let expected = expect_i64(name, expected)?;
    let new = expect_i64(name, new)?;
    let success = expect_memory_order(name, success)?;
    let failure = load_ordering(name, expect_memory_order(name, failure)?)?;
    let observed = match atomic
        .cell
        .compare_exchange(expected, new, success, failure)
    {
        Ok(prev) => prev,
        Err(current) => current,
    };
    Ok(Value::I64(observed))
}

/// `fence(order MemoryOrder) -> void`: a standalone memory fence mapping to
/// `std::sync::atomic::fence(order)`. `order` must be `acquire`/`release`/
/// `acq_rel`/`seq_cst` (a `relaxed` fence is meaningless and `std` panics on it).
pub fn builtin_fence(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = "fence";
    let [order]: [Value; 1] = args
        .try_into()
        .map_err(|args: Vec<Value>| ordering_arity(name, 1, args.len()))?;
    let order = fence_ordering(name, expect_memory_order(name, order)?)?;
    std::sync::atomic::fence(order);
    Ok(Value::Void)
}

/// Create a fresh unbounded `i64` channel as a `Value::Chan`. Shared by the AST
/// and IR interpreters so both keep identical channel semantics.
pub fn new_chan() -> Value {
    let (sender, receiver) = std::sync::mpsc::channel::<Value>();
    Value::Chan(Chan {
        sender,
        receiver: Arc::new(Mutex::new(receiver)),
    })
}

/// Join a spawned thread once: take the `JoinHandle` out of the shared slot and
/// wait for it, propagating a worker error or panic. A second `join` (the slot is
/// already `None`) is a no-op that returns `void`. Shared by both interpreters.
pub fn join_task(task: &Task) -> Result<Value, RuntimeError> {
    let handle = {
        let mut slot = task
            .handle
            .lock()
            .map_err(|_| RuntimeError::new("L0401", "join on a poisoned task handle"))?;
        slot.take()
    };
    match handle {
        // Already joined: joining again is a harmless no-op.
        None => Ok(Value::Void),
        Some(handle) => match handle.join() {
            Ok(result) => result.map(|_| Value::Void),
            Err(_) => Err(RuntimeError::new("L0401", "spawned thread panicked")),
        },
    }
}

/// Await a future once: take the `JoinHandle` out of the shared slot, wait for
/// the spawned thread, and return the `Value` it produced (unlike `join_task`,
/// which discards the value). A worker error propagates; a panic is `L0401`. A
/// second `await` on the same handle (the slot is already `None`) returns
/// `void` — a defensive no-op, though semantics binds each future to one
/// `await`. Shared by both interpreters so `await` has identical semantics.
pub fn await_future(future: &Future) -> Result<Value, RuntimeError> {
    let handle = {
        let mut slot = future
            .handle
            .lock()
            .map_err(|_| RuntimeError::new("L0401", "await on a poisoned future handle"))?;
        slot.take()
    };
    match handle {
        None => Ok(Value::Void),
        Some(handle) => match handle.join() {
            Ok(result) => result,
            Err(_) => Err(RuntimeError::new("L0401", "awaited thread panicked")),
        },
    }
}
