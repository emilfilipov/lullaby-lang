# Enum / Tagged-Union and Pattern-Matching Design

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

This note designs Lullaby's enum (tagged-union) types and the `match` form that
consumes them. The guiding constraint is the language aesthetic: token-minimal,
indentation-only, no braces, no semicolons, strongly typed. Enums reuse the
shapes the language already has (the `struct` declaration shape for the variant
list, call spelling for variant payloads) so little new syntax is introduced.

## Declaration

An enum is declared at the top level with `enum` and an indentation-only variant
list. Each variant is a `Name` optionally followed by positional, unnamed
payload types spelled exactly like function-parameter types (but without
names):

```lby
enum Color
    Red
    Green
    Blue

enum Shape
    Circle f64
    Rect f64 f64
    Empty
```

Rules:
- Variants are ordered; an enum type is nominal (identified by its name).
- Duplicate enum names and duplicate variant names (within one enum) are errors.
- Payload types may be any existing type, including structs, arrays, and other
  enums.
- A variant with no payload types is a unit variant.

## Construction

Construction reuses the language's existing spelling exactly like structs do —
a variant with payloads reads like a call, a unit variant reads like a bare
name — and is resolved semantically, so **no new construction syntax is added**:

```lby
let c Color = Red
let s Shape = Circle(2.0)
let r Shape = Rect(3.0, 4.0)
```

- A bare variant name (`Red`, `Empty`) constructs a unit variant. It parses as a
  variable; semantics resolves it to enum construction when the name is a known
  unit variant rather than a local.
- `Variant(args)` constructs a variant with payloads. It parses as a call;
  semantics resolves it to enum construction when the name is a known variant
  (before falling through to function lookup), and checks argument count and
  per-payload types against the declaration.

To keep this resolution unambiguous in the first increment, **variant names are
globally unique across all enums** (a collision is a compile-time error). This
mirrors how struct construction already reuses call spelling and needs zero
parser changes. A qualified `Enum.Variant` spelling is a possible later
convenience once postfix call-on-path parsing exists.

## Pattern matching

`match` selects on an enum value with an indentation-only arm list. Each arm is
a pattern, `->`, and either an inline expression or an indented block whose last
expression is the arm's value:

```lby
fn area s Shape -> f64
    match s
        Circle(r) -> 3.14159 * r * r
        Rect(w, h) -> w * h
        Empty -> 0.0
```

- Payload bindings (`r`, `w`, `h`) introduce locals scoped to that arm, typed by
  the variant's declared payload types.
- An arm body is either an inline expression or an indented **block** whose final
  expression is the arm's value; the block may run several statements (side
  effects, mutations of an enclosing binding, nested `match`es) first.
- Like `if`/`else` and `try`/`catch`, a `match` is an **expression**: when every
  arm yields the same type it produces that value; otherwise it is a void
  statement. Its value is usable in value position — a `let`/assignment
  right-hand side (`let x = match ...`), a `return` value, a function's final
  (tail) expression, and a nested arm's value. Using a `match` where a value is
  required but the arms yield differing types is rejected with `L0440`.
- Matching must be **exhaustive** over the enum's variants, in value position and
  statement position alike. A wildcard arm `_ -> expr` covers the rest; a
  non-exhaustive match without `_` is a compile-time error (`L0384`).
- A duplicate or unknown variant arm is a compile-time error.

Implementation surface: value-position `match` is delivered on the three
interpreter tiers (AST, IR, bytecode VM) at parity. The IR/bytecode lowerer
desugars a value-position `match` into a hoisted result temporary written by the
arms (the same shape as the `?` and inline-conditional desugars), so no new IR or
bytecode node is required; the AST interpreter evaluates the arm blocks against
the live environment so their side effects and mutations are observed. The native
and WASM backends do not yet compile `match`, so a function that uses one is
demoted to the interpreter via the existing gate (a clean skip, never a wrong
result). Nesting a `match` inside a larger expression — for example directly as a
call argument, `f(match ...)` — is not yet parsed (the flat expression parser
cannot span the indented arm block) and is a planned follow-up; the current
positions cover the common `let`/`return`/tail/nested-arm uses.

## Representation and backends

- Runtime value: an enum value carries its enum name, the variant name, and an
  ordered payload vector — `Enum { name, variant, payload: Vec<Value> }`.
- Type: a `TypeRef` whose name is the enum name (nominal), mirroring structs.
- Enums and `match` run identically on the AST, IR, and bytecode backends,
  including under optimization. `match` lowers to the same tag-compare +
  payload-bind sequence on each backend so results stay at parity.

## Why these choices

- **Variants as `Name type...`**: reuses the parameter/field spelling the reader
  already knows; zero new punctuation.
- **`Enum.Variant` construction**: unambiguous, self-documenting, and reuses the
  call parser for payloads.
- **`match ... arm -> value`**: the terse arrow already used elsewhere, with
  indentation for arms — no `case`/`of`/braces.
- **Nominal typing + exhaustiveness**: the simplest strong-typing story, and the
  foundation for the standard `option<T>` / `result<T, E>` core types.

## Scope and sequencing

1. Enum declaration, construction, and type checking (this design's first
   increment).
2. `match` with payload binding, exhaustiveness, and wildcard.
3. `option<T>` / `result<T, E>` as library enums once user generics exist; until
   then, concrete instantiations can be provided as built-in enums.

Deferred: enum methods / associated functions (shared with struct methods),
generics over enums, and guard clauses on match arms.
