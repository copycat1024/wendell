// ast/expr.rs

use scanner::token::Token;

#[derive(Debug)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Empty,
}

#[derive(Debug)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug)]
pub struct Literal {
    pub value: Token,
}

#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: Token,
}

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, n: &Expr) -> T;
    fn visit_assign(&mut self, n: &Assign) -> T;
    fn visit_binary(&mut self, n: &Binary) -> T;
    fn visit_grouping(&mut self, n: &Grouping) -> T;
    fn visit_literal(&mut self, n: &Literal) -> T;
    fn visit_unary(&mut self, n: &Unary) -> T;
    fn visit_variable(&mut self, n: &Variable) -> T;
}

