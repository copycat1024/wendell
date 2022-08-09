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
    pub fn new(name: &Token, params: &[Token], body: &Stmt) -> Self {
        Self {
            name: name.clone(),
            params: params.to_vec(),
            body: body.clone(),
        }
    }
}

impl Callable for AulUserFunction {
    fn call(
        &self,
        worker: &mut Worker,
        _paren: &Token,
        arguments: &[Instance],
    ) -> Result<Instance, Error> {
        worker.stack.push();
        for (i, p) in self.params.iter().enumerate() {
            if let Some(ins) = arguments.get(i) {
                worker.stack.define(p, ins.clone())?;
            } else {
                worker.stack.define(p, Instance::Nil)?;
            }
        }

        worker.execute(&self.body)?;
        worker.stack.pop();
        Ok(Instance::Nil)
    }
}
