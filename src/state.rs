// state.rs

extern crate std;

use error::Error;
use scanner::token::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Instance {
    Nil,
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(Clone)]
pub struct State {
    values: HashMap<String, Instance>,
    enclosing: Option<Box<State>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn enclose(state: &State) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(state.to_owned())),
        }
    }

    pub fn define(&mut self, name: &Token, value: Instance) -> Result<(), Error> {
        let Token { lexeme, kind, line } = name;
        if let TokenKind::Identifier = kind {
            self.values.insert(lexeme.to_string(), value);
            Ok(())
        } else {
            self.error(
                format!("Expecting an identifier, found '{}' instead.", lexeme),
                *line,
            )
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<Instance, Error> {
        let Token { lexeme, kind, line } = name;
        if let TokenKind::Identifier = kind {
            if let Some(ins) = self.values.get(lexeme) {
                return Ok(ins.clone());
            }

            if let Some(ref mut state) = self.enclosing {
                return state.as_mut().get(name);
            }

            self.error(format!("Undefined variable '{}'.", lexeme), *line)
        } else {
            self.error(
                format!("Expecting an identifier, found '{}' instead.", lexeme),
                *line,
            )
        }
    }

    pub fn assign(&mut self, name: &Token, value: Instance) -> Result<Instance, Error> {
        let Token { lexeme, kind, line } = name;
        if let TokenKind::Identifier = kind {
            if let Some(ref mut entry) = self.values.get_mut(lexeme) {
                **entry = value.clone();
                return Ok(value);
            }

            if let Some(ref mut state) = self.enclosing {
                return state.as_mut().assign(name, value);
            }

            self.error(format!("Undefined variable '{}'.", lexeme), *line)
        } else {
            self.error(
                format!("Expecting an identifier, found '{}' instead.", lexeme),
                *line,
            )
        }
    }

    fn error<T>(&self, msg: String, line: u32) -> Result<T, Error> {
        Err(Error {
            line: line,
            msg: msg,
        })
    }
}
