// function/callable.rs

use error::Error;
use scanner::token::Token;
use stack::Instance;
use std::fmt::Debug;
use worker::Worker;

pub trait CallableClone {
    fn clone_box(&self) -> Box<Callable>;
}

impl<T> CallableClone for T
where
    T: 'static + Callable + Clone,
{
    fn clone_box(&self) -> Box<Callable> {
        Box::new(self.clone())
    }
}

pub trait Callable: Debug + CallableClone {
    fn call(
        &self,
        worker: &mut Worker,
        paren: &Token,
        arguments: &Vec<Instance>,
    ) -> Result<Instance, Error>;
}

impl Clone for Box<Callable> {
    fn clone(&self) -> Box<Callable> {
        self.clone_box()
    }
}
