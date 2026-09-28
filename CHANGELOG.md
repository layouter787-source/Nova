# Changelog

## v0.5

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
