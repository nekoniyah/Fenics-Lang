# Interpreter CLI

Fenics ships with a Rust interpreter. Build and run with Cargo.

## Build & Run

```
cd interpreter
cargo run -- ../samples/example.fenics
```

## Error Handling

- Try/catch constructs are supported in the language
- The VS Code extension also emits diagnostics for common issues:
  - Missing module on `import name`
  - Missing file on `import "path"`
  - Undeclared function in `lib` export block
  - Simple declaration hints (possible missing `:`)

## Search Paths for Imports

- Bare module names are resolved against standard locations:
  - `module.fenics`
  - `libs/module.fenics`
  - `../libs/module.fenics`
  - `samples/module.fenics`
  - `../samples/module.fenics`

## Modules and Method Calls

- When importing a module, its exported functions are callable via `module.fn(...)`
- Methods dispatch to functions stored inside module objects
