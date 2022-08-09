extern crate wendell;

use std::env;
use wendell::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut intr = Interpreter::default();

    match args.len() {
        0 | 1 => {
            println!("wendell 0.0.1 interpreter.");
            println!("Press Ctrl^Z to exit.");
            intr.run_prompt();
        }
        2 => {
            intr.run_file(&args[1]);
        }
        _ => {
            println!("Usage: cargo run <script_file>");
        }
    };
}
