# Nova Interpreter (Bootstrap)

This is the first bootstrap interpreter for the Nova language.

It is written in Python and currently supports a very small subset of the language:

- Comments (`#`)
- Variables
- `print`
- Basic arithmetic (`+`, `-`, `*`, `/`)
- Simple expressions

## How to run

```bash
python nova.py examples/hello.nv
```

or

```bash
python nova.py path/to/your/file.nv
```

## Current limitations

- No functions yet
- No if / for / while yet
- No lists or maps yet
- Very basic error handling

This is intentionally minimal so we can grow the language step by step.