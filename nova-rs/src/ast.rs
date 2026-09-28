use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Var(String),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub struct FunDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Print(Expr),
    Assign(Expr, Expr),
    ExprStmt(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    For(String, Expr, Expr, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Fun(Rc<FunDef>),
    Struct(Rc<StructDef>),
    Import(String),
    Return(Option<Expr>),
    Break,
}
