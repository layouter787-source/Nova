# Roadmap to Self-Hosting Nova

## Phase 0 — Current (done)
- Python bootstrap interpreter (v0.3)
- Basic language features working

## Phase 1 — Language improvements needed for self-hosting
We need to add to the Python bootstrap:

- [ ] File reading (`read_file "path.nv"`)
- [ ] String operations (split, length, substring, etc.)
- [ ] Better lists (append, index access, length)
- [ ] Better maps
- [ ] Parentheses and proper operator precedence
- [ ] String interpolation or concatenation helpers

## Phase 2 — Write a real interpreter in Nova
- Lexer (turn source text into tokens)
- Parser (turn tokens into a simple AST or directly execute)
- Evaluator
- Ability to run simple Nova programs from inside Nova

## Phase 3 — Self-host
- The Nova interpreter can run itself
- We can stop depending on the Python version for development

## Phase 4 — Beyond
- Compile to native or bytecode
- Performance improvements
- Standard library

---

We move step by step. No magic jumps.
