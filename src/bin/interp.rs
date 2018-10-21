// bin/interp.rs

extern crate aulac;

use std::env;
use aulac::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut intr = Interpreter::new();
    if args.len() > 2 {
        println!("Usage: cargo run <script_file>");
    } else if args.len() == 2 {
        intr.run_file(&args[1]);
    } else {
        println!("Aulac 0.0.1 interpreter.");
        println!("Press Ctrl^Z to exit.");
        intr.run_prompt();
    }
}
