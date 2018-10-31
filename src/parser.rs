// parser.rs

use ast::expr::*;
use ast::stmt::*;
use error::Error;
use scanner::token::TokenKind::*;
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
            if self.previous().kind == Semicolon {
                return;
            }
            match self.peek().kind {
                Class | Fun | Var | For | If | While | Print | Return => {
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
            Var => self.decl_var(),
            Fun => self.decl_fun("function"),
            _ => self.statement(),
        }
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        let token = self.peek();
        match token.kind {
            LeftBrace => self.stmt_block(),
            If => self.stmt_if(),
            While => self.stmt_while(),
            For => self.stmt_for(),
            Print => self.stmt_print(),
            _ => self.stmt_expression(),
        }
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.expr_assignment()
    }

    fn decl_var(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat var token
        let name = self.consume(&Identifier("".to_string()), "Expect variable name.")?;
        let init = if self.match_token(&[Equal]) {
            self.expression()?
        } else {
            Expr::new_literal(Token::new(Nil, name.line))
        };
        self.consume(&Semicolon, "Expect ';' after var statement.")?;

        Ok(Stmt::new_var(name, init))
    }

    fn decl_fun(&mut self, kind: &str) -> Result<Stmt, Error> {
        self.advance(); // eat fun token
        let name = self.consume(
            &Identifier("".to_string()),
            &format!("Expect {} name.", kind),
        )?;
        self.consume(&LeftParen, &format!("Expect '(' after {} name.", kind))?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(&RightParen) {
            while {
                parameters
                    .push(self.consume(&Identifier("".to_string()), "Expect parameter name.")?);
                self.match_token(&[Comma])
            } {}
        }
        self.consume(&RightParen, "Expect ')' after parameters.")?;

        if !self.check(&LeftBrace) {
            self.error::<()>(format!("Expect '{{' before {} body.", kind))?;
        }

        let body = self.stmt_block()?;
        Ok(Stmt::new_function(name, parameters, Box::new(body)))
    }

    fn stmt_block(&mut self) -> Result<Stmt, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        self.advance(); // eat '{' token

        while !self.check(&RightBrace) && !self.is_eof() {
            statements.push(self.declaration()?);
        }

        self.consume(&RightBrace, "Expect '}' after print statement.")?;
        Ok(Stmt::new_block(statements))
    }

    fn stmt_if(&mut self) -> Result<Stmt, Error> {
        let token = self.advance(); // eat 'if' token
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after if condition.")?;

        let then_block = self.statement()?;
        let else_block = if self.match_token(&[Else]) {
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
        self.consume(&LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&RightParen, "Expect ')' after while condition.")?;

        let body = self.statement()?;

        Ok(Stmt::new_while(token.line, condition, Box::new(body)))
    }

    fn stmt_for(&mut self) -> Result<Stmt, Error> {
        let token = self.advance(); // eat 'for' token

        self.consume(&LeftParen, "Expect '(' after 'if'.")?;

        let initializer = if self.match_token(&[Semicolon]) {
            Stmt::Empty
        } else if self.check(&Var) {
            self.decl_var()?
        } else {
            self.stmt_expression()?
        };

        let condition = if self.check(&Semicolon) {
            Expr::new_literal(Token::new(True, token.line))
        } else {
            self.expression()?
        };
        self.consume(&Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(&RightParen) {
            Stmt::Empty
        } else {
            Stmt::new_expression(self.expression()?)
        };
        self.consume(&RightParen, "Expect ')' after for clauses.")?;

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
        self.consume(&Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::new_expression(expr))
    }

    fn stmt_print(&mut self) -> Result<Stmt, Error> {
        self.advance(); // eat print token
        let expr = self.expression()?;
        self.consume(&Semicolon, "Expect ';' after print statement.")?;
        Ok(Stmt::new_print(expr))
    }

    fn expr_assignment(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_or()?;
        if self.match_token(&[Equal]) {
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

        while self.match_token(&[Or]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_equality()?;

        while self.match_token(&[And]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_comparison()?;

        while self.match_token(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.expr_comparison()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_addition()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.expr_addition()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_addition(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_multiplication()?;

        while self.match_token(&[Plus, Minus]) {
            let operator = self.previous();
            let right = self.expr_multiplication()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_multiplication(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_unary()?;

        while self.match_token(&[Star, Slash]) {
            let operator = self.previous();
            let right = self.expr_unary()?;
            Self::extend_binary(&mut expr, operator, right);
        }

        Ok(expr)
    }

    fn expr_unary(&mut self) -> Result<Expr, Error> {
        while self.match_token(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.expr_unary()?;
            return Ok(Expr::new_unary(operator, Box::new(right)));
        }

        self.expr_call()
    }

    fn expr_call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.expr_primary()?;

        loop {
            if self.match_token(&[LeftParen]) {
                self.finish_expr_call(&mut expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn expr_primary(&mut self) -> Result<Expr, Error> {
        let token = self.advance();
        match token.kind {
            False | True | Nil | NumberLiteral(_) | StringLiteral(_) => {
                Ok(Expr::new_literal(token))
            }
            Identifier(_) => Ok(Expr::new_variable(token)),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(&RightParen, "Expect ')' after expression.")?;
                Ok(Expr::new_grouping(Box::new(expr)))
            }
            _ => self.error(format!("Unexpected token '{}'", token.to_string())),
        }
    }

    fn finish_expr_call(&mut self, callee: &mut Expr) -> Result<(), Error> {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(&RightParen) {
            while {
                arguments.push(self.expression()?);
                self.match_token(&[Comma])
            } {}
        }
        let paren = self.consume(&RightParen, "Expect ')' after arguments.")?;

        let old_callee = replace(callee, Expr::Empty);
        replace(
            callee,
            Expr::new_call(Box::new(old_callee), paren, arguments),
        );

        Ok(())
    }

    pub fn is_eof(&self) -> bool {
        self.peek().kind == Eof
    }

    fn advance(&mut self) -> Token {
        if !self.is_eof() {
            self.current += 1;
        }
        self.previous()
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds.iter() {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, kind: &TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        self.peek().kind.match_kind(kind)
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, kind: &TokenKind, msg: &str) -> Result<Token, Error> {
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
