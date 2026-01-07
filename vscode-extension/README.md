# Fenics Language Support for VS Code

Adds syntax highlighting, snippets, and IntelliSense support for the Fenics programming language.

## Features

### Syntax Highlighting

Full syntax highlighting support for Fenics including:

- Keywords (`if`, `else`, `for`, `while`, `loop`, `fn`, `return`, etc.)
- Types (`Int`, `Float`, `String`, `Boolean`, `List`, `Pairs`, etc.)
- Storage modifiers (`const`, `mut`, `global`)
- Built-in functions (`print`, `input`, `len`, `type`, etc.)
- String interpolation (`#{}`)
- Comments (`//`)
- Operators and literals
- Ephemeral variables (`#1`, `#tempVar`)

### Language Configuration

- Auto-closing brackets, parentheses, and quotes
- Auto-indentation after colons (`:`)
- Comment toggling with `//`
- Bracket matching
- Code folding support

### Snippets

Quick snippets for common Fenics constructs:

- `const` - Constant variable
- `var` - Mutable variable
- `fn` - Function definition
- `for` - For loop
- `while` - While loop
- `if` - If statement
- `ifelse` - If-else statement
- `try` - Try-catch block
- `print` - Print statement
- `arr` - Array literal
- `obj` - Object literal
- And many more...

Type the prefix and press `Tab` to expand the snippet.

## Installation

1. Copy the `vscode-extension` folder contents
2. Open VS Code
3. Press `F5` to open Extension Development Host
4. Or package and install: `vsce package` then install the `.vsix` file

## Quick Start

Create a file with `.fenics` extension and start coding:

```fenics
// Define variables
const greeting: "Hello, World!"
counter: 0  // mutable by default

// Create a function
fn greet(name: String) -> String:
    return "Hello, #{name}!"

// Call the function
print(greet("Alice"))

// Control flow
if counter < 10:
    print("Counter is less than 10")
    counter++
```

## Language Features

### Variables

- Immutable with `const` keyword
- Mutable by default (just `identifier: value`)
- Optional `mut` keyword for clarity
- Global scope with `global` keyword
- Type annotations supported

### Functions

- Define with `fn` keyword
- Optional return types
- Parameter type annotations

### Control Flow

- `if`, `else`, `else if` statements
- Ternary operators: `if...then...otherwise` or `? :`
- `for` loops with key-value iteration
- `while` loops
- `loop` statements

### Data Types

- Primitives: `Int`, `Float`, `String`, `Boolean`
- Collections: `Array`, `List(T)`, `Pairs(K, V)`
- Special values: `null`, `undefined`, `nil`, `NaN`

### String Interpolation

```fenics
name: "World"
print("Hello, #{name}!")
```

### Error Handling

```fenics
try:
    riskyOperation()
catch (err)
    print("Error: #{err.message}")
```

## Contributing

Found a bug or want to contribute? Visit the [repository](#) to report issues or submit pull requests.

## License

MIT
