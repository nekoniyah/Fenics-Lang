# Bridges (Native Modules)

Bridges let Fenics call native Rust functionality through a trait-based system. A bridge exposes methods that can be invoked like module methods.

## Filesystem Bridge (`fs`)

- `fs.read(path) -> String` — Read file contents
- `fs.exists(path) -> Boolean` — Return true if file exists
- `fs.write(path, content) -> Boolean` — Write content to file

## Usage

```
content: fs.read("./README.md")
print("Length: #{len(content)}")

ok: fs.write("./out.txt", "Hello")
print("Wrote? #{ok}")
```

## HTTP Bridge (`http`)

- `http.get(url) -> String` — Perform an HTTP GET and return response text
- `http.get_json(url) -> Object|Array|...` — GET and parse JSON into Fenics values
- `http.post(url, body) -> String` — POST raw string body and return response text

### Usage

```
text: http.get("https://example.com")
print(text)

data: http.get_json("https://httpbin.org/json")
print("Keys: #{len(data.keys())}")

resp: http.post("https://httpbin.org/post", "hello from fenics")
print(resp)
```

## Implementing a Bridge (Rust)

- Define a trait method set with `call(name, args)` semantics
- Register bridge in the interpreter so values like `fs` resolve to the native module
- Ensure argument evaluation happens before borrowing the bridge to avoid Rust borrow conflicts
