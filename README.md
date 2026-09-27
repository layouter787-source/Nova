# Nova

**Nova** (extension `.nv` / `.nova`) is an ultra-simple and powerful programming language, designed to be **much easier to learn than Python**, while being capable of doing **everything**:

- Train AI models
- Create websites and full-stack applications
- Desktop programs and scripts
- Anything else

## Main Goals

- Extremely simple and readable syntax
- Zero boilerplate
- One single language for frontend, backend, AI and scripts
- High productivity with minimal code
- Focus on reducing programming time and AI training costs

## Current Status

Early design + first bootstrap interpreter.

**Keywords are in English.**

### What works right now

- `print`
- Variables
- Basic arithmetic (`+ - * /`)
- Comments

### How to run examples

```bash
cd interpreter
python nova.py ../examples/hello.nv
python nova.py ../examples/variables.nv
```

## Repository Structure

- `SPEC.md` — Language specification (living draft)
- `examples/` — Example programs
- `interpreter/` — Bootstrap interpreter written in Python

## Next planned features

- `if` / `else`
- `for` / `while`
- Functions (`fun`)
- Lists and maps
- Better error messages

---

Created with help from Grok.