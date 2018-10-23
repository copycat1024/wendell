// interpreter.rs

extern crate std;

use ast::stmt::Stmt;
use error::Error;
use parser::Parser;
use scanner::token::Token;
use scanner::Scanner;
use stack::Stack;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use worker::Worker;

pub struct Interpreter {
    error_flag: bool,
    stack: Stack,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            error_flag: false,
            stack: Stack::new(),
        }
    }

    pub fn run_file(&mut self, file_name: &str) {
        let mut fh = File::open(file_name).expect("File not found");

        let mut contents = String::new();
        fh.read_to_string(&mut contents)
            .expect("Error while reading the file.");

        self.run(contents, 1).ok();
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        let mut iter = stdin.lock().lines();
        let mut line_num = 1;
        let print_head = |num: &u32| {
            print!("{:>4}> ", num);
            io::stdout().flush().unwrap();
        };
        print_head(&line_num);
        while let Some(line) = iter.next() {
            self.error_flag = false;
            if let Ok(_) = self.run(line.unwrap(), line_num) {
                line_num += 1;
            }
            print_head(&line_num);
        }
        println!("");
        println!("Exited on end of stream.");
    }

    fn run(&mut self, code: String, start_line: u32) -> Result<(), ()> {
        let Scanner { tokens, .. } = self.scan(code, start_line)?;
        let Parser { stmts, .. } = self.parse(tokens)?;
        self.execute(stmts)
    }

    fn scan(&mut self, code: String, start_line: u32) -> Result<Scanner, ()> {
        let mut scanner = Scanner::new(code, start_line);
        while let Err(e) = scanner.scan_all_tokens() {
            self.report_error(e)
        }
        if self.error_flag {
            Err(())
        } else {
            Ok(scanner)
        }
    }

    fn parse(&mut self, tokens: Vec<Token>) -> Result<Parser, ()> {
        let mut parser = Parser::new(tokens);
        while let Err(e) = parser.parse() {
            self.report_error(e);
            parser.synchronize();
        }
        if self.error_flag {
            Err(())
        } else {
            Ok(parser)
        }
    }

    fn execute(&mut self, stmts: Vec<Stmt>) -> Result<(), ()> {
        let result = {
            let mut worker = Worker::new(&mut self.stack);
            worker.run(&stmts)
        };
        if let Err(e) = result {
            self.report_error(e);
            Err(())
        } else {
            Ok(())
        }
    }

    fn report_error(&mut self, e: Error) {
        self.report(e.line, e.msg);
    }

    fn report(&mut self, line: u32, msg: String) {
        println!("[line {}] Error: {}", line, msg);
        self.error_flag = true;
    }
}
