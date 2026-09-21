//! tiny REPL binary

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let source = std::fs::read_to_string(&args[1])
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", args[1]));
        let mut env = tinylang::global_env();
        if let Err(e) = tinylang::run(&source, &mut env) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    println!("tinylang 1.0.0 — type :q to quit");
    tinylang::repl().unwrap();
}
