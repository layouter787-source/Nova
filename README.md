# Nova

**Nova** (`.nv` / `.nova`) is an ultra-simple and powerful programming language, designed to be **much easier to learn than Python**, while being capable of doing **everything**.

## Current Status — v0.4

Working bootstrap interpreter with:

- `print`, variables, arithmetic, comparisons
- Lists, Maps, indexing (`list[0]`)
- `if` / `else`, `for`, `while`, `break`
- Functions (`fun` + `return`)
- **New in v0.4 (self-host foundation):**
  - `read_file(path)`
  - `len`, `str`, `int`, `float`
  - `append`, `split`, `join`
  - `type`

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