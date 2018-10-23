// ast/expr.rs

use scanner::token::Token;

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: Token,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable {
        name: Token,
    },

    Empty,
}

impl Expr {
    pub fn accept<R, T:ExprVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
        Expr::Assign {
            ref name,
            ref value,
        } => visitor.visit_assign(
            name,
            value,
        ),
        Expr::Binary {
            ref left,
            ref operator,
            ref right,
        } => visitor.visit_binary(
            left,
            operator,
            right,
        ),
        Expr::Grouping {
            ref expression,
        } => visitor.visit_grouping(
            expression,
        ),
        Expr::Literal {
            ref value,
        } => visitor.visit_literal(
            value,
        ),
        Expr::Unary {
            ref operator,
            ref right,
        } => visitor.visit_unary(
            operator,
            right,
        ),
        Expr::Variable {
            ref name,
        } => visitor.visit_variable(
            name,
        ),
            Expr::Empty => visitor.visit_empty_expr(),
        }
    }

    pub fn new_assign(
        name: Token,
        value: Box<Expr>,
    ) -> Self {
        Expr::Assign {
        name: name,
        value: value,
        }
    }

    pub fn new_binary(
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    ) -> Self {
        Expr::Binary {
        left: left,
        operator: operator,
        right: right,
        }
    }

    pub fn new_grouping(
        expression: Box<Expr>,
    ) -> Self {
        Expr::Grouping {
        expression: expression,
        }
    }

    pub fn new_literal(
        value: Token,
    ) -> Self {
        Expr::Literal {
        value: value,
        }
    }

    pub fn new_unary(
        operator: Token,
        right: Box<Expr>,
    ) -> Self {
        Expr::Unary {
        operator: operator,
        right: right,
        }
    }

    pub fn new_variable(
        name: Token,
    ) -> Self {
        Expr::Variable {
        name: name,
        }
    }
}

pub trait ExprVisitor<R> {
    fn visit_assign(&mut self, name: &Token, value: &Box<Expr>) -> R;
    fn visit_binary(&mut self, left: &Box<Expr>, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_grouping(&mut self, expression: &Box<Expr>) -> R;
    fn visit_literal(&mut self, value: &Token) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Box<Expr>) -> R;
    fn visit_variable(&mut self, name: &Token) -> R;
    fn visit_empty_expr(&mut self) -> R;
}

