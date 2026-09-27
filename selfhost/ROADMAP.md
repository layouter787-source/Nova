# Roadmap to Self-Hosting Nova

## Phase 0 — Done
- Python bootstrap interpreter

## Phase 1 — Language improvements (mostly done in v0.4)
- [x] `read_file(path)`
- [x] `len`
- [x] Indexing `list[index]`
- [x] `append`, `split`, `join`
- [x] `str`, `int`, `float`, `type`
- [ ] Better parentheses support
- [ ] String interpolation (nice to have)

## Phase 2 — Write a real interpreter in Nova (current focus)
- [ ] Lexer (source → tokens)
- [ ] Parser / direct executor
- [ ] Ability to run simple Nova programs from Nova
- [ ] Grow feature coverage

## Phase 3 — Self-host
- The Nova interpreter can run itself
- Drop dependency on the Python version for daily development

## Phase 4 — Beyond
- Native / bytecode backend
- Standard library
- Package system

---

We move step by step.
