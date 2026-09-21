//! Tree-walking evaluator with lexical closures.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::parser::{BinOp, Expr, LogicalOp, Stmt};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Unit,
    Func { params: Vec<String>, body: Vec<Stmt>, closure: Rc<RefCell<Env>> },
    Native(NativeFn),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({n})"),
            Value::Str(s) => write!(f, "Str({s:?})"),
            Value::Bool(b) => write!(f, "Bool({b})"),
            Value::Unit => write!(f, "Unit"),
            Value::Func { .. } => write!(f, "Func"),
            Value::Native(_) => write!(f, "Native"),
        }
    }
}

#[allow(clippy::type_complexity)]
type NativeFn = Rc<dyn Fn(&[Value]) -> Value>;

pub struct Env {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn global() -> Env {
        Env { values: HashMap::new(), parent: None }
    }

    pub fn child(parent: &Rc<RefCell<Env>>) -> Env {
        Env { values: HashMap::new(), parent: Some(parent.clone()) }
    }

    pub fn snapshot(&self) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            values: self.values.clone(),
            parent: self.parent.clone(),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(format!("undefined variable {name:?}"))
    }
}



type EvalResult = Result<Value, String>;

pub fn eval_block(stmts: &[Stmt], env: &mut Env) -> EvalResult {
    let mut last = Value::Unit;
    for stmt in stmts {
        last = eval_stmt(stmt, env)?;
    }
    Ok(last)
}

fn eval_stmt(stmt: &Stmt, env: &mut Env) -> EvalResult {
    match stmt {
        Stmt::Expr(expr) => eval_expr(expr, env),
        Stmt::Print(expr) => {
            let value = eval_expr(expr, env)?;
            crate::print_value(&value);
            Ok(Value::Unit)
        }
        Stmt::FnDecl { name, params, body } => {
            let func = Value::Func {
                params: params.clone(),
                body: body.clone(),
                closure: env.snapshot(),
            };
            env.define(name, func);
            Ok(Value::Unit)
        }
    }
}

fn eval_expr(expr: &Expr, env: &mut Env) -> EvalResult {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Ident(name) => env
            .get(name)
            .ok_or_else(|| format!("undefined variable {name:?}")),
        Expr::Binary { op, left, right } => {
            let l = eval_expr(left, env)?;
            let r = eval_expr(right, env)?;
            binary(*op, &l, &r)
        }
        Expr::Logical { op, left, right } => {
            let l = eval_expr(left, env)?;
            let truth = truthy(&l);
            match op {
                LogicalOp::And => {
                    if !truth { return Ok(Value::Bool(false)); }
                    let r = eval_expr(right, env)?;
                    Ok(Value::Bool(truthy(&r)))
                }
                LogicalOp::Or => {
                    if truth { return Ok(Value::Bool(true)); }
                    let r = eval_expr(right, env)?;
                    Ok(Value::Bool(truthy(&r)))
                }
            }
        }
        Expr::Unary { op, operand } => {
            let v = eval_expr(operand, env)?;
            match op {
                '-' => match v {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err("cannot negate a non-number".into()),
                },
                '!' => Ok(Value::Bool(!truthy(&v))),
                _ => Err("unknown unary op".into()),
            }
        }
        Expr::Call { callee, args } => {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval_expr(arg, env)?);
            }
            let func = env
                .get(callee)
                .ok_or_else(|| format!("undefined function {callee:?}"))?;
            call(&func, &values)
        }
        Expr::Assign { name, value } => {
            let v = eval_expr(value, env)?;
            env.assign(name, v.clone())?;
            Ok(v)
        }
        Expr::Let { name, value } => {
            let v = eval_expr(value, env)?;
            env.define(name, v.clone());
            Ok(v)
        }
        Expr::If { cond, then_branch, else_branch } => {
            let c = eval_expr(cond, env)?;
            if truthy(&c) {
                eval_block(then_branch, env)
            } else {
                eval_block(else_branch, env)
            }
        }
        Expr::While { cond, body } => {
            loop {
                let c = eval_expr(cond, env)?;
                if !truthy(&c) {
                    break;
                }
                eval_block(body, env)?;
            }
            Ok(Value::Unit)
        }
    }
}

fn call(func: &Value, args: &[Value]) -> EvalResult {
    match func {
        Value::Native(f) => Ok(f(args)),
        Value::Func { params, body, closure } => {
            if args.len() != params.len() {
                return Err(format!(
                    "expected {} argument(s), got {}",
                    params.len(),
                    args.len()
                ));
            }
            let mut local = Env::child(closure);
            for (param, arg) in params.iter().zip(args) {
                local.define(param, arg.clone());
            }
            eval_block(body, &mut local)
        }
        _ => Err("value is not callable".into()),
    }
}

fn binary(op: BinOp, l: &Value, r: &Value) -> EvalResult {
    use BinOp::*;
    match op {
        Add => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            (Value::Str(a), other) => Ok(Value::Str(format!("{a}{}", crate::display(other)))),
            _ => Err("bad operands for +".into()),
        },
        Sub | Mul | Div | Mod => {
            let (Value::Number(a), Value::Number(b)) = (l, r) else {
                return Err("arithmetic needs numbers".into());
            };
            match op {
                Sub => Ok(Value::Number(a - b)),
                Mul => Ok(Value::Number(a * b)),
                Div => {
                    if *b == 0.0 {
                        Err("division by zero".into())
                    } else {
                        Ok(Value::Number(a / b))
                    }
                }
                Mod => Ok(Value::Number(a % b)),
                _ => unreachable!(),
            }
        }
        Eq => Ok(Value::Bool(values_eq(l, r))),
        Ne => Ok(Value::Bool(!values_eq(l, r))),
        Lt | Le | Gt | Ge => {
            let ord = match (l, r) {
                (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
                (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
                _ => None,
            }
            .ok_or_else(|| "incomparable operands".to_string())?;
            match op {
                Lt => Ok(Value::Bool(ord == std::cmp::Ordering::Less)),
                Le => Ok(Value::Bool(ord != std::cmp::Ordering::Greater)),
                Gt => Ok(Value::Bool(ord == std::cmp::Ordering::Greater)),
                Ge => Ok(Value::Bool(ord != std::cmp::Ordering::Less)),
                _ => unreachable!(),
            }
        }
    }
}

fn values_eq(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Unit | Value::Func { .. } | Value::Native(_) => true,
        Value::Number(n) => *n != 0.0,
        Value::Str(s) => !s.is_empty(),
    }
}

