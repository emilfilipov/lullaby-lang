# Nous Lang (nlang) Syntax Design Guide

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

## Overview

This guide covers the foundational syntax design for **Nous Lang** (nlang), a minimalistic compiled systems programming language optimized for LLM generation and understanding.

---

## Core Philosophy

### Design Principles

1. **Minimalism**: Eliminate syntactic noise - no curly braces, semicolons, parentheses, or other punctuation clutter
2. **Token Efficiency**: Minimize token count per semantic operation (aim for 40-60% reduction vs traditional languages)
3. **LLM-Friendly**: Structures that tiny models (<1B parameters) can parse and understand with high reliability
4. **Declarative + Implicit**: Combine explicit declarations with intelligent inference based on context and naming

### Key Differentiators from Existing Languages

| Traditional Language | Nous Lang Approach |
|---------------------|--------------------|
| Curly braces for blocks (Java, C++, Python) | **Keyword-based block delimiters** (`begin`, `end`) or **indentation contexts** |
| Semicolons to end statements | **No statement terminators** - implicit line boundaries |
| Parentheses for grouping/function calls | **Prefix notation** with explicit operators or **implicit precedence** via naming |
| Type annotations everywhere | **Type inference from usage + selective annotation** (`type` keyword only when needed) |
| Verbose variable declarations | **Concise binding** - name alone implies declaration and scope |

---

## Syntax Elements

### 1. Variable Declaration & Binding

**Traditional**: `int x = 5;` (6 tokens: int, x, =, 5, ; )
**Nous Lang**: `x is 5` or `x = 5` (3-4 tokens)
**Advanced**: `let x := 5` with optional type hint (`let x [int] := 5`)

#### Declaration Styles
```nlang
# Style 1: Implicit binding (name implies declaration)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
x is 5
names are ["alice", "bob"]

# Style 2: Explicit with type annotation

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
x [int] = 5
data [array[string]] = ["hello", "world"]

# Style 3: Pattern matching declaration

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
when x > 0 then x is positive
```

#### Variable Categories
- **local**: Function-scoped, automatically cleaned on exit (`var`, `let`)
- **static/global**: Module-scoped, persistent across calls (`static`, `global`)
- **mutable**: Can be reassigned after declaration (`mut`)
- **immutable**: Cannot be reassigned (default without `mut` keyword)

### 2. Control Structures

#### If Statements (No Braces)
```nlang
# Traditional: if x > 0 then return x

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
if x > 0 then return x

# Multiple conditions chained

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
if x > 0 then a is positive
else if x < 0 then a is negative
else a is zero

# Simplified single branch (no else needed for common cases)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
when x > 10 then log "large value"
```

#### Loops (Context-Aware)
```nlang
# For loop with explicit range

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
for i from 0 to 9 do print(i)

# While loop with condition check

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
while data.length > 0 do process(data.pop())

# Range-based iteration

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
foreach item in items do handle(item)

# Stream processing (lazy evaluation optimized for LLM understanding)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
stream line in file do parse(line)
```

#### Functions/Methods
```nlang
# Function definition (concise parameter notation)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
fn add a, b
    a + b
# Becomes:

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
add = lambda a, b -> a + b

# Method on type

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
class Point [x: float, y: float]
    method distance(other) -> float
# method body ends by dedent

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
```

### 3. Data Structures

#### Arrays & Collections
```nlang
# Simple array initialization

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
numbers = [1, 2, 3, 4, 5]

# Typed array (implicit from context when all elements same type)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
matrix = [[1, 2], [3, 4]]  # Inferred as array[array[int]]

# Dictionary/Map

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
config = map name: "app", version: "1.0"

# Set (unordered collection of unique items)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
tags = set "alpha", "beta", "gamma"  # Duplicates auto-removed

# Stream (lazy evaluation)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
stream nums from 0 to 100
```

#### Memory Layout (Systems Programming Focus)
```nlang
# Direct memory access (pointer syntax simplified)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
ptr x [int] points_to(0x1234)
deref(ptr_x)  # Returns value at address
addr(x)        # Returns address of variable

# Buffer/Region allocation

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
region buf: size=1024, align=8
init(buf, data)
```

### 4. Type System Integration in Syntax

```nlang
# Primitive types (no need for explicit type in most cases - inferred from usage)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
count = 5          # int inferred
name = "hello"     # string inferred
active = true      # bool inferred
ratio = 3.14       # float inferred

# Explicit type annotation (when inference ambiguous or optional types needed)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
temperature [float] = 98.6
users [array[User]] = [...]

# Custom/Record types

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
type Person
    name: string
    age: int
    height: float
end_type

p = Person(name: "Alice", age: 30, height: 5.7)
```

### 5. Operators (Minimal & Expressive)

#### Arithmetic
- `+` addition
- `-` subtraction
- `*` multiplication
- `/` division
- `%` modulo

#### Comparison
- `==`, `!=`, `<`, `>`, `<=`, `>=`

#### Logical
- `and`, `or`, `not` (natural English keywords instead of &&, ||, !)

#### Special Operators
```nlang
# Ternary conditional

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
result = x > 0 when true else false

# Function application (prefix notation)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
map(f, list)
reduce(add, list, initial=0)

# Arithmetic assignment (combine operation and assignment in one token!)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
x += 5     # Instead of: temp = x + 5; x = temp

# Null/Empty checks

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
is_null(x)    # Returns true if x is null or undefined
is_empty(arr) # Returns true if array/set is empty
```

