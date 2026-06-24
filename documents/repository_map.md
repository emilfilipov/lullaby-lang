# Repository Map

This file maps the repository layout and explains where to find core information. Keep it up to date whenever source files, directories, commands, tests, docs, or responsibilities change.

## Root

- [AGENTS.md](../AGENTS.md): operating guide for agents and contributors. Defines workflow, documentation rules, MCP/ClickUp/GitHub usage, testing expectations, Git rules, and where to find core language information.
- `.gitignore`: ignores local build outputs, caches, editor state, and generated artifacts once implementation begins.
- `documents/`: core language documents and planning material.

## Documents

- `documents/core_language_rules.md`: canonical global rules for `.nl` source files, indentation-only scope, forbidden block delimiters, and no semicolon terminators.
- `documents/language_specification.md`: top-level language specification and overview. Use this first for language behavior, philosophy, syntax reference, examples, and roadmap.
- `documents/implementation_plan.md`: implementation plan for the compiler, runtime, CLI, installer, tests, and release workflow.
- `documents/nous_lang_syntax_design.md`: syntax design details for declarations, functions, data structures, operators, naming, comments, and examples.
- `documents/nous_lang_type_system.md`: type-system details for primitives, composites, references, functions, inference, safety, aliases, generics, and OS-specific types.
- `documents/nous_lang_memory_management.md`: memory model covering regions, stack allocation, heap allocation, lifetime tracking, GC hooks, safety checks, runtime memory APIs, and kernel memory examples.
- `documents/nous_lang_control_structures.md`: control flow and operators, including conditionals, loops, switches, try/catch, coroutines, arithmetic/logical/bitwise operators, collections, conversions, and utility operations.
- `documents/nous_lang_input_output.md`: I/O and concurrency model, including files, streams, memory-mapped files, threads, processes, async, multiplexing, IPC, sockets, and performance strategies.
- `documents/nous_lang_error_handling.md`: error model, compact error tokens, compile-time and runtime categories, throw/catch/recovery behavior, diagnostics, and compiler integration.
- `documents/nous_lang_compilation_architecture.md`: compiler architecture from tokenization through semantic analysis, IR, optimization, code generation, linking, and binary verification.
- `documents/repository_map.md`: this file. Use it as the first navigation aid and update it with repository changes.

## Planned Source Layout

The implementation has not been created yet. Unless changed by an explicit architecture decision, use this layout when starting Rust implementation work:

- `crates/nous_lexer/`: source loading, tokenization, indentation scanning, and lexical diagnostics.
- `crates/nous_parser/`: AST model and parser for declarations, expressions, statements, and control flow.
- `crates/nous_semantics/`: symbol tables, type checking, memory analysis, and semantic diagnostics.
- `crates/nous_ir/`: semantic IR schema, lowering, validation, and debug serialization.
- `crates/nous_runtime/`: runtime execution prototype, memory APIs, I/O adapters, and error propagation.
- `crates/nous_cli/`: `nlang` command-line interface for `check`, `run`, `build`, `fmt`, `test`, and version reporting.
- `tests/fixtures/`: `.nl` source fixtures and expected outputs/diagnostics.
- `tests/integration/`: end-to-end tests from source through parse, semantic validation, runtime/backend execution, and output capture.

## Planning And Tracking

- ClickUp folder: `Nous Lang` under the available `general` space.
- Current ClickUp lists:
  - `01 Project Foundation`
  - `02 Lexer Parser AST`
  - `03 Type System`
  - `04 Memory Runtime`
  - `05 IO Errors Syscalls`
  - `06 IR Optimization Codegen`
  - `07 CLI Build Installer`
  - `08 Tests Docs Release`

## Update Rules

- Update this map when adding, moving, renaming, or deleting files or directories.
- Update this map when build, test, lint, fixture, release, or documentation commands change.
- Update this map when a document becomes canonical for a concept or stops being canonical.
- Keep this file factual. Do not use it for speculative design notes.
