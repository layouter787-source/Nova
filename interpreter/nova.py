#!/usr/bin/env python3
"""
Nova Language - Bootstrap Interpreter (v0.5)

Changes since v0.4:
  - FIX: parenthesized expressions like (2 + 3) * 4 now work (were silently
    crashing with "Cannot evaluate expression").
  - FIX: repeated / trailing unary minus (e.g. "5 - -3", "x = -y") no longer
    crashes with a TypeError.
  - FIX: bareword map keys ({ x: 1 }) were being looked up as variables
    instead of treated as the literal string key "x" - silently corrupted
    any map literal whose key name collided with an existing variable.
  - NEW: `elif` support in `if` statements.
  - NEW: logical `and`, `or`, `not` operators.
  - NEW: inline comments (`x = 5  # like this`), not just full-line ones.
  - NEW: string escapes (\\n, \\t, \\", \\\\) inside string literals.
  - NEW: index assignment (`list[0] = x`, `map["k"] = x`).
"""

import sys
import re
from typing import Any, List, Dict, Tuple


class ReturnValue(Exception):
    def __init__(self, value):
        self.value = value


class BreakException(Exception):
    pass


BLOCK_STARTERS = ("fun ", "if ", "for ", "while ")

ESCAPE_MAP = {
    "n": "\n",
    "t": "\t",
    "r": "\r",
    '"': '"',
    "'": "'",
    "\\": "\\",
}


