//! The AST interpreter's actor scheduler (stage 1: `spawn` + `tell`; stage 2:
//! `ask` + `await` + `Future<R>`; stage 4: supervision).
//!
//! **Supervision (stage 4) — failure is result-based, not panic-based.** This is
//! the one design point worth reading before the code. Lullaby's decided failure
//! semantics (A5) abort the program on a contract/memory-safety violation and
//! deliberately do **not** unwind, so a supervisor *cannot* catch a panicking
//! child — there is no unwinding to catch, and adding one for actors would carve
//! an exception into the decision that keeps aborts allocation-free and
//! freestanding-safe. A5's own reasoning already says recoverable errors flow
//! through `result`/`?`, so panics are reserved for bugs; supervision is that
//! principle applied to actors:
//!
//! - A **fallible handler** is one declared `-> result<R, E>`. Replying `err(e)`
//!   is an actor **failure** — a normal, expected, recoverable outcome.
//! - A **supervisor** (the spawning actor, named by a `spawn ... supervise
//!   POLICY` clause) observes that `err` and applies [`SupervisionPolicy`].
//! - A **genuine panic** inside a handler (an out-of-bounds index, a
//!   divide-by-zero) is a *bug*: it aborts the whole program exactly as it would
//!   anywhere else, and is never supervised. You do not restart an actor that hit
//!   a bug; you fix the bug.
//!
//! Supervision is **opt-in**: with no `supervise` clause an `err` reply is just
//! an ordinary value the asker matches on. It has to be — `err` is also the
//! ordinary recoverable-error channel (a `withdraw` handler replying
//! `err("insufficient funds")` is answering correctly, not failing), so only an
//! explicit clause can mark a child's `err` as a supervised failure.
//!
//! **No restart loop is possible.** The message that failed is *consumed* by the
//! turn that returned `err` (and its asker has already been given that `err`), so
//! a restart never re-delivers it. The poison-message restart storm that forces
//! Erlang-style supervisors to carry backoff/limit policies cannot arise here,
//! which is why none is needed.
//!
//! Actors run on a single-threaded, cooperative, deterministic scheduler. `spawn`
//! constructs an actor (zero-initializing its private `state`, then running its
//! `init`) and returns a typed [`Value::ActorRef`] handle. `tell` enqueues a
//! fire-and-forget message on a global FIFO mailbox; `ask` enqueues a request
//! carrying a one-shot reply slot and hands back a `Future<R>`
//! ([`Value::ActorFuture`]). `await` on that future drives the mailbox until the
//! slot is filled by the target handler's reply.
//!
//! **Turn model — non-reentrant run-to-completion.** An actor is *busy* for the
//! whole span of a message turn (including any nested `await`s). The scheduler
//! only ever runs the first *deliverable* message — one whose target actor is not
//! busy — so a single actor never runs two turns at once and its `state` stays a
//! single-writer resource with no data races. When nothing is on the stack (the
//! graceful drain before `main` returns), every message is deliverable, so the
//! order is plain FIFO and fully deterministic.
//!
//! **Ordering & fulfillment.** `await f` repeatedly runs the next deliverable
//! message until `f`'s reply slot is filled, then takes it. Because dispatch is
//! FIFO over deliverable messages on one thread, the sequence of turns — and thus
//! every reply and side effect — is identical on every run.
//!
//! **Deadlock.** If an `await` can only be satisfied by re-entering a busy actor
//! (a request cycle, e.g. A asks B while B is awaiting a reply from A), no message
//! is deliverable and the awaited slot can never fill. That is reported as a clean,
//! deterministic runtime error (`L0356`) rather than hanging.
//!
//! **Back-pressure (stage 5b).** Mailboxes are **bounded**: each actor carries a
//! capacity (`DEFAULT_MAILBOX_CAPACITY`, or the `spawn ... bound N` override) and
//! an occupancy counter of how many of its messages are currently queued. The
//! queue itself stays one global `VecDeque` — on a deterministic single thread,
//! per-actor queues drained by "lowest global sequence among deliverable heads"
//! *is* this scan, so counters buy per-target pressure with zero change to
//! delivery order. `tell`/`ask` to a full mailbox **block until space frees**,
//! cooperatively pumping `run_one_deliverable` exactly as `await` does; when
//! nothing is deliverable and the target is still full, no turn can ever free a
//! slot and that is the deterministic back-pressure deadlock `L0365`. `try_tell`
//! is the load-shedding escape hatch: it never pumps and answers `bool`.
//!
//! This is the AST-interpreter runtime; the IR/bytecode backends reject an actor
//! program (`L0355`) and the native/WASM backends cleanly skip it, so actors run
//! only here.

use std::collections::VecDeque;

use lullaby_parser::{
    ActorDecl, ActorHandler, CombinatorOp, StructField, SupervisionPolicy, TypeRef,
};

use super::*;

// ---------------------------------------------------------------------------
// The actor data model. These types live here, with the scheduler that owns
// them, rather than in `interpreter.rs`: the `Runtime` merely holds the tables,
// while every rule about what the fields mean is in this file.
// ---------------------------------------------------------------------------

