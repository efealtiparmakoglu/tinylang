//! Tokenizer.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Str(String),
    Ident(String),
    Let, Fn, If, Else, While, Print, True, False, And, Or, Not,
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    Assign,
    LParen, RParen, LBrace, RBrace, Comma, Semicolon,
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '#' => while i < chars.len() && chars[i] != '\n' { i += 1 },
            '"' => {
                let mut s = String::new();
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                        match chars[i] {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            other => s.push(other),
                        }
                    } else {
                        s.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("unterminated string".into());
                }
                i += 1;
                tokens.push(Token::Str(s));
            }
            '0'..='9' => {
                let start = i;
                let mut is_float = false;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    if chars[i] == '.' {
                        if is_float { break; }
                        is_float = true;
                    }
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(Token::Number(text.parse().map_err(
                    |_| format!("bad number {text:?}"))?));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                tokens.push(match word.as_str() {
                    "let" => Token::Let, "fn" => Token::Fn,
                    "if" => Token::If, "else" => Token::Else,
                    "while" => Token::While, "print" => Token::Print,
                    "true" => Token::True, "false" => Token::False,
                    "and" => Token::And, "or" => Token::Or, "not" => Token::Not,
                    _ => Token::Ident(word),
                });
            }
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '=' => {
                if chars.get(i + 1) == Some(&'=') {
                    tokens.push(Token::Eq); i += 2;
                } else {
                    tokens.push(Token::Assign); i += 1;
                }
            }
            '!' => {
                if chars.get(i + 1) == Some(&'=') {
                    tokens.push(Token::Ne); i += 2;
                } else {
                    tokens.push(Token::Not); i += 1;
                }
            }
            '<' => {
                if chars.get(i + 1) == Some(&'=') {
                    tokens.push(Token::Le); i += 2;
                } else {
                    tokens.push(Token::Lt); i += 1;
                }
            }
            '>' => {
                if chars.get(i + 1) == Some(&'=') {
                    tokens.push(Token::Ge); i += 2;
                } else {
                    tokens.push(Token::Gt); i += 1;
                }
            }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            '{' => { tokens.push(Token::LBrace); i += 1; }
            '}' => { tokens.push(Token::RBrace); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            ';' => { tokens.push(Token::Semicolon); i += 1; }
            other => return Err(format!("unexpected character {other:?}")),
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}
