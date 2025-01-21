use crate::lox::token_type::{LiteralValue, TokenType};

/// `Token` は、Lox 言語のトークンを表す構造体です。
///
/// 各トークンは、トークンの種類、元の文字列、オプションのリテラル値、および行番号を保持します。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// トークンの種類を示します（例: Identifier, Number, Keyword など）。
    pub token_type: TokenType,
    /// トークンの元の文字列表現です。
    pub lexeme: String,
    /// トークンに関連付けられたリテラル値（例: 数値や文字列）を保持します。
    /// 値が存在しない場合は `None` になります。
    pub literal: Option<LiteralValue>,
    /// トークンが現れたソースコードの行番号です。
    pub line: usize,
}

impl Token {
    /// 新しいトークンを作成するコンストラクタ関数です。
    ///
    /// # 引数
    /// - `token_type`: トークンの種類を指定します。
    /// - `lexeme`: トークンの元の文字列を指定します。
    /// - `literal`: トークンに関連付けられたリテラル値を指定します（存在しない場合は `None` を指定）。
    /// - `line`: トークンが現れたソースコードの行番号を指定します。
    ///
    /// # 戻り値
    /// 作成された新しい `Token` インスタンスを返します。
    ///
    /// # 使用例
    /// ```
    /// use crate::lox::token_type::{TokenType, LiteralValue};
    /// use crate::lox::token::Token;
    ///
    /// let token = Token::new(
    ///     TokenType::Identifier,
    ///     "example".to_string(),
    ///     None,
    ///     1
    /// );
    /// println!("{:?}", token);
    /// ```
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<LiteralValue>,
        line: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
