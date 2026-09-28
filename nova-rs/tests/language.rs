use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

#[derive(Clone)]
struct Buf(Rc<RefCell<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run(src: &str) -> Result<String, String> {
    let buf = Buf(Rc::new(RefCell::new(Vec::new())));
    let result = nova::run_source(src, Box::new(buf.clone()));
    match result {
        Ok(()) => Ok(String::from_utf8(buf.0.borrow().clone()).unwrap()),
        Err(e) => Err(e.to_string()),
    }
}

fn ok(src: &str) -> String {
    run(src).unwrap_or_else(|e| panic!("program failed: {}", e))
}

#[test]
fn hello() {
    assert_eq!(ok("print \"Hello, Nova!\""), "Hello, Nova!\n");
}

#[test]
fn precedence_and_parentheses() {
    assert_eq!(ok("print 2 + 3 * 4"), "14\n");
    assert_eq!(ok("print (2 + 3) * 4"), "20\n");
}

#[test]
fn unary_minus() {
    assert_eq!(ok("x = 5\nprint -x\nprint 3 * -2"), "-5\n-6\n");
}

#[test]
fn division_is_float() {
    assert_eq!(ok("print 7 / 2\nprint 4 / 2"), "3.5\n2.0\n");
}

#[test]
fn nested_index_assignment() {
    assert_eq!(
        ok("m = [[1, 2], [3, 4]]\nm[0][1] = 77\nprint m"),
        "[[1, 77], [3, 4]]\n"
    );
}

#[test]
fn negative_index() {
    assert_eq!(ok("a = [1, 2, 3]\nprint a[-1]"), "3\n");
}

#[test]
fn string_concat_with_number() {
    assert_eq!(ok("print \"score: \" + 5"), "score: 5\n");
}

#[test]
fn string_escapes() {
    assert_eq!(ok("print \"a\\nb\""), "a\nb\n");
}

#[test]
fn booleans_print_as_nova() {
    assert_eq!(ok("print 1 < 2\nprint false"), "true\nfalse\n");
}

#[test]
fn logic_operators() {
    assert_eq!(ok("print not false and true"), "true\n");
    assert_eq!(ok("print false or 0"), "false\n");
}

#[test]
fn else_if_chain() {
    let src = "x = 5\nif x > 10\n  print \"big\"\nelse if x > 3\n  print \"mid\"\nelse\n  print \"small\"\nend";
    assert_eq!(ok(src), "mid\n");
}

#[test]
fn recursion() {
    let src = "fun fib(n)\n  if n < 2\n    return n\n  end\n  return fib(n - 1) + fib(n - 2)\nend\nprint fib(10)";
    assert_eq!(ok(src), "55\n");
}

#[test]
fn functions_read_globals_but_do_not_leak_locals() {
    let src = "base = 10\nfun add(x)\n  return x + base\nend\nprint add(5)\nx = 1\nfun f()\n  x = 2\nend\nf()\nprint x";
    assert_eq!(ok(src), "15\n1\n");
}

#[test]
fn while_and_break() {
    let src = "n = 0\nwhile true\n  n = n + 1\n  if n == 3\n    break\n  end\nend\nprint n";
    assert_eq!(ok(src), "3\n");
}

#[test]
fn for_loop_is_inclusive() {
    assert_eq!(ok("total = 0\nfor i from 1 to 4\n  total = total + i\nend\nprint total"), "10\n");
}

#[test]
fn maps() {
    let src = "p = {name: \"Nova\", v: 1}\nprint p[\"name\"]\np[\"v\"] = 2\nprint p";
    assert_eq!(ok(src), "Nova\n{name: Nova, v: 2}\n");
}

#[test]
fn builtins() {
    assert_eq!(ok("print join(split(\"a,b,c\", \",\"), \"-\")"), "a-b-c\n");
    assert_eq!(ok("print len(\"h\u{e9}llo\")"), "5\n");
    assert_eq!(ok("l = [1]\nappend(l, 2)\nprint l\nprint type(l)"), "[1, 2]\nlist\n");
}

#[test]
fn multiline_list_literal() {
    assert_eq!(ok("x = [\n  1,\n  2\n]\nprint len(x)"), "2\n");
}

#[test]
fn error_undefined_variable() {
    let e = run("print y").unwrap_err();
    assert!(e.contains("Undefined variable: y"), "got: {}", e);
    assert!(e.contains("line 1"), "got: {}", e);
}

#[test]
fn error_syntax() {
    assert!(run("print 1 +").is_err());
}

#[test]
fn error_missing_end() {
    let e = run("if true\n  print 1\n").unwrap_err();
    assert!(e.contains("missing 'end'"), "got: {}", e);
}

#[test]
fn error_division_by_zero() {
    let e = run("print 1 / 0").unwrap_err();
    assert!(e.contains("division by zero"), "got: {}", e);
}

#[test]
fn deep_recursion_is_an_error_not_a_crash() {
    let e = run("fun f(n)\n  return f(n + 1)\nend\nprint f(0)").unwrap_err();
    assert!(e.contains("maximum call depth"), "got: {}", e);
}
