pub mod ast;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod parser;

use std::io::Write;

pub use error::NovaError;

/// Lex, parse (once, into an AST) and run a Nova program.
/// Program output goes to `out`, which is flushed before returning.
pub fn run_source(src: &str, out: Box<dyn Write>) -> Result<(), NovaError> {
    let tokens = lexer::tokenize(src)?;
    let program = parser::Parser::new(tokens).parse_program()?;
    let mut interpreter = interp::Interpreter::new(out);
    let result = interpreter.run(&program);
    interpreter.flush();
    result
}
