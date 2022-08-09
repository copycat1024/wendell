use error::Error;
use scanner::token::Token;
use stack::Instance;
use std::fmt::Debug;
use worker::Worker;

pub trait CallableClone {
    fn clone_box(&self) -> Box<dyn Callable>;
}

impl<T> CallableClone for T
where
    T: 'static + Callable + Clone,
{
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

pub trait Callable: Debug + CallableClone {
    fn call(
        &self,
        worker: &mut Worker,
        paren: &Token,
        arguments: &[Instance],
    ) -> Result<Instance, Error>;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.clone_box()
    }
}
