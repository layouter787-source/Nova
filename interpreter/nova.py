#!/usr/bin/env python3
"""
Nova Language - Bootstrap Interpreter (v0.3)
Supports: print, variables, arithmetic, if/else, for, while, functions, lists, maps
"""

import sys
import re
from typing import Any, List, Dict, Optional, Tuple


class ReturnValue(Exception):
    def __init__(self, value):
        self.value = value


class BreakException(Exception):
    pass


class NovaInterpreter:
    def __init__(self):
        self.variables: Dict[str, Any] = {}
        self.functions: Dict[str, Tuple[List[str], List[str]]] = {}  # name -> (params, body_lines)
        self.line_number = 0

    def run_file(self, path: str):
        with open(path, "r", encoding="utf-8") as f:
            source = f.read()
        self.run(source)

    def run(self, source: str):
        lines = source.splitlines()
        self.execute_block(lines, 0, len(lines))

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
                # Function definition
                if line.startswith("fun "):
                    i = self.parse_function(lines, i, end)
                    continue

                # If statement
                if line.startswith("if "):
                    i = self.parse_if(lines, i, end)
                    continue

                # For loop
                if line.startswith("for "):
                    i = self.parse_for(lines, i, end)
                    continue

                # While loop
                if line.startswith("while "):
                    i = self.parse_while(lines, i, end)
                    continue

                # Return
                if line.startswith("return"):
                    expr = line[6:].strip()
                    value = self.evaluate(expr) if expr else None
                    raise ReturnValue(value)

                # Break
                if line == "break":
                    raise BreakException()

                # End of block (should be handled by callers)
                if line == "end":
                    return i + 1

                # Normal statement
                self.execute_statement(line)
                i += 1

            except ReturnValue:
                raise
            except BreakException:
                raise
            except Exception as e:
                print(f"Error on line {self.line_number}: {e}")
                print(f"  --> {raw}")
                sys.exit(1)

        return end

    def parse_function(self, lines: List[str], start: int, end: int) -> int:
        header = lines[start].strip()
        # fun name(param1, param2)
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
            line = lines[i].strip()
            if line.startswith(("fun ", "if ", "for ", "while ")):
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

    def parse_if(self, lines: List[str], start: int, end: int) -> int:
        condition = lines[start].strip()[3:].strip()
        cond_value = self.evaluate(condition)

        # Collect then block and optional else block
        then_block = []
        else_block = []
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
            raise Exception("if statement is missing 'end'")

        if cond_value:
            self.execute_block(then_block, 0, len(then_block))
        else:
            self.execute_block(else_block, 0, len(else_block))

        return i + 1

    def parse_for(self, lines: List[str], start: int, end: int) -> int:
        header = lines[start].strip()
        # for i from 1 to 10
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
            line = lines[i].strip()
            if line.startswith(("fun ", "if ", "for ", "while ")):
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

    def parse_while(self, lines: List[str], start: int, end: int) -> int:
        condition = lines[start].strip()[6:].strip()

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
        # print
        if line.startswith("print "):
            expr = line[6:].strip()
            value = self.evaluate(expr)
            print(value)
            return

        # Function call as statement (ignore return value)
        if "(" in line and ")" in line and "=" not in line:
            self.evaluate(line)
            return

        # Assignment
        if "=" in line and not any(op in line for op in ["==", "!=", "<=", ">="]):
            # Check for simple assignment (not comparison)
            if re.search(r"(?<![!<>=])=(?!=)", line):
                parts = re.split(r"(?<![!<>=])=(?!=)", line, maxsplit=1)
                if len(parts) == 2:
                    name = parts[0].strip()
                    expr = parts[1].strip()

                    if not re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", name):
                        raise Exception(f"Invalid variable name: {name}")

                    value = self.evaluate(expr)
                    self.variables[name] = value
                    return

        raise Exception(f"Unknown statement: {line}")

    def evaluate(self, expr: str) -> Any:
        expr = expr.strip()
        if not expr:
            return None

        # String literal
        if (expr.startswith('"') and expr.endswith('"')) or (expr.startswith("'") and expr.endswith("'")):
            return expr[1:-1]

        # Boolean
        if expr == "true":
            return True
        if expr == "false":
            return False

        # List literal [1, 2, 3]
        if expr.startswith("[") and expr.endswith("]"):
            inner = expr[1:-1].strip()
            if not inner:
                return []
            items = self.split_args(inner)
            return [self.evaluate(item) for item in items]

        # Map literal {key: value}
        if expr.startswith("{") and expr.endswith("}"):
            inner = expr[1:-1].strip()
            if not inner:
                return {}
            result = {}
            pairs = self.split_args(inner)
            for pair in pairs:
                if ":" not in pair:
                    raise Exception(f"Invalid map pair: {pair}")
                k, v = pair.split(":", 1)
                key = self.evaluate(k.strip())
                result[key] = self.evaluate(v.strip())
            return result

        # Function call
        match = re.match(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)$", expr)
        if match:
            name = match.group(1)
            args_str = match.group(2).strip()
            args = [self.evaluate(a) for a in self.split_args(args_str)] if args_str else []

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

        # Comparisons and arithmetic (simple recursive approach)
        return self.evaluate_expression(expr)

    def evaluate_expression(self, expr: str) -> Any:
        # Handle comparisons first (lowest precedence for this simple version)
        for op in ["==", "!=", "<=", ">=", "<", ">"]:
            if op in expr:
                left, right = expr.split(op, 1)
                l = self.evaluate(left.strip())
                r = self.evaluate(right.strip())
                if op == "==": return l == r
                if op == "!=": return l != r
                if op == "<=": return l <= r
                if op == ">=": return l >= r
                if op == "<":  return l < r
                if op == ">":  return l > r

        # Arithmetic
        for op in ["+", "-"]:
            # Find the operator not inside strings (very simplified)
            parts = self.split_by_op(expr, op)
            if len(parts) > 1:
                result = self.evaluate(parts[0])
                for p in parts[1:]:
                    val = self.evaluate(p)
                    if op == "+": result = result + val
                    else: result = result - val
                return result

        for op in ["*", "/"]:
            parts = self.split_by_op(expr, op)
            if len(parts) > 1:
                result = self.evaluate(parts[0])
                for p in parts[1:]:
                    val = self.evaluate(p)
                    if op == "*": result = result * val
                    else: result = result / val
                return result

        raise Exception(f"Cannot evaluate expression: {expr}")

    def split_by_op(self, expr: str, op: str) -> List[str]:
        """Very simple split that ignores operators inside strings and parentheses (basic)."""
        parts = []
        current = ""
        depth = 0
        in_string = False
        string_char = None
        i = 0
        while i < len(expr):
            c = expr[i]
            if in_string:
                current += c
                if c == string_char:
                    in_string = False
            elif c in '"\'':
                in_string = True
                string_char = c
                current += c
            elif c == '(':
                depth += 1
                current += c
            elif c == ')':
                depth -= 1
                current += c
            elif depth == 0 and expr[i:i+len(op)] == op:
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
            elif c in "([{{":
                depth += 1
                current += c
            elif c in ")}]}":
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

        # Save current variables (simple local scope)
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


def main():
    if len(sys.argv) < 2:
        print("Usage: python nova.py <file.nv>")
        sys.exit(1)

    path = sys.argv[1]
    interpreter = NovaInterpreter()
    interpreter.run_file(path)


if __name__ == "__main__":
    main()
