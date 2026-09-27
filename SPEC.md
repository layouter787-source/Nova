# Nova Language Specification (draft v0.2)

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
2. **Zero boilerplate** — as little code as possible to do useful things
3. **One language** for frontend, backend, AI and scripts
4. **Extreme readability**
5. **Optional and smart typing**

## File extensions

- `.nv`
- `.nova`

## Basic Syntax (proposal)

### Comments
```nova
# this is a comment
```

### Variables
No need for `let`, `var` or `const`.
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

### Functions
```nova
fun add(a, b)
  return a + b
end
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

### Data structures
```nova
list = [1, 2, 3]
map = { name: "Nova", version: 0.2 }
```

## Keywords (English)

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

## Next steps

- Finalize keywords and operators
- Type system
- Modules and imports
- AI / model training integration
- Runtime / interpreter

---

This is a living draft. Everything can change.