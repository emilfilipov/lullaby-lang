# Lullaby Error Handling Documentation

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

## Overview

Lullaby implements a sophisticated yet minimal error handling system designed for systems programming. Unlike traditional languages that rely on verbose try-catch blocks and exception objects, Lullaby uses an operator-based approach optimized for both token efficiency and runtime performance.

## Current Error Reporting

The compiler reports compile-time and runtime failures with stable `L####` diagnostic codes. Lexing and parser diagnostics include source spans when available, semantic diagnostics include the function where the error was found, and runtime failures include a category label in CLI output.

Current diagnostic ranges:

| Range | Source | Example |
| :--- | :--- | :--- |
| `L0001-L0003` | Source path and host file loading/writing | Invalid extension, unreadable source file, or failed artifact write. |
| `L0101-L0104` | Lexer | Forbidden curly braces or semicolon terminators. |
| `L0201-L0211` | Parser | Missing function body indentation, malformed expression, or planned syntax rejected by the parser. |
| `L0300-L0329` | Semantic validation | Unknown name, type mismatch, invalid loop control, invalid builtin arguments, and invalid executable entry points. |
| `L0400-L0418` | Runtime and host resources | Missing `main`, division by zero, invalid pointer, missing file, failed command invocation. |
| `L0501` | IR lowering | Typed IR lowering failed after semantic validation. |
| `L0601` | Bytecode artifact | Compiled `.lbc` artifact is malformed or unsupported. |

Runtime CLI output uses:

```text
L0414 [resource]: failed to read `missing.txt`: ...
```

The implemented runtime categories are:

- `runtime`: execution errors such as division by zero, invalid pointer use, out-of-bounds array indexing, or wrong runtime value kind.
- `resource`: host resource failures such as failed file reads/writes/appends or failed command invocation.
- `ir`: typed IR lowering failures reported before an IR or bytecode backend starts executing.
- `bytecode`: compiled `.lbc` artifact loading failures before bytecode execution starts.

Language-level `throw`/`try`/`catch` and the postfix `?` propagation operator on `option`/`result` are implemented (see "Safe-Tier Failure Semantics" below and the fixtures `run_error_handling.lby`, `run_error_propagation.lby`). The compact `!0xXX` error-token surface remains planned; unimplemented planned keywords still produce `L0211 [parser error]` so users can distinguish future syntax from malformed code.

## Safe-Tier Failure Semantics (decision A5)

Lullaby's safe tier splits every runtime failure into two **disjoint** families with different guarantees. This is the decided A5 contract (`documents/road_to_1_0_stable.md`): *abort-with-diagnostic, no unwinding; recoverable errors flow through `result`/`?`/`throw`/`catch`, so panics are for bugs.*

### Aborts — contract / memory-safety violations (bugs)

A **contract violation** is a bug in the program: it indicates the code reached a state its own invariants were supposed to prevent. In the safe tier such a violation **aborts the program immediately** with a clear stable `L####` diagnostic and a non-zero exit status. It does **not** unwind the stack, does **not** run any `catch`, and is **not** recoverable — there is nothing sensible to recover *to*, and pretending otherwise would let a corrupted state propagate. Aborts are deterministic and allocation-free, which keeps them compatible with the GC-free arena model and the freestanding tier.

The contract violations and their diagnostics:

| Violation | Diagnostic | Notes |
| :--- | :--- | :--- |
| Array / `array<T>` index out of bounds (incl. negative) | `L0413` | `a[i]` on the interpreters; native traps (see below). |
| `string` index / substring range out of bounds | `L0413` | Char-indexed. |
| `list<T>` `get`/`set` index out of bounds | `L0413` | `get` returns the element directly, so out-of-range is a bug. |
| `pop` of an empty `list<T>` | `L0413` | No element to remove. |
| Index target is not indexable | `L0412` | Normally prevented statically (`L0325`). |
| Division / remainder by zero (`/`, `%`, `/=`) | `L0404` | Integer div/rem by a runtime `0`. |
| `for … by 0` (zero loop step) | `L0411` | A non-terminating loop is rejected. |
| `array_fill` with a negative length | `L0433` | |
| Wrong runtime value kind (should be unreachable after checks) | `L0407`/`L0408`/`L0409`/`L0417`/`L0418`/`L0421` | Type checker normally prevents these; if reached they abort, never silently coerce. |

There is deliberately **no forced-`unwrap` operator** that panics on `none`/`err`. An `option`/`result` payload is reached only through an **exhaustive `match`** (non-exhaustive is the compile error `L0384`) or the postfix **`?`** operator (recoverable — below). So "unwrap on `none`" is not an abort in Lullaby: it is either a compile-time error or a recoverable propagation. This is a stronger safety story than a language with an abort-on-`none` `unwrap`.

### Recoverable — modeled / expected failures

