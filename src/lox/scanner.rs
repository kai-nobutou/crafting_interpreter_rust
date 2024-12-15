use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue;
use crate::lox::token_type::TokenType;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), None, self.line));

        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            '!' | '=' | '<' | '>' => self.handle_two_char_token(c),
            '/' => self.handle_slash(),

            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,

            '"' => self.string(),

            _ => self.handle_default(c),
        }
    }

    fn handle_two_char_token(&mut self, c: char) {
        let token_type = match c {
            '!' => {
                if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '=' => {
                if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            _ => unreachable!(),
        };
        self.add_token(token_type);
    }

    fn handle_slash(&mut self) {
        if self.match_char('/') {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else {
            self.add_token(TokenType::Slash);
        }
    }

    fn handle_default(&mut self, c: char) {
        if c.is_ascii_digit() {
            self.number();
        } else if c.is_ascii_alphabetic() || c == '_' {
            self.identifier();
        } else {
            eprintln!("Unexpected character at line {}: {}", self.line, c);
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.substring(self.start, self.current);
        let token_type = self.keyword_lookup(&text);
        self.add_token(token_type);
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            eprintln!("Unterminated string at line {}", self.line);
            return;
        }

        self.advance(); // Consume the closing "
        let value = self.substring(self.start + 1, self.current - 1);
        self.add_token_with_literal(TokenType::StringLit, LiteralValue::String(value));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: f64 = self
            .substring(self.start, self.current)
            .parse()
            .expect("Failed to parse number literal");
        self.add_token_with_literal(TokenType::Number, LiteralValue::Number(value));
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.substring(self.start, self.current);
        self.tokens
            .push(Token::new(token_type, text, None, self.line));
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: LiteralValue) {
        let text = self.substring(self.start, self.current);
        self.tokens
            .push(Token::new(token_type, text, Some(literal), self.line));
    }

    fn substring(&self, start: usize, end: usize) -> String {
        self.source.chars().skip(start).take(end - start).collect()
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source[self.current..].chars().nth(1).unwrap_or('\0')
    }

    fn advance(&mut self) -> char {
        if let Some(c) = self.source[self.current..].chars().next() {
            self.current += c.len_utf8();
            c
        } else {
            '\0'
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn keyword_lookup(&self, text: &str) -> TokenType {
        match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }
}
