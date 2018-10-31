// api/mod.rs

use error::Error;
use function::callable::Callable;
use scanner::token::*;
use stack::*;
use worker::Worker;

#[derive(Debug, Clone)]
struct TestFun {}

impl Callable for TestFun {
    fn call(
        &self,
        _worker: &mut Worker,
        _paren: &Token,
        arguments: &Vec<Instance>,
    ) -> Result<Instance, Error> {
        println!("{}", "test");
        for arg in arguments {
            println!("{:?}", arg);
        }
        Ok(Instance::Nil)
    }
}

pub fn load_std_api(stack: &mut Stack) -> Result<(), Error> {
    stack.define(
        &Token {
            lexeme: "test".into(),
            line: 0,
            kind: TokenKind::Identifier,
        },
        Instance::Function(Box::new(TestFun {})),
    )
}
