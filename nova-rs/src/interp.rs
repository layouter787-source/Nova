use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::{BinOp, Expr, FunDef, Stmt, StmtKind, StructDef};
use crate::error::NovaError;

const MAX_DEPTH: usize = 2000;

type ListRef = Rc<RefCell<Vec<Value>>>;
type MapRef = Rc<RefCell<Vec<(Value, Value)>>>;
type StructRef = Rc<RefCell<Vec<(String, Value)>>>;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(ListRef),
    Map(MapRef),
    Struct(String, StructRef),
}

pub fn new_list(v: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(v)))
}

fn fmt_float(f: f64) -> String {
    if f.is_finite() && f.fract() == 0.0 && f.abs() < 1e16 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Struct(_, _) => "struct",
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Map(m) => !m.borrow().is_empty(),
            Value::Struct(_, _) => true,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Int(i) => i.to_string(),
            Value::Float(f) => fmt_float(*f),
            Value::Str(s) => s.clone(),
            Value::List(l) => {
                let parts: Vec<String> = l.borrow().iter().map(|v| v.display()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Map(m) => {
                let parts: Vec<String> = m
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.display(), v.display()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            Value::Struct(name, fields) => {
                let parts: Vec<String> = fields
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display()))
                    .collect();
                format!("{} {{ {} }}", name, parts.join(", "))
            }
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::List(x), Value::List(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let x = x.borrow();
            let y = y.borrow();
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| values_equal(p, q))
        }
        (Value::Map(x), Value::Map(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let x = x.borrow();
            let y = y.borrow();
            x.len() == y.len()
                && x.iter().all(|(k, v)| match map_get(&y, k) {
                    Some(other) => values_equal(v, &other),
                    None => false,
                })
        }
        (Value::Struct(n1, f1), Value::Struct(n2, f2)) => {
            if n1 != n2 {
                return false;
            }
            if Rc::ptr_eq(f1, f2) {
                return true;
            }
            let a = f1.borrow();
            let b = f2.borrow();
            a.len() == b.len()
                && a.iter().all(|(k, v)| {
                    b.iter()
                        .find(|(k2, _)| k2 == k)
                        .map(|(_, v2)| values_equal(v, v2))
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}

fn num_pair(a: &Value, b: &Value) -> Option<(f64, f64)> {
    let x = match a {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return None,
    };
    let y = match b {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return None,
    };
    Some((x, y))
}

fn compare(a: &Value, b: &Value) -> Result<Ordering, NovaError> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Str(x), Value::Str(y)) => Ok(x.cmp(y)),
        _ => match num_pair(a, b) {
            Some((x, y)) => x
                .partial_cmp(&y)
                .ok_or_else(|| NovaError::runtime("cannot compare NaN")),
            None => Err(NovaError::runtime(format!(
                "cannot compare {} and {}",
                a.type_name(),
                b.type_name()
            ))),
        },
    }
}

fn overflow() -> NovaError {
    NovaError::runtime("integer overflow")
}

fn add(l: Value, r: Value) -> Result<Value, NovaError> {
    match (&l, &r) {
        (Value::Str(_), _) | (_, Value::Str(_)) => {
            Ok(Value::Str(format!("{}{}", l.display(), r.display())))
        }
        (Value::Int(a), Value::Int(b)) => a.checked_add(*b).map(Value::Int).ok_or_else(overflow),
        (Value::List(a), Value::List(b)) => {
            let mut v: Vec<Value> = a.borrow().clone();
            v.extend(b.borrow().iter().cloned());
            Ok(new_list(v))
        }
        _ => match num_pair(&l, &r) {
            Some((x, y)) => Ok(Value::Float(x + y)),
            None => Err(NovaError::runtime(format!(
                "cannot add {} and {}",
                l.type_name(),
                r.type_name()
            ))),
        },
    }
}

fn arith(op: BinOp, l: Value, r: Value) -> Result<Value, NovaError> {
    let sym = match op {
        BinOp::Sub => "subtract",
        BinOp::Mul => "multiply",
        _ => "divide",
    };
    let type_err = || {
        NovaError::runtime(format!("cannot {} {} and {}", sym, l.type_name(), r.type_name()))
    };
    if op == BinOp::Div {
        return match num_pair(&l, &r) {
            Some((_, y)) if y == 0.0 => Err(NovaError::runtime("division by zero")),
            Some((x, y)) => Ok(Value::Float(x / y)),
            None => Err(type_err()),
        };
    }
    if let (Value::Int(a), Value::Int(b)) = (&l, &r) {
        let res = if op == BinOp::Sub { a.checked_sub(*b) } else { a.checked_mul(*b) };
        return res.map(Value::Int).ok_or_else(overflow);
    }
    match num_pair(&l, &r) {
        Some((x, y)) => Ok(Value::Float(if op == BinOp::Sub { x - y } else { x * y })),
        None => Err(type_err()),
    }
}

fn binary(op: BinOp, l: Value, r: Value) -> Result<Value, NovaError> {
    match op {
        BinOp::Eq => Ok(Value::Bool(values_equal(&l, &r))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(&l, &r))),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = compare(&l, &r)?;
            Ok(Value::Bool(match op {
                BinOp::Lt => ord == Ordering::Less,
                BinOp::Le => ord != Ordering::Greater,
                BinOp::Gt => ord == Ordering::Greater,
                _ => ord != Ordering::Less,
            }))
        }
        BinOp::Add => add(l, r),
        BinOp::Sub | BinOp::Mul | BinOp::Div => arith(op, l, r),
    }
}

