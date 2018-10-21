// ast/stmt.rs

use ast::expr::Expr;
use scanner::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression(Expression),
    Print(Print),
    Var(Var),
    Empty,
}

#[derive(Debug)]
pub struct Expression {
    pub expression: Expr,
}

#[derive(Debug)]
pub struct Print {
    pub expression: Expr,
}

#[derive(Debug)]
pub struct Var {
    pub name: Token,
    pub initializer: Expr,
}

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, n: &Stmt) -> T;
    fn visit_expression(&mut self, n: &Expression) -> T;
    fn visit_print(&mut self, n: &Print) -> T;
    fn visit_var(&mut self, n: &Var) -> T;
}

