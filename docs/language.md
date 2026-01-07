# Language Guide

Fenics emphasizes readable blocks and simple declarations.

## Types

- Primitive: Int, Float, String, Boolean (Bool alias)
- Collections: Array, Object
- Generics: List(T), Pairs(K, V)
- Regex literals: `/pattern/flags`

## Variables

- Constant: `const name: value`
- Typed constant: `Int const count: 1`
- Mutable: `name: value`
- Typed mutable: `Int name: 0`
- Global: `global cfg:` followed by indented object entries

## Functions

```
fn add(a, b) -> Int:
    return a + b

fn log(message):
    print(message)
```

- Parameters are identifiers; return type is optional.

## Control Flow

```
if condition:
    // ...
else if other:
    // ...
else:
    // ...

for item in items:
    // ...

while condition:
    // ...

loop condition:
    // reactive loop body

try:
    // risky
catch (err)
    // recovery
```

## Expressions & Operators

- Arithmetic: `+ - * / % ** ^`, increment/decrement: `++ --`
- Comparison: `== != === !== < <= > >= ~ !~`
- Logical: `and or not is`
- Assignment: `:` and augmented `+: -: *: /: %:`
- Ternary: `if cond then A otherwise B` or `cond ? A : B`

## Strings & Interpolation

```
name: "Fenics"
msg: "Hello, #{name}!"
print(msg)
```

- Use `#{...}` inside double quotes; supports arbitrary expressions.

## Arrays & Objects

```
nums: [1, 2, 3]
user:
    - name: "Ada",
    - age: 36
```

- Arrays use `[ ... ]`
- Objects use block style with dashed entries under `identifier:`

## Ephemeral Variables

- Quick inline values for interpolation and expressions: `#name`, `#1`
- Example: `print("#{#greet}")` or `total: #1 + 2`

## Blocks & Effects

- `block expr` prevents further effects and short-circuits side effects in certain contexts.
