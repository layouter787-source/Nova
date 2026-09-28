pub mod ast;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod parser;

use std::io::Write;
use std::path::PathBuf;

pub use error::NovaError;

/// Lex, parse (once, into an AST) and run a Nova program.
/// `base_dir` is the directory `import "..."` paths are resolved against
/// (normally the directory of the file being run).
/// Program output goes to `out`, which is flushed before returning.
pub fn run_source(src: &str, out: Box<dyn Write>, base_dir: PathBuf) -> Result<(), NovaError> {
    let tokens = lexer::tokenize(src)?;
    let program = parser::Parser::new(tokens).parse_program()?;
    let mut interpreter = interp::Interpreter::new(out, base_dir);
    let result = interpreter.run(&program);
    interpreter.flush();
    result
}
