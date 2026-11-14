# ZAK

**ZAK is a native-first programming language for building applications.**

The long-term goal is one language and toolchain that can target Android, iOS, macOS, Windows and Linux while using each platform's native UI and system capabilities.

ZAK is in its early compiler-foundation stage. The current 0.1 milestone focuses on a small, real language implementation: a Rust compiler frontend, an AST, diagnostics, a basic static type checker, LLVM IR generation and a command-line workflow.

> **Status:** experimental. The first milestone targets a native executable on the host machine. Mobile and cross-platform application backends are not implemented yet.

## Current implementation

- Rust Cargo workspace.
- Lexer and recursive-descent parser.
- AST for functions, variables, assignments, returns and expressions.
- Static type checking for Int, Bool, String and Unit.
- LLVM IR code generation using opaque pointers.
- zak new, zak check, zak build and zak run.
- Native host compilation through clang.

The current language subset supports one main function, local variables, integer and boolean expressions, strings, assignments to var declarations, and print(...).

## Prerequisites

- Rust toolchain with Cargo.
- LLVM/Clang installed and clang available on PATH.

The compiler emits textual LLVM IR (.ll) and invokes Clang to produce a host executable. This keeps LLVM installation and version selection outside the Rust dependency graph.

## Build the compiler

~~~bash
cargo build --workspace
cargo test --workspace
~~~

For a convenient local CLI:

~~~bash
cargo install --path crates/zak-cli
~~~

## Create and run a project

~~~bash
zak new hello
cd hello
zak check
zak build
zak run
~~~

A generated project looks like this:

~~~text
hello/
├── zak.toml
└── src/
    └── main.zak
~~~

Example source:

~~~rust
fn main() -> Int {
    print("Hello from ZAK");
    let answer: Int = 40 + 2;
    print(answer);
    return 0;
}
~~~

## Commands

| Command | Description |
|---|---|
| zak new <name> | Create a project |
| zak check | Parse and type-check source |
| zak build --target host | Generate LLVM IR and a native executable |
| zak run | Build and run the project |
| zak help | Show CLI usage |

The compiler writes generated files under build/, including main.ll.

## Language and architecture documents

- [ZAK language specification 0.1](docs/spec-v0.1.md)
- [Compiler architecture and implementation plan](docs/architecture/compiler-foundation.md)

## VS Code support

An initial declarative VS Code language extension lives in [editors/vscode](editors/vscode). It registers the `.zak` extension as language ID `zak`, provides ZAK 0.1 syntax highlighting, basic editing behaviour, snippets and a language icon.

To test it in VS Code, open `editors/vscode` as a folder and press **F5** to launch an Extension Development Host. This editor extension does not install the compiler. The compiler is a separate `zak` CLI tool.

## Roadmap

1. Stabilize the language frontend and type system.
2. Expand the language with control flow, functions, modules and memory-safety rules.
3. Build the platform-independent runtime and native UI intermediate representation.
4. Implement the Android native host and device APIs.
5. Add iOS/macOS, Windows and Linux native adapters.
6. Deliver package management, language-server support, debugging and a stable ecosystem.

## Design principles

- Native UI rather than a WebView-based application shell.
- One language with explicit platform adapters.
- Static types and safe-by-default memory semantics.
- Platform SDKs hidden behind the ZAK build and runtime tooling where practical.
- Firebase and other service providers remain optional integrations, not language requirements.

## Contributing

The language is not yet stable. Changes to syntax, semantics or platform APIs should be proposed through an RFC in docs/rfcs/ before becoming part of the stable specification.

## License

MIT. See LICENSE.