// ── maps (insertion-ordered, linear lookup for now) ────────────

fn check_key(k: &Value) -> Result<(), NovaError> {
    match k {
        Value::Str(_) | Value::Int(_) | Value::Bool(_) | Value::Float(_) => Ok(()),
        _ => Err(NovaError::runtime(format!(
            "map keys must be strings or numbers, got {}",
            k.type_name()
        ))),
    }
}

fn map_get(pairs: &[(Value, Value)], key: &Value) -> Option<Value> {
    pairs.iter().find(|(k, _)| values_equal(k, key)).map(|(_, v)| v.clone())
}

fn map_set(pairs: &mut Vec<(Value, Value)>, key: Value, val: Value) {
    for p in pairs.iter_mut() {
        if values_equal(&p.0, &key) {
            p.1 = val;
            return;
        }
    }
    pairs.push((key, val));
}

// ── indexing ───────────────────────────────────────────────────

fn as_int(v: &Value, what: &str) -> Result<i64, NovaError> {
    match v {
        Value::Int(i) => Ok(*i),
        other => Err(NovaError::runtime(format!(
            "{} must be an integer, got {}",
            what,
            other.type_name()
        ))),
    }
}

fn norm_index(i: i64, len: usize) -> Option<usize> {
    let l = len as i64;
    let j = if i < 0 { i + l } else { i };
    if j < 0 || j >= l {
        None
    } else {
        Some(j as usize)
    }
}

fn get_index(container: &Value, idx: &Value) -> Result<Value, NovaError> {
    match container {
        Value::List(list) => {
            let i = as_int(idx, "list index")?;
            let items = list.borrow();
            match norm_index(i, items.len()) {
                Some(j) => Ok(items[j].clone()),
                None => Err(NovaError::runtime(format!(
                    "list index {} out of range (length {})",
                    i,
                    items.len()
                ))),
            }
        }
        Value::Str(s) => {
            let i = as_int(idx, "string index")?;
            let n = s.chars().count();
            match norm_index(i, n) {
                Some(j) => Ok(Value::Str(s.chars().nth(j).unwrap_or(' ').to_string())),
                None => Err(NovaError::runtime(format!(
                    "string index {} out of range (length {})",
                    i, n
                ))),
            }
        }
        Value::Map(m) => match map_get(&m.borrow(), idx) {
            Some(v) => Ok(v),
            None => Err(NovaError::runtime(format!("key not found: {}", idx.display()))),
        },
        other => Err(NovaError::runtime(format!("cannot index {}", other.type_name()))),
    }
}

