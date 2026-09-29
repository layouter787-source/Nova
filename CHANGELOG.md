# Changelog

## nova-rs v0.7.0

Methods on structs.

Added:
- `fun Struct.method(self, ...) ... end` declares a method on `Struct`.
  The receiver is passed as the method's first parameter (call it whatever
  you like; `self` is just a convention).
- `p.method(args)` calls it, resolved by the struct's own name at call
  time — so `p.increment()` only looks at `Counter`'s methods if `p` is a
  `Counter`.
- Methods can mutate the receiver's fields (`self.value = self.value + 1`)
  since struct values are reference-counted, so the caller sees the change.
- Calling a method on a non-struct value, or a method the struct doesn't
  have, is a clear runtime error instead of silently doing nothing.

## nova-rs v0.6.0

Structs and modules, on top of the Rust rewrite.

Added:
- `struct Name \n field \n ... \n end` plus positional constructors: `p = Point(1, 2)`.
- Field access and assignment: `p.x`, `p.x = 9`.
- Structural equality for structs (`a == b` compares field values, not identity).
- `import "path.nv"` to pull functions/structs from another file into the
  global scope. Resolved relative to the importing file's directory,
  idempotent (importing the same file twice is a no-op), and circular
  imports are reported as an error instead of hanging or crashing.
- `type(x)` returns the struct's own name for struct values.

## nova-rs v0.5.0 (first Rust implementation)

Full rewrite of the interpreter in Rust (`nova-rs/`), replacing the
Python prototype's naive string-splitting evaluator with a real
lexer + parser producing an AST once (loops no longer re-parse their
body every iteration).

Added over the Python prototype:
- `and` / `or` / `not`, `else if`.
- String escapes (`\n`, `\t`, `\r`, `\"`, `\\`).
- Negative list/string indices (`a[-1]`).
- `write_file(path, content)`.
- A call-depth limit (clean error instead of a stack overflow).
- Integer overflow, NaN, and division-by-zero are reported as errors
  instead of silently producing wrong results or panicking.

Notes:
- `to` and `from` are now reserved words (used by `for i from 1 to 10`).
- The Python interpreter (`interpreter/nova.py`) remains the reference
  bootstrap prototype; it does not implement structs or imports.

## v0.5 (Python bootstrap interpreter)

Correctness pass on the bootstrap interpreter. The expression evaluator was rewritten around a real tokenizer and recursive-descent parser.

Fixed:
- Parentheses for grouping (`(2 + 3) * 4`) now work.
- Unary minus (`-x`, `3 * -2`) now works.
- Operator precedence is now correct.
- Nested indexing (`matrix[i][j]`) works.
- Index assignment (`list[0] = 5`, `matrix[i][j] = 1`) is now supported.
- `print` and `str()` render `true`/`false`/`null` instead of Python's `True`/`False`/`None`.
- `"score: " + 5` concatenates instead of crashing.
- Blocks missing `end` report which block was left open.

## v0.4

File I/O, better lists/strings, indexing and more builtins.
