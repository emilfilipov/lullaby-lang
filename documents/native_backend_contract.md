# Alpha 1 Native Backend Contract

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

This document records the first native backend contract for Alpha 1. The executable source of truth is `crates/lullaby_ir/src/native_contract.rs`.

## Status

Implemented now:

- A serializable `NativeBackendContract` data model in `lullaby_ir`.
- A deterministic `alpha1_native_backend_contract()` baseline.
- `alpha1_value_layout(TypeRef)` coverage for the current Alpha 1 type surface: `void`, `i64`, `bool`, `string`, `array<T>`, and `ptr_*`.
- Unit tests for target selection, current value layouts, cleanup sequencing, and JSON round-trip stability.
- A checked-in JSON snapshot under `crates/lullaby_ir/tests/snapshots/alpha1_native_backend_contract.json`.
- A first `x86_64-pc-windows-msvc` COFF object emitter in `crates/lullaby_ir/src/native_object.rs` for zero-argument `main` functions that return a literal `i64`, literal `bool`, `void`, stack-backed `i64` local arithmetic, or straight-line `i64` local assignment arithmetic.
- Checked-in object-emission snapshots under `crates/lullaby_ir/tests/snapshots/alpha1_return_42.coff.json`, `crates/lullaby_ir/tests/snapshots/alpha1_locals_add.coff.json`, and `crates/lullaby_ir/tests/snapshots/alpha1_assignments.coff.json`.

Not implemented yet:

- General machine-code lowering beyond the current literal, `i64` local, assignment, and simple arithmetic subset.
- Object file writing for non-COFF targets.
- Linker orchestration.
- Native runtime packaging.

## Targets

The first native prototype target is `x86_64-pc-windows-msvc` with COFF object emission. The contract also records the intended 64-bit target family for later work:

- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

All current contract targets are 64-bit little-endian targets.

## Calling Convention

The Alpha 1 backend uses an internal Lullaby ABI before adapting to platform object and linker conventions:

- Parameters lower in source order.
- `main` remains the zero-argument entry function for executable validation.
- Scalar and handle return values are returned directly.
- Variadic calls are not part of Alpha 1.
- Call boundaries require 16-byte alignment.

## Stack Frame

Native lowering must model stable slots for:

- parameters
- locals
- temporaries
- spills
- cleanup records

Cleanup order is driven by `IrMemoryOperation.sequence`, matching the bytecode artifact memory metadata.

## Value Layout

The current Alpha 1 value layouts are:

| Type pattern | Class | Size | Alignment | Pass/return mode |
| :--- | :--- | :--- | :--- | :--- |
| `void` | no payload | 0 bytes | 1 byte | no payload |
| `i64` | integer | 8 bytes | 8 bytes | direct value |
| `bool` | boolean | 1 byte | 1 byte | direct value |
| `string` | runtime handle | 8 bytes | 8 bytes | pointer-sized handle |
| `array<T>` | runtime descriptor handle | 8 bytes | 8 bytes | pointer-sized handle |
| `ptr_*` | heap pointer handle | 8 bytes | 8 bytes | pointer-sized handle |

The contract intentionally treats strings, arrays, and heap pointers as pointer-sized handles. Inline string bytes, array element storage, and heap-slot contents remain runtime-managed.

## Pointer And Array Rules

Safe source operations must not lower null pointer values. Native lowering for `load`, `store`, and `dealloc` must preserve the same live-resource requirements recorded in memory-operation metadata.

Arrays lower as runtime descriptor handles. The descriptor contains a logical `length: i64` and a pointer-sized handle to contiguous element storage. Indexing must preserve bounds-check semantics before element access.

## Cleanup And Diagnostics

Explicit release and future compiler cleanup must share `IrMemoryOperation.sequence` so bytecode and native backends make the same resource-order decisions.

Native backend diagnostics must use the shared `L####` diagnostic model. Target-specific failures must include the target triple.

## Prototype Object Emission

`lullaby_ir::native_object` emits the first prototype COFF object for `x86_64-pc-windows-msvc`. The current emitter deliberately supports a small reviewable native slice:

- source is still validated, lowered to typed IR, and lowered to bytecode before object emission
- the entry function must be zero-argument `main`
- return type must be `void`, `i64`, or `bool`
- the body may be empty `void`, `return <literal>`, a final literal expression, `i64` local bindings, `i64` `=`, `+=`, `-=`, and `*=` assignment, and an `i64` return expression using locals, literals, `+`, `-`, and `*`
- unsupported bytecode returns a structured `NativeObjectError`

For literal `i64`, the prototype emits `mov rax, imm64; ret`. For `bool`, it emits `mov eax, imm32; ret`. For `void`, it emits `ret`. For local `i64` arithmetic and assignment, it emits a frame pointer prologue, 16-byte-aligned stack slots, local loads/stores, arithmetic into `rax`, and a frame epilogue. Native `i64` division and `/=` remain unsupported until signed division lowering and trap behavior are specified.

## Next Backend Work

The next native backend checkpoint should expand object emission to calls, control flow, division semantics, or stronger object validation with platform object tools when available; linker workflow for `x86_64-pc-windows-msvc` remains pending. It should not bypass the AST runtime, typed IR validation, bytecode VM, or existing release verification.
