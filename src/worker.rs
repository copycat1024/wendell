// worker.rs

extern crate std;

use ast::expr::*;
use ast::stmt::*;
use error::Error;
use scanner::token::{Token, TokenKind};
use state::*;

pub struct Worker<'a> {
    state: &'a mut State,
}

impl<'a> Worker<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state: state }
    }

    pub fn execute(&mut self, stmts: Vec<Stmt>) -> Result<(), Error> {
        for stmt in stmts.into_iter() {
            self.visit_stmt(&stmt)?;
        }
        Ok(())
    }

    fn primitive_not(&mut self, operator: &Token, value: &Instance) -> Result<Instance, Error> {
        match value {
            Instance::Bool(v) => Ok(Instance::Bool(!v)),
            _ => self.unary_error(operator, "Bool", &value),
        }
    }

    fn primitive_neg(&mut self, operator: &Token, value: &Instance) -> Result<Instance, Error> {
        match value {
            Instance::Number(v) => Ok(Instance::Number(-v)),
            _ => self.unary_error(operator, "Number", &value),
        }
    }

    fn primitive_add(
        &mut self,
        operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Number(v1 + v2)),
                _ => self.binary_error(operator, "Number", &value1, &value2),
            },
            Instance::String(ref v1) => match value2 {
                Instance::String(v2) => Ok(Instance::String(format!("{}{}", v1, v2))),
                _ => self.binary_error(operator, "String", &value1, &value2),
            },
            _ => self.binary_error(operator, "Number | String", &value1, &value2),
        }
    }

    fn primitive_sub(
        &mut self,
        operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Number(v1 - v2)),
                _ => self.binary_error(operator, "Number", &value1, &value2),
            },
            _ => self.binary_error(operator, "Number", &value1, &value2),
        }
    }

    fn primitive_mul(
        &mut self,
        operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Number(v1 * v2)),
                _ => self.binary_error(operator, "Number", &value1, &value2),
            },
            _ => self.binary_error(operator, "Number", &value1, &value2),
        }
    }

    fn primitive_div(
        &mut self,
        operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Number(v1 / v2)),
                _ => self.binary_error(operator, "Number", &value1, &value2),
            },
            _ => self.binary_error(operator, "Number", &value1, &value2),
        }
    }

    fn primitive_eq(
        &self,
        _operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Bool(v1 == v2)),
                _ => Ok(Instance::Bool(false)),
            },
            Instance::String(v1) => match value2 {
                Instance::String(v2) => Ok(Instance::Bool(v1 == v2)),
                _ => Ok(Instance::Bool(false)),
            },
            Instance::Bool(v1) => match value2 {
                Instance::Bool(v2) => Ok(Instance::Bool(v1 == v2)),
                _ => Ok(Instance::Bool(false)),
            },
            Instance::Nil => match value2 {
                Instance::Nil => Ok(Instance::Bool(true)),
                _ => Ok(Instance::Bool(false)),
            },
        }
    }

    fn primitive_less(
        &self,
        operator: &Token,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        match value1 {
            Instance::Number(v1) => match value2 {
                Instance::Number(v2) => Ok(Instance::Bool(v1 < v2)),
                _ => self.binary_error(operator, "Number", &value1, &value2),
            },
            Instance::String(ref v1) => match value2 {
                Instance::String(ref v2) => Ok(Instance::Bool(v1 < v2)),
                _ => self.binary_error(operator, "String", &value1, &value2),
            },
            _ => self.binary_error(operator, "Number | String", &value1, &value2),
        }
    }

    fn raw_not(&self, value: Result<Instance, Error>) -> Result<Instance, Error> {
        match value? {
            Instance::Bool(v) => Ok(Instance::Bool(!v)),
            _ => self.error("raw_not was called on none Bool".into(), 0),
        }
    }

    fn raw_or(
        &self,
        value1: Result<Instance, Error>,
        value2: Result<Instance, Error>,
    ) -> Result<Instance, Error> {
        match value1? {
            Instance::Bool(v1) => match value2? {
                Instance::Bool(v2) => Ok(Instance::Bool(v1 || v2)),
                _ => self.error("raw_or was called on none Bool".into(), 0),
            },
            _ => self.error("raw_or was called on none Bool".into(), 0),
        }
    }

    fn operator_error(&self, operator: &Token, operator_kind: &str) -> Result<Instance, Error> {
        let Token { kind, lexeme, line } = operator;
        self.error(
            format!(
                "{:?} operator '{}' is not a {} operator.",
                kind, lexeme, operator_kind
            ),
            *line,
        )
    }

    fn unary_error(
        &self,
        operator: &Token,
        expected: &str,
        value: &Instance,
    ) -> Result<Instance, Error> {
        let Token { kind, lexeme, line } = operator;
        self.error(
            format!(
                "{:?} operator '{}' expected type '{}', found '{:?}' instead.",
                kind, lexeme, expected, value
            ),
            *line,
        )
    }

    fn binary_error(
        &self,
        operator: &Token,
        expected: &str,
        value1: &Instance,
        value2: &Instance,
    ) -> Result<Instance, Error> {
        let Token { kind, lexeme, line } = operator;
        self.error(
            format!(
                "{:?} operator '{}' expected type '{}', found '{:?}' and '{:?}' instead.",
                kind, lexeme, expected, value1, value2
            ),
            *line,
        )
    }

    fn error<T>(&self, msg: String, line: u32) -> Result<T, Error> {
        Err(Error {
            line: line,
            msg: msg,
        })
    }
}

