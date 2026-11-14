# Compiler Foundation Architecture

## 1. Scope

The first implementation milestone establishes a real end-to-end compiler for a small ZAK language subset. It does not attempt to implement native mobile UI or every target platform.

The current workspace deliberately has no third-party Rust dependencies. LLVM integration is performed through textual LLVM IR and the installed clang executable.

## 2. Workspace

- zakc: lexer, parser, AST, diagnostics, type checker and LLVM IR generator.
- zak-cli: the zak command-line interface and host build orchestration.

The CLI owns filesystem operations and external tool invocation. The compiler library remains deterministic and does not invoke shell commands.

## 3. Frontend pipeline

source -> lexer -> tokens -> parser -> AST -> type checker -> LLVM IR

### Lexer

The lexer owns source scanning and token locations. It recognizes identifiers, keywords, literals, punctuation and operators. Lexical errors are reported with line and column.

### Parser

A recursive-descent parser implements the 0.1 grammar and produces an AST. Binary expression parsing follows the precedence specified in spec-v0.1.md.

### AST

The AST contains program and function declarations, statements, typed names, expressions and source spans. It is independent of LLVM.

### Type checker

The type checker verifies program shape, declarations, mutability, variable references, built-in calls and expression types before code generation. LLVM IR is never used as a substitute for source-level semantic checks.

### Code generator

The code generator lowers the supported AST into LLVM IR. The initial backend emits a single native main function and uses the host C runtime for the print intrinsic (puts and printf).

LLVM opaque pointers (ptr) are used in generated IR. Source-level types are tracked by ZAK and are not inferred from LLVM pointer types.

## 4. CLI and build pipeline

- zak new creates a project manifest and src/main.zak.
- zak check runs the frontend and type checker.
- zak build writes build/main.ll and invokes clang to produce a host executable.
- zak run builds the executable and launches it.

The first release supports --target host only. Other targets must fail explicitly rather than silently producing host binaries.

## 5. Error handling

Compiler diagnostics include a phase (lexer, parser, type) and source line/column. The CLI reports diagnostics to stderr and exits nonzero.

Build failures from Clang are surfaced with the compiler's stderr output.

## 6. Testing strategy

The zakc crate contains tests for successful LLVM generation, immutable-variable assignment rejection and type mismatch rejection.

Future test layers:

1. Lexer token and span tests.
2. Parser AST-shape tests.
3. Type checker positive and negative tests.
4. LLVM verification and executable integration tests.
5. CLI end-to-end tests.
6. Cross-platform backend tests once those backends exist.

## 7. Next compiler increments

After the first executable works:

1. Improve diagnostics with source excerpts and error recovery.
2. Add block scopes, if and while.
3. Add user-defined functions and call lowering.
4. Add floating point and richer string handling.
5. Specify ownership and borrowing before exposing heap-managed objects.
6. Introduce a platform-independent MIR before building additional native backends.
7. Add an explicit target triple, target ABI and platform linker configuration.

## 8. Platform boundary

The eventual native runtime should be split into a platform-independent core and platform adapters behind a stable C ABI. The compiler frontend and language semantics must not depend on Android, Apple, Windows or Linux SDK types.

The future platform build graph is:

ZAK source -> ZAK frontend -> HIR/MIR -> LLVM target backend -> native runtime + platform host -> signed package

Android, Apple, Windows and Linux adapters are separate deliverables. Their existence must be represented in the CLI only when the corresponding implementation and toolchain integration are functional.
