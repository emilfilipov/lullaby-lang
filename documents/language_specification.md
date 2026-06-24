# Nous Lang (nlang) - Complete Language Specification

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

## Overview

Nous Lang is a next-generation compiled systems programming language designed with three fundamental goals:

1. **Minimalistic Syntax**: No squiggly brackets, semicolons, or other noise. Clean, readable syntax optimized for human understanding.
2. **Token Efficiency**: Minimal token expenditure during generation by LLMs, enabling smaller models to generate correct code.
3. **LLM-Friendly Design**: Simple enough that even tiny language models (<1B parameters) can understand and write in the language without issues.

**Target Use Case**: Systems programming for operating system development and other low-level applications requiring type safety, performance optimization, and memory efficiency.

## Language Philosophy

Nous Lang rejects traditional design patterns that prioritize compiler convenience over code clarity:

- **No Braces**: Control blocks defined through indentation only (Python-inspired but simpler)
- **No Semicolons**: Line-based statements without terminator requirements
- **Flat Structure**: Single-level control flow instead of deep nesting
- **Type Inference**: Automatic type detection reducing explicit annotations
- **Reference Counting**: Automatic memory management without garbage collection pauses

## Core Language Components

### 1. Syntax Design (See: `nous_lang_syntax_design.md`)

The syntax is intentionally minimal and predictable:

**Variables**:
```nlang
type name = value   // Type prefix required for clarity
name               // Type inferred from context
```

**Functions**:
```nlang
def function_name(params):
    return statement
end_function
```

**Indentation-based Scoping**: Blocks defined by indentation levels only, no braces needed.

### 2. Memory Management (See: `nous_lang_memory_management.md`)

Reference-counted memory with automatic lifetime management:

- No garbage collection pauses
- Deterministic object cleanup
- Region-based organization
- Type-safe operations

**Key Types**:
- `num` - Unified integers and floats
- `str` - Strings
- `bool` - Boolean values
- `array<T>` - Homogeneous arrays
- `map<K,V>` - Key-value mappings
- `ptr<T>`, `ref<T>` - Pointers and references

### 3. Control Structures (See: `nous_lang_control_structures.md`)

Flat control flow without nesting complexity:

**Conditionals**:
```nlang
if condition:
    then_statements
elif other_condition:
    else_statements
else:
    final_statements
end_if
```

**Loops**:
```nlang
for variable from start to end [by step]:
    loop_body

while condition:
    loop_body

loop:
    unconditionally_executed_statements

break  // Exit loop
continue  // Skip remaining statements in iteration
```

### 4. Type System (See: `nous_lang_type_system.md`)

Pragmatic type system for systems programming:

- **Zero-cost abstractions**: Types compile away at runtime
- **Union types**: Single type can hold multiple representations
- **Pattern matching**: Declarative value decomposition
- **Type inference**: Automatic type detection from usage

**Core Types**:
```nlang
type Integer is num  int, uint
type Float is num  float, double
type Bool is bool  false, true
type Text is str  string, char
```

### 5. I/O and Concurrency (See: `nous_lang_input_output.md`)

Efficient systems programming primitives:

**File Operations**:
- Read/write with automatic buffering
- Type-aware file operations
- Memory-mapped access for large files
- Stream-based processing

**Concurrency Patterns**:
- Simple thread/async models
- Automatic synchronization
- I/O multiplexing
- Message queues and shared memory

## Complete Syntax Reference

### Primitives
```nlang
# Boolean values

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
false, true

# Numeric literals

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
integer: 0, -123, +456
float: 3.14159, 2.71828

# String literals

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
"hello world"
'single quotes also supported'
"""multi-line strings"""

# None/null (no pointer dereferencing needed)

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
none  // Represents absence of value
```

### Operators

**Arithmetic**: `+ - * / % ^ //`
**Comparison**: `== != < > <= >= is_none is_defined`
**Logical**: `and or not xor`
**Assignment**: `+= -= *= /= ^= +=` (compound operators)
**Bitwise**: `& | ^ << >> inv(x)`
**Functional**: `map reduce fold select min max avg sum mode`

### Control Flow
```nlang
if condition:
    statements

switch value:
    case pattern1:
        code_block
    case pattern2 when additional_condition:
        alternative_code

end_switch
```

### Functions
```nlang
def function_name(params):
    # Code statements
    return result_value

end_function
# or simply:

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
function_name(params) -> result_type:
    # Code statements
```

### Structs and Objects
```nlang
struct Type
    field1: type1  // Explicit typing
    field2        // Inferred typing
    method(params): return_type:
        code

ptr_type = new Type()   // Pointer creation
ref_ref = ref(ptr_type) // Reference copy

type_instance = Type(field1: value1, field2: value2)  // Direct construction
```

### Collections
```nlang
array[10] = [v1, v2, ..., v10]
map[key] = [key_value_pairs]

len(collection)      // Length check
index(collection, i) // Element access
slice(arr, start, end)  // Range extraction
contains(coll, item)  // Membership test
sort(coll)           // Ordering
filter(coll, pred)   // Selective collection
```

