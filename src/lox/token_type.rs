use crate::lox::ast::Stmt;
use crate::lox::token::Token;
use std::fmt;

/// `TokenType` は、Lox 言語で使用されるすべてのトークン型を定義する列挙型です。
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenType {
    // Single-character tokens
    /// `(` トークン
    LeftParen,
    /// `)` トークン
    RightParen,
    /// `{` トークン
    LeftBrace,
    /// `}` トークン
    RightBrace,
    /// `,` トークン
    Comma,
    /// `.` トークン
    Dot,
    /// `-` トークン
    Minus,
    /// `+` トークン
    Plus,
    /// `;` トークン
    Semicolon,
    /// `/` トークン
    Slash,
    /// `*` トークン
    Star,
    /// `%` トークン
    Percent,

    // One or two character tokens
    /// `!` トークン
    Bang,
    /// `!=` トークン
    BangEqual,
    /// `=` トークン
    Equal,
    /// `==` トークン
    EqualEqual,
    /// `>` トークン
    Greater,
    /// `>=` トークン
    GreaterEqual,
    /// `<` トークン
    Less,
    /// `<=` トークン
    LessEqual,

    // Literals
    /// 識別子トークン
    Identifier,
    /// 文字列リテラルトークン
    StringLit,
    /// 数値リテラルトークン
    Number,

    // Keywords
    /// `and` キーワード
    And,
    /// `class` キーワード
    Class,
    /// `else` キーワード
    Else,
    /// `false` キーワード
    False,
    /// `fun` キーワード
    Fun,
    /// `for` キーワード
    For,
    /// `if` キーワード
    If,
    /// `nil` キーワード
    Nil,
    /// `or` キーワード
    Or,
    /// `print` キーワード
    Print,
    /// `return` キーワード
    Return,
    /// `super` キーワード
    Super,
    /// `this` キーワード
    This,
    /// `true` キーワード
    True,
    /// `var` キーワード
    Var,
    /// `while` キーワード
    While,

    /// ファイルの終端を示すトークン
    Eof,
}

/// `LiteralValue` は、Lox 言語で使用されるリテラル値を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    /// 文字列型のリテラル値
    String(String),
    /// 数値型のリテラル値
    Number(f64),
    /// ブール型のリテラル値
    Boolean(bool),
    /// `nil` を表すリテラル値
    Nil,
    /// ユーザー定義関数
    Function {
        /// 関数の名前
        name: String,
        /// 関数の引数リスト
        params: Vec<Token>,
        /// 関数の本体
        body: Vec<Stmt>,
    },
    /// `return` 文による返り値
    Return(Box<LiteralValue>),
}

/// `LiteralValue` を文字列形式に変換するための実装。
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
    /// 数値リテラルを取得します。
    ///
    /// # 戻り値
    /// 数値型のリテラル値の場合は `Some(f64)` を返します。
    /// それ以外の場合は `None` を返します。
    pub fn as_number(&self) -> Option<f64> {
        match self {
            LiteralValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}