fn set_index(container: &Value, idx: &Value, val: Value) -> Result<(), NovaError> {
    match container {
        Value::List(list) => {
            let i = as_int(idx, "list index")?;
            let mut items = list.borrow_mut();
            let len = items.len();
            match norm_index(i, len) {
                Some(j) => {
                    items[j] = val;
                    Ok(())
                }
                None => Err(NovaError::runtime(format!(
                    "list index {} out of range (length {})",
                    i, len
                ))),
            }
        }
        Value::Map(m) => {
            check_key(idx)?;
            map_set(&mut m.borrow_mut(), idx.clone(), val);
            Ok(())
        }
        other => Err(NovaError::runtime(format!(
            "cannot assign to an index of {}",
            other.type_name()
        ))),
    }
}

// ── built-in functions ─────────────────────────────────────────

fn expect_args(name: &str, args: &[Value], min: usize, max: usize) -> Result<(), NovaError> {
    if args.len() < min || args.len() > max {
        let want = if min == max { format!("{}", min) } else { format!("{}-{}", min, max) };
        return Err(NovaError::runtime(format!(
            "{}() expects {} argument(s), got {}",
            name,
            want,
            args.len()
        )));
    }
    Ok(())
}

fn as_str<'a>(v: &'a Value, fname: &str) -> Result<&'a str, NovaError> {
    match v {
        Value::Str(s) => Ok(s.as_str()),
        other => Err(NovaError::runtime(format!(
            "{}() expects a string, got {}",
            fname,
            other.type_name()
        ))),
    }
}

fn b_len(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("len", args, 1, 1)?;
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Map(m) => Ok(Value::Int(m.borrow().len() as i64)),
        other => Err(NovaError::runtime(format!("len() not supported for {}", other.type_name()))),
    }
}

fn b_str(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("str", args, 1, 1)?;
    Ok(Value::Str(args[0].display()))
}

fn b_int(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("int", args, 1, 1)?;
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => {
            if f.is_finite() {
                Ok(Value::Int(*f as i64))
            } else {
                Err(NovaError::runtime("cannot convert a non-finite float to int"))
            }
        }
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Str(s) => s
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| NovaError::runtime(format!("cannot convert \"{}\" to int", s))),
        other => Err(NovaError::runtime(format!("cannot convert {} to int", other.type_name()))),
    }
}

fn b_float(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("float", args, 1, 1)?;
    match &args[0] {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Str(s) => s
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| NovaError::runtime(format!("cannot convert \"{}\" to float", s))),
        other => Err(NovaError::runtime(format!("cannot convert {} to float", other.type_name()))),
    }
}

fn b_append(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("append", args, 2, 2)?;
    match &args[0] {
        Value::List(l) => {
            l.borrow_mut().push(args[1].clone());
            Ok(args[0].clone())
        }
        _ => Err(NovaError::runtime("append expects a list")),
    }
}

fn b_split(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("split", args, 1, 2)?;
    let s = as_str(&args[0], "split")?;
    let sep = if args.len() == 2 { as_str(&args[1], "split")? } else { " " };
    if sep.is_empty() {
        return Err(NovaError::runtime("split() separator cannot be empty"));
    }
    Ok(new_list(s.split(sep).map(|p| Value::Str(p.to_string())).collect()))
}

fn b_join(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("join", args, 1, 2)?;
    let sep = if args.len() == 2 { as_str(&args[1], "join")?.to_string() } else { String::new() };
    match &args[0] {
        Value::List(l) => {
            let parts: Vec<String> = l.borrow().iter().map(|v| v.display()).collect();
            Ok(Value::Str(parts.join(&sep)))
        }
        other => Err(NovaError::runtime(format!("join() expects a list, got {}", other.type_name()))),
    }
}

fn b_type(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("type", args, 1, 1)?;
    if let Value::Struct(name, _) = &args[0] {
        return Ok(Value::Str(name.clone()));
    }
    Ok(Value::Str(args[0].type_name().to_string()))
}

fn b_read_file(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("read_file", args, 1, 1)?;
    let path = as_str(&args[0], "read_file")?;
    std::fs::read_to_string(path)
        .map(Value::Str)
        .map_err(|e| NovaError::runtime(format!("cannot read '{}': {}", path, e)))
}