### Memory Operations
```nlang
alloc(size/type)      // Allocate memory
dealloc(ptr)          // Free allocated memory
ref(ptr)             // Create reference copy
ptr(type)            // Create pointer
swap(a, b)           // Exchange values
duplicate(value)     // Deep copy
```

### I/O Operations
```nlang
io.read(path)         // Read entire file
io.readlines(path, max_lines=N)  // Read limited lines
io.open(path, mode)   // Open stream for reading/writing
io.write(path, data)  // Write to file

# Memory-mapped files

Canonical language rules: see [core_language_rules.md](core_language_rules.md).
io.memory_map(path, size)
mm_data = mm_file.data_pointer
```

### Concurrency Primitives
```nlang
thread func = spawn_thread(func, args)
result = wait(thread)

mutex sync = create_mutex()
lock(sync)
    protected_code
end_lock

async def async_func(params):
    result = await operation()
    return result

tasks = []
for param in params:
    task = spawn_task(async_func, param)
    tasks.append(task)

results = await_all(tasks)
```

## Design Principles Summary

### 1. Minimalism
- Remove unnecessary syntax (no braces, no semicolons, no parentheses where not needed)
- Each line contains one clear operation
- Single keywords for common operations instead of verbose alternatives

### 2. Type Safety
- Compile-time type checking prevents errors before runtime
- Automatic inference reduces annotation burden
- Zero-cost abstractions maintain performance

### 3. Memory Efficiency
- Reference counting eliminates GC overhead
- Region-based organization improves cache locality
- Explicit lifetime management ensures determinism

### 4. LLM Optimization
- Predictable structure enables pattern-based generation
- Flat syntax reduces complexity for model understanding
- Limited keyword set (approx. 50 core keywords)

### 5. Systems Focus
- Designed specifically for systems programming needs
- Direct hardware abstraction without hidden layers
- Explicit memory and resource management

## Comparison with Existing Languages

| Feature | C/C++/Java | Python | Nous Lang |
|---------|------------|--------|-----------|
| Block syntax | Brace-delimited blocks | Indentation | **Indentation only** |
| Statement terminator | Semicolon `;` | None (new line) | **No terminators** |
| Memory model | Manual/GC | GC (pause risks) | **Reference counting** |
| Type system | Static/Static | Dynamic | **Static+Inference hybrid** |
| Control flow | Nested blocks | Indentation | **Flat indentation** |
| Keyword count | 50-80 | 30-40 | **~25 core keywords** |

## Implementation Roadmap

1. **Phase 1**: Core language specification and compiler design
   - Grammar definition
   - Type system specification
   - Compiler frontend (parsing, semantic analysis)

2. **Phase 2**: Runtime infrastructure development
   - Memory allocator implementation
   - Virtual machine design
   - JIT/AOT compilation strategies

3. **Phase 3**: Standard library creation
   - Basic I/O operations
   - Concurrency primitives
   - Data structure implementations

4. **Phase 4**: Systems programming toolkit
   - OS development templates
   - Hardware abstraction layer
   - Performance optimization guides

## Getting Started Examples

### Hello World
```nlang
def main():
    print("Hello, Nous Lang!")

end_function

main()
```

### Simple Calculator
```nlang
num a = 10
num b = 25

sum_result = a + b
diff_result = a - b
prod_result = a * b
quotient = b // a if b >= a else (a / b)

print("Sum:", sum_result)
print("Difference:", diff_result)
print("Product:", prod_result)
print("Quotient:", quotient)
end_if

def main():
    calculate_numbers()

end_function

main()
```

### Array Processing
```nlang
array numbers[5] = [1, 2, 3, 4, 5]

sum = 0
for i from 0 to 4:
    sum += numbers[i]

avg_value = num(sum / len(numbers))

print("Array:", numbers)
print("Sum:", sum)
print("Average:", avg_value)

end_for

def main():
    process_numbers()

end_function

main()
```

## Future Extensions (Planned Features)

- **Generative AI Integration**: Built-in model training and inference APIs
- **WebAssembly Target**: Native WASM compilation for browser deployment
- **SQL Query Language**: Embedded database query syntax
- **DSL Support**: Domain-specific language embedding capabilities
- **Testing Framework**: Assertion-based testing primitives
- **Profiling Tools**: Performance analysis and optimization hints

## Conclusion

Nous Lang represents a fresh approach to systems programming, combining the performance and type safety of compiled languages with the simplicity and LLM-friendliness of modern design. By eliminating traditional sources of complexity (braces, semicolons, deep nesting) while maintaining rigorous type checking and memory safety, Nous Lang enables developers to write clear, efficient code that can be reliably generated by smaller language models.

This specification provides the foundation for both human developers building complex systems programs and AI models generating correct, optimized code. The minimalist design philosophy ensures that as LLM capabilities improve, Nous Lang will continue to benefit from more sophisticated generation while maintaining its core advantages of simplicity and efficiency.

---
**Language Version**: 1.0 (Alpha Specification)
**Design Goals Achieved**: Minimalism yes | Token Efficiency yes | LLM-Friendly yes | Type Safety yes | Uniqueness yes | Systems Programming yes