A **modeled failure** is an outcome the program anticipates (a parse can fail, a lookup can miss, a resource can be unavailable). These are **values**, not aborts, and flow through:

- **`result<T, E>` / `option<T>`** returned from functions and consumed by `match`.
- **`?`** — postfix propagation: `expr?` yields the payload on `ok`/`some` and otherwise returns the failure (`err(e)`/`none`) from the enclosing function unchanged. The enclosing return type must be compatible (`L0427`/`L0428`/`L0429`).
- **`throw` / `try` / `catch`** — `throw "msg"` raises a catchable user error (`L0420`); a surrounding `try` … `catch name` recovers it and binds the message. `assert(cond)` raises the same catchable `L0420` on a false condition.

A recoverable failure lets the program **continue to a normal exit**. Only a `throw` that escapes *every* enclosing `try`/`catch` becomes the abort `L0420` ("uncaught thrown error") — the boundary of the recoverable model.

### The line between them

`try`/`catch` catches **only** user-thrown `L0420` (from `throw`/`assert`). A contract-violation abort (`L0404`, `L0413`, `L0412`, `L0411`, `L0433`, …) propagates straight through any `catch` and terminates the program. This is the "no unwinding through a safety abort" rule: a safety abort is not an exception you can intercept.

### Backend consistency and the freestanding tier

The three interpreters (AST, IR, bytecode) enforce this identically — same diagnostic code and message for each violation, and `try`/`catch` recovers only `L0420` on all three (regression-locked by `crates/lullaby_cli/tests/cli/suite13.rs`). The **native** backend turns the same contract violations into a hardware trap rather than a formatted diagnostic: an out-of-bounds index, an out-of-range substring, an empty-separator `split`, and heap exhaustion emit a `ud2` illegal-instruction trap, which surfaces as `STATUS_ILLEGAL_INSTRUCTION` (`0xC000001D`) on Windows; an integer divide-by-zero surfaces as the CPU `#DE` (`STATUS_INTEGER_DIVIDE_BY_ZERO`, `0xC0000094`). Both are defined, deterministic, non-zero-exit aborts with no unwinding — the same guarantee, expressed as a trap because native code has no diagnostic printer. See `documents/native_backend_contract.md`.