fn b_write_file(args: &[Value]) -> Result<Value, NovaError> {
    expect_args("write_file", args, 2, 2)?;
    let path = as_str(&args[0], "write_file")?;
    let content = args[1].display();
    std::fs::write(path, content)
        .map(|_| Value::Null)
        .map_err(|e| NovaError::runtime(format!("cannot write '{}': {}", path, e)))
}

fn builtin(name: &str, args: &[Value]) -> Option<Result<Value, NovaError>> {
    Some(match name {
        "len" => b_len(args),
        "str" => b_str(args),
        "int" => b_int(args),
        "float" => b_float(args),
        "append" => b_append(args),
        "split" => b_split(args),
        "join" => b_join(args),
        "type" => b_type(args),
        "read_file" => b_read_file(args),
        "write_file" => b_write_file(args),
        _ => return None,
    })
}

// ── interpreter ────────────────────────────────────────────────

enum Flow {
    Normal,
    Break,
    Return(Value),
}

pub struct Interpreter {
    frames: Vec<HashMap<String, Value>>,
    functions: HashMap<String, Rc<FunDef>>,
    structs: HashMap<String, Rc<StructDef>>,
    methods: HashMap<(String, String), Rc<FunDef>>,
    base_dir: PathBuf,
    imported: HashSet<PathBuf>,
    importing: HashSet<PathBuf>,
    out: Box<dyn Write>,
    depth: usize,
}

impl Interpreter {
    pub fn new(out: Box<dyn Write>, base_dir: PathBuf) -> Self {
        Interpreter {
            frames: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            methods: HashMap::new(),
            base_dir,
            imported: HashSet::new(),
            importing: HashSet::new(),
            out,
            depth: 0,
        }
    }

    pub fn flush(&mut self) {
        let _ = self.out.flush();
    }

    pub fn run(&mut self, prog: &[Stmt]) -> Result<(), NovaError> {
        match self.exec_block(prog)? {
            Flow::Normal => Ok(()),
            Flow::Break => Err(NovaError::runtime("'break' outside of a loop")),
            Flow::Return(_) => Err(NovaError::runtime("'return' outside of a function")),
        }
    }