/// One scheduled actor: its declared type name and its private `state`, held as
/// an ordered `(field, value)` list exactly like a struct's fields. The state is
/// touched only by the actor's own handlers, one message at a time, so it is a
/// single-writer resource with no locking.
///
/// The remaining fields carry stage-4 **supervision** (see `runtime_actor.rs`):
/// who supervises this actor, what policy applies when one of its fallible
/// handlers replies `err`, the `spawn` arguments a `restart` replays into
/// `init`, and whether it has been stopped.
#[derive(Debug, Clone)]
pub(crate) struct ActorInstance {
    pub(crate) actor_name: String,
    pub(crate) state: Vec<(String, Value)>,
    /// The actor that spawned this one — its **supervisor**, and the actor an
    /// `escalate` propagates to. `None` when it was spawned outside any actor
    /// turn (from `main` or a free function it called), i.e. it is a root actor
    /// and an escalation from it has nowhere to go but the program's exit.
    ///
    /// A supervisor is always spawned before its children, so `supervisor` is
    /// strictly less than the instance's own id and the supervision links form a
    /// forest — an escalation chain therefore always terminates.
    pub(crate) supervisor: Option<usize>,
    /// The policy from this actor's `spawn ... supervise POLICY` clause, or
    /// `None` when it is unsupervised (an `err` reply is then an ordinary value
    /// with no supervisory effect).
    pub(crate) policy: Option<SupervisionPolicy>,
    /// The evaluated `spawn` arguments, retained so a `restart` can re-run `init`
    /// with exactly the arguments the actor was originally constructed with.
    pub(crate) spawn_args: Vec<Value>,
    /// True once the actor has been stopped (by a `stop` policy, or by an
    /// `escalate` that terminated it). A stopped actor runs no further turns:
    /// `tell`s to it are dropped and `ask`s to it resolve to `L0359`.
    pub(crate) stopped: bool,
    /// This actor's **mailbox capacity**: the most messages that may be pending
    /// for it at once. [`DEFAULT_MAILBOX_CAPACITY`] unless the `spawn` carried a
    /// `bound N` clause. Always at least 1 (the parser rejects `bound 0`).
    pub(crate) capacity: usize,
    /// How many messages targeting this actor are currently sitting on the global
    /// mailbox — its **occupancy** against `capacity`. Incremented when a message
    /// for this actor is enqueued, decremented when the scheduler dequeues one
    /// for dispatch, and zeroed when a supervision `stop` purges the actor's
    /// queued messages. A `restart` deliberately leaves it alone: a restart
    /// preserves the mailbox, and those preserved messages still occupy memory
    /// and will still be delivered.
    pub(crate) pending: usize,
}

/// The mailbox capacity an actor gets when its `spawn` carries no `bound N`
/// clause. Unbounded mailboxes are a memory-safety hazard — a fast producer can
/// grow the queue without limit — so every actor is bounded by default, and the
/// default is set high enough that ordinary programs never notice it.
pub(crate) const DEFAULT_MAILBOX_CAPACITY: usize = 1024;

/// A supervisory action to carry out on an actor: what a [`SupervisionPolicy`]
/// resolves to once a failure has actually occurred. (`escalate` is a policy but
/// not an action — it *stops* the escalating actor and hands the failure to its
/// supervisor, whose own policy then resolves to one of these.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SupervisionAction {
    /// Discard the actor's state and re-run its `init` with the original `spawn`
    /// arguments.
    Restart,
    /// Terminate the actor: it runs no further turns.
    Stop,
}

/// The state of a one-shot `ask` reply slot. A slot is tri-state rather than an
/// `Option<Value>` because a stopped actor must fail its pending askers
/// *deterministically* instead of leaving them to wait forever: a reply that can
/// never be produced is [`ReplySlot::Unavailable`], and `await` on it raises
/// `L0359` rather than hanging.
///
/// The failure cannot be modeled as a synthesized `err(e)` value: the reply type
/// is the user's own `result<R, E>`, and the runtime has no way to conjure an
/// inhabitant of an arbitrary `E`. A clean, deterministic runtime error is the
/// honest outcome.
#[derive(Debug, Clone)]
pub(crate) enum ReplySlot {
    /// The handler's turn has not run yet; the reply is still to come.
    Pending,
    /// The handler replied with this value (taken by the awaiting `await`).
    Filled(Value),
    /// The reply can never be produced: the target actor was stopped before it
    /// could run this request's turn.
    Unavailable,
}

/// One pending message on the global mailbox: which actor it targets, which
/// handler to run, and the (already-evaluated, moved-across-the-boundary)
/// argument values. Drained FIFO before `main` returns.
#[derive(Debug, Clone)]
pub(crate) struct ActorMessage {
    pub(crate) actor_id: usize,
    pub(crate) handler: String,
    pub(crate) args: Vec<Value>,
    /// For an `ask` request, the index of the `actor_reply_slots` entry the
    /// handler's reply value is written into once the turn completes; `None` for a
    /// fire-and-forget `tell`.
    pub(crate) reply_slot: Option<usize>,
}

/// True when `value` is the `err(e)` variant of a `result` — the shape a fallible
/// actor handler replies with to report a failure. Paired with a check that the
/// handler is *declared* `-> result<R, E>` (see `dispatch_message`), this is the
/// entire supervised-failure trigger.
fn is_err_value(value: &Value) -> bool {
    matches!(value, Value::Enum(inner) if inner.enum_name == "result" && inner.variant == "err")
}

impl<'a> Runtime<'a> {
    /// `spawn NAME(args) [supervise POLICY]`: allocate an actor instance with
    /// zero-initialized state, run its `init` (if any) with `args`, register it
    /// on the scheduler, and return its handle. Semantics has already checked the
    /// actor exists and the argument count/types match `init`, so the runtime
    /// guards here are defensive.
    ///
    /// The new actor's **supervisor** is the actor whose turn is spawning it
    /// (`current_actor`), or `None` when spawned from `main` — a root actor. Its
    /// `policy` comes from the `supervise` clause, and the evaluated `args` are
    /// retained so a `restart` can replay them into `init`.
    ///
    /// `bound` is the `bound N` clause's mailbox capacity, or `None` for
    /// [`DEFAULT_MAILBOX_CAPACITY`].
    pub(crate) fn spawn_actor(
        &mut self,
        actor_name: &str,
        args: Vec<Value>,
        policy: Option<SupervisionPolicy>,
        bound: Option<usize>,
    ) -> Result<Value, RuntimeError> {
        let decl: &'a ActorDecl = match self.actors.get(actor_name).copied() {
            Some(decl) => decl,
            None => {
                return Err(RuntimeError::new(
                    "L0401",
                    format!("`spawn` of unknown actor `{actor_name}`"),
                ));
            }
        };
        // Zero-initialize every state field before `init` runs. A well-formed
        // `init` overwrites the fields it needs; the zero is the value a handler
        // would see for any field the author leaves unset.
        let state: Vec<(String, Value)> = decl
            .state
            .iter()
            .map(|field| (field.name.clone(), self.zero_value(&field.ty)))
            .collect();
        let id = self.actor_instances.len();
        self.actor_instances.push(ActorInstance {
            actor_name: actor_name.to_string(),
            state,
            supervisor: self.current_actor,
            policy,
            spawn_args: args.clone(),
            stopped: false,
            capacity: bound.unwrap_or(DEFAULT_MAILBOX_CAPACITY),
            pending: 0,
        });

