//! tinylang: a tree-walking interpreter for a small dynamic language.
//!
//! Features: numbers, strings, booleans, arithmetic, comparison, and/or/not,
//! let bindings, if/else, while, functions with closures, and print.
//! Pipeline: tokenize -> Pratt parse -> AST -> evaluate.

pub mod lexer;
pub mod parser;
pub mod interp;

use std::io::Write;
use std::rc::Rc;

use interp::{Env, Value};

pub fn run(source: &str, env: &mut Env) -> Result<Value, String> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(&tokens)?;
    interp::eval_block(&ast, env)
}

pub fn run_std(source: &str) -> Result<Value, String> {
    let mut env = Env::global();
    run(source, &mut env)
}

/// REPL: line-based, keeps the environment between inputs.
pub fn repl() -> Result<(), std::io::Error> {
    let mut env = Env::global();
    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        print!("tl> ");
        std::io::stdout().flush()?;
        line.clear();
        if stdin.read_line(&mut line)? == 0 {
            println!();
            return Ok(());
        }
        let src = line.trim();
        if src.is_empty() || src == ":q" || src == ":quit" {
            if src == ":q" || src == ":quit" {
                return Ok(());
            }
            continue;
        }
        match run(src, &mut env) {
            Ok(value) => {
                if !matches!(value, Value::Unit) {
                    println!("{}", display(&value));
                }
            }
            Err(e) => println!("error: {e}"),
        }
    }
}

pub fn print_value(value: &Value) {
    println!("{}", display(value));
}

pub fn display(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        Value::Str(s) => s.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Unit => "()".to_string(),
        Value::Func { .. } => "<fn>".to_string(),
        Value::Native(_) => "<native>".to_string(),
    }
}

pub fn global_env() -> Env {
    let mut env = Env::global();
    env.define("print", Value::Native(Rc::new(|args| {
        let line = args.iter().map(display).collect::<Vec<_>>().join(" ");
        println!("{line}");
        Value::Unit
    })));
    env.define("clock", Value::Native(Rc::new(|_args| {
        Value::Number(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0))
    })));
    env
}
