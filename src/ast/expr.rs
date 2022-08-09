use scanner::token::Token;

#[derive(Debug, Clone)]
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

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    Variable {
        name: Token,
    },

    Empty,
}

impl Expr {
    pub fn accept<R, T: ExprVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
            Expr::Assign {
                ref name,
                ref value,
            } => visitor.visit_assign(name, value),
            Expr::Binary {
                ref left,
                ref operator,
                ref right,
            } => visitor.visit_binary(left, operator, right),
            Expr::Grouping { ref expression } => visitor.visit_grouping(expression),
            Expr::Literal { ref value } => visitor.visit_literal(value),
            Expr::Unary {
                ref operator,
                ref right,
            } => visitor.visit_unary(operator, right),
            Expr::Call {
                ref callee,
                ref paren,
                ref arguments,
            } => visitor.visit_call(callee, paren, arguments),
            Expr::Variable { ref name } => visitor.visit_variable(name),
            Expr::Empty => visitor.visit_empty_expr(),
        }
    }

    pub fn new_assign(name: Token, value: Box<Expr>) -> Self {
        Expr::Assign { name, value }
    }

    pub fn new_binary(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
        Expr::Binary {
            left,
            operator,
            right,
        }
    }

    pub fn new_grouping(expression: Box<Expr>) -> Self {
        Expr::Grouping { expression }
    }

    pub fn new_literal(value: Token) -> Self {
        Expr::Literal { value }
    }

    pub fn new_unary(operator: Token, right: Box<Expr>) -> Self {
        Expr::Unary { operator, right }
    }

    pub fn new_call(callee: Box<Expr>, paren: Token, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            callee,
            paren,
            arguments,
        }
    }

    pub fn new_variable(name: Token) -> Self {
        Expr::Variable { name }
    }
}

pub trait ExprVisitor<R> {
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> R;
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_literal(&mut self, value: &Token) -> R;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> R;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &[Expr]) -> R;
    fn visit_variable(&mut self, name: &Token) -> R;
    fn visit_empty_expr(&mut self) -> R;
}
