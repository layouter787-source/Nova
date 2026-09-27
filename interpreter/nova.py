#!/usr/bin/env python3
"""
Nova Language - Bootstrap Interpreter (v0.1)
Very minimal implementation to start running Nova code.
"""

import sys
import re

class NovaInterpreter:
    def __init__(self):
        self.variables = {}

    def run_file(self, path):
        with open(path, "r", encoding="utf-8") as f:
            source = f.read()
        self.run(source)

    def run(self, source):
        lines = source.splitlines()
        for line_num, raw_line in enumerate(lines, 1):
            line = raw_line.strip()

            # Skip empty lines and comments
            if not line or line.startswith("#"):
                continue

            try:
                self.execute(line)
            except Exception as e:
                print(f"Error on line {line_num}: {e}")
                print(f"  --> {raw_line}")
                sys.exit(1)

    def execute(self, line):
        # print statement
        if line.startswith("print "):
            expr = line[6:].strip()
            value = self.evaluate(expr)
            print(value)
            return

        # assignment: name = expression
        if "=" in line and not line.startswith("=="):
            # Simple split on first =
            parts = line.split("=", 1)
            if len(parts) == 2:
                name = parts[0].strip()
                expr = parts[1].strip()

                if not re.match(r"^[a-zA-Z_][a-zA-Z0-9_]*$", name):
                    raise Exception(f"Invalid variable name: {name}")

                value = self.evaluate(expr)
                self.variables[name] = value
                return

        raise Exception(f"Unknown statement: {line}")

    def evaluate(self, expr):
        expr = expr.strip()

        # String literal
        if (expr.startswith('"') and expr.endswith('"')) or \
           (expr.startswith("'") and expr.endswith("'")):
            return expr[1:-1]

        # Boolean
        if expr == "true":
            return True
        if expr == "false":
            return False

        # Number
        try:
            if "." in expr:
                return float(expr)
            return int(expr)
        except ValueError:
            pass

        # Variable
        if expr in self.variables:
            return self.variables[expr]

        # Simple arithmetic (very basic, left to right)
        # Support: a + b, a - b, a * b, a / b
        for op in ["+", "-", "*", "/"]:
            if op in expr:
                left, right = expr.split(op, 1)
                left_val = self.evaluate(left.strip())
                right_val = self.evaluate(right.strip())

                if op == "+":
                    return left_val + right_val
                if op == "-":
                    return left_val - right_val
                if op == "*":
                    return left_val * right_val
                if op == "/":
                    return left_val / right_val

        raise Exception(f"Cannot evaluate: {expr}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python nova.py <file.nv>")
        sys.exit(1)

    path = sys.argv[1]
    interpreter = NovaInterpreter()
    interpreter.run_file(path)


if __name__ == "__main__":
    main()