impl<'a> ExprVisitor<Result<Instance, Error>> for Worker<'a> {
    fn visit_expr(&mut self, n: &Expr) -> Result<Instance, Error> {
        match n {
            Expr::Binary(n) => self.visit_binary(n),
            Expr::Grouping(n) => self.visit_grouping(n),
            Expr::Literal(n) => self.visit_literal(n),
            Expr::Unary(n) => self.visit_unary(n),
            Expr::Variable(n) => self.visit_variable(n),
            Expr::Assign(n) => self.visit_assign(n),
            Expr::Empty => self.error("Found empty Expr.".into(), 0),
        }
    }

    fn visit_assign(&mut self, n: &Assign) -> Result<Instance, Error> {
        let Assign { name, value } = n;
        let assign_value = self.visit_expr(value)?;
        self.state.assign(name, assign_value)
    }

    fn visit_grouping(&mut self, n: &Grouping) -> Result<Instance, Error> {
        self.visit_expr(&n.expression)
    }

    fn visit_binary(&mut self, n: &Binary) -> Result<Instance, Error> {
        let Binary {
            operator,
            left,
            right,
        } = n;

        let left = self.visit_expr(&left)?;
        let right = self.visit_expr(&right)?;

        match operator.kind {
            TokenKind::Plus => self.primitive_add(operator, &left, &right),
            TokenKind::Minus => self.primitive_sub(operator, &left, &right),
            TokenKind::Star => self.primitive_mul(operator, &left, &right),
            TokenKind::Slash => self.primitive_div(operator, &left, &right),
            TokenKind::EqualEqual => self.primitive_eq(operator, &left, &right),
            TokenKind::BangEqual => self.raw_not(self.primitive_eq(operator, &left, &right)),
            TokenKind::Less => self.primitive_less(operator, &left, &right),
            TokenKind::Greater => self.raw_not(self.raw_or(
                self.primitive_less(operator, &left, &right),
                self.primitive_eq(operator, &left, &right),
            )),
            TokenKind::LessEqual => self.raw_or(
                self.primitive_less(operator, &left, &right),
                self.primitive_eq(operator, &left, &right),
            ),
            TokenKind::GreaterEqual => self.raw_not(self.primitive_less(operator, &left, &right)),
            _ => self.operator_error(operator, "binary"),
        }
    }

    fn visit_unary(&mut self, n: &Unary) -> Result<Instance, Error> {
        let Unary { operator, right } = n;

        let right = self.visit_expr(&right)?;

        match operator.kind {
            TokenKind::Bang => self.primitive_not(operator, &right),
            TokenKind::Minus => self.primitive_neg(operator, &right),
            _ => self.operator_error(operator, "unary"),
        }
    }

    fn visit_literal(&mut self, n: &Literal) -> Result<Instance, Error> {
        let Literal {
            value: Token { kind, lexeme, line },
        } = n;

        let ins = match kind {
            TokenKind::NumberLiteral => match lexeme.parse::<f64>() {
                Ok(n) => Instance::Number(n),
                Err(e) => self.error(
                    format!("Cannot parse '{}' into number ({}).", lexeme, e),
                    *line,
                )?,
            },
            TokenKind::StringLiteral => Instance::String(lexeme.to_string()),
            TokenKind::True => Instance::Bool(true),
            TokenKind::False => Instance::Bool(false),
            TokenKind::Nil => Instance::Nil,
            _ => self.error(format!("'{}' is not a literal.", lexeme), *line)?,
        };

        Ok(ins)
    }

    fn visit_variable(&mut self, n: &Variable) -> Result<Instance, Error> {
        let Variable { name } = n;
        self.state.get(name)
    }
}

impl<'a> StmtVisitor<Result<(), Error>> for Worker<'a> {
    fn visit_stmt(&mut self, n: &Stmt) -> Result<(), Error> {
        match n {
            Stmt::Expression(n) => self.visit_expression(n),
            Stmt::Print(n) => self.visit_print(n),
            Stmt::Var(n) => self.visit_var(n),
            Stmt::Empty => self.error("Found empty Expr.".into(), 0),
        }
    }

    fn visit_print(&mut self, n: &Print) -> Result<(), Error> {
        let n = &n.expression;

        let v = self.visit_expr(n)?;
        match v {
            Instance::String(s) => println!("{}", s),
            Instance::Number(n) => println!("{}", n),
            Instance::Bool(b) => println!("{}", b),
            Instance::Nil => println!("nil"),
        }

        Ok(())
    }

    fn visit_expression(&mut self, n: &Expression) -> Result<(), Error> {
        let n = &n.expression;
        let v = self.visit_expr(n)?;

        match v {
            _ => {
                println!("{:?}", v);
                Ok(())
            }
        }
    }

    fn visit_var(&mut self, n: &Var) -> Result<(), Error> {
        let Var { name, initializer } = n;
        let value = self.visit_expr(initializer)?;
        self.state.define(name, value)
    }
}
