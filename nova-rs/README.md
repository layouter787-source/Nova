# Nova (Rust implementation)

The native implementation of the Nova language. The Python interpreter in
`../interpreter` stays as the reference/bootstrap prototype (no structs, no
imports, no methods).

## Build and test

```
cd nova-rs
cargo test
cargo build --release
./target/release/nova ../examples/complete.nv
./target/release/nova ../examples/rust-only/methods.nv
```

## Architecture

- `lexer.rs`  source text -> tokens (with line numbers, escapes, multi-line literals)
- `parser.rs` tokens -> AST, parsed **once** (the Python prototype re-parsed loop bodies on every iteration)
- `interp.rs` tree-walking evaluator, values, builtins, structs, methods, imports
- `error.rs`  errors with line numbers

No external dependencies.

## Language additions over the Python prototype

`and` / `or` / `not`, `else if`, string escapes (`\n \t \" \\`), negative indices,
`write_file`, call-depth limit instead of a crash, `struct`/field access
(`p.x`), methods (`fun Struct.method(self, ...)` + `p.method(...)`),
`import "file.nv"`.

`to`, `from`, `struct` and `import` are now reserved words.

## Structs

```
struct Point
  x
  y
end

p = Point(1, 2)
print p.x        # 1
p.x = 9
print p           # Point { x: 9, y: 2 }
print type(p)      # Point
```

Fields are positional at construction time, ordered as declared. Equality
(`==`) is structural: same struct name and equal field values.

## Methods

```
fun Point.length_sq(self)
  return self.x * self.x + self.y * self.y
end

p = Point(3, 4)
print p.length_sq()   # 25
```

The receiver is just the method's first parameter — call it `self` by
convention, Nova doesn't require the name. Methods are resolved by the
receiver's *own* struct name at call time, so `a.foo()` only finds
methods declared for `a`'s struct. Since struct values are reference-counted,
a method that assigns to `self.field` mutates the caller's value — there's
no need to `return self` just to see the change (though it's handy for
chaining, as in `examples/rust-only/methods.nv`).

## Modules

```
import "geometry.nv"
```

Resolved relative to the *importing file's* directory. Runs the target
file once at global scope (its top-level `fun`/`struct`/variable
definitions land in the current program's globals); importing the same
file again is a no-op. A cycle (`a.nv` imports `b.nv` imports `a.nv`)
is reported as an error instead of hanging. There is no namespacing yet —
imported names go straight into the global scope, so avoid name clashes
between files for now.

## Known limitations (next steps)

- Maps use linear lookup (insertion ordered); needs a real hash map.
- `s[i]` on strings is O(n) (UTF-8 char indexing).
- Tree-walking evaluator; the next step is a bytecode VM.
- No closures, no inheritance/traits between structs.
- No import namespacing (`import "x.nv" as x`) or selective imports yet.
- Self-hosting: rewrite lexer/parser/interpreter in Nova using this as the seed compiler.
