// stack.rs

use error::Error;
use function::callable::Callable;
use scanner::token::*;
use std::collections::HashMap;
use std::mem::replace;

#[derive(Debug, Clone)]
pub enum Instance {
    Nil,
    Number(f64),
    String(String),
    Bool(bool),
    Function(Box<Callable>),
}

pub struct Stack {
    pub height: usize,
    values: HashMap<String, Instance>,
    next: Link,
}

type Link = Option<Box<Stack>>;

impl Stack {
    pub fn new() -> Self {
        Self::raw_new(0)
    }

    pub fn push(&mut self) {
        let height = self.height;
        let old_self = replace(self, Self::raw_new(height + 1));
        self.next = Some(Box::new(old_self));
    }

    pub fn pop(&mut self) {
        let new_self = match replace(&mut self.next, None) {
            Some(link) => link,
            None => panic!("Cannot pop global scope"),
        };
        replace(self, *new_self);
    }

    pub fn define(&mut self, name: &Token, value: Instance) -> Result<(), Error> {
        let Token { kind, .. } = name;
        if let TokenKind::Identifier(var_name) = kind {
            self.values.insert(var_name.to_string(), value);
            return Ok(());
        }
        unreachable!()
    }

    pub fn get(&self, name: &Token) -> Result<Instance, Error> {
        let Token { kind, line } = name;
        if let TokenKind::Identifier(var_name) = kind {
            if let Some(ins) = self.values.get(var_name) {
                return Ok(ins.clone());
            }

            if let Some(ref stack) = self.next {
                return stack.get(name);
            }

            return self.error(format!("Undefined variable '{}'.", var_name), *line);
        }
        unreachable!()
    }

    pub fn assign(&mut self, name: &Token, value: Instance) -> Result<Instance, Error> {
        let Token { kind, line } = name;
        if let TokenKind::Identifier(var_name) = kind {
            if let Some(ref mut entry) = self.values.get_mut(var_name) {
                **entry = value.clone();
                return Ok(value);
            }

            if let Some(ref mut stack) = self.next {
                return stack.as_mut().assign(name, value);
            }

            return self.error(format!("Undefined variable '{}'.", var_name), *line);
        }
        unreachable!()
    }

    fn raw_new(height: usize) -> Self {
        Self {
            values: HashMap::new(),
            height: height,
            next: None,
        }
    }

    fn error<T>(&self, msg: String, line: u32) -> Result<T, Error> {
        Err(Error {
            line: line,
            msg: msg,
        })
    }
}
