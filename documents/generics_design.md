# User-Defined Generics Design

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

Lullaby already has built-in generic types (`array<T>`, `list<T>`, `map<K, V>`,
`option<T>`, `result<T, E>`). This note adds **user-defined generics** so a
programmer can write their own type-parameterized *functions*. Generic user
*types* (`struct Box<T>`) are a follow-up; this increment is functions only.

Key property: generics are **erased at runtime**. A type parameter `T` is just
`Value` at run time, so the AST interpreter, IR interpreter, and bytecode VM
need no monomorphization and no new value representation — this is a
type-checker feature. The only backend-adjacent work is that the IR lowerer,
which re-derives expression types, must run the same call-site inference as
semantics (exactly as it already does for `none`/`ok`/`err` and `list_new`).

## Syntax

A function declares type parameters in angle brackets after its name, then uses
them as ordinary types in parameters and the return type:

```lby
fn identity<T> x T -> T
    x

fn first<T> xs list<T> -> option<T>
    match len(xs)
        0 -> none
        _ -> some(get(xs, 0))

fn pair_max<T> a T b T -> T
    a           # (illustrative; real body would compare)
```

- `<T>` or `<T, U>` — one or more comma-separated type-parameter names.
- A type-parameter name is in scope as a type only within that function's
  signature and body.
- Duplicate type-parameter names, or a `<T>` that shadows a real type, are
  errors (`L0394`).

## Type variables and inference

- Within a generic function, a type-parameter name (e.g. `T`) is a **type
  variable**: it unifies with whatever concrete type flows into it.
- At a **call site**, the type variables are inferred from the argument types by
  unification, then substituted into the return type:
  - `identity(3)` → unify `T` with `i64` → returns `i64`.
  - `identity("hi")` → `T = string` → returns `string`.
  - `first(nums)` where `nums: list<i64>` → `T = i64` → returns `option<i64>`.
- If two occurrences of the same type variable receive different concrete types
  (`same(1, "x")` for `fn same<T> a T b T`), that is a conflict error (`L0395`).
- If a type variable cannot be inferred from the arguments (it appears only in
  the return type with no argument to pin it), report `L0396`; such functions
  would need explicit type arguments, which are deferred.

Inside the generic function body, a value of type `T` supports only the
operations valid for *every* type (assignment, passing, returning, equality via
`==` where the language already allows same-type equality, and use as an
argument to another generic). Type-specific operations (arithmetic, ordering)
on a bare `T` are rejected until trait bounds exist — this keeps the first
increment sound without a trait system.

## Where inference lives

- **Semantics** (`crates/lullaby_semantics`): when checking a call whose target
  is a generic function, build a substitution by unifying each argument's type
  against the (possibly type-variable-containing) parameter type, then apply it
  to the return type. Report `L0394`/`L0395`/`L0396` as above. Type-check the
  generic body with each `T` treated as an opaque type that only supports
  universal operations.
- **IR lowerer** (`crates/lullaby_ir`): `call_return_type` for a generic
  function must perform the same unify-and-substitute against the lowered
  argument types, so the IR result type matches what semantics assigned. This
  mirrors the existing context-inference threading — no new runtime value.
- **Runtime / bytecode**: unchanged; `T` is `Value`.

## Backends and parity

Because generics are erased, a `run_generics.lby` fixture (calling generic
functions at several concrete types and combining the results into a
deterministic `i64`) runs identically on the AST, IR, bytecode, and optimized
IR/bytecode backends — verified by the auto-discovering parity harness.

## Diagnostics

- `L0394` — invalid type-parameter list (duplicate `<T>`, or shadows a type).
- `L0395` — conflicting inference for a type variable.
- `L0396` — a type variable cannot be inferred from the arguments.

## Scope and sequencing

First increment: generic **functions** with call-site inference, erased runtime,
no bounds. Deferred: generic user **types** (`struct Box<T>`, `enum Tree<T>`),
explicit type arguments (`identity<i64>(3)`), and **trait bounds** (`<T: Ord>`),
which arrive with the trait system and let bare `T` values use bounded
operations. Generic functions are the foundation the trait work builds on.

## Why these choices

- **Erased generics**: the runtime is already `Value`-based, so erasure is free
  and keeps all three backends and the optimizer untouched.
- **Inference from arguments**: matches how `option`/`result`/`list` already
  infer; no explicit type-argument syntax needed for the common case.
- **No bare-`T` arithmetic without bounds**: keeps the first increment sound
  and defers the harder trait-bound machinery to the trait ticket.
