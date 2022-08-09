mod api;
mod ast;
mod error;
mod function;
mod parser;
mod scanner;
mod stack;
mod worker;

pub mod interpreter;

#[cfg(test)]
mod test {
    #[test]
    fn main() {
    }
}
