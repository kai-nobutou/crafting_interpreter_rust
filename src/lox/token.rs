use crate::lox::token_type::{TokenType, LiteralValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: usize,
}


impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<LiteralValue>, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}