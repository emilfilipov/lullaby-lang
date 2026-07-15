//! The AST interpreter's actor scheduler (stage 1: `spawn` + `tell`).
//!
//! Actors run on a single-threaded, cooperative, deterministic scheduler. `spawn`
//! constructs an actor (zero-initializing its private `state`, then running its
//! `init`) and returns a typed [`Value::ActorRef`] handle. `tell` enqueues a
//! fire-and-forget message on a global FIFO mailbox. Every outstanding message is
//! drained — run-to-completion, one at a time — by [`Runtime::drain_actors`]
//! before `main` returns, so a `tell` with an observable side effect (e.g.
//! `print`) produces deterministic output identical on every run. Because only
//! one message runs at a time and each actor's `state` is touched only by its own
//! handlers, the state is a single-writer resource with no data races.
//!
//! This is the AST-interpreter runtime; the IR/bytecode backends reject an actor
//! program (`L0355`) and the native/WASM backends cleanly skip it, so stage 1
//! actors run only here.

use lullaby_parser::{ActorDecl, ActorHandler, StructField, TypeRef};

use super::*;

impl<'a> Runtime<'a> {
    /// `spawn NAME(args)`: allocate an actor instance with zero-initialized
    /// state, run its `init` (if any) with `args`, register it on the scheduler,
    /// and return its handle. Semantics has already checked the actor exists and
    /// the argument count/types match `init`, so the runtime guards here are
    /// defensive.
    pub(crate) fn spawn_actor(
        &mut self,
        actor_name: &str,
        args: Vec<Value>,
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

    /// `tell TARGET.HANDLER(args)`: enqueue a fire-and-forget message on the
    /// target actor's mailbox and return `void`. The message is processed later,
    /// during the graceful drain before `main` returns.
    pub(crate) fn tell_actor(
        &mut self,
        target: Value,
        handler: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let Value::ActorRef(actor_id) = target else {
            return Err(RuntimeError::new(
                "L0401",
                format!("`tell` target is not an actor handle: `{target}`"),
            ));
        };
        if actor_id >= self.actor_instances.len() {
            return Err(RuntimeError::new(
                "L0401",
                format!("`tell` to unknown actor handle `{actor_id}`"),
            ));
        }
        self.actor_mailbox.push_back(ActorMessage {
            actor_id,
            handler: handler.to_string(),
            args,
        });
        Ok(Value::Void)
    }

    /// Drain the actor mailbox: run every outstanding message to completion, one
    /// at a time, in FIFO order. A handler may itself `tell` (or `spawn`) during
    /// its turn, appending to the queue; the loop continues until the queue is
    /// empty. Single-threaded and deterministic.
    pub(crate) fn drain_actors(&mut self) -> Result<(), RuntimeError> {
        while let Some(message) = self.actor_mailbox.pop_front() {
            self.dispatch_message(message)?;
        }
        Ok(())
    }

    /// Run one mailbox message on its target actor: locate the handler on the
    /// actor's declaration and run its body as a turn over the actor's state.
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
        self.run_actor_turn(
            message.actor_id,
            &decl.state,
            &param_names,
            &handler.body,
            message.args,
        )
    }

    /// Run one actor turn: take the instance's state out, bind it (plus the
    /// handler/init parameters) into a fresh environment, evaluate the body to
    /// completion, then read the (possibly mutated) state fields back into the
    /// instance. Taking the state out for the duration of the turn is safe
    /// because the scheduler is single-threaded and processes one message at a
    /// time, so nothing else can observe the instance mid-turn.
    fn run_actor_turn(
        &mut self,
        id: usize,
        state_fields: &[StructField],
        param_names: &[String],
        body: &'a [Stmt],
        args: Vec<Value>,
    ) -> Result<(), RuntimeError> {
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

        let control = self.eval_block(body, &mut env)?;
        if matches!(control, Control::Break | Control::Continue) {
            return Err(RuntimeError::new(
                "L0410",
                "loop control escaped an actor handler body",
            ));
        }

        // Read the state fields back out of the environment. Every field was
        // bound at turn start, so each read resolves.
        let mut new_state = Vec::with_capacity(state_fields.len());
        for field in state_fields {
            new_state.push((field.name.clone(), env.get(&field.name)?));
        }
        self.actor_instances[id].state = new_state;
        Ok(())
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
