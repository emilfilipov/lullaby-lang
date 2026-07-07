# Modules, Imports, and Visibility Design

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

This note designs multi-file Lullaby programs: how a file pulls in declarations
from another file (`import`), and how a file controls what it exposes (`pub`).
The design keeps the token-minimal, indentation-only aesthetic and — crucially —
is a **frontend-only** change: imports are resolved and all files are merged
into a single `Program` before semantic analysis, so the type checker, all three
backends (AST, IR, bytecode), and the optimizer are unchanged.

## Files are modules

Each `.lby` file is a module. A module's top-level declarations (`fn`, `struct`,
`enum`, `alias`) are **file-private by default** and become importable only when
marked `pub`:

```lby
# geometry.lby
pub struct Point
    x i64
    y i64

pub fn dot a Point b Point -> i64
    a.x * b.x + a.y * b.y

fn helper n i64 -> i64      # private: not importable
    n * n
```

## Import

A file imports another module by name at the top of the file, before any
declaration:

```lby
# main.lby
import geometry

fn main -> i64
    let p Point = Point(3, 4)
    dot(p, p)
```

Rules:
- `import NAME` loads `NAME.lby` from the **same directory** as the importing
  file. (A dotted `import pkg.sub` maps to `pkg/sub.lby` — deferred; first
  increment resolves flat single-name imports in the entry file's directory.)
- An import makes the imported module's `pub` items available **unqualified** in
  the importing file. This is the most token-minimal option and avoids a new
  qualifier token (a qualified `Module.item` would collide with field/method
  access, and `Module::item` would add a new token; flat import keeps the
  surface unchanged).
- A name that is `pub` in two different imported modules, or that collides with
  a local declaration, is a compile-time error (`L0391`) — flat namespacing
  trades brevity for a no-shadowing rule.
- Imports are transitive for **loading** (importing `a` which imports `b` loads
  `b`), but names are **not** re-exported: `main` importing `geometry` does not
  automatically see what `geometry` imported. Each file sees only its own
  imports plus its own declarations.

## Visibility

- `pub` may prefix a top-level `fn`, `struct`, `enum`, or `alias`. Only `pub`
  items are importable; unmarked items are private to their file.
- Struct fields and enum variants inherit their type's visibility in the first
  increment (a `pub struct` exposes all its fields). Per-field visibility is
  deferred.
- Using a non-`pub` item from another module is a compile-time error
  (`L0392`) — reported at resolution time.

## Loading pipeline (frontend only)

A new module-loader stage runs before semantic analysis:

1. Start from the entry file. Lex+parse it, collect its `import` names.
2. For each import, resolve `NAME.lby` in the entry file's directory, lex+parse
   it, and recurse over its imports. Detect and reject import cycles (`L0393`)
   and missing module files (a resolver diagnostic).
3. Merge every loaded module's declarations into one `Program`, tagging each
   declaration with its source module and visibility, and record per-file import
   sets so name resolution can enforce visibility and the no-shadowing rule.
4. Hand the merged `Program` to the existing semantic analyzer and backends
   unchanged. Because everything is one `Program`, execution, optimization, and
   bytecode artifacts need no changes.

The parser already reports `L0211` for `import`/`module` as planned syntax;
that rejection is removed for `import` and `pub` exactly as `struct`/`enum`/
`match` were un-rejected.

## AST and CLI

- Parser: accept a leading `import NAME` line list, and an optional `pub`
  modifier on top-level declarations. Add `imports: Vec<String>` and a `pub`
  flag (or a `visibility` field) to the relevant AST nodes (serde-defaulted so
  single-file artifacts stay compatible).
- CLI: `lullaby run/check/compile <entry.lby>` invokes the loader to build the
  merged program. Single-file programs with no imports behave exactly as today.

## Diagnostics

- `L0391` — name collision across imports / with a local declaration.
- `L0392` — use of a non-`pub` item from another module.
- `L0393` — import cycle.
- Plus a resource-style diagnostic for a missing module file.

## Scope and sequencing

First increment: flat single-name imports in the entry directory, `pub`
visibility on top-level declarations, unqualified merged namespace with a
no-shadowing rule, cycle/missing detection. Deferred: dotted module paths and a
package root, qualified access, per-field visibility, and re-exports — these
layer on once a package manifest exists (see the packaging ticket).

## Why these choices

- **File = module, `pub` to export**: the simplest mental model; matches how
  most systems languages scope visibility.
- **Frontend-only merge**: keeps the three backends and the optimizer untouched,
  so multi-file support carries zero backend risk and stays at parity for free.
- **Flat unqualified import**: no new tokens, no collision with `.` access;
  the no-shadowing rule keeps it unambiguous.
