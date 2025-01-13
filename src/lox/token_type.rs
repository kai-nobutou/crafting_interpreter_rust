use crate::lox::token::Token;
use crate::lox::ast::Stmt;
use crate::lox::evaluator::Environment;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Percent,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    StringLit,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Function {
        name: String,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return(Box<LiteralValue>),
}

use std::fmt;

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "{}", s),
            LiteralValue::Boolean(b) => write!(f, "{}", b),
            LiteralValue::Nil => write!(f, "nil"),
            LiteralValue::Function { name, .. } => {
                write!(f, "<fn {}>", name)
            }
            LiteralValue::Return(r) => write!(f, "{}", r),
        }
    }
}

impl LiteralValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            LiteralValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}