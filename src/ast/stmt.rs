// ast/stmt.rs

use ast::expr::Expr;
use scanner::token::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Var {
        name: Token,
        initializer: Expr,
    },

    Block {
        statements: Vec<Stmt>,
    },

    If {
        line_number: u32,
        condition: Expr,
        then_block: Box<Stmt>,
        else_block: Box<Stmt>,
    },

    While {
        line_number: u32,
        condition: Expr,
        body: Box<Stmt>,
    },

    Function {
        name: Token,
        params: Vec<Token>,
        body: Box<Stmt>,
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
    pub fn accept<R, T: StmtVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
            Stmt::Var {
                ref name,
                ref initializer,
            } => visitor.visit_var(name, initializer),
            Stmt::Block { ref statements } => visitor.visit_block(statements),
            Stmt::If {
                ref line_number,
                ref condition,
                ref then_block,
                ref else_block,
            } => visitor.visit_if(line_number, condition, then_block, else_block),
            Stmt::While {
                ref line_number,
                ref condition,
                ref body,
            } => visitor.visit_while(line_number, condition, body),
            Stmt::Function {
                ref name,
                ref params,
                ref body,
            } => visitor.visit_function(name, params, body),
            Stmt::Expression { ref expression } => visitor.visit_expression(expression),
            Stmt::Print { ref expression } => visitor.visit_print(expression),
            Stmt::Empty => visitor.visit_empty_stmt(),
        }
    }

    pub fn new_var(name: Token, initializer: Expr) -> Self {
        Stmt::Var {
            name: name,
            initializer: initializer,
        }
    }

    pub fn new_block(statements: Vec<Stmt>) -> Self {
        Stmt::Block {
            statements: statements,
        }
    }

    pub fn new_if(
        line_number: u32,
        condition: Expr,
        then_block: Box<Stmt>,
        else_block: Box<Stmt>,
    ) -> Self {
        Stmt::If {
            line_number: line_number,
            condition: condition,
            then_block: then_block,
            else_block: else_block,
        }
    }

    pub fn new_while(line_number: u32, condition: Expr, body: Box<Stmt>) -> Self {
        Stmt::While {
            line_number: line_number,
            condition: condition,
            body: body,
        }
    }

    pub fn new_function(name: Token, params: Vec<Token>, body: Box<Stmt>) -> Self {
        Stmt::Function {
            name: name,
            params: params,
            body: body,
        }
    }

    pub fn new_expression(expression: Expr) -> Self {
        Stmt::Expression {
            expression: expression,
        }
    }

    pub fn new_print(expression: Expr) -> Self {
        Stmt::Print {
            expression: expression,
        }
    }
}

pub trait StmtVisitor<R> {
    fn visit_var(&mut self, name: &Token, initializer: &Expr) -> R;
    fn visit_block(&mut self, statements: &Vec<Stmt>) -> R;
    fn visit_if(
        &mut self,
        line_number: &u32,
        condition: &Expr,
        then_block: &Box<Stmt>,
        else_block: &Box<Stmt>,
    ) -> R;
    fn visit_while(&mut self, line_number: &u32, condition: &Expr, body: &Box<Stmt>) -> R;
    fn visit_function(&mut self, name: &Token, params: &Vec<Token>, body: &Box<Stmt>) -> R;
    fn visit_expression(&mut self, expression: &Expr) -> R;
    fn visit_print(&mut self, expression: &Expr) -> R;
    fn visit_empty_stmt(&mut self) -> R;
}
