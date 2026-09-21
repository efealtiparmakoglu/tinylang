//! Pratt parser: tokens -> AST.

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Logical { op: LogicalOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: char, operand: Box<Expr> },
    Call { callee: String, args: Vec<Expr> },
    Assign { name: String, value: Box<Expr> },
    Let { name: String, value: Box<Expr> },
    If { cond: Box<Expr>, then_branch: Vec<Stmt>, else_branch: Vec<Stmt> },
    While { cond: Box<Expr>, body: Vec<Stmt> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Le, Gt, Ge }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOp { And, Or }

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    FnDecl { name: String, params: Vec<String>, body: Vec<Stmt> },
}

pub type Program = Vec<Stmt>;

pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 };
    let mut program = Vec::new();
    while p.peek() != &Token::Eof {
        program.push(p.statement()?);
        // optional semicolon separator
        if p.peek() == &Token::Semicolon {
            p.advance();
        }
    }
    Ok(program)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        t
    }
    fn expect(&mut self, token: &Token) -> Result<(), String> {
        if self.peek() == token {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected {token:?}, got {:?}", self.peek()))
        }
    }
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            stmts.push(self.statement()?);
            if self.peek() == &Token::Semicolon {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        // assignment: Ident = expr
        if let Token::Ident(name) = self.peek().clone() {
            if self.tokens.get(self.pos + 1) == Some(&Token::Assign) {
                self.advance(); // name
                self.advance(); // =
                let value = self.expression()?;
                return Ok(Stmt::Expr(Expr::Assign {
                    name,
                    value: Box::new(value),
                }));
            }
        }
        match self.peek() {
            Token::Print => {
                self.advance();
                Ok(Stmt::Print(self.expression()?))
            }
            Token::Let => {
                self.advance();
                let name = match self.advance() {
                    Token::Ident(name) => name,
                    t => return Err(format!("expected name after let, got {t:?}")),
                };
                self.expect(&Token::Assign)?;
                let value = self.expression()?;
                Ok(Stmt::Expr(Expr::Let { name, value: Box::new(value) }))
            }
            Token::Fn => {
                self.advance();
                let name = match self.advance() {
                    Token::Ident(name) => name,
                    t => return Err(format!("expected fn name, got {t:?}")),
                };
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                if self.peek() != &Token::RParen {
                    loop {
                        match self.advance() {
                            Token::Ident(p) => params.push(p),
                            t => return Err(format!("bad param {t:?}")),
                        }
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RParen)?;
                let body = self.block()?;
                Ok(Stmt::FnDecl { name, params, body })
            }
            Token::If => {
                self.advance();
                let cond = self.expression()?;
                let then_branch = self.block()?;
                let else_branch = if self.peek() == &Token::Else {
                    self.advance();
                    self.block()?
                } else {
                    Vec::new()
                };
                Ok(Stmt::Expr(Expr::If {
                    cond: Box::new(cond),
                    then_branch,
                    else_branch,
                }))
            }
            Token::While => {
                self.advance();
                let cond = self.expression()?;
                let body = self.block()?;
                Ok(Stmt::Expr(Expr::While { cond: Box::new(cond), body }))
            }
            _ => Ok(Stmt::Expr(self.expression()?)),
        }
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.and_expr()?;
        while self.peek() == &Token::Or {
            self.advance();
            left = Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(self.and_expr()?),
            };
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.comparison()?;
        while self.peek() == &Token::And {
            self.advance();
            left = Expr::Logical {
                op: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(self.comparison()?),
            };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.term()?;
        let op = match self.peek() {
            Token::Eq => BinOp::Eq, Token::Ne => BinOp::Ne,
            Token::Lt => BinOp::Lt, Token::Le => BinOp::Le,
            Token::Gt => BinOp::Gt, Token::Ge => BinOp::Ge,
            _ => return Ok(left),
        };
        self.advance();
        Ok(Expr::Binary {
            op, left: Box::new(left), right: Box::new(self.term()?),
        })
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.factor()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            left = Expr::Binary {
                op, left: Box::new(left), right: Box::new(self.factor()?),
            };
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            left = Expr::Binary {
                op, left: Box::new(left), right: Box::new(self.unary()?),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Ok(Expr::Unary { op: '-', operand: Box::new(self.unary()?) })
            }
            Token::Not => {
                self.advance();
                Ok(Expr::Unary { op: '!', operand: Box::new(self.unary()?) })
            }
            _ => self.primary_with_calls(),
        }
    }

    fn primary_with_calls(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        while self.peek() == &Token::LParen {
            self.advance();
            let mut args = Vec::new();
            if self.peek() != &Token::RParen {
                loop {
                    args.push(self.expression()?);
                    if self.peek() == &Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.expect(&Token::RParen)?;
            let callee = match expr {
                Expr::Ident(ref name) => name.clone(),
                _ => return Err("only named functions are callable".into()),
            };
            expr = Expr::Call { callee, args };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let t = self.advance();
        match t {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Str(s) => Ok(Expr::Str(s)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Ident(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let expr = self.expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            other => Err(format!("unexpected {other:?}")),
        }
    }
}
