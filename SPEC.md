# Nova Language Specification (v0.3)

## Vision

Nova is an ultra-simple, full-stack, high-productivity programming language.
It is designed to be **significantly easier to learn and use than Python**, while remaining powerful enough for:

- Training AI models
- Building websites and web apps
- Desktop programs
- Scripts and automation
- Anything else

## Design Principles

1. **Minimum keywords and symbols**
2. **Zero boilerplate**
3. **One language** for frontend, backend, AI and scripts
4. **Extreme readability**
5. **Optional and smart typing** (future)

## File extensions

- `.nv`
- `.nova`

## Current Features (v0.3)

### Comments
```nova
# this is a comment
```

### Variables
```nova
name = "Nova"
age = 1
native = true
```

### Output
```nova
print "Hello, world!"
print name
```

### Arithmetic & Comparisons
```nova
x = 10 + 5 * 2
if x >= 20
  print "big"
end
```

### Lists
```nova
numbers = [1, 2, 3, 4]
print numbers
```

### Maps
```nova
person = { name: "Nova", version: 0.3 }
print person
```

### Functions
```nova
fun add(a, b)
  return a + b
end

result = add(3, 4)
print result
```

### Conditionals
```nova
if age > 18
  print "Adult"
else
  print "Minor"
end
```

### Loops
```nova
for i from 1 to 10
  print i
end

while true
  print "loop"
  break
end
```

## Keywords

| Keyword   | Purpose              |
|-----------|----------------------|
| `print`   | Output               |
| `fun`     | Define function      |
| `return`  | Return value         |
| `end`     | End block            |
| `if`      | Conditional          |
| `else`    | Else branch          |
| `for`     | For loop             |
| `from`    | Range start          |
| `to`      | Range end            |
| `while`   | While loop           |
| `break`   | Exit loop            |
| `true`    | Boolean true         |
| `false`   | Boolean false        |

## How to run

```bash
cd interpreter
python nova.py ../examples/complete.nv
```

## Next planned features

- Better operator precedence & parentheses
- String interpolation
- Modules / import
- Classes / objects
- Native AI training helpers
- Web server & UI primitives
- Compiled backend (future)

---

This is a living draft.