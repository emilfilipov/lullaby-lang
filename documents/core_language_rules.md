# Nous Lang Core Language Rules

This file is the canonical location for global Nous Lang rules that apply across all subsystem documents and implementation work.

## Canonical Source Extension

Nous Lang source files use the `.nl` file extension.

Examples:

```text
main.nl
kernel.nl
memory.nl
driver.nl
allocator.nl
```

The compiler, installer, tests, examples, CLI, diagnostics, generated project templates, and documentation must consistently use `.nl` unless the language specification is intentionally changed.

## Indentation-Only Scope

Nous Lang / nlang uses indentation-only scope.

Curly brace characters are forbidden as block delimiters in:

- functions
- conditionals
- loops
- structs and unions
- regions
- unsafe and unchecked blocks
- classes
- modules
- try and catch error-handling blocks
- every other scoped language construct

Canonical block form:

```nlang
fn add x y
    x + y

fn max a b
    if a > b
        a
    else
        b

fn count_to n
    mut i = 0
    while i < n
        out i
        i = i + 1

struct Point
    x f64
    y f64

region temp size 1024 align 8
    buf = alloc temp
```

Rules:

- indentation is the only block delimiter
- no semicolons terminate statements
- a block begins after a line that introduces scope
- a block ends when indentation returns to the previous level

## Documentation Rule

Do not duplicate this block in subsystem documents. Link to this file instead.
