# Fenics Lang Documentation

Welcome to Fenics Lang. This documentation covers the language syntax, standard features, module system, native bridges, interpreter usage, and VS Code tooling.

- Language Guide: language.md
- Modules and Imports: modules.md
- Bridges (Native Modules): bridges.md
- Interpreter CLI: interpreter.md
- VS Code Extension: vscode-extension.md

## Installation

- Rust toolchain: install from https://www.rust-lang.org/tools/install
- Clone repository and build interpreter
  ```bash
  cd interpreter
  cargo run -- ../samples/example.fenics
  ```
- Install VSIX extension
  ```bash
  code --install-extension "c:\\Users\\noebo\\Documents\\Dev\\Fenics Lang\\vscode-extension\\fenics-lang-0.1.0.vsix"
  ```

## Samples

See `samples/` for runnable examples such as `example.fenics`, `mylib.fenics`, and bridge usages.
