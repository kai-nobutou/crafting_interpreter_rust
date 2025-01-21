/// プロジェクト全体で使用する共通エラー型。
///
/// この列挙型は、Loxインタプリタ全体で発生する可能性のあるさまざまなエラーを表します。
/// 各エラーには、その原因や関連情報が含まれています。
#[derive(Debug, PartialEq)]
pub enum LoxError {
    /// 指定されたファイルが見つからない場合のエラー。
    ///
    /// # 引数
    /// - `String`: 見つからなかったファイル名。
    FileNotFound(String),

    /// 型変換が無効な場合のエラー。
    ///
    /// # 引数
    /// - `String`: エラーの詳細メッセージ。
    InvalidTypeConversion(String),

    /// 入出力エラー。
    ///
    /// # 引数
    /// - `String`: エラーの詳細メッセージ。
    IoError(String),

    /// パースエラー。
    ///
    /// # 引数
    /// - `String`: エラーの詳細メッセージ。
    ParseError(String),

    /// 未終了の文字列リテラルのエラー。
    ///
    /// # 引数
    /// - `String`: エラーの詳細メッセージ。
    UnterminatedString(String),

    /// ソースコードに予期しない文字が含まれている場合のエラー。
    ///
    /// # 引数
    /// - `char`: 予期しない文字。
    UnexpectedCharacter(char),

    /// 未定義の変数を参照した場合のエラー。
    ///
    /// # 引数
    /// - `String`: 未定義の変数名。
    UndefinedVariable(String),

    /// 0での除算を試みた場合のエラー。
    DivisionByZero,

    /// 条件式がブール値でない場合のエラー。
    ///
    /// # 引数
    /// - `String`: 条件式の文字列表現。
    NonBooleanCondition(String),

    /// 関数外で `return` 文を使用した場合のエラー。
    ReturnOutsideFunction,

    /// 関数のパラメータ名が重複している場合のエラー。
    ///
    /// # 引数
    /// - `String`: 重複しているパラメータ名。
    DuplicateParameterName(String),

    /// 実行時に発生するエラー。
    ///
    /// # 引数
    /// - `String`: エラーの詳細メッセージ。
    RuntimeError(String),
}

impl std::fmt::Display for LoxError {
    /// エラーを人間が読みやすい形式でフォーマットします。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxError::FileNotFound(file) => write!(f, "[Error: File not found '{}']", file),
            LoxError::InvalidTypeConversion(msg) => {
                write!(f, "[Error: Invalid type conversion '{}']", msg)
            }
            LoxError::IoError(msg) => write!(f, "[Error: IO error '{}']", msg),
            LoxError::ParseError(msg) => write!(f, "[Error: Parse error '{}']", msg),
            LoxError::UnterminatedString(msg) => {
                write!(f, "[Error: Unterminated string '{}']", msg)
            }
            LoxError::UnexpectedCharacter(c) => write!(f, "[Error: Unexpected character '{}']", c),
            LoxError::UndefinedVariable(name) => {
                write!(f, "[Error: Undefined variable '{}']", name)
            }
            LoxError::DivisionByZero => write!(f, "[Error: Division by zero]"),
            LoxError::NonBooleanCondition(cond) => {
                write!(f, "[Error: Non-boolean condition '{}']", cond)
            }
            LoxError::ReturnOutsideFunction => {
                write!(f, "[Error: Cannot return from outside a function.]")
            }
            LoxError::DuplicateParameterName(param) => {
                write!(f, "[Error: Duplicate parameter name '{}']", param)
            }
            LoxError::RuntimeError(msg) => write!(f, "[Error: Runtime error '{}']", msg),
        }
    }
}

impl std::error::Error for LoxError {}
