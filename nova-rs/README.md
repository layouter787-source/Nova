# Nova (Rust implementation)

The native implementation of the Nova language. The Python interpreter in
`../interpreter` stays as the reference/bootstrap prototype.

## Build and test

```
cd nova-rs
cargo test
cargo build --release
./target/release/nova ../examples/complete.nv
```

## Architecture

- `lexer.rs`  source text -> tokens (with line numbers, escapes, multi-line literals)
- `parser.rs` tokens -> AST, parsed **once** (the Python prototype re-parsed loop bodies on every iteration)
- `interp.rs` tree-walking evaluator, values, builtins
- `error.rs`  errors with line numbers

No external dependencies.

## Language additions over the Python prototype

`and` / `or` / `not`, `else if`, string escapes (`\n \t \" \\`), negative indices,
`write_file`, call-depth limit instead of a crash.

`to` and `from` are now reserved words (used by `for i from 1 to 10`).

## Known limitations (next steps)

- Maps use linear lookup (insertion ordered); needs a real hash map.
- `s[i]` on strings is O(n) (UTF-8 char indexing).
- Tree-walking evaluator; the next step is a bytecode VM.
- No modules/imports, classes/structs, or closures yet.
- Self-hosting: rewrite lexer/parser/interpreter in Nova using this as the seed compiler.