class NovaInterpreter:
    def __init__(self):
        self.variables: Dict[str, Any] = {}
        self.functions: Dict[str, Tuple[List[str], List[str]]] = {}
        self.line_number = 0

        # Built-in functions
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

    # ── Comments ───────────────────────────────────────────────

    def strip_comment(self, line: str) -> str:
        """Remove a trailing '# ...' comment, ignoring '#' inside strings."""
        in_string = False
        string_char = None
        for i, c in enumerate(line):
            if in_string:
                if c == "\\":
                    continue
                if c == string_char:
                    in_string = False
            elif c in '"\'':
                in_string = True
                string_char = c
            elif c == "#":
                return line[:i]
        return line

    # ── Block execution ────────────────────────────────────────

    def execute_block(self, lines: List[str], start: int, end: int) -> int:
        i = start
        while i < end:
            raw = lines[i]
            line = self.strip_comment(raw).strip()
            self.line_number = i + 1

            if not line:
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
        header = self.strip_comment(lines[start]).strip()
        match = re.match(r"fun\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)", header)
        if not match:
            raise Exception(f"Invalid function definition: {header}")
        name = match.group(1)
        params_str = match.group(2).strip()
        params = [p.strip() for p in params_str.split(",") if p.strip()] if params_str else []

        body = []
        i = start + 1
        depth = 1
        while i < end:
            line = self.strip_comment(lines[i]).strip()
            if line.startswith(BLOCK_STARTERS):
                depth += 1
            elif line == "end":
                depth -= 1
                if depth == 0:
                    break
            body.append(lines[i])
            i += 1
        if depth != 0:
            raise Exception(f"Function '{name}' is missing 'end'")
        self.functions[name] = (params, body)
        return i + 1

    def parse_if(self, lines, start, end):
        """Supports if / elif (any number) / else / end."""
        condition = self.strip_comment(lines[start]).strip()[3:].strip()
        branches = []  # list of (condition_or_None, block_lines)
        current_block: List[str] = []
        branches.append((condition, current_block))

        i = start + 1
        depth = 1
        closed = False
        while i < end:
            raw_line = lines[i]
            line = self.strip_comment(raw_line).strip()

            if depth == 1 and line.startswith("elif "):
                current_block = []
                branches.append((line[5:].strip(), current_block))
                i += 1
                continue
            if depth == 1 and line == "else":
                current_block = []
                branches.append((None, current_block))
                i += 1
                continue

            if line.startswith(BLOCK_STARTERS):
                depth += 1
            elif line == "end":
                depth -= 1
                if depth == 0:
                    i += 1
                    closed = True
                    break

            current_block.append(raw_line)
            i += 1

        if not closed:
            raise Exception("if statement is missing 'end'")

        for cond, block in branches:
            if cond is None or self.evaluate(cond):
                self.execute_block(block, 0, len(block))
                break
        return i

    def parse_for(self, lines, start, end):
        header = self.strip_comment(lines[start]).strip()
        match = re.match(r"for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+from\s+(.+)\s+to\s+(.+)", header)
        if not match:
            raise Exception(f"Invalid for loop: {header}")
        var_name = match.group(1)
        start_val = int(self.evaluate(match.group(2).strip()))
        end_val = int(self.evaluate(match.group(3).strip()))

        body = []
        i = start + 1
        depth = 1
        while i < end:
            line = self.strip_comment(lines[i]).strip()
            if line.startswith(BLOCK_STARTERS):
                depth += 1
            elif line == "end":
                depth -= 1
                if depth == 0:
                    break
            body.append(lines[i])
            i += 1
        if depth != 0:
            raise Exception("for loop is missing 'end'")

        for val in range(start_val, end_val + 1):
            self.variables[var_name] = val
            try:
                self.execute_block(body, 0, len(body))
            except BreakException:
                break
        return i + 1

    def parse_while(self, lines, start, end):
        condition = self.strip_comment(lines[start]).strip()[6:].strip()
        body = []
        i = start + 1
        depth = 1
        while i < end:
            line = self.strip_comment(lines[i]).strip()
            if line.startswith(BLOCK_STARTERS):
                depth += 1
            elif line == "end":
                depth -= 1
                if depth == 0:
                    break
            body.append(lines[i])
            i += 1
        if depth != 0:
            raise Exception("while loop is missing 'end'")

        while self.evaluate(condition):
            try:
                self.execute_block(body, 0, len(body))
            except BreakException:
                break
        return i + 1

    def execute_statement(self, line: str):
        if line.startswith("print "):
            value = self.evaluate(line[6:].strip())
            print(value)
            return

        # Function call as statement
        if re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*\s*\(.*\)$", line):
            self.evaluate(line)
            return

        # Index / key assignment: container[index] = expr
        idx_assign = re.match(r"^(.+)\[(.+)\]\s*(?<![!<>=])=(?!=)\s*(.+)$", line)
        if idx_assign:
            container = self.evaluate(idx_assign.group(1).strip())
            index = self.evaluate(idx_assign.group(2).strip())
            value = self.evaluate(idx_assign.group(3).strip())
            try:
                container[index] = value
                return
            except Exception as e:
                raise Exception(f"Cannot assign to index: {e}")

        # Plain assignment
        if re.search(r"(?<![!<>=])=(?!=)", line):
            parts = re.split(r"(?<![!<>=])=(?!=)", line, maxsplit=1)
            if len(parts) == 2:
                left = parts[0].strip()
                expr = parts[1].strip()

                if re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", left):
                    self.variables[left] = self.evaluate(expr)
                    return

        raise Exception(f"Unknown statement: {line}")

    # ── Expression evaluation ─────────────────────────────────

    def evaluate(self, expr: str) -> Any:
        expr = expr.strip()
        if not expr:
            return None

        # Fully-parenthesized grouping: (expr)
        stripped = self.strip_outer_parens(expr)
        if stripped != expr:
            return self.evaluate(stripped)

        # String literal
        if (expr.startswith('"') and expr.endswith('"') and len(expr) >= 2) or (
            expr.startswith("'") and expr.endswith("'") and len(expr) >= 2
        ):
            return self.decode_string(expr[1:-1])

        # Boolean
        if expr == "true":
            return True
        if expr == "false":
            return False

        # Logical not
        if expr == "not" or expr.startswith("not "):
            return not self.evaluate(expr[3:].strip())

        # List literal
        if expr.startswith("[") and expr.endswith("]"):
            inner = expr[1:-1].strip()
            if not inner:
                return []
            return [self.evaluate(item) for item in self.split_args(inner)]

        # Map literal
        if expr.startswith("{") and expr.endswith("}"):
            inner = expr[1:-1].strip()
            if not inner:
                return {}
            result = {}
            for pair in self.split_args(inner):
                if ":" not in pair:
                    raise Exception(f"Invalid map pair: {pair}")
                k, v = pair.split(":", 1)
                k = k.strip()
                # Bareword keys (name: 1) are literal string keys, like in
                # every other language with object-literal syntax - they
                # must NOT be looked up as variables.
                if re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", k):
                    key = k
                else:
                    key = self.evaluate(k)
                result[key] = self.evaluate(v.strip())
            return result

        # Indexing: something[index]
        idx_match = re.match(r"^(.+)\[(.+)\]$", expr)
        if idx_match:
            container = self.evaluate(idx_match.group(1).strip())
            index = self.evaluate(idx_match.group(2).strip())
            try:
                return container[index]
            except Exception:
                raise Exception(f"Cannot index {container} with {index}")

        # Function / builtin call
        call_match = re.match(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)$", expr)
        if call_match:
            name = call_match.group(1)
            args_str = call_match.group(2).strip()
            args = [self.evaluate(a) for a in self.split_args(args_str)] if args_str else []

            if name in self.builtins:
                return self.builtins[name](*args)
            if name in self.functions:
                return self.call_function(name, args)
            raise Exception(f"Unknown function: {name}")

        # Number
        try:
            if "." in expr:
                return float(expr)
            return int(expr)
        except ValueError:
            pass

        # Variable
        if re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", expr):
            if expr in self.variables:
                return self.variables[expr]
            raise Exception(f"Undefined variable: {expr}")

        return self.evaluate_expression(expr)

    def evaluate_expression(self, expr: str) -> Any:
        # Logical operators (lowest precedence)
        parts = self.split_by_op(expr, "or")
        if len(parts) > 1:
            result = False
            for p in parts:
                if self.evaluate(p):
                    result = True
            return result

        parts = self.split_by_op(expr, "and")
        if len(parts) > 1:
            for p in parts:
                if not self.evaluate(p):
                    return False
            return True

        for op in ["==", "!=", "<=", ">=", "<", ">"]:
            parts = self.split_by_op(expr, op)
            if len(parts) > 1:
                l = self.evaluate(parts[0])
                r = self.evaluate(parts[1])
                if op == "==": return l == r
                if op == "!=": return l != r
                if op == "<=": return l <= r
                if op == ">=": return l >= r
                if op == "<":  return l < r
                if op == ">":  return l > r

        for op in ["+", "-"]:
            parts = self.split_by_op(expr, op)
            if len(parts) > 1:
                result = self.evaluate(parts[0])
                for p in parts[1:]:
                    val = self.evaluate(p)
                    result = result + val if op == "+" else result - val
                return result

        for op in ["*", "/"]:
            parts = self.split_by_op(expr, op)
            if len(parts) > 1:
                result = self.evaluate(parts[0])
                for p in parts[1:]:
                    val = self.evaluate(p)
                    result = result * val if op == "*" else result / val
                return result

        raise Exception(f"Cannot evaluate expression: {expr}")

    # ── Helpers ────────────────────────────────────────────────

    def strip_outer_parens(self, expr: str) -> str:
        """If expr is fully wrapped in one matching pair of parens, unwrap it."""
        if not (expr.startswith("(") and expr.endswith(")")):
            return expr
        depth = 0
        in_string = False
        string_char = None
        for i, c in enumerate(expr):
            if in_string:
                if c == string_char:
                    in_string = False
                continue
            if c in '"\'':
                in_string = True
                string_char = c
                continue
            if c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
                if depth == 0 and i != len(expr) - 1:
                    return expr  # closes early -> not a single wrapping group
        return expr[1:-1].strip()

    def decode_string(self, s: str) -> str:
        out = []
        i = 0
        while i < len(s):
            c = s[i]
            if c == "\\" and i + 1 < len(s) and s[i + 1] in ESCAPE_MAP:
                out.append(ESCAPE_MAP[s[i + 1]])
                i += 2
            else:
                out.append(c)
                i += 1
        return "".join(out)

    def split_by_op(self, expr: str, op: str) -> List[str]:
        parts = []
        current = ""
        depth = 0
        in_string = False
        string_char = None
        is_word_op = op.isalpha()
        i = 0
        while i < len(expr):
            c = expr[i]
            if in_string:
                current += c
                if c == "\\" and i + 1 < len(expr):
                    current += expr[i + 1]
                    i += 2
                    continue
                if c == string_char:
                    in_string = False
            elif c in '"\'':
                in_string = True
                string_char = c
                current += c
            elif c in "([{":
                depth += 1
                current += c
            elif c in ")]}":
                depth -= 1
                current += c
            elif (
                depth == 0
                and expr[i:i + len(op)] == op
                and (
                    not is_word_op
                    or (
                        (i == 0 or not (expr[i - 1].isalnum() or expr[i - 1] == "_"))
                        and (
                            i + len(op) >= len(expr)
                            or not (expr[i + len(op)].isalnum() or expr[i + len(op)] == "_")
                        )
                    )
                )
            ):
                if op in ("+", "-") and current.strip() == "":
                    # unary +/- : keep it attached to the operand, don't split
                    current += c
                    i += 1
                    continue
                parts.append(current.strip())
                current = ""
                i += len(op) - 1
            else:
                current += c
            i += 1
        if current.strip():
            parts.append(current.strip())
        return parts if len(parts) > 1 else [expr]

    def split_args(self, s: str) -> List[str]:
        args = []
        current = ""
        depth = 0
        in_string = False
        string_char = None
        for c in s:
            if in_string:
                current += c
                if c == string_char:
                    in_string = False
            elif c in '"\'':
                in_string = True
                string_char = c
                current += c
            elif c in "([{":
                depth += 1
                current += c
            elif c in ")]}":
                depth -= 1
                current += c
            elif c == "," and depth == 0:
                args.append(current.strip())
                current = ""
            else:
                current += c
        if current.strip():
            args.append(current.strip())
        return args

    def call_function(self, name: str, args: List[Any]) -> Any:
        params, body = self.functions[name]
        if len(args) != len(params):
            raise Exception(f"Function '{name}' expects {len(params)} arguments, got {len(args)}")
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

    # ── Built-ins ──────────────────────────────────────────────

    def builtin_read_file(self, path: str) -> str:
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def builtin_len(self, x) -> int:
        return len(x)

    def builtin_str(self, x) -> str:
        return str(x)

    def builtin_int(self, x) -> int:
        return int(x)

    def builtin_float(self, x) -> float:
        return float(x)

    def builtin_append(self, lst, item):
        if not isinstance(lst, list):
            raise Exception("append expects a list")
        lst.append(item)
        return lst

    def builtin_split(self, s: str, sep: str = " ") -> list:
        return s.split(sep)

    def builtin_join(self, lst, sep: str = "") -> str:
        return sep.join(str(x) for x in lst)

    def builtin_type(self, x) -> str:
        if isinstance(x, bool): return "bool"
        if isinstance(x, int): return "int"
        if isinstance(x, float): return "float"
        if isinstance(x, str): return "string"
        if isinstance(x, list): return "list"
        if isinstance(x, dict): return "map"
        return "unknown"


def main():
    if len(sys.argv) < 2:
        print("Usage: python nova.py <file.nv>")
        sys.exit(1)
    interpreter = NovaInterpreter()
    interpreter.run_file(sys.argv[1])


if __name__ == "__main__":
    main()
