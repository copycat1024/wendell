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
            TokenKind::LeftBrace => self.stmt_block(),
            TokenKind::If => self.stmt_if(),
            TokenKind::While => self.stmt_while(),
            TokenKind::For => self.stmt_for(),
            TokenKind::Print => self.stmt_print(),
            _ => self.stmt_expression(),
        }
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.expr_assignment()
    }

    fn decl_var(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat var token
        let name = self.consume(TokenKind::Identifier, "Expect variable name.")?;
        let init = if self.match_token(&[TokenKind::Equal]) {
            self.expression()?
        } else {
            Expr::new_literal(Token {
                kind: TokenKind::Nil,
                line: name.line,
                lexeme: "nil".into(),
            })
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after var statement.")?;

        Ok(Stmt::new_var(name, init))
    }

    fn stmt_block(&mut self) -> Result<Stmt, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        self.advance(); // eat '{' token

        while !self.check(TokenKind::RightBrace) && !self.is_eof() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after print statement.")?;
        Ok(Stmt::new_block(statements))
    }

    fn stmt_if(&mut self) -> Result<Stmt, Error> {
        let token = self.advance(); // eat 'if' token
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after if condition.")?;

        let then_block = self.statement()?;
        let else_block = if self.match_token(&[TokenKind::Else]) {
            self.statement()?
        } else {
            Stmt::Empty
        };

        Ok(Stmt::new_if(
            token.line,
            condition,
            Box::new(then_block),
            Box::new(else_block),
        ))
    }

    fn stmt_while(&mut self) -> Result<Stmt, Error> {
        let token = self.advance(); // eat 'while' token
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after while condition.")?;

        let body = self.statement()?;

        Ok(Stmt::new_while(token.line, condition, Box::new(body)))
    }

    fn stmt_for(&mut self) -> Result<Stmt, Error> {
        let token = self.advance(); // eat 'for' token

        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")?;

        let initializer = if self.match_token(&[TokenKind::Semicolon]) {
            Stmt::Empty
        } else if self.check(TokenKind::Var) {
            self.decl_var()?
        } else {
            self.stmt_expression()?
        };

        let condition = if self.check(TokenKind::Semicolon) {
            Expr::new_literal(Token {
                kind: TokenKind::True,
                lexeme: "true".to_string(),
                line: token.line,
            })
        } else {
            self.expression()?
        };
        self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TokenKind::RightParen) {
            Stmt::Empty
        } else {
            Stmt::new_expression(self.expression()?)
        };
        self.consume(TokenKind::RightParen, "Expect ')' after for clauses.")?;

        let body = self.statement()?;
        Ok(Stmt::new_block(vec![
            initializer,
            Stmt::new_while(
                token.line,
                condition,
                Box::new(Stmt::new_block(vec![body, increment])),
            ),
        ]))
    }

    fn stmt_expression(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::new_expression(expr))
    }

    fn stmt_print(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat print token
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after print statement.")?;
        Ok(Stmt::new_print(expr))
    }

    fn expr_assignment(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_or()?;
        if self.match_token(&[TokenKind::Equal]) {
            let value = self.expr_assignment()?;

            let new_expr = if let Expr::Variable { ref name } = expr {
                let name = name.clone();
                Some(Expr::new_assign(name, Box::new(value)))
            } else {
                None
            };

            if let Some(ex) = new_expr {
                replace(&mut expr, ex);
            } else {
                return self.error("Invalid expr_assignment target.".into());
            }
        }
        Ok(expr)
    }

    fn expr_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_and()?;

        while self.match_token(&[TokenKind::Or]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_equality()?;

        while self.match_token(&[TokenKind::And]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_comparison()?;

        while self.match_token(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_addition()?;

        while self.match_token(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.expr_addition()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_addition(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_multiplication()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.expr_multiplication()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_multiplication(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_unary()?;

        while self.match_token(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous();
            let right = self.expr_unary()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_unary(&mut self) -> Result<Expr, Error> {
        while self.match_token(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.expr_unary()?;
            return Ok(Expr::new_unary(operator, Box::new(right)));
        }

        self.expr_primary()
    }

    fn expr_call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_primary()?;

        loop {
            if self.match_token(&[TokenKind::LeftParen]) {
                self.finish_expr_call(&mut expr);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn expr_primary(&mut self) -> Result<Expr, Error> {
        let token = self.advance();
        match token.kind {
            TokenKind::False
            | TokenKind::True
            | TokenKind::Nil
            | TokenKind::NumberLiteral
            | TokenKind::StringLiteral => Ok(Expr::new_literal(token)),
            TokenKind::Identifier => Ok(Expr::new_variable(token)),
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::new_grouping(Box::new(expr)))
            }
            _ => self.error(format!("Unexpected token '{}'", token.to_string())),
        }
    }

    fn finish_expr_call(&mut self, callee: &mut Expr) -> Result<(), Error> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenKind::RightParen) {
            while {
                arguments.push(self.expression()?);
                self.match_token(&[TokenKind::Comma])
            } {}
        }
        let paren = self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;

        let old_callee = replace(callee, Expr::Empty);
        replace(callee, Expr::new_call(Box::new(old_callee), paren, arguments));

        Ok(())
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

        self.error(format!("{} Found '{:?}'", msg, self.peek()))
    }

    fn extend_binary(expr: &mut Expr, operator: Token, right: Expr) {
        let old_expr = replace(expr, Expr::Empty);
        let new_expr = Expr::new_binary(Box::new(old_expr), operator, Box::new(right));
        replace(expr, new_expr);
    }

    fn error<T>(&self, msg: String) -> Result<T, Error> {
        Err(Error {
            line: self.peek().line,
            msg: msg,
        })
    }
}
