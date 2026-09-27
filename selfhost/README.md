# Nova Self-Hosting

Goal: write the Nova interpreter **in Nova itself**.

## Roadmap

1. **Bootstrap (Python)** — current `interpreter/nova.py` (v0.3)
2. **Nova interpreter written in Nova** (this folder)
3. Run the Nova-written interpreter using the Python bootstrap
4. Improve until the Nova interpreter can run itself → **self-host**

## Current status

Very early. We are starting with a minimal evaluator written in Nova.

## How it will work later

```bash
# Run the Nova interpreter (written in Nova) using the Python bootstrap
python ../interpreter/nova.py nova_interpreter.nv your_program.nv
```

Eventually:

```bash
# Once self-hosted
./nova your_program.nv
```
