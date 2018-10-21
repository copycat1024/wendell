// parser.rs

extern crate std;

use ast::expr::*;
use ast::stmt::*;
use error::Error;
use scanner::token::{Token, TokenKind};
use std::mem::replace;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub stmts: Vec<Stmt>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens,
            current: 0,
            stmts: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        while !self.is_eof() {
            let decl = self.declaration()?;
            self.stmts.push(decl);
        }

        Ok(())
    }

    pub fn synchronize(&mut self) {
        self.advance();

        while !self.is_eof() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => {
                    return;
                }
                _ => (),
            }
            self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        let token = self.peek();
        match token.kind {
            TokenKind::Var => self.decl_var(),
            _ => self.statement(),
        }
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        let token = self.peek();
        match token.kind {
            TokenKind::Print => self.stmt_print(),
            _ => self.stmt_expression(),
        }
    }

    fn decl_var(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat var token
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?;
        let init = if self.match_token(&[TokenKind::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Literal {
                value: Token {
                    kind: TokenKind::Nil,
                    line: name.line,
                    lexeme: "nil".into(),
                },
            })
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after print statement.")?;

        Ok(Stmt::Var(Var {
            name: name,
            initializer: init,
        }))
    }

    fn stmt_print(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat print token
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after print statement.")?;
        Ok(Stmt::Print(Print { expression: expr }))
    }

    fn stmt_expression(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Expression { expression: expr }))
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;
        if self.match_token(&[TokenKind::Equal]) {
            let value = self.assignment()?;

            let new_expr = if let Expr::Variable(ref v) = expr {
                let name = v.name.clone();
                drop(v);
                Some(Expr::Assign(Assign {
                    name: name,
                    value: Box::new(value),
                }))
            } else {
                None
            };

            if let Some(ex) = new_expr {
                replace(&mut expr, ex);
            } else {
                return self.error("Invalid assignment target.".into());
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            let old_expr = replace(&mut expr, Expr::Empty);
            let new_expr = Expr::Binary(Binary {
                left: Box::new(old_expr),
                operator: operator,
                right: Box::new(right),
            });
            replace(&mut expr, new_expr);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.addition()?;

        while self.match_token(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.addition()?;
            let old_expr = replace(&mut expr, Expr::Empty);
            let new_expr = Expr::Binary(Binary {
                left: Box::new(old_expr),
                operator: operator,
                right: Box::new(right),
            });
            replace(&mut expr, new_expr);
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, Error> {
        let mut expr = self.multiplication()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.multiplication()?;
            let old_expr = replace(&mut expr, Expr::Empty);
            let new_expr = Expr::Binary(Binary {
                left: Box::new(old_expr),
                operator: operator,
                right: Box::new(right),
            });
            replace(&mut expr, new_expr);
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;
            let old_expr = replace(&mut expr, Expr::Empty);
            let new_expr = Expr::Binary(Binary {
                left: Box::new(old_expr),
                operator: operator,
                right: Box::new(right),
            });
            replace(&mut expr, new_expr);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        while self.match_token(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary {
                operator: operator,
                right: Box::new(right),
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = self.advance();
        match token.kind {
            TokenKind::False
            | TokenKind::True
            | TokenKind::Nil
            | TokenKind::NumberLiteral
            | TokenKind::StringLiteral => Ok(Expr::Literal(Literal { value: token })),
            TokenKind::Identifier => Ok(Expr::Variable(Variable { name: token })),
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Grouping {
                    expression: Box::new(expr),
                }))
            }
            _ => self.error(format!("Unexpected token '{}'", token.to_string())),
        }
    }

    pub fn is_eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        if !self.is_eof() {
            self.current += 1;
        }
        self.previous()
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds.iter() {
            if self.check(*kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        self.peek().kind == kind
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, Error> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        Err(Error {
            line: self.peek().line,
            msg: msg.into(),
        })
    }

    fn error<T>(&self, msg: String) -> Result<T, Error> {
        Err(Error {
            line: self.peek().line,
            msg: msg,
        })
    }
}
