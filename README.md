# Nova

**Nova** (`.nv` / `.nova`) is an ultra-simple and powerful programming language, designed to be **much easier to learn than Python**, while being capable of doing **everything**.

## Current Status — v0.5

Working bootstrap interpreter with:

- `print`, variables, arithmetic, comparisons
- Parentheses for grouping: `(2 + 3) * 4`
- Logical operators: `and`, `or`, `not`
- Lists, Maps, indexing (`list[0]`) and index assignment (`list[0] = x`)
- `if` / `elif` / `else`, `for`, `while`, `break`
- Functions (`fun` + `return`)
- Inline comments (`x = 5  # comment`)
- String escapes (`\n`, `\t`, `\"`, `\\`)
- **From v0.4 (self-host foundation):**
  - `read_file(path)`
  - `len`, `str`, `int`, `float`
  - `append`, `split`, `join`
  - `type`

### v0.5 fixes

v0.4 had three real correctness bugs, now fixed:
- Parenthesized expressions crashed instead of grouping.
- Repeated/trailing unary minus (`5 - -3`) crashed with a TypeError.
- Bareword map keys (`{ x: 1 }`) were silently looked up as variables instead
  of used as the literal string key `"x"` — this could corrupt data with no
  error message if a variable named `x` already existed.

## Quick Start

```bash
cd interpreter
python nova.py ../examples/hello.nv
python nova.py ../examples/complete.nv
python nova.py ../examples/builtins.nv
python nova.py ../selfhost/echo_file.nv
```

## Road to Self-Hosting

See `selfhost/ROADMAP.md`.

We now have the basic tools (`read_file` + string/list helpers) needed to start writing a real interpreter **in Nova**.

## Repository Structure

- `SPEC.md` — Language specification
- `examples/` — Example programs
- `interpreter/` — Bootstrap interpreter (Python)
- `selfhost/` — Path to self-hosting

---

Created with help from Grok.
