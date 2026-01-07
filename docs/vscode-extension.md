# VS Code Extension

The Fenics VS Code extension provides syntax support, code completion, hovers, and diagnostics.

## Installation

- Install VSIX: `vscode-extension/fenics-lang-0.1.0.vsix`

```
code --install-extension "c:\\Users\\noebo\\Documents\\Dev\\Fenics Lang\\vscode-extension\\fenics-lang-0.1.0.vsix"
```

## Syntax Highlighting

- Keywords: `if`, `else`, `for`, `while`, `loop`, `return`, `try`, `catch`, `then`, `otherwise`, `in`, `block`, `lib`, `import`, `as`
- Operators: assignment `:`, augmented `+: -: *: /: %:`, arithmetic, comparisons, ternary `?:`
- Strings and regex literals; string interpolation blocks `#{...}`

## Snippets

- Functions, variables, typed declarations, loops, control flow
- Library export block and imports (`import`, `importas`, `importpath`)

## Completions

- Keywords, builtins, types
- After `import `, module names discovered in the workspace
- After `module.`, methods for exported functions

## Hovers

- Builtins: docs + signature
- Variables: declared type + implicit type, value preview, object keys
- Functions: signature and declaration line
- Import alias: maps alias to module and shows its exports
- Ephemeral variables: `#name` hover

## Diagnostics

- Import by path not found
- Missing module for bare `import name`
- `lib` export entries that don’t match declared functions
- Declaration hints for likely missing colon

## Command

- `Fenics: Refresh Module Index` — Re-scan `.fenics` files to rebuild the module/export index
