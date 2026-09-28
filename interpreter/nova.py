#!/usr/bin/env python3
"""
Nova Language - Bootstrap Interpreter (v0.5)

Changelog v0.5 (correctness pass):
  - Real tokenizer + recursive-descent parser for expressions.
    Fixes several bugs from v0.4:
      * Parentheses for grouping now actually work: (2 + 3) * 4
      * Unary minus now works: -x, 3 * -2, -f(x)
      * Correct operator precedence: == != <= >= < >  <  + -  <  * /
      * Chained/nested indexing works: matrix[i][j]
      * Index assignment now works: list[0] = 5, matrix[i][j] = 1
  - print now renders Nova values (true/false/null) instead of Python's
    True/False/None.
  - str(x) uses the same Nova-style rendering.
  - '+' with a string operand now does string concatenation
    (e.g. "score: " + 5 works, instead of crashing).
"""

import sys
import re
from typing import Any, List, Dict, Tuple, Optional


class ReturnValue(Exception):
    def __init__(self, value):
        self.value = value


class BreakException(Exception):
    pass


class NovaError(Exception):
    pass


# ───────────────────────── Tokenizer ─────────────────────────

TOKEN_SPEC = [
    ("FLOAT", r"\d+\.\d+"),
    ("INT", r"\d+"),
    ("STRING", r'"[^"]*"|\'[^\']*\''),
    ("EQEQ", r"=="),
    ("NEQ", r"!="),
    ("LE", r"<="),
    ("GE", r">="),
    ("LT", r"<"),
    ("GT", r">"),
    ("PLUS", r"\+"),
    ("MINUS", r"-"),
    ("STAR", r"\*"),
    ("SLASH", r"/"),
    ("LPAREN", r"\("),
    ("RPAREN", r"\)"),
    ("LBRACK", r"\["),
    ("RBRACK", r"\]"),
    ("LBRACE", r"\{"),
    ("RBRACE", r"\}"),
    ("COMMA", r","),
    ("COLON", r":"),
    ("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*"),
    ("SKIP", r"\s+"),
    ("MISMATCH", r"."),
]

MASTER_REGEX = re.compile("|".join(f"(?P<{name}>{pattern})" for name, pattern in TOKEN_SPEC))


class Tokenizer:
    def __init__(self, text: str):
        self.text = text

    def tokenize(self) -> List[Tuple[str, Any]]:
        tokens = []
        for m in MASTER_REGEX.finditer(self.text):
            kind = m.lastgroup
            value: Any = m.group()
            if kind == "SKIP":
                continue
            if kind == "MISMATCH":
                raise NovaError(f"Unexpected character: {value!r}")
            if kind == "STRING":
                value = value[1:-1]
            elif kind == "INT":
                value = int(value)
            elif kind == "FLOAT":
                value = float(value)
            tokens.append((kind, value))
        return tokens


# ───────────────────────── Parser ─────────────────────────
# Produces a small tuple-based AST:
#   ('num', v) ('str', v) ('bool', v)
#   ('list', [nodes]) ('map', [(keynode, valnode), ...])
#   ('var', name)
#   ('index', containernode, indexnode)
#   ('call', name, [argnodes])
#   ('unary', 'MINUS', node)
#   ('binop', kind, left, right)

CMP_OPS = ("EQEQ", "NEQ", "LE", "GE", "LT", "GT")
ADD_OPS = ("PLUS", "MINUS")
MUL_OPS = ("STAR", "SLASH")


