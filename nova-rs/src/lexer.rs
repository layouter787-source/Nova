use crate::error::NovaError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Float(f64),
    Str(String),
    Ident(String),
    // keywords
    Fun,
    If,
    Else,
    For,
    From,
    To,
    While,
    End,
    Return,
    Break,
    Print,
    True,
    False,
    And,
    Or,
    Not,
    // symbols
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Assign,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

fn keyword(s: &str) -> Option<Tok> {
    Some(match s {
        "fun" => Tok::Fun,
        "if" => Tok::If,
        "else" => Tok::Else,
        "for" => Tok::For,
        "from" => Tok::From,
        "to" => Tok::To,
        "while" => Tok::While,
        "end" => Tok::End,
        "return" => Tok::Return,
        "break" => Tok::Break,
        "print" => Tok::Print,
        "true" => Tok::True,
        "false" => Tok::False,
        "and" => Tok::And,
        "or" => Tok::Or,
        "not" => Tok::Not,
        _ => return None,
    })
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, NovaError> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut line = 1usize;
    // Inside (), [] and {} newlines are ignored so literals can span lines.
    let mut depth: i32 = 0;
    let mut out: Vec<Token> = Vec::new();

    while i < n {
        let c = chars[i];

        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
        } else if c == '\n' {
            let last_is_content = out.last().map(|t| t.tok != Tok::Newline).unwrap_or(false);
            if depth <= 0 && last_is_content {
                out.push(Token { tok: Tok::Newline, line });
            }
            line += 1;
            i += 1;
        } else if c == '#' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' || c == '\'' {
            let quote = c;
            let start_line = line;
            i += 1;
            let mut s = String::new();
            loop {
                if i >= n {
                    return Err(NovaError::new(start_line, "unterminated string"));
                }
                let ch = chars[i];
                if ch == quote {
                    i += 1;
                    break;
                }
                if ch == '\\' {
                    i += 1;
                    if i >= n {
                        return Err(NovaError::new(start_line, "unterminated string"));
                    }
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '\'' => s.push('\''),
                        other => {
                            s.push('\\');
                            s.push(other);
                        }
                    }
                    i += 1;
                } else {
                    if ch == '\n' {
                        line += 1;
                    }
                    s.push(ch);
                    i += 1;
                }
            }
            out.push(Token { tok: Tok::Str(s), line: start_line });
        } else if c.is_ascii_digit() {
            let start = i;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
            let mut is_float = false;
            if i + 1 < n && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
                is_float = true;
                i += 1;
                while i < n && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let text: String = chars[start..i].iter().collect();
            if is_float {
                match text.parse::<f64>() {
                    Ok(f) => out.push(Token { tok: Tok::Float(f), line }),
                    Err(_) => return Err(NovaError::new(line, format!("invalid number '{}'", text))),
                }
            } else {
                match text.parse::<i64>() {
                    Ok(v) => out.push(Token { tok: Tok::Int(v), line }),
                    Err(_) => {
                        return Err(NovaError::new(line, format!("integer '{}' is too large", text)))
                    }
                }
            }
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let tok = match keyword(&text) {
                Some(k) => k,
                None => Tok::Ident(text),
            };
            out.push(Token { tok, line });
        } else {
            let next = chars.get(i + 1).copied();
            let (tok, adv) = match c {
                '+' => (Tok::Plus, 1),
                '-' => (Tok::Minus, 1),
                '*' => (Tok::Star, 1),
                '/' => (Tok::Slash, 1),
                '(' => {
                    depth += 1;
                    (Tok::LParen, 1)
                }
                ')' => {
                    depth -= 1;
                    (Tok::RParen, 1)
                }
                '[' => {
                    depth += 1;
                    (Tok::LBracket, 1)
                }
                ']' => {
                    depth -= 1;
                    (Tok::RBracket, 1)
                }
                '{' => {
                    depth += 1;
                    (Tok::LBrace, 1)
                }
                '}' => {
                    depth -= 1;
                    (Tok::RBrace, 1)
                }
                ',' => (Tok::Comma, 1),
                ':' => (Tok::Colon, 1),
                '=' => {
                    if next == Some('=') {
                        (Tok::EqEq, 2)
                    } else {
                        (Tok::Assign, 1)
                    }
                }
                '!' => {
                    if next == Some('=') {
                        (Tok::NotEq, 2)
                    } else {
                        return Err(NovaError::new(line, "unexpected character '!' (did you mean '!=' or 'not'?)"));
                    }
                }
                '<' => {
                    if next == Some('=') {
                        (Tok::Le, 2)
                    } else {
                        (Tok::Lt, 1)
                    }
                }
                '>' => {
                    if next == Some('=') {
                        (Tok::Ge, 2)
                    } else {
                        (Tok::Gt, 1)
                    }
                }
                other => {
                    return Err(NovaError::new(line, format!("unexpected character '{}'", other)));
                }
            };
            out.push(Token { tok, line });
            i += adv;
        }
    }

    let last_is_content = out.last().map(|t| t.tok != Tok::Newline).unwrap_or(false);
    if last_is_content {
        out.push(Token { tok: Tok::Newline, line });
    }
    out.push(Token { tok: Tok::Eof, line });
    Ok(out)
}
