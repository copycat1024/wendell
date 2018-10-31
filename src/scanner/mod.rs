// scanner/mod.rs

pub mod token;

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

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".into(),
            line: self.line,
        });

        Ok(())
    }

    fn scan_token<'b>(&'b mut self) -> Result<(), Error> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                let kind = if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(kind);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind);
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_eof() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
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

    fn is_eof(&self) -> bool {
        return self.current >= self.source.len();
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
        return true;
    }

    fn peek(&mut self) -> char {
        if self.is_eof() {
            return '\0';
        };
        return self.source[self.current];
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source[self.current + 1];
    }

    fn add_token(&mut self, kind: TokenKind) {
        let lexeme_slice: &[char] = &(self.source)[self.start..self.current];
        let lexeme: String = lexeme_slice.iter().collect();
        self.tokens.push(Token {
            kind: kind,
            lexeme: lexeme,
            line: self.line,
        });
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
        let lexeme_slice: &[char] = &(self.source)[self.start + 1..self.current - 1];
        let lexeme: String = lexeme_slice.iter().collect();
        self.tokens.push(Token {
            kind: TokenKind::StringLiteral,
            lexeme: lexeme,
            line: self.line,
        });
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

        let lexeme_slice: &[char] = &(self.source)[self.start..self.current];
        let lexeme: String = lexeme_slice.iter().collect();
        self.tokens.push(Token {
            kind: TokenKind::NumberLiteral,
            lexeme: lexeme,
            line: self.line,
        });
    }

    fn add_identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let lexeme_slice: &[char] = &(self.source)[self.start..self.current];
        let lexeme: String = lexeme_slice.iter().collect();
        let kind = match &lexeme as &str {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };
        self.tokens.push(Token {
            kind: kind,
            lexeme: lexeme,
            line: self.line,
        });
    }

    fn error(&self, msg: String) -> Result<(), Error> {
        Err(Error {
            line: self.line,
            msg: msg,
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
