use ast::expr::*;
use ast::stmt::*;
use error::Error;
use function::callable::Callable;
use function::*;
use scanner::token::{Token, TokenKind};
use stack::*;

pub struct Worker<'a> {
    pub stack: &'a mut Stack,
}

impl<'a> Worker<'a> {
    pub fn new(stack: &'a mut Stack) -> Self {
        Self { stack }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        for stmt in stmts.iter() {
            self.execute(stmt)?;
        }
        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), Error> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Instance, Error> {
        expr.accept(self)
    }

    fn primitive_not(&mut self, operator: &Token, value: &Instance) -> Result<Instance, Error> {
        match value {
            Instance::Bool(v) => Ok(Instance::Bool(!v)),
            _ => self.unary_error(operator, "Bool", value),
        }
    }

    fn primitive_neg(&mut self, operator: &Token, value: &Instance) -> Result<Instance, Error> {
        match value {
            Instance::Number(v) => Ok(Instance::Number(-v)),
            _ => self.unary_error(operator, "Number", value),
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
                _ => self.binary_error(operator, "Number", value1, value2),
            },
            Instance::String(ref v1) => match value2 {
                Instance::String(v2) => Ok(Instance::String(format!("{}{}", v1, v2))),
                _ => self.binary_error(operator, "String", value1, value2),
            },
            _ => self.binary_error(operator, "Number | String", value1, value2),
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
                _ => self.binary_error(operator, "Number", value1, value2),
            },
            _ => self.binary_error(operator, "Number", value1, value2),
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
                _ => self.binary_error(operator, "Number", value1, value2),
            },
            _ => self.binary_error(operator, "Number", value1, value2),
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
                _ => self.binary_error(operator, "Number", value1, value2),
            },
            _ => self.binary_error(operator, "Number", value1, value2),
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
            _ => Ok(Instance::Bool(false)),
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
                _ => self.binary_error(operator, "Number", value1, value2),
            },
            Instance::String(ref v1) => match value2 {
                Instance::String(ref v2) => Ok(Instance::Bool(v1 < v2)),
                _ => self.binary_error(operator, "String", value1, value2),
            },
            _ => self.binary_error(operator, "Number | String", value1, value2),
        }
    }

    fn primitive_or(
        &mut self,
        operator: &Token,
        expr1: &Expr,
        expr2: &Expr,
    ) -> Result<Instance, Error> {
        let value1 = match self.evaluate(expr1)? {
            Instance::Bool(value1) => value1,
            value1 => return self.unary_error(operator, "Bool", &value1),
        };
        if value1 {
            Ok(Instance::Bool(value1))
        } else {
            match self.evaluate(expr2)? {
                Instance::Bool(value2) => Ok(Instance::Bool(value2)),
                value2 => self.unary_error(operator, "Bool", &value2),
            }
        }
    }

    fn primitive_and(
        &mut self,
        operator: &Token,
        expr1: &Expr,
        expr2: &Expr,
    ) -> Result<Instance, Error> {
        let value1 = match self.evaluate(expr1)? {
            Instance::Bool(value1) => value1,
            value1 => return self.unary_error(operator, "Bool", &value1),
        };
        if !value1 {
            Ok(Instance::Bool(value1))
        } else {
            match self.evaluate(expr2)? {
                Instance::Bool(value2) => Ok(Instance::Bool(value2)),
                value2 => self.unary_error(operator, "Bool", &value2),
            }
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
        let Token { kind, line } = operator;
        self.error(
            format!("{:?} operator is not a {} operator.", kind, operator_kind),
            *line,
        )
    }

    fn unary_error(
        &self,
        operator: &Token,
        expected: &str,
        value: &Instance,
    ) -> Result<Instance, Error> {
        let Token { kind, line } = operator;
        self.error(
            format!(
                "{:?} operator expected type '{}', found '{:?}' instead.",
                kind, expected, value
            ),
            *line,
        )
    }

    fn condition_error(
        &self,
        statement_kind: &str,
        line: &u32,
        value: &Instance,
    ) -> Result<(), Error> {
        self.error(
            format!(
                "{} statement condition must be 'Bool', found '{:?}' instead.",
                statement_kind, value
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
        let Token { kind, line } = operator;
        self.error(
            format!(
                "{:?} operator expected type '{}', found '{:?}' and '{:?}' instead.",
                kind, expected, value1, value2
            ),
            *line,
        )
    }

    fn error<T>(&self, msg: String, line: u32) -> Result<T, Error> {
        Err(Error { line, msg })
    }
}

impl<'a> ExprVisitor<Result<Instance, Error>> for Worker<'a> {
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> Result<Instance, Error> {
        let assign_value = self.evaluate(value)?;
        self.stack.assign(name, assign_value)
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Result<Instance, Error> {
        self.evaluate(expression)
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Instance, Error> {
        match operator.kind {
            TokenKind::Or => return self.primitive_or(operator, left, right),
            TokenKind::And => return self.primitive_and(operator, left, right),
            _ => (),
        };

        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

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

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Result<Instance, Error> {
        let right = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Bang => self.primitive_not(operator, &right),
            TokenKind::Minus => self.primitive_neg(operator, &right),
            _ => self.operator_error(operator, "unary"),
        }
    }

    fn visit_call(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
    ) -> Result<Instance, Error> {
        let callee = self.evaluate(callee)?;
        let mut unpacked_arg: Vec<Instance> = Vec::new();

        for arg in arguments {
            unpacked_arg.push(self.evaluate(arg)?);
        }

        if let Instance::Function(fun) = callee {
            fun.call(self, paren, &unpacked_arg)
        } else {
            self.error(
                format!("Expected a function, found '{:?}' instead", callee),
                paren.line,
            )
        }
    }

    fn visit_literal(&mut self, value: &Token) -> Result<Instance, Error> {
        let Token { kind, line } = value;

        let ins = match kind {
            TokenKind::NumberLiteral(value_string) => match value_string.parse::<f64>() {
                Ok(n) => Instance::Number(n),
                Err(e) => self.error(
                    format!("Cannot parse '{}' into number ({}).", value_string, e),
                    *line,
                )?,
            },
            TokenKind::StringLiteral(value_string) => Instance::String(value_string.to_string()),
            TokenKind::True => Instance::Bool(true),
            TokenKind::False => Instance::Bool(false),
            TokenKind::Nil => Instance::Nil,
            _ => unreachable!(),
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
    fn visit_var(&mut self, name: &Token, initializer: &Expr) -> Result<(), Error> {
        let value = self.evaluate(initializer)?;
        self.stack.define(name, value)
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        self.stack.push();
        for stmt in statements {
            self.execute(stmt)?;
        }
        self.stack.pop();
        Ok(())
    }

    fn visit_if(
        &mut self,
        line_number: &u32,
        condition: &Expr,
        then_block: &Stmt,
        else_block: &Stmt,
    ) -> Result<(), Error> {
        match self.evaluate(condition)? {
            Instance::Bool(con) => {
                if con {
                    self.execute(then_block)
                } else {
                    self.execute(else_block)
                }
            }
            other => self.condition_error("If", line_number, &other),
        }
    }

    fn visit_while(
        &mut self,
        line_number: &u32,
        condition: &Expr,
        body: &Stmt,
    ) -> Result<(), Error> {
        loop {
            match self.evaluate(condition)? {
                Instance::Bool(con) => {
                    if con {
                        self.execute(body)?;
                    } else {
                        break;
                    }
                }
                other => self.condition_error("While", line_number, &other)?,
            }
        }
        Ok(())
    }

    fn visit_function(
        &mut self,
        name: &Token,
        params: &[Token],
        body: &Stmt,
    ) -> Result<(), Error> {
        let fun = AulUserFunction::new(name, params, body);
        let wrapped_fun = Box::new(fun) as Box<dyn Callable>;
        self.stack.define(name, Instance::Function(wrapped_fun))?;
        Ok(())
    }

    fn visit_expression(&mut self, expression: &Expr) -> Result<(), Error> {
        self.evaluate(expression)?;
        Ok(())
    }

    fn visit_print(&mut self, expression: &Expr) -> Result<(), Error> {
        let value = self.evaluate(expression)?;
        match value {
            Instance::String(s) => println!("{}", s),
            Instance::Number(n) => println!("{}", n),
            Instance::Bool(b) => println!("{}", b),
            Instance::Function(_) => println!("function"),
            Instance::Nil => println!("nil"),
        }

        Ok(())
    }

    fn visit_empty_stmt(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
