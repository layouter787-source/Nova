use std::rc::Rc;

use crate::ast::{BinOp, Expr, FunDef, Stmt, StmtKind, StructDef};
use crate::error::NovaError;
use crate::lexer::{Tok, Token};

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

fn describe_tok(t: &Tok) -> String {
    match t {
        Tok::Int(n) => format!("number {}", n),
        Tok::Float(f) => format!("number {}", f),
        Tok::Str(_) => "a string".to_string(),
        Tok::Ident(s) => format!("'{}'", s),
        Tok::Newline => "end of line".to_string(),
        Tok::Eof => "end of file".to_string(),
        Tok::End => "'end'".to_string(),
        Tok::Else => "'else'".to_string(),
        other => format!("{:?}", other),
    }
}

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Parser { toks, pos: 0 }
    }

    // ── token helpers ──────────────────────────────────────────

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }

    fn line(&self) -> usize {
        self.toks[self.pos].line
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn check(&self, t: &Tok) -> bool {
        self.peek() == t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.check(t) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: Tok, what: &str) -> Result<Token, NovaError> {
        if self.check(&t) {
            Ok(self.advance())
        } else {
            Err(NovaError::new(
                self.line(),
                format!("expected {}, found {}", what, describe_tok(self.peek())),
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&Tok::Newline) {
            self.advance();
        }
    }

    fn end_of_stmt(&mut self) -> Result<(), NovaError> {
        if matches!(self.peek(), Tok::Newline | Tok::Eof) {
            Ok(())
        } else {
            Err(NovaError::new(
                self.line(),
                format!("unexpected {} after statement", describe_tok(self.peek())),
            ))
        }
    }

    // ── statements ─────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, NovaError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.check(&Tok::Eof) {
            if matches!(self.peek(), Tok::End | Tok::Else) {
                return Err(NovaError::new(
                    self.line(),
                    format!("unexpected {} without a matching block", describe_tok(self.peek())),
                ));
            }
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn parse_block(&mut self, terms: &[Tok], open_line: usize, what: &str) -> Result<Vec<Stmt>, NovaError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        loop {
            if self.check(&Tok::Eof) {
                return Err(NovaError::new(open_line, format!("{} is missing 'end'", what)));
            }
            if terms.iter().any(|t| self.check(t)) {
                break;
            }
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, NovaError> {
        let line = self.line();
        let kind = match self.peek().clone() {
            Tok::Fun => self.parse_fun()?,
            Tok::Struct => self.parse_struct()?,
            Tok::Import => self.parse_import()?,
            Tok::If => self.parse_if()?,
            Tok::For => self.parse_for()?,
            Tok::While => self.parse_while()?,
            Tok::Return => {
                self.advance();
                if matches!(self.peek(), Tok::Newline | Tok::Eof) {
                    StmtKind::Return(None)
                } else {
                    StmtKind::Return(Some(self.parse_expr()?))
                }
            }
            Tok::Break => {
                self.advance();
                StmtKind::Break
            }
            Tok::Print => {
                self.advance();
                StmtKind::Print(self.parse_expr()?)
            }
            _ => {
                let e = self.parse_expr()?;
                if self.eat(&Tok::Assign) {
                    match e {
                        Expr::Var(_) | Expr::Index(..) | Expr::Field(..) => {}
                        _ => return Err(NovaError::new(line, "invalid assignment target")),
                    }
                    let v = self.parse_expr()?;
                    StmtKind::Assign(e, v)
                } else {
                    StmtKind::ExprStmt(e)
                }
            }
        };
        self.end_of_stmt()?;
        Ok(Stmt { kind, line })
    }

    fn parse_fun(&mut self) -> Result<StmtKind, NovaError> {
        let open_line = self.line();
        self.advance(); // fun
        let name = match self.advance() {
            Token { tok: Tok::Ident(n), .. } => n,
            t => return Err(NovaError::new(t.line, "expected function name after 'fun'")),
        };
        self.expect(Tok::LParen, "'('")?;
        let mut params = Vec::new();
        if !self.check(&Tok::RParen) {
            loop {
                match self.advance() {
                    Token { tok: Tok::Ident(p), .. } => params.push(p),
                    t => return Err(NovaError::new(t.line, "expected parameter name")),
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }
        self.expect(Tok::RParen, "')'")?;
        self.end_of_stmt()?;
        let body = self.parse_block(&[Tok::End], open_line, &format!("function '{}'", name))?;
        self.expect(Tok::End, "'end'")?;
        Ok(StmtKind::Fun(Rc::new(FunDef { name, params, body })))
    }

    fn parse_struct(&mut self) -> Result<StmtKind, NovaError> {
        let open_line = self.line();
        self.advance(); // struct
        let name = match self.advance() {
            Token { tok: Tok::Ident(n), .. } => n,
            t => return Err(NovaError::new(t.line, "expected struct name after 'struct'")),
        };
        self.end_of_stmt()?;
        self.skip_newlines();
        let mut fields = Vec::new();
        loop {
            if self.check(&Tok::End) {
                break;
            }
            if self.check(&Tok::Eof) {
                return Err(NovaError::new(open_line, format!("struct '{}' is missing 'end'", name)));
            }
            match self.advance() {
                Token { tok: Tok::Ident(f), .. } => fields.push(f),
                t => return Err(NovaError::new(t.line, "expected a field name in struct body")),
            }
            self.end_of_stmt()?;
            self.skip_newlines();
        }
        self.expect(Tok::End, "'end'")?;
        Ok(StmtKind::Struct(Rc::new(StructDef { name, fields })))
    }

    fn parse_import(&mut self) -> Result<StmtKind, NovaError> {
        self.advance(); // import
        let path = match self.advance() {
            Token { tok: Tok::Str(s), .. } => s,
            t => return Err(NovaError::new(t.line, "expected a string path after 'import'")),
        };
        Ok(StmtKind::Import(path))
    }

    /// Parses `if ... [else if ...] [else ...] end`. Consumes the final 'end'.
    fn parse_if(&mut self) -> Result<StmtKind, NovaError> {
        let open_line = self.line();
        self.advance(); // if
        let cond = self.parse_expr()?;
        self.end_of_stmt()?;
        let then_body = self.parse_block(&[Tok::Else, Tok::End], open_line, "if")?;
        let mut else_body = Vec::new();
        if self.eat(&Tok::Else) {
            if self.check(&Tok::If) {
                let nested_line = self.line();
                let nested = self.parse_if()?; // consumes the shared 'end'
                else_body.push(Stmt { kind: nested, line: nested_line });
                return Ok(StmtKind::If(cond, then_body, else_body));
            }
            self.end_of_stmt()?;
            else_body = self.parse_block(&[Tok::End], open_line, "else")?;
        }
        self.expect(Tok::End, "'end'")?;
        Ok(StmtKind::If(cond, then_body, else_body))
    }

    fn parse_for(&mut self) -> Result<StmtKind, NovaError> {
        let open_line = self.line();
        self.advance(); // for
        let var = match self.advance() {
            Token { tok: Tok::Ident(n), .. } => n,
            t => return Err(NovaError::new(t.line, "expected loop variable after 'for'")),
        };
        self.expect(Tok::From, "'from'")?;
        let start = self.parse_expr()?;
        self.expect(Tok::To, "'to'")?;
        let end = self.parse_expr()?;
        self.end_of_stmt()?;
        let body = self.parse_block(&[Tok::End], open_line, "for loop")?;
        self.expect(Tok::End, "'end'")?;
        Ok(StmtKind::For(var, start, end, body))
    }

    fn parse_while(&mut self) -> Result<StmtKind, NovaError> {
        let open_line = self.line();
        self.advance(); // while
        let cond = self.parse_expr()?;
        self.end_of_stmt()?;
        let body = self.parse_block(&[Tok::End], open_line, "while loop")?;
        self.expect(Tok::End, "'end'")?;
        Ok(StmtKind::While(cond, body))
    }

    // ── expressions (lowest to highest precedence) ─────────────
    // or < and < not < comparison < + - < * / < unary minus < index/call/field

    pub fn parse_expr(&mut self) -> Result<Expr, NovaError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.parse_and()?;
        while self.eat(&Tok::Or) {
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.parse_not()?;
        while self.eat(&Tok::And) {
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, NovaError> {
        if self.eat(&Tok::Not) {
            let inner = self.parse_not()?;
            Ok(Expr::Not(Box::new(inner)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => BinOp::Eq,
                Tok::NotEq => BinOp::Ne,
                Tok::Lt => BinOp::Lt,
                Tok::Le => BinOp::Le,
                Tok::Gt => BinOp::Gt,
                Tok::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_add()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, NovaError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, NovaError> {
        if self.eat(&Tok::Minus) {
            let inner = self.parse_unary()?;
            Ok(Expr::Neg(Box::new(inner)))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, NovaError> {
        let mut node = self.parse_primary()?;
        loop {
            if self.check(&Tok::LBracket) {
                self.advance();
                let idx = self.parse_expression()?;
                self.expect(Tok::RBracket, "']'")?;
                node = Expr::Index(Box::new(node), Box::new(idx));
            } else if self.check(&Tok::Dot) {
                self.advance();
                let name = match self.advance() {
                    Token { tok: Tok::Ident(n), .. } => n,
                    t => return Err(NovaError::new(t.line, "expected a field name after '.'")),
                };
                node = Expr::Field(Box::new(node), name);
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<Expr, NovaError> {
        let line = self.line();
        match self.advance().tok {
            Tok::Int(n) => Ok(Expr::Int(n)),
            Tok::Float(f) => Ok(Expr::Float(f)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::True => Ok(Expr::Bool(true)),
            Tok::False => Ok(Expr::Bool(false)),
            Tok::Ident(name) => {
                if self.eat(&Tok::LParen) {
                    let mut args = Vec::new();
                    if !self.check(&Tok::RParen) {
                        args.push(self.parse_expr()?);
                        while self.eat(&Tok::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(Tok::RParen, "')'")?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Tok::LParen => {
                let e = self.parse_expr()?;
                self.expect(Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::LBracket => {
                let mut items = Vec::new();
                if !self.check(&Tok::RBracket) {
                    items.push(self.parse_expr()?);
                    while self.eat(&Tok::Comma) {
                        if self.check(&Tok::RBracket) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                }
                self.expect(Tok::RBracket, "']'")?;
                Ok(Expr::List(items))
            }
            Tok::LBrace => {
                let mut pairs = Vec::new();
                if !self.check(&Tok::RBrace) {
                    pairs.push(self.parse_pair()?);
                    while self.eat(&Tok::Comma) {
                        if self.check(&Tok::RBrace) {
                            break;
                        }
                        pairs.push(self.parse_pair()?);
                    }
                }
                self.expect(Tok::RBrace, "'}'")?;
                Ok(Expr::Map(pairs))
            }
            other => Err(NovaError::new(line, format!("unexpected {}", describe_tok(&other)))),
        }
    }

    fn parse_pair(&mut self) -> Result<(Expr, Expr), NovaError> {
        let kind = self.peek().clone();
        let key_node = if let Tok::Ident(v) = kind {
            if self.peek_at(1) == &Tok::Colon {
                self.advance();
                Expr::Str(v)
            } else {
                self.parse_expr()?
            }
        } else {
            self.parse_expr()?
        };
        self.expect(Tok::Colon, "':'")?;
        let val_node = self.parse_expr()?;
        Ok((key_node, val_node))
    }
}
