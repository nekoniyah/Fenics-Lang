# Fenics Lang

Fenics is a small, expressive programming language with a friendly syntax, a Rust-based interpreter, a VS Code extension, and an extensible "bridge" system to call native functionality.

## Features

- Clean block syntax with colon-based declarations
- Variables: constant, mutable, global; optional types
- Functions with optional return types
- Control flow: `if`, `else if`, `else`, `for`, `while`, `loop`, `try/catch`
- Expressions: arithmetic, comparison, logical, ternary (`then/otherwise` and `?:`)
- Strings with interpolation: "Hello #{name}"
- Arrays, object blocks, typed collections (`List`, `Pairs`)
- Ephemeral variables: `#name` or `#1` for quick inline values
- Modules: `lib` export blocks, `import` (by path or name) with `as` alias
- Bridges: call native modules implemented in Rust (e.g., `fs.read`, `fs.write`)
- VS Code: syntax highlighting, snippets, completions, hovers, diagnostics

## Quick Start

### Run a sample

```bash
cargo run -- interpreter ../samples/example.fenics
```

Or from `interpreter/`:

```bash
cargo run -- ../samples/example.fenics
```

### Install the VS Code extension

- Package is at `vscode-extension/fenics-lang-0.1.0.vsix`
- Install with:

```bash
code --install-extension "c:\Users\noebo\Documents\Dev\Fenics Lang\vscode-extension\fenics-lang-0.1.0.vsix"
```

## Language Overview

- Declarations use a colon:
  - `const count: 1`
  - `Int const count: 1`
  - `name: "Fenics"`
  - `global config:` then indented `- key: value` entries
- Functions:
  - `fn add(a, b) -> Int:` body and `return a + b`
- Control flow:
  - `if cond:` ... `else if other:` ... `else:` ...
  - `for item in items:`
  - `while condition:`
  - `loop condition:` reactive loop
  - `try:` ... `catch (err)` ...
- Operators: `+ - * / % ** ^ ++ --`, comparisons `== != === !== < <= > >= ~ !~`, logical `and or not is`, assignments `:` and augmented `+: -: *: /: %:`
- Strings: double-quoted, with interpolation via `#{expr}`
- Arrays: `[1, 2, 3]`; Objects: block with `- key: value` lines
- Ephemeral variables: `#name`, `#1` usable in interpolation and expressions

## Modules and Imports

- Export a module:

```
fn greet(name):
    print("Hello, #{name}!")

lib mylib:
    - greet
```

- Import by name (searches common locations):

```
import mylib
mylib.greet("World")
```

- Import by path with optional alias:

```
import "../samples/mylib.fenics" as m
m.greet("World")
```

Search order for bare names: `module.fenics`, `libs/module.fenics`, `../libs/module.fenics`, `samples/module.fenics`, `../samples/module.fenics`.

## Bridges (Native Modules)

Built via a Rust trait; bundled example `fs`:

- `fs.read(path) -> String`
- `fs.exists(path) -> Boolean`
- `fs.write(path, content) -> Boolean`

## VS Code Extension

- Syntax highlighting and snippets for language constructs
- Completions: keywords, builtins, types, modules, and module methods
- Hovers: variables (declared + implicit types, value preview), functions (signature), builtins (signature), import aliases
- Diagnostics: missing modules/paths, undeclared exports, simple declaration hints

## Documentation

Full documentation lives under `docs/`.

- Getting Started: docs/index.md
- Language Guide: docs/language.md
- Modules: docs/modules.md
- Bridges: docs/bridges.md
- Interpreter CLI: docs/interpreter.md
- VS Code Extension: docs/vscode-extension.md

## Contributing

PRs welcome. Please open issues for bugs or feature requests.

## License

TBD
