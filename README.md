# Nova

**Nova** (extension `.nv` / `.nova`) is an ultra-simple and powerful programming language, designed to be **much easier to learn than Python**, while being capable of doing **everything**:

- Train AI models
- Create websites and full-stack applications
- Desktop programs and scripts
- Anything else

## Current Status — v0.3

Working bootstrap interpreter with the following features:

- `print`
- Variables
- Arithmetic (`+ - * /`) and comparisons
- Lists and Maps
- `if` / `else`
- `for` / `while` + `break`
- Functions (`fun` + `return`)

## Quick Start

```bash
cd interpreter
python nova.py ../examples/hello.nv
python nova.py ../examples/variables.nv
python nova.py ../examples/complete.nv
```

## Repository Structure

- `SPEC.md` — Language specification
- `examples/` — Example programs
- `interpreter/` — Bootstrap interpreter (Python)

## Example

```nova
fun greet(name)
  print "Hello,"
  print name
end

for i from 1 to 3
  greet("Nova")
end

if 10 > 5
  print "Math works"
end
```

## Next Goals

- Better expression parsing (parentheses, precedence)
- String interpolation
- Modules
- Native support for AI training pipelines
- Web & UI primitives

---

Created with help from Grok.