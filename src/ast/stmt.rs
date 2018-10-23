// ast/stmt.rs

use ast::expr::Expr;
use scanner::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Var {
        name: Token,
        initializer: Expr,
    },

    Block {
        statements: Vec<Stmt>,
    },

    Expression {
        expression: Expr,
    },

    Print {
        expression: Expr,
    },

    Empty,
}

impl Stmt {
    pub fn accept<R, T:StmtVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
        Stmt::Var {
            ref name,
            ref initializer,
        } => visitor.visit_var(
            name,
            initializer,
        ),
        Stmt::Block {
            ref statements,
        } => visitor.visit_block(
            statements,
        ),
        Stmt::Expression {
            ref expression,
        } => visitor.visit_expression(
            expression,
        ),
        Stmt::Print {
            ref expression,
        } => visitor.visit_print(
            expression,
        ),
            Stmt::Empty => visitor.visit_empty_stmt(),
        }
    }

    pub fn new_var(
        name: Token,
        initializer: Expr,
    ) -> Self {
        Stmt::Var {
        name: name,
        initializer: initializer,
        }
    }

    pub fn new_block(
        statements: Vec<Stmt>,
    ) -> Self {
        Stmt::Block {
        statements: statements,
        }
    }

    pub fn new_expression(
        expression: Expr,
    ) -> Self {
        Stmt::Expression {
        expression: expression,
        }
    }

    pub fn new_print(
        expression: Expr,
    ) -> Self {
        Stmt::Print {
        expression: expression,
        }
    }
}

pub trait StmtVisitor<R> {
    fn visit_var(&mut self, name: &Token, initializer: &Expr) -> R;
    fn visit_block(&mut self, statements: &Vec<Stmt>) -> R;
    fn visit_expression(&mut self, expression: &Expr) -> R;
    fn visit_print(&mut self, expression: &Expr) -> R;
    fn visit_empty_stmt(&mut self) -> R;
}

