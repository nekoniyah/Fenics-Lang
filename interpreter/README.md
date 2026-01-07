# Fenics Interpreter

A Rust-based interpreter for the Fenics programming language.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -- path/to/file.fenics
```

Or after building:

```bash
./target/release/fenics-interpreter path/to/file.fenics
```

## Example

```bash
cargo run -- ../samples/example.fenics
```

## Features Implemented

- ✅ Variable declarations (const and mutable)
- ✅ Function declarations and calls
- ✅ Control flow (if/else, for, while, loop)
- ✅ Built-in functions (print, len)
- ✅ Built-in methods (reverse, has, split, keys)
- ✅ Property access (.length, .first, .last)
- ✅ Array and object literals
- ✅ Try-catch error handling
- ✅ Ternary operators
- ⏳ String interpolation (partial)
- ⏳ Binary/unary operators (partial)
- ⏳ Ephemeral variables
- ⏳ All built-in methods

## Structure

- `src/ast.rs` - Abstract Syntax Tree definitions
- `src/parser.rs` - Pest-based parser
- `src/interpreter.rs` - Tree-walking interpreter/evaluator
- `src/main.rs` - Entry point

## Grammar

The grammar is defined in `../grammar/fenics.pest` using the Pest parser generator.
