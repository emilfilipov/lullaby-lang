# First-Class Functions and Closures Design

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

This note designs first-class function values for Lullaby. The goal is to let
functions be passed to and returned from other functions (higher-order
functions like a `map`/`filter`/callback style) while staying token-minimal and
consistent with the value-semantics interpreter that runs on all three backends.

## Increments

1. **First-class function values (this increment).** A top-level function can be
   referred to by name in value position, stored in a `let`, passed as an
   argument, returned, and called through a variable. No environment capture.
2. **Closures with capture (follow-up).** Nested function literals that capture
   surrounding locals by value. Deferred to keep the first increment small and
   the value model simple.

## Function types

A function type is spelled `fn(T1, T2) -> R` (zero or more parameter types, an
arrow, a return type). Examples: `fn(i64) -> i64`, `fn(i64, i64) -> bool`,
`fn() -> void`. This reuses the existing generic-type parsing style (a new
recognized spelling in `expect_type`); the canonical string form is
`fn(i64, i64) -> bool`.

## Referring to a function as a value

A bare function name in value position evaluates to a function value:

```lby
fn inc x i64 -> i64
    x + 1

fn apply f fn(i64) -> i64 v i64 -> i64
    f(v)

fn main -> i64
    let g fn(i64) -> i64 = inc
    apply(inc, 10) + g(5)
```

- The parser already produces `Variable(name)` for a bare name; semantics
  resolves it to a function value when `name` is a declared function and is not
  a local, giving it type `fn(params) -> ret`.
- Runtime value: `Value::Func(name)` — a handle to a top-level function by name
  (no captured environment in this increment).

## Calling a function value

Calling uses the existing call spelling. `f(args)` where `f` is a local of
function type is dispatched at runtime by resolving the referenced function and
invoking it with the arguments. In the parser this is already `Call { name: f,
args }`; semantics checks `f`'s type is `fn(A...) -> R`, checks argument arity
and types against the function-type parameters, and yields `R`. The runtime and
IR interpreters, when a call name resolves to a local holding a `Value::Func`,
invoke the referenced function (rather than looking `f` up in the function
table).

UFCS method calls still desugar `x.m(a)` to `m(x, a)`; that is unaffected.

## Type checking

- A function name used as a value has type `fn(param types) -> return type`
  built from its declaration.
- `apply(f fn(i64) -> i64, ...)` checks the argument is a function value with a
  matching signature (string-equal canonical `fn(...)` type).
- Calling a function-typed local checks arity and parameter types and produces
  the declared return type.
- Passing a function with the wrong signature, or calling a non-function local,
  reports a new semantic code (`L0390`).

## Backends and parity

- `Value::Func(name)` is a shared runtime value; construction (name-as-value)
  and call-through-value run identically on the AST interpreter, IR interpreter,
  and bytecode VM. A `run_first_class_fn.lby` parity fixture exercises passing a
  function to a higher-order function and calling it through a local, verified
  on all five backend variants.

## Why these choices

- **`fn(T) -> R` type spelling**: mirrors the declaration arrow the reader
  already knows; no new keywords.
- **Name-as-value + normal call spelling**: zero new expression syntax; a
  function value is produced and consumed with shapes the language already has.
- **Deferring capture**: true closures need an environment representation and
  capture semantics; first-class top-level functions deliver most of the
  higher-order-function value immediately and keep the value model trivial.