class Parser:
    def __init__(self, tokens: List[Tuple[str, Any]]):
        self.tokens = tokens
        self.pos = 0

    def peek(self) -> Tuple[Optional[str], Any]:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return (None, None)

    def advance(self) -> Tuple[Optional[str], Any]:
        tok = self.peek()
        self.pos += 1
        return tok

    def at_end(self) -> bool:
        return self.pos >= len(self.tokens)

    def expect(self, kind: str) -> Tuple[str, Any]:
        tok = self.peek()
        if tok[0] != kind:
            raise NovaError(f"Expected {kind} but got {tok[0]!r}")
        return self.advance()

    def parse_expression(self):
        return self.parse_equality()

    def parse_equality(self):
        left = self.parse_additive()
        while self.peek()[0] in CMP_OPS:
            op, _ = self.advance()
            right = self.parse_additive()
            left = ("binop", op, left, right)
        return left

    def parse_additive(self):
        left = self.parse_multiplicative()
        while self.peek()[0] in ADD_OPS:
            op, _ = self.advance()
            right = self.parse_multiplicative()
            left = ("binop", op, left, right)
        return left

    def parse_multiplicative(self):
        left = self.parse_unary()
        while self.peek()[0] in MUL_OPS:
            op, _ = self.advance()
            right = self.parse_unary()
            left = ("binop", op, left, right)
        return left

    def parse_unary(self):
        if self.peek()[0] == "MINUS":
            self.advance()
            node = self.parse_unary()
            return ("unary", "MINUS", node)
        return self.parse_postfix()

    def parse_postfix(self):
        node = self.parse_primary()
        while self.peek()[0] == "LBRACK":
            self.advance()
            idx = self.parse_expression()
            self.expect("RBRACK")
            node = ("index", node, idx)
        return node

    def parse_primary(self):
        kind, value = self.peek()

        if kind in ("INT", "FLOAT"):
            self.advance()
            return ("num", value)

        if kind == "STRING":
            self.advance()
            return ("str", value)

        if kind == "IDENT":
            self.advance()
            if value == "true":
                return ("bool", True)
            if value == "false":
                return ("bool", False)
            if self.peek()[0] == "LPAREN":
                self.advance()
                args = []
                if self.peek()[0] != "RPAREN":
                    args.append(self.parse_expression())
                    while self.peek()[0] == "COMMA":
                        self.advance()
                        args.append(self.parse_expression())
                self.expect("RPAREN")
                return ("call", value, args)
            return ("var", value)

        if kind == "LPAREN":
            self.advance()
            node = self.parse_expression()
            self.expect("RPAREN")
            return node

        if kind == "LBRACK":
            self.advance()
            items = []
            if self.peek()[0] != "RBRACK":
                items.append(self.parse_expression())
                while self.peek()[0] == "COMMA":
                    self.advance()
                    items.append(self.parse_expression())
            self.expect("RBRACK")
            return ("list", items)

        if kind == "LBRACE":
            self.advance()
            pairs = []
            if self.peek()[0] != "RBRACE":
                pairs.append(self.parse_pair())
                while self.peek()[0] == "COMMA":
                    self.advance()
                    pairs.append(self.parse_pair())
            self.expect("RBRACE")
            return ("map", pairs)

        raise NovaError(f"Unexpected token: {kind} {value!r}")

    def parse_pair(self):
        kind, value = self.peek()
        if kind == "IDENT":
            self.advance()
            key_node = ("str", value)
        else:
            key_node = self.parse_expression()
        self.expect("COLON")
        val_node = self.parse_expression()
        return (key_node, val_node)


# ───────────────────────── Interpreter ─────────────────────────

