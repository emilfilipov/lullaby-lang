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

Construction is qualified by the enum name to stay unambiguous and readable:

```lby
let c Color = Color.Red
let s Shape = Shape.Circle(2.0)
let r Shape = Shape.Rect(3.0, 4.0)
```

- `Enum.Variant` constructs a unit variant.
- `Enum.Variant(args)` constructs a variant with payloads; argument count and
  per-payload types are checked against the declaration.

Qualification (`Enum.` prefix) is chosen over bare `Variant` so construction
never collides with function calls, struct construction, or variables, and so a
reader always sees the type at the construction site.

## Pattern matching

`match` selects on an enum value with an indentation-only arm list. Each arm is
a pattern, `->`, and either an inline expression or an indented block whose last
expression is the arm's value:

```lby
fn area s Shape -> f64
    match s
        Shape.Circle(r) -> 3.14159 * r * r
        Shape.Rect(w, h) -> w * h
        Shape.Empty -> 0.0
```

- Payload bindings (`r`, `w`, `h`) introduce locals scoped to that arm, typed by
  the variant's declared payload types.
- Like `if`/`else` and `try`/`catch`, a `match` is an expression: when every arm
  yields the same type it produces that value (so it can be a function's final
  expression); otherwise it is a void statement.
- Matching must be **exhaustive** over the enum's variants. A wildcard arm
  `_ -> expr` covers the rest; a non-exhaustive match without `_` is a
  compile-time error.
- A duplicate or unknown variant arm is a compile-time error.

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
