# Nous Lang (nlang) Compiler & Installer Implementation Plan

Canonical language rules: see [core_language_rules.md](core_language_rules.md).

**Goal:** Create a complete toolchain and installer for Nous Lang (nlang) that allows users to compile and run `.nl` source files easily via the command line.

**Key Dependencies:** All 8 core language documents in this directory must be fully implemented and integrated into this plan.

## Current Implementation Checkpoint

The repository now contains the first Rust workspace scaffold:

- `nous_lexer`: validates `.nl` source paths, tokenizes source text, emits indentation/dedent tokens, and rejects curly braces and semicolon terminators.
- `nous_parser`: parses the first supported function syntax, typed parameters, return types, indentation-based bodies, `return`, expression statements, and `if`/`elif`/`else` blocks into an AST.
- `nous_semantics`: performs the first semantic validation pass for non-void function return values.
- `nous_cli`: exposes `nlang check <file.nl>` through `cargo run -p nous_cli -- check <file.nl>`.
- `nous_ir` and `nous_runtime`: placeholder crates for the next implementation phase.
- `tests/fixtures`: valid and invalid `.nl` fixtures for frontend and CLI smoke checks.

## Epic 1: Core Toolchain Implementation (Compiler & Runtime)
*Objective: Implement the core components defined by our design docs to parse, analyze, and execute nlang code.*

| Story | Description | Dependencies | Estimated Effort | Status |
| :--- | :--- | :--- | :--- | :--- |
| **1.1** | **Lexer & Parser Implementation:** Implement the core components to read raw nlang source code and convert it into an Abstract Syntax Tree (AST). | `nous_lang_syntax_design.md` | High | In Progress |
| **1.2** | **Type System Integration:** Implement the type checker based on `nous_lang_type_system.md`. Ensure all AST nodes are correctly typed and checked before execution. | `nous_lang_type_system.md` | High | To Do |
| **1.3** | **Memory System Implementation:** Implement the memory allocator/deallocator based on `nous_lang_memory_management.md`. Ensure ARC and explicit allocation work correctly in the runtime environment. | `nous_lang_memory_management.md` | High | To Do |
| **1.4** | **Runtime Execution Engine:** Implement the core execution loop that traverses the AST, manages memory, resolves types, and executes nlang instructions. | All previous steps | Critical | To Do |

## Epic 2: System Integration & I/O Layer
*Objective: Connect the runtime to the operating system and enable basic interaction.*

| Story | Description | Dependencies | Estimated Effort | Status |
| :--- | :--- | :--- | :--- | :--- |
| **2.1** | **I/O System Integration:** Implement the I/O layer based on `nous_lang_input_output.md`. Focus on file system access (reading/writing) for standard library operations. | `nous_lang_input_output.md` | High | To Do |
| **2.2** | **Error Handling Integration:** Integrate the error handling system (`nous_lang_error_handling.md`) into the runtime to ensure all compilation and runtime errors are gracefully reported in a structured format. | `nous_lang_error_handling.md` | Medium | To Do |
| **2.3** | **System Call Abstraction:** Define the interface for executing low-level OS commands (e.g., system calls) that nlang code can invoke safely. | N/A (New Design) | High | To Do |

## Epic 3: Build System & Distribution
*Objective: Create the infrastructure to compile the source code into an executable and package it for distribution.*

| Story | Description | Dependencies | Estimated Effort | Status |
| :--- | :--- | :--- | :--- | :--- |
| **3.1** | **Compiler Toolchain:** Implement the full compiler pipeline defined in `nous_lang_compilation_architecture.md` to handle source code compilation into machine-readable bytecode or an intermediate representation. | All Runtime Components | High | To Do |
| **3.2** | **Build Script Generation:** Create a robust, platform-agnostic build script (e.g., using CMake or a custom script) that orchestrates the compilation of the compiler and runtime into a single binary. | 3.1 | Medium | To Do |
| **3.3** | **Installer Creation:** Develop the installer logic to bundle the compiled nlang executable, necessary libraries, and documentation into a single user-friendly package (e.g., .exe or system package). | 3.2 | High | To Do |

## Epic 4: User Experience & Final Delivery
*Objective: Create the final, easy-to-use installation method.*

| Story | Description | Dependencies | Estimated Effort | Status |
| :--- | :--- | :--- | :--- | :--- |
| **4.1** | **CLI Tool Implementation:** Implement the command-line interface (CLI) tool that allows users to invoke the compiled nlang executable (`nlang run script.nl`). | 3.3 | Medium | In Progress |
| **4.2** | **Installation & Setup:** Finalize the installation process, ensuring minimal user interaction and clear setup instructions are provided upon first launch. | 3.3, 4.1 | High | To Do |
| **4.3** | **Documentation Finalization:** Review all documentation to ensure they align with the final installed product's usage patterns. | All previous steps | Low | To Do |

## Epic 5: Testing & Verification (The Regression Shield)
*Objective: Establish a continuous feedback loop to ensure correctness, prevent regressions, and verify that all components interact as designed.*

| Story | Description | Dependencies | Estimated Effort | Status |
| :--- | :--- | :--- | :--- | :--- |
| **5.1** | **Unit Test Framework Setup:** Define the structure for unit tests (e.g., using a Python-based harness or custom nlang testing runner). This must be lightweight and fast, aligning with our minimalistic philosophy. | All previous components | Medium | In Progress |
| **5.2** | **Component Unit Testing:** Implement unit tests for each major component: Lexer (tokenization), Parser (AST generation), Memory Manager (allocation/deallocation), and Type Checker. | Stories 1.1, 1.2, 1.3 | High | To Do |
| **5.3** | **Integration Test Suite:** Develop end-to-end integration tests that verify the entire pipeline: `Source Code` -> `AST` -> `Runtime Execution`. This ensures the compiler and runtime work together correctly. | All previous steps | Critical | To Do |
| **5.4** | **Regression Test Protocol:** Establish a protocol for running the full suite (Unit + Integration) before any major feature addition or refactoring is committed to the codebase. | All previous steps | Medium | To Do |
