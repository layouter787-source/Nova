# Nova Interpreter (v0.3)

Bootstrap interpreter for the Nova language written in Python.

## Supported Features

- Comments (`#`)
- Variables
- `print`
- Arithmetic (`+ - * /`) and comparisons (`== != < > <= >=`)
- Lists `[1, 2, 3]`
- Maps `{ key: value }`
- `if` / `else` / `end`
- `for i from 1 to 10` / `end`
- `while condition` / `end`
- `break`
- Functions (`fun name(params)` ... `return` ... `end`)

## How to run

```bash
python nova.py ../examples/hello.nv
python nova.py ../examples/variables.nv
python nova.py ../examples/complete.nv
```

## Notes

This is still a bootstrap implementation. The goal is to keep growing the language while keeping the interpreter simple and easy to understand.

Next improvements planned:
- Proper operator precedence and parentheses
- Better error messages with context
- String interpolation
- Modules