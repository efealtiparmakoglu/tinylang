# tinylang

**A small tree-walking interpreter in pure Rust — lexer, Pratt parser, closures, REPL. Zero dependencies.**

[![CI](https://github.com/efealtiparmakoglu/tinylang/actions/workflows/ci.yml/badge.svg)](https://github.com/efealtiparmakoglu/tinylang/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

```
$ tinylang
tl> fn fib(n) { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
tl> fib(20)
6765
```

tinylang is a complete little dynamic language: a hand-written lexer, a
Pratt-style recursive-descent parser, and a tree-walking evaluator with
lexical closures.

## Language

```
let greeting = "hi " + "there"
print greeting                    # hi there

fn fib(n) { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
print fib(15)                     # 610

let i = 0
while i < 3 { print i; i = i + 1 }
```

- Numbers (floats), strings (with escapes), booleans
- Arithmetic `+ - * / %`, comparison `== != < <= > >=`, `and`/`or`/`not`
- `let` bindings, assignment, `if`/`else`, `while`
- First-class functions with closures (environment snapshot)
- `print` builtin; REPL with persistent environment

## Development

```sh
cargo test
cargo clippy --all-targets
```

Tests cover arithmetic precedence, comparison/logic, string concat,
variables, branches, loops, functions with closures, and runtime error
handling.

## License

MIT — see [LICENSE](LICENSE).