In the **freestanding / `no-runtime` tier** the safe-tier guarantee is preserved but the abort is routed to a **user-provided panic handler** (cf. Rust's `#[panic_handler]`) instead of an OS abort, so a kernel or embedded target decides what a bounds/contract failure does (halt, reset, log). The check machinery is the same; only the terminal action is pluggable. See `documents/execution_tiers_and_1_0_scope.md`.

## Epic 6 Diagnostics UX

The compiler has three CLI diagnostic modes:

```text
lullaby check file.lby
lullaby check --verbose file.lby
lullaby check --format json file.lby
```

The same flags are available for `lullaby compile`, `lullaby build`, `lullaby inspect`, and `lullaby run`. `lullaby run` defaults to the AST runtime and accepts `--backend ir` and `--backend bytecode` for the current implemented subset. `lullaby compile` emits a versioned `.lbc` bytecode artifact with function-table and memory-operation metadata, `lullaby build` is the same artifact-generation path under a build-oriented command name, `lullaby inspect file.lbc` summarizes that artifact, and `lullaby run file.lbc` executes it. IR lowering failures use code `L0501` and phase `ir`; bytecode artifact failures use code `L0601` and phase `bytecode`. The alias `--diagnostic-format json` is also accepted. Extra positional arguments are rejected so tools do not accidentally ignore misspelled paths or flags.

### Concise Output

Concise output is the default. It is intended for quick terminal feedback:

```text
L0303 [semantic error] at tests/fixtures/invalid/type_mismatch.lby:2:22 in `main`: binding `value` declares `bool` but initializer has `i64`
```

### Verbose Output

Verbose output is intended for humans and LLM agents that need enough context to repair the source:

```text
L0102 [lexer error] at tests/fixtures/invalid/brace.lby:2:5: curly braces are not block delimiters in Lullaby

Source:
   2 |     {
     |     ^

Problem:
  Lullaby uses indentation-only blocks.

Root cause:
  The source contains a curly brace, which is not a block delimiter.

Suggested fix:
  Remove the brace and express the block with indentation.
```

Runtime failures include lightweight tracebacks when execution has entered user code:

```text
Traceback:
  in `main` at 1:1
```

### JSON Output

JSON mode is deterministic and intended for editors, CI systems, and LLM agents. Failure diagnostics are written to stderr and keep a non-zero exit status. Successful JSON runs write this to stdout:

```json
{"status":"ok","diagnostics":[]}
```

Failure JSON uses the diagnostic registry fields:

```json
{"status":"error","diagnostics":[{"code":"L0313","phase":"semantic","severity":"error","message":"argument 2 for `sys_status` must be `array<string>` but got `array<i64>`","source_path":"tests/fixtures/invalid/sys_args_type.lby","span":{"line":2,"column":24},"function":"bad","explanation":"Function and builtin arguments are statically type checked.","root_cause":"The argument expression type does not match the parameter type.","suggested_fix":"Pass a value of the expected type or change the called function signature.","notes":[],"traceback":[]}]}
```

See [diagnostic_registry.md](diagnostic_registry.md) for the full stable code registry and JSON field contract.

## Core Error Concepts

### Error Tokens
Errors are represented as compact tokens rather than full exception objects:
- `!` - Error marker prefix (replaces "throw" keyword)
- Errors encoded as 3-digit hexadecimal values
- Example: `!0x4c` represents a specific error code

### Error Categories
1. **Runtime Errors** (`!0xXX`) - Occur during execution
2. **Compilation Errors** (`#err`) - Caught at compile time
3. **Type Errors** (`#tpe`) - Type mismatches and violations
4. **Resource Errors** (`#res`) - Memory, file, or I/O issues

## Planned Error Operators

### The Throw Operator
```lullaby
!0x4c message
```
- Throws error code 0x4c (segmentation fault) with optional message
- Minimal token count: 2 tokens minimum
- No parentheses, no brackets needed

### Try-Catch Pattern
```lullaby
try code !catch ErrorCode message handler_code end
```
- Wraps potentially failing code in try block
- Catches specified error codes using the ! operator
- Executes fallback logic on error
- End marks closure of try block

### Error Recovery
```lullaby
err RecoveryCode:
  recovery_action1
  recovery_action2
end
```
- Defines specific recovery procedures for error codes
- Can be called automatically or manually from catch blocks
- Supports multiple recovery paths per error code

## Compile-Time Error Detection

The compiler performs extensive static analysis to detect errors before runtime:

### Static Analysis Checks
- Type compatibility verification
- Memory allocation validation
- Resource access permissions
- Control flow consistency

### Error Categories at Compile Time
1. **Syntax Errors** - Invalid syntax constructs
2. **Type Mismatches** - Incompatible type operations
3. **Unused Declarations** - Unreachable or unused code
4. **Resource Conflicts** - File/driver access issues

## Error Reporting Format

Currently:
```text
L0102 at 1:9: curly braces are not block delimiters in Lullaby
L0313 in `main`: argument 2 for `sys_status` must be `array<string>` but got `array<i64>`
L0414 [resource]: failed to read `missing.txt`: ...
```

Planned compact language-level representation:
```lullaby
!0x4c MemoryOverflow: Allocation exceeds limit of 64KB
#tpe ExpectedInt: Got Float in integer parameter position
#res NoFilePermission: Cannot open '/etc/config' (permission denied)
```

### Message Encoding
- Compact format with error code and descriptive message
- Messages stored in Unicode but limited to 32 characters max
- Includes location information when available (line number, function name)

## Error Handling Best Practices

1. **Fail Fast** - Detect errors immediately rather than continuing on corrupted state
2. **Graceful Degradation** - Provide fallback behaviors when errors occur
3. **Explicit Recovery** - Define recovery paths for critical error codes
4. **Error Propagation** - Bubble up significant errors through call stack automatically

## Example Usage

The following is planned syntax, not accepted by the current parser:

```lullaby
# Memory-safe array operation with error handling
func process_data(data: Array[int]): bool
  try
    allocate_buffer(data.size)
    for i from 0 to data.size do
      load_value(i, data[i])
      store_result(i, computed_value(data[i]))

    # Check memory usage before operation
    if mem_usage > mem_limit then
      !0x41 MemoryLimitExceeded: Current usage at mem_usageKB
      fallback_to_minimal_allocation()
    end

    return success(true)
  catch ErrorCode handler_code end

  return success(false)
end

// Error recovery for I/O operations
err FileReadError:
  attempt_alternate_source()
  report_missing_data()

err PermissionDenied:
  request_admin_access()
  log_security_event()
end
```

## Performance Considerations

- Error handling adds minimal overhead (< 2% in typical workloads)
- Compile-time errors detected before any runtime cost
- Token-efficient error representation saves ~60% vs traditional exceptions
- Fast error lookup using hash-based code mapping

## Integration with Compilation Pipeline

Error handling is integrated into the compilation phases:

1. **Semantic Analysis** - Type checking and type error detection (#tpe)
2. **Static Optimization** - Resource validation and #res errors
3. **Code Generation** - Insert try-catch patterns automatically where needed
4. **Final Verification** - Runtime error handler injection (!catch blocks)

## Conclusion

The Lullaby error handling system provides:
- Minimal token overhead for LLM understanding
- Clear separation between compile-time and runtime errors
- Automatic recovery mechanisms without verbose exception objects
- Strong safety guarantees through static analysis
- Efficient error codes suitable for systems programming

This design maintains the minimalistic philosophy while providing robust error handling essential for reliable OS development.