class NovaInterpreter:
    def __init__(self):
        self.variables: Dict[str, Any] = {}
        self.functions: Dict[str, Tuple[List[str], List[str]]] = {}
        self.line_number = 0

        self.builtins = {
            "read_file": self.builtin_read_file,
            "len": self.builtin_len,
            "str": self.builtin_str,
            "int": self.builtin_int,
            "float": self.builtin_float,
            "append": self.builtin_append,
            "split": self.builtin_split,
            "join": self.builtin_join,
            "type": self.builtin_type,
        }

    def run_file(self, path: str):
        with open(path, "r", encoding="utf-8") as f:
            source = f.read()
        self.run(source)

    def run(self, source: str):
        lines = source.splitlines()
        self.execute_block(lines, 0, len(lines))

    # ── Block execution / control flow ──────────────────────

    def execute_block(self, lines: List[str], start: int, end: int) -> int:
        i = start
        while i < end:
            raw = lines[i]
            line = raw.strip()
            self.line_number = i + 1

            if not line or line.startswith("#"):
                i += 1
                continue

            try:
                if line.startswith("fun "):
                    i = self.parse_function(lines, i, end)
                    continue
                if line.startswith("if "):
                    i = self.parse_if(lines, i, end)
                    continue
                if line.startswith("for "):
                    i = self.parse_for(lines, i, end)
                    continue
                if line.startswith("while "):
                    i = self.parse_while(lines, i, end)
                    continue
                if line.startswith("return"):
                    expr = line[6:].strip()
                    value = self.evaluate(expr) if expr else None
                    raise ReturnValue(value)
                if line == "break":
                    raise BreakException()
                if line == "end":
                    return i + 1

                self.execute_statement(line)
                i += 1

            except (ReturnValue, BreakException):
                raise
            except Exception as e:
                print(f"Error on line {self.line_number}: {e}")
                print(f"  --> {raw}")
                sys.exit(1)
        return end

    def parse_function(self, lines, start, end):
        header = lines[start].strip()
        match = re.match(r"fun\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)", header)
        if not match:
            raise NovaError(f"Invalid function definition: {header}")
        name = match.group(1)
        params_str = match.group(2).strip()
        params = [p.strip() for p in params_str.split(",") if p.strip()] if params_str else []

        body, i = self.collect_block(lines, start, end, name)
        self.functions[name] = (params, body)
        return i + 1

    def parse_if(self, lines, start, end):
        condition = lines[start].strip()[3:].strip()
        cond_value = self.evaluate(condition)

        then_block, else_block = [], []
        i = start + 1
        depth = 1
        in_else = False
        while i < end:
            line = lines[i].strip()
            if line.startswith(("fun ", "if ", "for ", "while ")):
                depth += 1
            elif line == "else" and depth == 1:
                in_else = True
                i += 1
                continue
            elif line == "end":
                depth -= 1
                if depth == 0:
                    break
            if in_else:
                else_block.append(lines[i])
            else:
                then_block.append(lines[i])
            i += 1
        if depth != 0:
            raise NovaError("if statement is missing 'end'")

        if cond_value:
            self.execute_block(then_block, 0, len(then_block))
        else:
            self.execute_block(else_block, 0, len(else_block))
        return i + 1

    def parse_for(self, lines, start, end):
        header = lines[start].strip()
        match = re.match(r"for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+from\s+(.+)\s+to\s+(.+)", header)
        if not match:
            raise NovaError(f"Invalid for loop: {header}")
        var_name = match.group(1)
        start_val = self.evaluate(match.group(2).strip())
        end_val = self.evaluate(match.group(3).strip())
        if not isinstance(start_val, int) or not isinstance(end_val, int):
            raise NovaError("for..from..to bounds must be integers")

        body, i = self.collect_block(lines, start, end, "for")

        for val in range(start_val, end_val + 1):
            self.variables[var_name] = val
            try:
                self.execute_block(body, 0, len(body))
            except BreakException:
                break
        return i + 1

    def parse_while(self, lines, start, end):
        condition = lines[start].strip()[6:].strip()
        body, i = self.collect_block(lines, start, end, "while")

        while self.evaluate(condition):
            try:
                self.execute_block(body, 0, len(body))
            except BreakException:
                break
        return i + 1

    def collect_block(self, lines, start, end, label):
        """Collect the raw lines of a block until the matching 'end'."""
        body = []
        i = start + 1
        depth = 1
        while i < end:
            line = lines[i].strip()
            if line.startswith(("fun ", "if ", "for ", "while ")):
                depth += 1
            elif line == "end":
                depth -= 1
                if depth == 0:
                    return body, i
            body.append(lines[i])
            i += 1
        raise NovaError(f"'{label}' block is missing 'end'")

    # ── Statements ───────────────────────────────────────────

    def execute_statement(self, line: str):
        if line.startswith("print "):
            value = self.evaluate(line[6:].strip())
            print(self.stringify(value))
            return

        eq_idx = self.find_assignment_index(line)
        if eq_idx is not None:
            target_str = line[:eq_idx].strip()
            expr_str = line[eq_idx + 1:].strip()
            if not target_str or not expr_str:
                raise NovaError(f"Invalid assignment: {line}")
            target_node = self.parse(target_str)
            value = self.evaluate(expr_str)
            self.assign_to(target_node, value)
            return

        # Bare expression statement (function call, etc.)
        self.evaluate(line)

    def find_assignment_index(self, line: str) -> Optional[int]:
        """Find the position of a top-level '=' that is a plain assignment
        (not part of ==, !=, <=, >=), skipping over strings/brackets."""
        depth = 0
        in_string = False
        string_char = None
        i = 0
        n = len(line)
        while i < n:
            c = line[i]
            if in_string:
                if c == string_char:
                    in_string = False
            elif c in "\"'":
                in_string = True
                string_char = c
            elif c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
            elif c == "=" and depth == 0:
                prev_c = line[i - 1] if i > 0 else ""
                next_c = line[i + 1] if i + 1 < n else ""
                if prev_c not in "!<>=" and next_c != "=":
                    return i
            i += 1
        return None

    def assign_to(self, node, value):
        if node[0] == "var":
            self.variables[node[1]] = value
            return
        if node[0] == "index":
            container = self.eval_node(node[1])
            index = self.eval_node(node[2])
            try:
                container[index] = value
            except Exception:
                raise NovaError(f"Cannot assign to index {index} of {self.stringify(container)}")
            return
        raise NovaError("Invalid assignment target")

    # ── Expressions ──────────────────────────────────────────

    def parse(self, expr: str):
        tokens = Tokenizer(expr).tokenize()
        parser = Parser(tokens)
        node = parser.parse_expression()
        if not parser.at_end():
            raise NovaError(f"Unexpected token near: {parser.peek()}")
        return node

    def evaluate(self, expr: str) -> Any:
        expr = expr.strip()
        if not expr:
            return None
        node = self.parse(expr)
        return self.eval_node(node)

    def eval_node(self, node) -> Any:
        kind = node[0]

        if kind == "num":
            return node[1]
        if kind == "str":
            return node[1]
        if kind == "bool":
            return node[1]
        if kind == "list":
            return [self.eval_node(n) for n in node[1]]
        if kind == "map":
            return {self.eval_node(k): self.eval_node(v) for k, v in node[1]}
        if kind == "var":
            name = node[1]
            if name in self.variables:
                return self.variables[name]
            raise NovaError(f"Undefined variable: {name}")
        if kind == "index":
            container = self.eval_node(node[1])
            index = self.eval_node(node[2])
            try:
                return container[index]
            except Exception:
                raise NovaError(f"Cannot index {self.stringify(container)} with {self.stringify(index)}")
        if kind == "call":
            name = node[1]
            args = [self.eval_node(a) for a in node[2]]
            if name in self.builtins:
                return self.builtins[name](*args)
            if name in self.functions:
                return self.call_function(name, args)
            raise NovaError(f"Unknown function: {name}")
        if kind == "unary":
            val = self.eval_node(node[2])
            if node[1] == "MINUS":
                return -val
            raise NovaError(f"Unknown unary operator: {node[1]}")
        if kind == "binop":
            op = node[1]
            l = self.eval_node(node[2])
            r = self.eval_node(node[3])
            return self.apply_binop(op, l, r)

        raise NovaError(f"Unknown AST node: {node}")

    def apply_binop(self, op: str, l: Any, r: Any) -> Any:
        if op == "PLUS":
            if isinstance(l, str) or isinstance(r, str):
                return self.stringify(l) + self.stringify(r)
            return l + r
        if op == "MINUS":
            return l - r
        if op == "STAR":
            return l * r
        if op == "SLASH":
            return l / r
        if op == "EQEQ":
            return l == r
        if op == "NEQ":
            return l != r
        if op == "LE":
            return l <= r
        if op == "GE":
            return l >= r
        if op == "LT":
            return l < r
        if op == "GT":
            return l > r
        raise NovaError(f"Unknown operator: {op}")

    def call_function(self, name: str, args: List[Any]) -> Any:
        params, body = self.functions[name]
        if len(args) != len(params):
            raise NovaError(f"Function '{name}' expects {len(params)} arguments, got {len(args)}")
        old_vars = self.variables.copy()
        for param, arg in zip(params, args):
            self.variables[param] = arg
        try:
            self.execute_block(body, 0, len(body))
            return None
        except ReturnValue as r:
            return r.value
        finally:
            self.variables = old_vars

    # ── Value rendering ──────────────────────────────────────

    def stringify(self, value: Any) -> str:
        if isinstance(value, bool):
            return "true" if value else "false"
        if value is None:
            return "null"
        if isinstance(value, list):
            return "[" + ", ".join(self.stringify(v) for v in value) + "]"
        if isinstance(value, dict):
            return "{" + ", ".join(f"{k}: {self.stringify(v)}" for k, v in value.items()) + "}"
        return str(value)

    # ── Built-ins ──────────────────────────────────────────

    def builtin_read_file(self, path: str) -> str:
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def builtin_len(self, x) -> int:
        return len(x)

    def builtin_str(self, x) -> str:
        return self.stringify(x)

    def builtin_int(self, x) -> int:
        return int(x)

    def builtin_float(self, x) -> float:
        return float(x)

    def builtin_append(self, lst, item):
        if not isinstance(lst, list):
            raise NovaError("append expects a list")
        lst.append(item)
        return lst

    def builtin_split(self, s: str, sep: str = " ") -> list:
        return s.split(sep)

    def builtin_join(self, lst, sep: str = "") -> str:
        return sep.join(self.stringify(x) for x in lst)

    def builtin_type(self, x) -> str:
        if isinstance(x, bool):
            return "bool"
        if isinstance(x, int):
            return "int"
        if isinstance(x, float):
            return "float"
        if isinstance(x, str):
            return "string"
        if isinstance(x, list):
            return "list"
        if isinstance(x, dict):
            return "map"
        return "unknown"


def main():
    if len(sys.argv) < 2:
        print("Usage: python nova.py <file.nv>")
        sys.exit(1)
    interpreter = NovaInterpreter()
    interpreter.run_file(sys.argv[1])


if __name__ == "__main__":
    main()
