// worker.rs

extern crate std;

use ast::expr::*;
use ast::stmt::*;
use error::Error;
use scanner::token::{Token, TokenKind};
use stack::*;

pub struct Worker<'a> {
    stack: &'a mut Stack,
}

impl<'a> Worker<'a> {
    pub fn new(stack: &'a mut Stack) -> Self {
        Self { stack: stack }
    }

    pub fn execute(&mut self, stmts: Vec<Stmt>) -> Result<(), Error> {
        for stmt in stmts.into_iter() {
            stmt.accept(self)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Instance, Error> {
        expr.accept(self)
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
    fn visit_assign(&mut self, name: &Token, value: &Box<Expr>) -> Result<Instance, Error> {
        let assign_value = self.evaluate(value)?;
        self.stack.assign(name, assign_value)
    }

    fn visit_grouping(&mut self, expression: &Box<Expr>) -> Result<Instance, Error> {
        self.evaluate(expression)
    }

    fn visit_binary(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<Instance, Error> {
        let left = self.evaluate(&left)?;
        let right = self.evaluate(&right)?;

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

    fn visit_unary(&mut self, operator: &Token, right: &Box<Expr>) -> Result<Instance, Error> {
        let right = self.evaluate(&right)?;

        match operator.kind {
            TokenKind::Bang => self.primitive_not(operator, &right),
            TokenKind::Minus => self.primitive_neg(operator, &right),
            _ => self.operator_error(operator, "unary"),
        }
    }

    fn visit_literal(&mut self, value: &Token) -> Result<Instance, Error> {
        let Token { kind, lexeme, line } = value;

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

    fn visit_variable(&mut self, name: &Token) -> Result<Instance, Error> {
        self.stack.get(name)
    }

    fn visit_empty_expr(&mut self) -> Result<Instance, Error> {
        self.error("Found empty Expr.".into(), 0)
    }
}

impl<'a> StmtVisitor<Result<(), Error>> for Worker<'a> {
    fn visit_print(&mut self, expression: &Expr) -> Result<(), Error> {
        let value = self.evaluate(expression)?;
        match value {
            Instance::String(s) => println!("{}", s),
            Instance::Number(n) => println!("{}", n),
            Instance::Bool(b) => println!("{}", b),
            Instance::Nil => println!("nil"),
        }

        Ok(())
    }

    fn visit_expression(&mut self, expression: &Expr) -> Result<(), Error> {
        let value = self.evaluate(expression)?;

        match value {
            _ => {
                println!("{:?}", value);
                Ok(())
            }
        }
    }

    fn visit_var(&mut self, name: &Token, initializer: &Expr) -> Result<(), Error> {
        let value = self.evaluate(initializer)?;
        self.stack.define(name, value)
    }

    fn visit_empty_stmt(&mut self) -> Result<(), Error> {
        self.error("Found empty Expr.".into(), 0)
    }

    fn visit_block(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        self.stack.push();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.stack.pop();
        Ok(())
    }
}