---

## Syntax Rules & Conventions

### Line-Based Parsing (No Statement Termination)

Each physical line in nlang source code represents a complete logical statement. No semicolons needed - implicit newline acts as statement delimiter.

```nlang
x = 5          # Single assignment
y = x * 2      # Multiple operations chained on same line
z = y + 10     # Continues chain
print(z)       # New logical statement (new line)
```

### Expression Continuation

Multi-line expressions use continuation indicator:
```nlang
total = sum(items)
         + calculate_tax(total)
         * apply_discount(0.9)
```

Or implicit via context awareness (LLM can infer expression boundaries):
```nlang
result = complex_calculation(
    a, b, c
)
# LLM understands this spans one logical unit based on parentheses or indentation

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
```

### Naming Conventions for Clarity

```nlang
# camelCase for variables and functions

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
myVariable, calculateTotal

# PascalCase for types and classes

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
MyType, MyClass

# snake_case for module names and constants (when using UPPER_CASE convention)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
user_data, MAX_BUFFER_SIZE

# descriptive names encouraged (no abbreviations preferred for LLM readability)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
```

### Comments

```nlang
# Single-line comment (hash symbol only - no other punctuation needed)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
# This is a comment

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
# x = 5 + y * z     # Inline comment after code

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

/*
 * Multi-line comment block
 */
```

---

## Token Optimization Features

### Compact Keyword Set
Only essential keywords (~12 core words):
- `is`, `are` (declaration)
- `let`, `var` (variable scoping)
- `func`, `method` (function definition)
- `if`, `else`, `when`, `then` (conditionals)
- `for`, `while`, `foreach`, `stream` (iteration)
- `type`, `end_type` (type definitions)
- `class` (object definitions)
- `import`, `export` (module system)
- `return`, `yield` (function output)

### Zero-Bloat Operators
Operators use single-character or natural English:
- No `++`, `--` -> Use `inc()`, `dec()` functions
- No `&&`, `||`, `!` -> Use `and`, `or`, `not`
- No ternary operator -> Use `x when condition else value`

### Type Inference Priority
Types inferred from:
1. Literal values (`5` -> int, `"text"` -> string)
2. Usage context (parameter types propagate to variables)
3. Return statements in functions

Explicit annotations only when:
- Ambiguity exists (multiple possible types for a value)
- Optional/Union types needed
- Memory layout constraints require specific types

---

## LLM Optimization Strategies

### Structured Token Patterns
Each statement follows predictable token sequence, enabling tiny models to learn patterns quickly:
```nlang
# Pattern 1: Assignment

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
[NAME] [operator] [VALUE]

# Pattern 2: Conditional

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
IF [CONDITION] THEN [ACTION] [ELSE...] END_IF

# Pattern 3: Loop

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
FOR [ITERATOR] FROM [START] TO [END] DO [BODY] END_FOR

# Pattern 4: Function Call

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
[FUNCTION_NAME]([ARGUMENTS])
```

### Semantic Token Grouping
Related operations grouped into logical phrases, reducing token fragmentation:
```nlang
# Instead of separate tokens for each operation:

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
x = 5 * 2 + 10
# Becomes semantically grouped by the LLM as one calculation unit

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
```

### Contextual Cues for Understanding
- Variable names provide type hints through naming patterns (`count*` -> int, `name*` -> string)
- Operator precedence inferred from operator symbols and operand order
- Scope boundaries clear from keyword declarations (`let`, `var`, `func`)

---

## Example Code Samples

### Basic Program with Indentation Scope
```nlang
# Hello World in Nous Lang

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
fn main
    message is "Hello, world!"
    print(message)
run(main)
```

### Data Processing
```nlang
process_data = func(data: array[float]) -> float
    sum is 0
    for value in data do
        sum = sum + value
    end_for

    count is len(data)
    average = when count > 0 then sum / count else 0

    return average

# Usage

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
numbers = [1.5, 2.3, 3.7, 4.2]
result = process_data(numbers)
print(result)
```

### Systems Programming Example (OS Kernel Task)
```nlang
class process_manager
    processes: array[Process]
    current_process_id: int

    init() -> self
        processes is []
        current_process_id is 0

        # Initialize memory regions
        region kernel_code: size=4096, align=4096
        region kernel_data: size=8192, align=8192

        return self

    create(process_name: string) -> int
        pid = current_process_id + 1
        if pid > max_pid then
            error("Process table full")
        end_if

        # Create process entry
        proc_data [struct] = Process(
            id: pid,
            name: process_name,
            state: ProcessState.New
        )

        processes.append(proc_data)
        current_process_id is pid + 1

        return pid

# Usage

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
km = process_manager.init()
pid = km.create("shell")
```

---

## Next Steps for Language Design

After establishing the syntax foundation, subsequent documents should cover:

1. **Memory Management System**: Heap allocator with garbage collection or manual memory tracking
2. **Compilation Architecture**: Lexer/parser design optimized for LLM-assisted generation
3. **Type System Implementation**: Type checker and inference engine
4. **Control Flow Translation**: AST construction and optimization
5. **Runtime Environment**: VM execution model and syscall interface

---

*This guide provides the complete syntax specification for Nous Lang. Each subsequent document builds upon these foundational concepts while addressing specific subsystem requirements.*

**Version**: 1.0
**Last Updated**: June 24, 2026
**Author**: Nous Lang Design Team