    fn set_var(&mut self, name: &str, v: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), v);
        }
    }

    // Functions read their own locals first, then globals.
    fn get_var(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.frames.last().and_then(|f| f.get(name)) {
            return Some(v.clone());
        }
        self.frames.first().and_then(|f| f.get(name)).cloned()
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Flow, NovaError> {
        for s in stmts {
            match self.exec(s)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec(&mut self, s: &Stmt) -> Result<Flow, NovaError> {
        self.exec_kind(&s.kind).map_err(|e| e.at(s.line))
    }

    fn exec_kind(&mut self, kind: &StmtKind) -> Result<Flow, NovaError> {
        match kind {
            StmtKind::Print(e) => {
                let v = self.eval(e)?;
                writeln!(self.out, "{}", v.display())
                    .map_err(|err| NovaError::runtime(format!("write error: {}", err)))?;
                Ok(Flow::Normal)
            }
            StmtKind::Assign(target, value) => {
                let v = self.eval(value)?;
                match target {
                    Expr::Var(name) => self.set_var(name, v),
                    Expr::Index(c, i) => {
                        let cv = self.eval(c)?;
                        let iv = self.eval(i)?;
                        set_index(&cv, &iv, v)?;
                    }
                    Expr::Field(base, fname) => {
                        let bv = self.eval(base)?;
                        match bv {
                            Value::Struct(sname, fields) => {
                                let mut fs = fields.borrow_mut();
                                match fs.iter_mut().find(|(k, _)| k == fname) {
                                    Some(entry) => entry.1 = v,
                                    None => {
                                        return Err(NovaError::runtime(format!(
                                            "struct '{}' has no field '{}'",
                                            sname, fname
                                        )))
                                    }
                                }
                            }
                            other => {
                                return Err(NovaError::runtime(format!(
                                    "cannot assign field '{}' of {}",
                                    fname,
                                    other.type_name()
                                )))
                            }
                        }
                    }
                    _ => return Err(NovaError::runtime("invalid assignment target")),
                }
                Ok(Flow::Normal)
            }
            StmtKind::ExprStmt(e) => {
                self.eval(e)?;
                Ok(Flow::Normal)
            }
            StmtKind::If(cond, then_body, else_body) => {
                if self.eval(cond)?.truthy() {
                    self.exec_block(then_body)
                } else {
                    self.exec_block(else_body)
                }
            }
            StmtKind::For(var, start, end, body) => {
                let s = self.eval(start)?;
                let e = self.eval(end)?;
                let (s, e) = match (s, e) {
                    (Value::Int(s), Value::Int(e)) => (s, e),
                    _ => return Err(NovaError::runtime("for..from..to bounds must be integers")),
                };
                let mut i = s;
                while i <= e {
                    self.set_var(var, Value::Int(i));
                    match self.exec_block(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal => {}
                    }
                    if i == i64::MAX {
                        break;
                    }
                    i += 1;
                }
                Ok(Flow::Normal)
            }
            StmtKind::While(cond, body) => {
                while self.eval(cond)?.truthy() {
                    match self.exec_block(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal => {}
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::Fun(def) => {
                match &def.owner {
                    Some(owner) => {
                        self.methods.insert((owner.clone(), def.name.clone()), def.clone());
                    }
                    None => {
                        self.functions.insert(def.name.clone(), def.clone());
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::Struct(def) => {
                self.structs.insert(def.name.clone(), def.clone());
                Ok(Flow::Normal)
            }
            StmtKind::Import(path) => {
                self.do_import(path)?;
                Ok(Flow::Normal)
            }
            StmtKind::Return(opt) => {
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(v))
            }
            StmtKind::Break => Ok(Flow::Break),
        }
    }

    fn do_import(&mut self, rel_path: &str) -> Result<(), NovaError> {
        let full = self.base_dir.join(rel_path);
        let canon = full.canonicalize().unwrap_or_else(|_| full.clone());

        if self.imported.contains(&canon) {
            return Ok(());
        }
        if self.importing.contains(&canon) {
            return Err(NovaError::runtime(format!("circular import: {}", rel_path)));
        }

        let src = std::fs::read_to_string(&full)
            .map_err(|e| NovaError::runtime(format!("cannot import '{}': {}", rel_path, e)))?;

        self.importing.insert(canon.clone());
        let tokens = crate::lexer::tokenize(&src)?;
        let program = crate::parser::Parser::new(tokens).parse_program()?;
        let result = self.exec_block(&program);
        self.importing.remove(&canon);

        match result? {
            Flow::Normal => {}
            Flow::Break => {
                return Err(NovaError::runtime(format!("'break' outside of a loop (in '{}')", rel_path)))
            }
            Flow::Return(_) => {
                return Err(NovaError::runtime(format!(
                    "'return' outside of a function (in '{}')",
                    rel_path
                )))
            }
        }

        self.imported.insert(canon);
        Ok(())
    }

    fn call_function(&mut self, def: &Rc<FunDef>, args: Vec<Value>) -> Result<Value, NovaError> {
        if args.len() != def.params.len() {
            let label = match &def.owner {
                Some(o) => format!("{}.{}", o, def.name),
                None => def.name.clone(),
            };
            return Err(NovaError::runtime(format!(
                "'{}' expects {} argument(s), got {}",
                label,
                def.params.len(),
                args.len()
            )));
        }
        if self.depth >= MAX_DEPTH {
            return Err(NovaError::runtime("maximum call depth exceeded"));
        }
        let mut frame = HashMap::new();
        for (p, a) in def.params.iter().zip(args.into_iter()) {
            frame.insert(p.clone(), a);
        }
        self.frames.push(frame);
        self.depth += 1;
        let result = self.exec_block(&def.body);
        self.depth -= 1;
        self.frames.pop();
        match result? {
            Flow::Return(v) => Ok(v),
            Flow::Normal => Ok(Value::Null),
            Flow::Break => Err(NovaError::runtime("'break' outside of a loop")),
        }
    }

    fn eval(&mut self, e: &Expr) -> Result<Value, NovaError> {
        match e {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for it in items {
                    v.push(self.eval(it)?);
                }
                Ok(new_list(v))
            }
            Expr::Map(pairs) => {
                let mut v: Vec<(Value, Value)> = Vec::new();
                for (k, val) in pairs {
                    let kv = self.eval(k)?;
                    check_key(&kv)?;
                    let vv = self.eval(val)?;
                    map_set(&mut v, kv, vv);
                }
                Ok(Value::Map(Rc::new(RefCell::new(v))))
            }
            Expr::Var(name) => self
                .get_var(name)
                .ok_or_else(|| NovaError::runtime(format!("Undefined variable: {}", name))),
            Expr::Index(c, i) => {
                let cv = self.eval(c)?;
                let iv = self.eval(i)?;
                get_index(&cv, &iv)
            }
            Expr::Field(base, name) => {
                let bv = self.eval(base)?;
                match bv {
                    Value::Struct(sname, fields) => {
                        let fs = fields.borrow();
                        match fs.iter().find(|(k, _)| k == name) {
                            Some((_, v)) => Ok(v.clone()),
                            None => Err(NovaError::runtime(format!(
                                "struct '{}' has no field '{}'",
                                sname, name
                            ))),
                        }
                    }
                    other => Err(NovaError::runtime(format!(
                        "cannot access field '{}' of {}",
                        name,
                        other.type_name()
                    ))),
                }
            }
            Expr::Call(name, args) => {
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                if let Some(sdef) = self.structs.get(name.as_str()).cloned() {
                    if argv.len() != sdef.fields.len() {
                        return Err(NovaError::runtime(format!(
                            "struct '{}' expects {} field(s), got {}",
                            sdef.name,
                            sdef.fields.len(),
                            argv.len()
                        )));
                    }
                    let pairs: Vec<(String, Value)> =
                        sdef.fields.iter().cloned().zip(argv.into_iter()).collect();
                    return Ok(Value::Struct(sdef.name.clone(), Rc::new(RefCell::new(pairs))));
                }
                if let Some(r) = builtin(name, &argv) {
                    return r;
                }
                let def = self.functions.get(name.as_str()).cloned();
                match def {
                    Some(d) => self.call_function(&d, argv),
                    None => Err(NovaError::runtime(format!("Unknown function: {}", name))),
                }
            }
            Expr::MethodCall(recv, method, args) => {
                let rv = self.eval(recv)?;
                match &rv {
                    Value::Struct(sname, _) => {
                        let def = self.methods.get(&(sname.clone(), method.clone())).cloned();
                        match def {
                            Some(d) => {
                                let mut argv = Vec::with_capacity(args.len() + 1);
                                argv.push(rv.clone());
                                for a in args {
                                    argv.push(self.eval(a)?);
                                }
                                self.call_function(&d, argv)
                            }
                            None => Err(NovaError::runtime(format!(
                                "struct '{}' has no method '{}'",
                                sname, method
                            ))),
                        }
                    }
                    other => Err(NovaError::runtime(format!(
                        "cannot call method '{}' on {}",
                        method,
                        other.type_name()
                    ))),
                }
            }
            Expr::Neg(x) => match self.eval(x)? {
                Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(overflow),
                Value::Float(f) => Ok(Value::Float(-f)),
                v => Err(NovaError::runtime(format!("cannot negate {}", v.type_name()))),
            },
            Expr::Not(x) => {
                let v = self.eval(x)?;
                Ok(Value::Bool(!v.truthy()))
            }
            Expr::And(a, b) => {
                if !self.eval(a)?.truthy() {
                    return Ok(Value::Bool(false));
                }
                let r = self.eval(b)?;
                Ok(Value::Bool(r.truthy()))
            }
            Expr::Or(a, b) => {
                if self.eval(a)?.truthy() {
                    return Ok(Value::Bool(true));
                }
                let r = self.eval(b)?;
                Ok(Value::Bool(r.truthy()))
            }
            Expr::Binary(op, a, b) => {
                let l = self.eval(a)?;
                let r = self.eval(b)?;
                binary(*op, l, r)
            }
        }
    }
}
