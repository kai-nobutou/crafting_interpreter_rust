use crate::lox::error::LoxError;
use crate::lox::token::Token;
use crate::lox::token_type::{LiteralValue, TokenType};

/// 字句解析器（Scanner）
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    /// 新しい `Scanner` の生成
    ///
    /// # 引数
    /// - `source`: ソースコードの文字列
    ///
    /// # 戻り値
    /// 新しい `Scanner` インスタンス
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    /// トークンのスキャン処理
    ///
    /// ソースコード全体を解析してトークンのリストを生成する。
    ///
    /// # 戻り値
    /// 成功時はトークンのリスト、失敗時は `LoxError`
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?; // 各トークンをスキャン
        }

        // 終端トークンを追加
        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), None, self.line));

        Ok(self.tokens.clone())
    }

    /// 入力の終端判定
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// トークンのスキャン
    fn scan_token(&mut self) -> Result<(), LoxError> {
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
            '%' => self.add_token(TokenType::Percent),
            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_char('/') {
                    // 行コメントのスキップ
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    // ブロックコメントのスキップ
                    self.skip_block_comment()?;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '"' => self.string()?,
            ' ' | '\r' | '\t' => {} // 空白のスキップ
            '\n' => self.line += 1, // 行番号のインクリメント
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphanumeric() || c == '_' {
                    self.identifier();
                } else {
                    return Err(LoxError::UnexpectedCharacter(c));
                }
            }
        }
        Ok(())
    }

    /// 現在位置の文字を取得して次に進む
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += 1;
        c
    }

    /// 特定の文字との一致確認
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap_or('\0') != expected {
            return false;
        }
        self.current += 1;
        true
    }

    /// トークンの追加
    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, None, self.line));
    }

    /// リテラルを持つトークンの追加
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: LiteralValue) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, Some(literal), self.line));
    }

    /// 文字列リテラルの解析
    fn string(&mut self) -> Result<(), LoxError> {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxError::UnterminatedString(format!(
                "Unterminated string literal at line {}.",
                self.line
            )));
        }

        self.advance();
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenType::StringLit, LiteralValue::String(value));
        Ok(())
    }

    /// 数字の解析
    fn number(&mut self) {
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current]
            .parse()
            .expect("Failed to parse number.");
        self.add_token_with_literal(TokenType::Number, LiteralValue::Number(value));
    }

    /// 識別子の解析
    fn identifier(&mut self) {
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        let token_type = match text.as_str() {
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
        };
        self.add_token(token_type);
    }

    /// 次の文字の取得
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }

    /// 次の次の文字の取得
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    /// ブロックコメントのスキップ
    fn skip_block_comment(&mut self) -> Result<(), LoxError> {
        let mut depth = 1;

        while depth > 0 && !self.is_at_end() {
            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                depth -= 1;
            } else if self.peek() == '\n' {
                self.line += 1;
                self.advance();
            } else {
                self.advance();
            }
        }

        if depth > 0 {
            return Err(LoxError::UnterminatedString(format!(
                "Unterminated string literal at line {}.",
                self.line
            )));
        }
        Ok(())
    }
}