        match &decl.init {
            Some(init) => {
                let param_names: Vec<String> =
                    init.params.iter().map(|param| param.name.clone()).collect();
                self.run_actor_turn(id, &decl.state, &param_names, &init.body, args)?;
            }
            None if args.is_empty() => {}
            None => {
                return Err(RuntimeError::new(
                    "L0402",
                    format!(
                        "actor `{actor_name}` declares no `init` but `spawn` was given {} argument(s)",
                        args.len()
                    ),
                ));
            }
        }
        Ok(Value::ActorRef(id))
    }

    /// Resolve a message-send target to an actor id, rejecting a non-handle or a
    /// dangling handle with the shared `L0401` guard. `verb` names the send form
    /// for the diagnostic. Semantics has already proved the target is an
    /// `Actor<T>`, so both arms are defensive.
    fn send_target_id(&self, target: &Value, verb: &str) -> Result<usize, RuntimeError> {
        let Value::ActorRef(actor_id) = target else {
            return Err(RuntimeError::new(
                "L0401",
                format!("`{verb}` target is not an actor handle: `{target}`"),
            ));
        };
        if *actor_id >= self.actor_instances.len() {
            return Err(RuntimeError::new(
                "L0401",
                format!("`{verb}` to unknown actor handle `{actor_id}`"),
            ));
        }
        Ok(*actor_id)
    }

    /// Enqueue a message on the global mailbox, charging it against its target's
    /// [`ActorInstance::pending`] occupancy. Every enqueue goes through here so
    /// the counter and the queue can never disagree.
    fn enqueue_message(&mut self, message: ActorMessage) {
        self.actor_instances[message.actor_id].pending += 1;
        self.actor_mailbox.push_back(message);
    }

    /// **Back-pressure.** Block until `actor_id` has room in its mailbox, by
    /// cooperatively pumping the scheduler: while the target is at capacity, run
    /// the next deliverable message (freeing a slot whenever that message is one
    /// of the target's own) and re-check. Returns once occupancy is below
    /// capacity.
    ///
    /// **Why this stays deterministic.** The pump is the *same*
    /// [`Runtime::run_one_deliverable`] `await`, `join_all` and `select` already
    /// drive: one thread, run-to-completion turns, and the message chosen is
    /// always the earliest queued deliverable one. Re-entering the scheduler from
    /// the middle of an expression is likewise not new — `await_actor_future`
    /// has always run other actors' turns nested on the Rust call stack. So a
    /// pumping `tell` adds no new ordering freedom: the sequence of turns, and
    /// therefore every side effect, is identical on every run.
    ///
    /// When nothing is deliverable and the target is still full, no future turn
    /// can ever free a slot (the queue only shrinks by dispatch), so the send can
    /// never complete: that is the deterministic back-pressure deadlock `L0365`,
    /// reported rather than hung. The usual shape is a send to an actor that is
    /// itself mid-turn — its own queued messages are not deliverable while it
    /// runs, so only it could drain them, and it is blocked sending.
    fn reserve_mailbox_capacity(
        &mut self,
        actor_id: usize,
        verb: &str,
    ) -> Result<(), RuntimeError> {
        loop {
            let instance = &self.actor_instances[actor_id];
            if instance.pending < instance.capacity {
                return Ok(());
            }
            if !self.run_one_deliverable()? {
                let instance = &self.actor_instances[actor_id];
                return Err(RuntimeError::new(
                    "L0365",
                    format!(
                        "`{verb}` can never complete: actor `{}`'s mailbox is full ({} of {} messages pending) and no queued message is deliverable to free a slot — only that actor can drain its own mailbox, and it is mid-turn (or every queued message targets a busy actor) — this is a deterministic back-pressure deadlock",
                        instance.actor_name, instance.pending, instance.capacity
                    ),
                ));
            }
        }
    }

    /// `tell TARGET.HANDLER(args)`: enqueue a fire-and-forget message on the
    /// target actor's mailbox and return `void`. The message is processed later,
    /// during the graceful drain before `main` returns.
    ///
    /// **Back-pressure.** Mailboxes are bounded (see
    /// [`DEFAULT_MAILBOX_CAPACITY`] and the `spawn ... bound N` clause). A `tell`
    /// to a full mailbox does not fail and does not silently grow the queue: it
    /// **blocks until space frees**, cooperatively pumping the scheduler through
    /// [`Runtime::reserve_mailbox_capacity`], which is where the `L0365`
    /// deadlock — and the determinism argument for the pump — lives. Use
    /// `try_tell` to shed instead of blocking.
    ///
    /// A `tell` to a **stopped** actor is dropped: the message is not enqueued and
    /// the send still evaluates to `void`. That is what stopping means — the actor
    /// runs no further turns — and a fire-and-forget send has no channel on which
    /// to report anything back, so silently discarding it is the only coherent
    /// outcome (an `ask` to a stopped actor, which *does* have a channel, reports
    /// `L0359` instead).
    pub(crate) fn tell_actor(
        &mut self,
        target: Value,
        handler: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let actor_id = self.send_target_id(&target, "tell")?;
        if self.actor_instances[actor_id].stopped {
            return Ok(Value::Void);
        }
        self.reserve_mailbox_capacity(actor_id, "tell")?;
        // Pumping ran other actors' turns, one of which may have stopped the
        // target through supervision. The drop rule applies at the moment of
        // enqueue, not at the moment the send was written.
        if self.actor_instances[actor_id].stopped {
            return Ok(Value::Void);
        }
        self.enqueue_message(ActorMessage {
            actor_id,
            handler: handler.to_string(),
            args,
            reply_slot: None,
        });
        Ok(Value::Void)
    }

    /// `try_tell TARGET.HANDLER(args)`: the **load-shedding** variant of `tell`.
    /// It has `tell`'s fire-and-forget semantics but never blocks: if the target's
    /// mailbox is already at capacity the message is dropped and the send
    /// evaluates to `false`; otherwise it is enqueued exactly as `tell` would and
    /// the send evaluates to `true`.
    ///
    /// Because it never pumps, `try_tell` can never raise the back-pressure
    /// deadlock `L0365` — that is the whole point of having it. A `try_tell` to a
    /// **stopped** actor likewise yields `false`: the message was not enqueued,
    /// and reporting `true` for a message that can never be delivered would be a
    /// lie about the only thing this form promises to answer.
    pub(crate) fn try_tell_actor(
        &mut self,
        target: Value,
        handler: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let actor_id = self.send_target_id(&target, "try_tell")?;
        let instance = &self.actor_instances[actor_id];
        if instance.stopped || instance.pending >= instance.capacity {
            return Ok(Value::Bool(false));
        }
        self.enqueue_message(ActorMessage {
            actor_id,
            handler: handler.to_string(),
            args,
            reply_slot: None,
        });
        Ok(Value::Bool(true))
    }

    /// `ask TARGET.HANDLER(args)`: enqueue a request-reply message on the target
    /// actor's mailbox and return a `Future<R>` ([`Value::ActorFuture`]) for the
    /// reply. A fresh one-shot reply slot is allocated; the handler's turn writes
    /// its reply value into it, and `await` on the returned future drives the
    /// scheduler until the slot is filled.
    ///
    /// An `ask` to a **stopped** actor still hands back a well-formed `Future<R>`,
    /// but its slot is [`ReplySlot::Unavailable`] from the start: the reply can
    /// never be produced, so `await`ing it reports `L0359` deterministically
    /// instead of waiting for a turn that will never run.
    ///
    /// **Back-pressure.** Only the *request* is bounded, exactly as for `tell`: a
    /// request to a full mailbox blocks until space frees (see
    /// [`Runtime::reserve_mailbox_capacity`], including its `L0365` deadlock).
    /// The **reply** is never bounded — `dispatch_message` writes it straight
    /// into the reply slot rather than onto the asker's mailbox — so a full
    /// mailbox can never lose, delay, or deadlock a reply.
    ///
    /// The capacity is reserved *before* the reply slot is allocated: pumping may
    /// itself run turns that `ask`, and allocating first would hand this request
    /// a slot index from before those requests rather than after them.
    pub(crate) fn ask_actor(
        &mut self,
        target: Value,
        handler: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let actor_id = self.send_target_id(&target, "ask")?;
        if !self.actor_instances[actor_id].stopped {
            self.reserve_mailbox_capacity(actor_id, "ask")?;
        }
        let slot = self.actor_reply_slots.len();
        // Re-read `stopped`: the actor may have been stopped by a turn the pump
        // above ran, and a request to a stopped actor can never be answered.
        if self.actor_instances[actor_id].stopped {
            self.actor_reply_slots.push(ReplySlot::Unavailable);
            return Ok(Value::ActorFuture(slot));
        }
        self.actor_reply_slots.push(ReplySlot::Pending);
        self.enqueue_message(ActorMessage {
            actor_id,
            handler: handler.to_string(),
            args,
            reply_slot: Some(slot),
        });
        Ok(Value::ActorFuture(slot))
    }

    /// `await` on an actor request-reply future: drive the mailbox until the
    /// awaited reply slot is filled, then take and return the reply value. Each
    /// iteration runs the next *deliverable* message (one whose target actor is not
    /// mid-turn). If no message is deliverable and the slot is still empty, the
    /// reply can never be produced — a deterministic deadlock reported as `L0356`
    /// rather than a hang.
    ///
    /// A slot the scheduler has marked [`ReplySlot::Unavailable`] — the target was
    /// stopped by supervision before it could run this request — likewise can never
    /// fill, and is reported as `L0359`. Both exits are errors rather than hangs:
    /// an `await` in this scheduler always terminates.
    pub(crate) fn await_actor_future(
        &mut self,
        slot: usize,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        loop {
            match self.actor_reply_slots.get_mut(slot) {
                Some(entry @ ReplySlot::Filled(_)) => {
                    // Take the reply, leaving the slot `Pending` — the state it
                    // held before the handler replied. A `Future<R>` is one-shot,
                    // so this slot is never legitimately read again; resetting to
                    // `Pending` (rather than `Unavailable`) keeps a stray second
                    // `await` on the same future reporting the stage-2 deadlock
                    // `L0356`, instead of misattributing it to supervision.
                    let ReplySlot::Filled(reply) = std::mem::replace(entry, ReplySlot::Pending)
                    else {
                        // The arm above already proved the slot is `Filled`.
                        unreachable!("slot matched as filled")
                    };
                    return Ok(reply);
                }
                Some(ReplySlot::Unavailable) => {
                    return Err(RuntimeError::new(
                        "L0359",
                        "`await` can never complete: the actor that would produce this reply was stopped by supervision, so the request will never run a turn",
                    )
                    .with_span(span));
                }
                Some(ReplySlot::Pending) | None => {}
            }
            if !self.run_one_deliverable()? {
                return Err(RuntimeError::new(
                    "L0356",
                    "`await` can never complete: the awaited actor reply cannot be produced because no queued message is deliverable (a request cycle re-enters an actor that is already running) — this is a deterministic deadlock",
                )
                .with_span(span));
            }
        }
    }

    /// Drain the actor mailbox: run every outstanding message to completion, one
    /// at a time, until nothing is deliverable. At the top-level graceful drain no
    /// actor is busy, so every message is deliverable and the queue empties fully
    /// in FIFO order. A handler may itself `tell`/`ask`/`spawn` during its turn,
    /// appending to the queue; the loop continues until it is empty.
    pub(crate) fn drain_actors(&mut self) -> Result<(), RuntimeError> {
        while self.run_one_deliverable()? {}
        Ok(())
    }

    /// `join_all EXPR` / `select EXPR`: the entry point for both `Future<T>`
    /// combinators. The operand has already been evaluated to a value that
    /// semantics guarantees is a `list<Future<T>>` — a [`Value::Array`] of
    /// [`Value::ActorFuture`] reply slots. The guards here are defensive (a
    /// mis-typed operand that slipped past semantics is a clean `L0364`, never a
    /// panic), mirroring the actor scheduler's other runtime guards.
    pub(crate) fn eval_combinator(
        &mut self,
        op: CombinatorOp,
        operand: &Expr,
        env: &mut Env,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let operand = self.eval_expr(operand, env)?;
        let Value::Array(items) = operand else {
            return Err(RuntimeError::new(
                "L0364",
                format!(
                    "`{}` expects a `list<Future<T>>` but its operand is not a collection: `{operand}`",
                    op.as_str()
                ),
            )
            .with_span(span));
        };
        let mut slots = Vec::with_capacity(items.len());
        for item in items.iter() {
            let Value::ActorFuture(slot) = item else {
                return Err(RuntimeError::new(
                    "L0364",
                    format!(
                        "`{}` collection element is not a `Future<T>`: `{item}`",
                        op.as_str()
                    ),
                )
                .with_span(span));
            };
            slots.push(*slot);
        }
        match op {
            CombinatorOp::JoinAll => self.join_all_futures(&slots, span),
            CombinatorOp::Select => self.select_futures(&slots, span),
        }
    }

    /// `join_all`: wait for **every** future to resolve and return a `list<T>` of
    /// the results in input order. Deterministic: each slot is `await`ed in turn
    /// (driving the same deterministic mailbox `await` drives), so the result
    /// order is exactly the input order and repeated runs are byte-identical. A
    /// deadlock or a supervision-stopped target surfaces through `await`'s own
    /// `L0356`/`L0359`, unchanged.
    fn join_all_futures(&mut self, slots: &[usize], span: Span) -> Result<Value, RuntimeError> {
        let mut results = Vec::with_capacity(slots.len());
        for &slot in slots {
            results.push(self.await_actor_future(slot, span)?);
        }
        Ok(Value::Array(results.into_boxed_slice()))
    }

    /// `select`: wait for the **first** future to resolve and return a
    /// `Selected<T>` { `index i64`, `value T` } naming the winning input position
    /// and its reply. Determinism and the tie-break rule:
    ///
    /// - On each step the slots are scanned in **input order**; the first slot
    ///   already [`ReplySlot::Filled`] wins, so when several are ready at once the
    ///   **lowest input index** is chosen. This makes a tie fully deterministic.
    /// - If no slot is filled yet, one deliverable message is run (the same pump
    ///   `await` uses) and the scan repeats. The single-threaded run-to-completion
    ///   schedule is fixed, so the winner — and thus the whole result — is
    ///   identical on every run.
    ///
    /// Only the winner is consumed (its slot is taken and reset to `Pending`, as
    /// `await` does); the losing futures are left untouched and remain awaitable.
    ///
    /// Termination mirrors `await`: an empty collection, or one whose futures can
    /// never resolve, is a clean deterministic error rather than a hang. When the
    /// pump stalls with pending futures still outstanding it is the same
    /// request-cycle deadlock as `await` (`L0356`); when every future's target was
    /// stopped by supervision it is `L0359`.
    fn select_futures(&mut self, slots: &[usize], span: Span) -> Result<Value, RuntimeError> {
        if slots.is_empty() {
            return Err(RuntimeError::new(
                "L0356",
                "`select` over an empty collection can never complete: there is no future to wait for",
            )
            .with_span(span));
        }
        loop {
            // Scan in input order so the lowest index wins a tie.
            for (index, &slot) in slots.iter().enumerate() {
                if let Some(ReplySlot::Filled(_)) = self.actor_reply_slots.get(slot) {
                    let entry = &mut self.actor_reply_slots[slot];
                    let ReplySlot::Filled(reply) = std::mem::replace(entry, ReplySlot::Pending)
                    else {
                        unreachable!("slot matched as filled")
                    };
                    return Ok(Value::Struct(Box::new(StructValue {
                        name: "Selected".to_string(),
                        fields: vec![
                            ("index".to_string(), Value::I64(index as i64)),
                            ("value".to_string(), reply),
                        ],
                    })));
                }
            }
            // No slot resolved. If every future's target was stopped by
            // supervision, none can ever resolve — report that, not a deadlock.
            let all_unavailable = slots.iter().all(|&slot| {
                matches!(
                    self.actor_reply_slots.get(slot),
                    Some(ReplySlot::Unavailable)
                )
            });
            if all_unavailable {
                return Err(RuntimeError::new(
                    "L0359",
                    "`select` can never complete: every future's target actor was stopped by supervision, so no reply will ever be produced",
                )
                .with_span(span));
            }
            if !self.run_one_deliverable()? {
                return Err(RuntimeError::new(
                    "L0356",
                    "`select` can never complete: no queued message is deliverable while futures are still pending (a request cycle re-enters an actor that is already running) — this is a deterministic deadlock",
                )
                .with_span(span));
            }
        }
    }

    /// Run the first *deliverable* mailbox message — the earliest queued message
    /// whose target actor is not currently running a turn — to completion. Returns
    /// `true` if one ran, `false` if none is deliverable (the queue is empty, or
    /// every pending message targets a busy actor). Skipping a message bound for a
    /// busy actor preserves per-target FIFO (that actor's messages stay in order,
    /// merely delayed) while upholding the non-reentrant run-to-completion
    /// guarantee.
    ///
    /// A stopped actor's messages are purged from the mailbox when it stops, so no
    /// queued message can target one; the `stopped` guard here is the belt-and-
    /// braces half of that invariant, keeping a stopped actor unschedulable even
    /// if a message reached the queue by another path.
    ///
    /// This is also the only place a message *leaves* the queue by delivery, so it
    /// is where the target's [`ActorInstance::pending`] occupancy is released —
    /// at dequeue, before the turn runs, because the mailbox slot is free from
    /// that moment on. (The other exit is a supervision `stop`'s purge; see
    /// [`Runtime::stop_actor`].)
    fn run_one_deliverable(&mut self) -> Result<bool, RuntimeError> {
        let Some(index) = self.actor_mailbox.iter().position(|message| {
            !self.busy_actors.contains(&message.actor_id)
                && !self.actor_instances[message.actor_id].stopped
        }) else {
            return Ok(false);
        };
        // `index` was just computed from the live queue, so the removal resolves.
        let message = self
            .actor_mailbox
            .remove(index)
            .expect("deliverable index is valid");
        let instance = &mut self.actor_instances[message.actor_id];
        // Every queued message was charged to its target by `enqueue_message`, so
        // this cannot underflow.
        debug_assert!(
            instance.pending > 0,
            "dequeued a message for actor `{}` with zero pending occupancy",
            instance.actor_name
        );
        instance.pending = instance.pending.saturating_sub(1);
        self.dispatch_message(message)?;
        Ok(true)
    }

    /// Run one mailbox message on its target actor: locate the handler on the
    /// actor's declaration, run its body as a turn over the actor's state, and —
    /// for an `ask` request — write the turn's reply value into its reply slot.
    ///
    /// **Supervision order (load-bearing).** The reply is published into its slot
    /// *before* any policy is applied, so an asker awaiting a failing supervised
    /// child always receives the child's own `err(e)` — the failure reaches the
    /// asker as an ordinary value even as the supervisor restarts or stops the
    /// child. Restarting first would strand that asker.
    fn dispatch_message(&mut self, message: ActorMessage) -> Result<(), RuntimeError> {
        let actor_name = self.actor_instances[message.actor_id].actor_name.clone();
        let decl: &'a ActorDecl = match self.actors.get(actor_name.as_str()).copied() {
            Some(decl) => decl,
            None => {
                return Err(RuntimeError::new(
                    "L0401",
                    format!("actor instance references unknown actor `{actor_name}`"),
                ));
            }
        };
        let handler: &'a ActorHandler = decl
            .handlers
            .iter()
            .find(|handler| handler.name == message.handler)
            .ok_or_else(|| {
                RuntimeError::new(
                    "L0401",
                    format!("actor `{actor_name}` has no handler `{}`", message.handler),
                )
            })?;
        let param_names: Vec<String> = handler
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let reply = self.run_actor_turn(
            message.actor_id,
            &decl.state,
            &param_names,
            &handler.body,
            message.args,
        )?;
        // A supervised failure is a **fallible** handler — one declared
        // `-> result<R, E>` — replying `err(e)`. Both halves are required: the
        // declared signature is what makes `err` a *failure* rather than an
        // incidental value, so a fire-and-forget handler whose body merely happens
        // to end in an `err` value never triggers supervision.
        let failed = handler
            .reply_type
            .as_ref()
            .is_some_and(|reply_type| reply_type.result_args().is_some())
            && is_err_value(&reply);
        // For an `ask` request, the handler's turn value is the reply: publish it
        // into the reply slot so an `await` on the corresponding future resolves.
        // This happens first — see the note on this function — so a failure is
        // delivered to its asker even when the policy then stops the child.
        if let Some(slot) = message.reply_slot {
            self.actor_reply_slots[slot] = ReplySlot::Filled(reply);
        }
        if failed {
            self.supervise_failure(message.actor_id)?;
        }
        Ok(())
    }

    /// Apply the supervision policy of the actor that just failed (its fallible
    /// handler replied `err`). An unsupervised actor — no `supervise` clause at
    /// its `spawn` — has no policy, so its `err` is simply an ordinary value and
    /// nothing happens here.
    ///
    /// **Termination.** `escalate` walks from the failed actor to its supervisor,
    /// and a supervisor is always spawned before its children, so each step
    /// strictly decreases the actor id. The walk is therefore finite and cannot
    /// loop; it ends at a `restart`/`stop`, or at a root/unsupervised parent,
    /// which is the `L0362` program-terminating outcome.
    fn supervise_failure(&mut self, failed: usize) -> Result<(), RuntimeError> {
        let mut current = failed;
        loop {
            let Some(policy) = self.actor_instances[current].policy else {
                // Reached an unsupervised actor. If this is the actor that
                // originally failed, its `err` is just a value — nothing to do.
                // If we arrived here by escalation, the failure has run out of
                // supervisors and terminates the program.
                if current == failed {
                    return Ok(());
                }
                return Err(self.escalation_reached_root(failed, Some(current)));
            };
            match policy {
                SupervisionPolicy::Restart => {
                    return self.apply_or_defer(current, SupervisionAction::Restart);
                }
                SupervisionPolicy::Stop => {
                    return self.apply_or_defer(current, SupervisionAction::Stop);
                }
                SupervisionPolicy::Escalate => {
                    // The escalating actor itself stops; its supervisor now bears
                    // the failure and applies its own policy.
                    self.apply_or_defer(current, SupervisionAction::Stop)?;
                    match self.actor_instances[current].supervisor {
                        Some(supervisor) => current = supervisor,
                        None => return Err(self.escalation_reached_root(failed, None)),
                    }
                }
            }
        }
    }

    /// Apply a supervisory action, or defer it to the end of the target's current
    /// turn if it is **busy**.
    ///
    /// This upholds run-to-completion, and it is load-bearing rather than a nicety.
    /// The common escalation shape is a child failing while its supervisor is
    /// mid-turn, blocked in `await ask child...`. Restarting that supervisor
    /// immediately would re-run its `init` underneath its own running handler —
    /// and because a turn holds the actor's `state` in its environment and writes
    /// it back on completion, the outer turn would then clobber the fresh state
    /// with the stale copy it had taken, silently resurrecting the failed child's
    /// handle. A turn always finishes; the supervisory action lands cleanly after
    /// it, on state nobody else is holding.
    ///
    /// Deferral is deterministic and unambiguous: an actor's action is a function
    /// of its own single `policy`, so repeated failures within one turn agree on
    /// the action and the first record stands.
    fn apply_or_defer(&mut self, id: usize, action: SupervisionAction) -> Result<(), RuntimeError> {
        if self.busy_actors.contains(&id) {
            self.pending_supervision.entry(id).or_insert(action);
            return Ok(());
        }
        self.apply_supervision(id, action)
    }

    /// Carry out a supervisory action on an actor that is not mid-turn.
    fn apply_supervision(
        &mut self,
        id: usize,
        action: SupervisionAction,
    ) -> Result<(), RuntimeError> {
        match action {
            SupervisionAction::Stop => {
                self.stop_actor(id);
                Ok(())
            }
            SupervisionAction::Restart => self.restart_actor(id),
        }
    }

    /// The terminal outcome of an escalation that ran out of supervisors: the
    /// failure reached a root actor (spawned from `main`, so it has no supervisor)
    /// or an unsupervised parent that declared no policy. There is nobody left to
    /// handle it, so the program stops with a deterministic diagnostic rather than
    /// discarding the failure.
    fn escalation_reached_root(&self, failed: usize, unsupervised: Option<usize>) -> RuntimeError {
        let origin = &self.actor_instances[failed].actor_name;
        let detail = match unsupervised {
            Some(id) => format!(
                "actor `{}`, which declares no `supervise` policy",
                self.actor_instances[id].actor_name
            ),
            None => "the root (`main`), which has no supervisor".to_string(),
        };
        RuntimeError::new(
            "L0362",
            format!(
                "actor failure escalated with nowhere left to go: a fallible handler of actor `{origin}` replied `err` under `supervise escalate`, and the escalation reached {detail}"
            ),
        )
    }

    /// `supervise stop`: terminate an actor. It runs no further turns, so every
    /// message queued for it is purged — a `tell` is dropped, and an `ask` has its
    /// reply slot marked [`ReplySlot::Unavailable`] so the asker gets a
    /// deterministic `L0359` instead of waiting forever for a turn that will never
    /// run. Stopping is idempotent.
    ///
    /// The purge is the second way a message leaves the queue, so it releases the
    /// actor's [`ActorInstance::pending`] occupancy too — every message charged to
    /// this actor is being dropped here, so the counter goes to zero. Leaving it
    /// stale would leave the scheduler believing a mailbox is full that holds
    /// nothing.
    fn stop_actor(&mut self, id: usize) {
        if self.actor_instances[id].stopped {
            return;
        }
        self.actor_instances[id].stopped = true;
        // Purge in queue order; the retained messages keep their relative order,
        // so the surviving actors' FIFO is untouched.
        let mut retained = VecDeque::with_capacity(self.actor_mailbox.len());
        let mut purged = 0usize;
        for message in std::mem::take(&mut self.actor_mailbox) {
            if message.actor_id != id {
                retained.push_back(message);
                continue;
            }
            purged += 1;
            if let Some(slot) = message.reply_slot {
                self.actor_reply_slots[slot] = ReplySlot::Unavailable;
            }
        }
        self.actor_mailbox = retained;
        let instance = &mut self.actor_instances[id];
        instance.pending = instance.pending.saturating_sub(purged);
        // Every message charged to this actor was just purged, so the occupancy
        // must land exactly on zero. A non-zero remainder means an enqueue or a
        // dequeue somewhere skipped the counter — the accounting is the whole
        // basis of back-pressure, so assert it rather than let it drift.
        debug_assert_eq!(
            instance.pending, 0,
            "mailbox occupancy for actor `{}` disagreed with the queue at stop",
            instance.actor_name
        );
    }

    /// `supervise restart`: give the actor a fresh start. Its `state` is discarded
    /// and zero-initialized, then its `init` re-runs with the arguments the
    /// original `spawn` supplied, so a restarted actor is indistinguishable from a
    /// newly spawned one.
    ///
    /// Deliberately unchanged by a restart:
    /// - **The handle.** `Actor<T>` keeps the same id, so every holder stays valid
    ///   and addresses the restarted actor — that is the point of restarting
    ///   rather than respawning.
    /// - **The mailbox.** Messages already queued for the actor are preserved and
    ///   are delivered, in order, to the restarted actor. The message that failed
    ///   is not among them: it was consumed by the turn that returned `err`, which
    ///   is precisely why a restart cannot loop on a poison message. Its
    ///   [`ActorInstance::pending`] occupancy is preserved with it: those messages
    ///   still hold memory and will still be delivered, so they must still count
    ///   against the capacity (unlike a `stop`, which drops them).
    /// - **Supervision links.** Its supervisor and policy carry over, so a
    ///   restarted actor is still supervised.
    ///
    /// A restart cannot loop on a poison message (the failing message is consumed
    /// by the turn that failed, never replayed), but it *could* loop on a failing
    /// `init` — one that spawns an escalating child and drives it to fail during
    /// construction, so that restarting re-fails immediately. That is a broken
    /// actor rather than a supervisable failure, and it is caught here as a
    /// deterministic `L0363` rather than spinning forever.
    fn restart_actor(&mut self, id: usize) -> Result<(), RuntimeError> {
        let actor_name = self.actor_instances[id].actor_name.clone();
        if !self.restarting.insert(id) {
            return Err(RuntimeError::new(
                "L0363",
                format!(
                    "restart loop: actor `{actor_name}` failed again while its `init` was re-running for a `supervise restart`, so restarting it can never produce a working actor — the failure is in `init` itself, not in a message it handled"
                ),
            ));
        }
        let restarted = self.restart_actor_inner(id, &actor_name);
        self.restarting.remove(&id);
        restarted
    }

    /// The body of a restart (see [`Runtime::restart_actor`], which wraps this
    /// with the re-entrancy guard).
    fn restart_actor_inner(&mut self, id: usize, actor_name: &str) -> Result<(), RuntimeError> {
        let decl: &'a ActorDecl = match self.actors.get(actor_name).copied() {
            Some(decl) => decl,
            None => {
                return Err(RuntimeError::new(
                    "L0401",
                    format!("actor instance references unknown actor `{actor_name}`"),
                ));
            }
        };
        self.actor_instances[id].state = decl
            .state
            .iter()
            .map(|field| (field.name.clone(), self.zero_value(&field.ty)))
            .collect();
        let Some(init) = &decl.init else {
            return Ok(());
        };
        let param_names: Vec<String> = init.params.iter().map(|param| param.name.clone()).collect();
        let args = self.actor_instances[id].spawn_args.clone();
        self.run_actor_turn(id, &decl.state, &param_names, &init.body, args)?;
        Ok(())
    }

    /// Run one actor turn and return its value (the handler/`init` block's final
    /// value — the reply value for an `ask` handler, `void` otherwise). Marks the
    /// actor busy for the whole turn (including nested `await`s) so no second turn
    /// for the same actor can start, upholding the non-reentrant run-to-completion
    /// guarantee; the busy mark is cleared on every exit path.
    ///
    /// The turn is also the scope of `current_actor`, which a `spawn` inside the
    /// body reads to record itself as the new actor's supervisor. The previous
    /// value is saved and restored rather than cleared, so an `await` that drives
    /// another actor's turn from inside this one leaves this frame's notion of
    /// "the running actor" intact when it returns.
    fn run_actor_turn(
        &mut self,
        id: usize,
        state_fields: &[StructField],
        param_names: &[String],
        body: &'a [Stmt],
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        self.busy_actors.insert(id);
        let outer_actor = self.current_actor.replace(id);
        let result = self.run_actor_turn_inner(id, state_fields, param_names, body, args);
        self.current_actor = outer_actor;
        self.busy_actors.remove(&id);
        // The turn is over and its state is written back, so a supervisory action
        // deferred while this actor was busy can now be applied safely. Skipped
        // when the turn itself failed: that error is already terminating the
        // program, and supervision has nothing to add to it.
        let value = result?;
        if let Some(action) = self.pending_supervision.remove(&id) {
            self.apply_supervision(id, action)?;
        }
        Ok(value)
    }

    /// The body of a single actor turn (see [`Runtime::run_actor_turn`], which
    /// wraps this with the busy guard): take the instance's state out, bind it
    /// (plus the handler/`init` parameters) into a fresh environment, evaluate the
    /// body to completion, read the (possibly mutated) state fields back into the
    /// instance, and return the block's value. Taking the state out for the turn is
    /// safe because an actor runs at most one turn at a time (enforced by the busy
    /// set), so nothing else can observe the instance mid-turn.
    fn run_actor_turn_inner(
        &mut self,
        id: usize,
        state_fields: &[StructField],
        param_names: &[String],
        body: &'a [Stmt],
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if param_names.len() != args.len() {
            return Err(RuntimeError::new(
                "L0402",
                format!(
                    "actor handler expects {} argument(s) but got {}",
                    param_names.len(),
                    args.len()
                ),
            ));
        }
        let current = std::mem::take(&mut self.actor_instances[id].state);
        let mut env = Env::default();
        for (name, value) in current {
            env.define(name, value);
        }
        // Parameters shadow a state field of the same name for the turn's scope.
        for (name, value) in param_names.iter().zip(args) {
            env.define(name.clone(), value);
        }

        // An actor turn runs in its own frame over its own `Env` (see
        // `raw_pointer.rs`); `?` below would skip the exit, so unpack explicitly.
        let outer_frame = self.raw_ptrs.enter_frame();
        let turn = self.eval_block(body, &mut env);
        self.raw_ptrs.exit_frame(outer_frame);
        let control = turn?;
        let reply = match control {
            Control::Value(value) | Control::Return(value) => value,
            Control::Break | Control::Continue => {
                return Err(RuntimeError::new(
                    "L0410",
                    "loop control escaped an actor handler body",
                ));
            }
        };

        // Read the state fields back out of the environment. Every field was
        // bound at turn start, so each read resolves.
        let mut new_state = Vec::with_capacity(state_fields.len());
        for field in state_fields {
            new_state.push((field.name.clone(), env.get(&field.name)?));
        }
        self.actor_instances[id].state = new_state;
        Ok(reply)
    }

    /// A type-appropriate zero value for an actor `state` field, used to
    /// initialize the field before `init` runs. Scalars get their numeric/empty
    /// zero, `string`/`char`/`byte` their empty/NUL/`0`, growable/`array`
    /// collections an empty container, `map` an empty map, `option<T>` `none`,
    /// and a struct a recursively zero-initialized value. Any other type (an
    /// `enum`, `result`, reference/pointer handle, `Actor<T>`, or function value)
    /// has no natural zero, so it defaults to `void`; a well-formed `init` sets
    /// such a field before any handler reads it.
    fn zero_value(&self, ty: &TypeRef) -> Value {
        match ty.name.as_str() {
            "i64" => return Value::I64(0),
            "f64" => return Value::F64(0.0),
            "f32" => return Value::F32(0.0),
            "bool" => return Value::Bool(false),
            "string" => return Value::String(String::new().into()),
            "char" => return Value::Char('\0'),
            "byte" => return Value::Byte(0),
            "i8" => return Value::int(0, IntKind::I8),
            "i16" => return Value::int(0, IntKind::I16),
            "i32" => return Value::int(0, IntKind::I32),
            "u8" => return Value::int(0, IntKind::U8),
            "u16" => return Value::int(0, IntKind::U16),
            "u32" => return Value::int(0, IntKind::U32),
            "u64" => return Value::int(0, IntKind::U64),
            "isize" => return Value::int(0, IntKind::Isize),
            "usize" => return Value::int(0, IntKind::Usize),
            _ => {}
        }
        if ty.list_element().is_some() || ty.array_element().is_some() {
            return Value::Array(Vec::new().into());
        }
        if ty.map_args().is_some() {
            return Value::Map(Box::default());
        }
        if ty.option_element().is_some() {
            return option_value(None);
        }
        // A non-generic user struct: build a value with each field zeroed.
        if let Some(decl) = self
            .program
            .structs
            .iter()
            .find(|decl| decl.name == ty.name && decl.type_params.is_empty())
        {
            let fields = decl
                .fields
                .iter()
                .map(|field| (field.name.clone(), self.zero_value(&field.ty)))
                .collect();
            return Value::Struct(Box::new(StructValue {
                name: decl.name.clone(),
                fields,
            }));
        }
        Value::Void
    }
}
