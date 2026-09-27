# Nova Language Specification (v0.5)

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

## Current Features (v0.5)

### Comments
```nova
# full-line comment
x = 5  # inline comment too
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

### Arithmetic, comparisons & grouping
```nova
x = (10 + 5) * 2
if x >= 20
  print "big"
end

y = 5 - -3   # unary minus works everywhere
```

### Logical operators
```nova
if age >= 18 and not is_banned
  print "welcome"
end

if role == "admin" or role == "owner"
  print "has access"
end
```

### Strings
```nova
print "line one\nline two\ttabbed"
```

### Lists
```nova
numbers = [1, 2, 3, 4]
numbers[0] = 99
print numbers
```

### Maps
```nova
person = { name: "Nova", version: 0.5 }
person["version"] = 0.6
print person
```
Bareword keys (`name:`) are always literal string keys — they are never
looked up as variables, even if a variable with that name exists.

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
elif age == 18
  print "Just turned adult"
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
| `elif`    | Else-if branch       |
| `else`    | Else branch          |
| `for`     | For loop             |
| `from`    | Range start          |
| `to`      | Range end            |
| `while`   | While loop           |
| `break`   | Exit loop            |
| `true`    | Boolean true         |
| `false`   | Boolean false        |
| `and`     | Logical AND          |
| `or`      | Logical OR           |
| `not`     | Logical NOT          |

## How to run

```bash
cd interpreter
python nova.py ../examples/complete.nv
```

## Next planned features

- String interpolation
- Modules / import
- Classes / objects
- Native AI training helpers
- Web server & UI primitives
- Compiled backend (future)

---

This is a living draft.
