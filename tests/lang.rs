#[allow(unused_imports)]
use tinylang::{global_env, run};

fn eval(src: &str) -> Result<tinylang::interp::Value, String> {
    let mut env = global_env();
    tinylang::run(src, &mut env)
}

fn eval_num(src: &str) -> f64 {
    match eval(src).unwrap() {
        tinylang::interp::Value::Number(n) => n,
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn arithmetic_precedence() {
    assert_eq!(eval_num("1 + 2 * 3"), 7.0);
    assert_eq!(eval_num("(1 + 2) * 3"), 9.0);
    assert_eq!(eval_num("10 % 3"), 1.0);
    assert_eq!(eval_num("-2 * 4"), -8.0);
}

#[test]
fn comparison_and_logic() {
    assert!(matches!(eval("1 < 2").unwrap(), tinylang::interp::Value::Bool(true)));
    assert!(matches!(eval("1 > 2 and 2 > 1").unwrap(), tinylang::interp::Value::Bool(false)));
    assert!(matches!(eval("1 > 2 or 2 > 1").unwrap(), tinylang::interp::Value::Bool(true)));
    assert!(matches!(eval("not true").unwrap(), tinylang::interp::Value::Bool(false)));
    assert!(matches!(eval("\"abc\" == \"abc\"").unwrap(), tinylang::interp::Value::Bool(true)));
}

#[test]
fn variables_and_strings() {
    let v = eval("let greeting = \"hi \" + \"there\"\ngreeting").unwrap();
    assert!(matches!(v, tinylang::interp::Value::Str(s) if s == "hi there"));
}

#[test]
fn if_else_branches() {
    assert_eq!(eval_num("if 1 < 2 { 10 } else { 20 }"), 10.0);
    assert_eq!(eval_num("if 2 < 1 { 10 } else { 20 }"), 20.0);
}

#[test]
fn while_loop() {
    assert_eq!(
        eval_num("let i = 0\nwhile i < 5 { i = i + 1 }\ni"),
        5.0
    );
}

#[test]
fn functions_and_closures() {
    assert_eq!(eval_num("fn add(a, b) { a + b }\nadd(2, 3)"), 5.0);
    // closure captures environment
    let value = eval(
        "fn make_counter() { let n = 0; fn inc() { n = n + 1; n } inc }\nlet c = make_counter()\nc()\nc()\nc()",
    )
    .unwrap();
    assert!(matches!(value, tinylang::interp::Value::Number(n) if n == 3.0));
}

#[test]
fn runtime_errors_reported() {
    assert!(eval("1 / 0").is_err());
    assert!(eval("undefined_var").is_err());
    assert!(eval("\"a\" + 1").is_ok()); // string concat coerces
    assert!(eval("\"a\" * 2").is_err());
}

#[test]
fn lexer_rejects_bad_input() {
    let mut env = global_env();
    assert!(tinylang::run("@@@", &mut env).is_err());
}
