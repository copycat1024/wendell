pub mod token;

use self::token::TokenKind::*;
use self::token::{Token, TokenKind};
use error::Error;

pub struct Scanner {
    pub source: Vec<char>,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: String, start_line: u32) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: start_line,
        }
    }

    pub fn scan_all_tokens(&mut self) -> Result<(), Error> {
        while !self.is_eof() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.add_token(Eof);

        Ok(())
    }

    fn scan_token(&mut self) -> Result<(), Error> {
        let c = self.advance();
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '!' => {
                let kind = if self.match_char('=') {
                    BangEqual
                } else {
                    Bang
                };
                self.add_token(kind);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(kind);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    LessEqual
                } else {
                    Less
                };
                self.add_token(kind);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(kind);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_eof() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash);
                }
            }
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => self.line += 1,
            '"' => self.add_string_literal()?,
            x if Self::is_digit(x) => self.add_number_literal(),
            x if Self::is_alpha(x) => self.add_identifier(),
            _ => return self.error(format!("Unknown character '{}'", c)),
        }
        Ok(())
    }

    fn add_string_literal(&mut self) -> Result<(), Error> {
        while self.peek() != '"' && !self.is_eof() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // Unterminated string
        if self.is_eof() {
            return self.error("Unterminated string.".into());
        }

        // The closing "
        self.advance();

        // Trim the surrounding quotes
        let lexeme: String = {
            let lexeme_slice: &[char] = &(self.source)[self.start + 1..self.current - 1];
            lexeme_slice.iter().collect()
        };
        self.add_token(StringLiteral(lexeme));
        Ok(())
    }

    fn add_number_literal(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();

            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme: String = {
            let lexeme_slice: &[char] = &(self.source)[self.start..self.current];
            lexeme_slice.iter().collect()
        };
        self.add_token(NumberLiteral(lexeme));
    }

    fn add_identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let lexeme: String = {
            let lexeme_slice: &[char] = &(self.source)[self.start..self.current];
            lexeme_slice.iter().collect()
        };

        let kind = match &lexeme as &str {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fun" => Fun,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            _ => Identifier(lexeme),
        };
        self.add_token(kind);
    }

    fn is_eof(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.is_eof() {
            return false;
        }
        if self.source[self.current] != c {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&mut self) -> char {
        if self.is_eof() {
            return '\0';
        };
        self.source[self.current]
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(kind, self.line));
    }

    fn error(&self, msg: String) -> Result<(), Error> {
        Err(Error {
            line: self.line,
            msg,
        })
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit() || c == '$'
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        Self::is_digit(c) || Self::is_alpha(c)
    }
}
