use std::fmt;

#[derive(Debug, Clone)]
pub struct NovaError {
    pub line: Option<usize>,
    pub msg: String,
}

impl NovaError {
    pub fn new(line: usize, msg: impl Into<String>) -> Self {
        NovaError { line: Some(line), msg: msg.into() }
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        NovaError { line: None, msg: msg.into() }
    }

    /// Attach a line number if the error does not have one yet.
    pub fn at(mut self, line: usize) -> Self {
        if self.line.is_none() {
            self.line = Some(line);
        }
        self
    }
}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(l) => write!(f, "Error on line {}: {}", l, self.msg),
            None => write!(f, "Error: {}", self.msg),
        }
    }
}

impl std::error::Error for NovaError {}
