# Modules and Imports

Fenics supports exporting functions from a file as a module and importing them elsewhere.

## Exporting

```
fn greet(name):
    print("Hello, #{name}!")

fn add(a, b) -> Int:
    return a + b

lib mylib:
    - greet,
    - add
```

- `lib moduleName:` starts an export block.
- Each `- identifier` exports a function from the same file.

## Importing

- By name (pathless):

```
import mylib
print(mylib.add(10, 20))
```

- By path with optional alias:

```
import "../samples/mylib.fenics" as m
m.greet("World")
```

## Pathless Import Resolution

Bare `import name` searches in order:

1. `name.fenics` in the current directory
2. `libs/name.fenics`
3. `../libs/name.fenics`
4. `samples/name.fenics`
5. `../samples/name.fenics`

## Calling Methods

Imported modules expose exported functions as methods:

```
mylib.greet("World")
result: mylib.add(2, 3)
```

## Aliases

Use `as` to create shorter names:

```
import mylib as m
m.add(2, 3)
```
