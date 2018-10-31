// function/mod.rs

pub mod callable;

use self::callable::Callable;
use ast::stmt::Stmt;
use error::Error;
use scanner::token::Token;
use stack::Instance;
use worker::Worker;

#[derive(Debug, Clone)]
pub struct AulUserFunction {
    name: Token,
    params: Vec<Token>,
    body: Stmt,
}

impl AulUserFunction {
    pub fn new(name: &Token, params: &Vec<Token>, body: &Box<Stmt>) -> Self {
        Self {
            name: name.clone(),
            params: params.clone(),
            body: body.as_ref().clone(),
        }
    }
}

impl Callable for AulUserFunction {
    fn call(
        &self,
        worker: &mut Worker,
        _paren: &Token,
        arguments: &Vec<Instance>,
    ) -> Result<Instance, Error> {
        let mut i = 0;
        worker.stack.push();
        for ref p in self.params.iter() {
            if let Some(ins) = arguments.get(i) {
                worker.stack.define(p, ins.clone())?;
            } else {
                worker.stack.define(p, Instance::Nil)?;
            }
            i += 1;
        }

        worker.execute(&self.body)?;
        worker.stack.pop();
        Ok(Instance::Nil)
    }
}
