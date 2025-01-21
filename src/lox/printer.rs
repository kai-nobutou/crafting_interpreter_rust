use crate::lox::ast::Expr;
use crate::lox::ast::Stmt;
use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue;

/// `Visitor` トレイトは、抽象構文木（AST）の各ノードを訪問するためのインターフェースを定義します。
///
/// # 型パラメータ
/// - `R`: 各訪問メソッドが返す結果の型。
pub trait Visitor<R> {
    // Expr 用

    /// バイナリ式（例: 加算や減算）を訪問します。
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;

    /// リテラル値（例: 数値や文字列）を訪問します。
    fn visit_literal(&mut self, value: &LiteralValue) -> R;

    /// グループ化された式（括弧内の式）を訪問します。
    fn visit_grouping(&mut self, expression: &Expr) -> R;

    /// 変数参照を訪問します。
    fn visit_variable(&mut self, name: &Token) -> R;

    /// 単項演算子（例: `-` や `!`）を訪問します。
    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> R;

    /// 変数の代入を訪問します。
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> R;

    /// 関数呼び出しを訪問します。
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> R;

    /// オブジェクトのプロパティ取得を訪問します。
    fn visit_get(&mut self, object: &Expr, name: &Token) -> R;

    /// オブジェクトのプロパティ設定を訪問します。
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> R;

    /// `this` キーワードを訪問します。
    fn visit_this(&mut self, keyword: &Token) -> R;

    /// `super` キーワードを訪問します。
    fn visit_super(&mut self, keyword: &Token, method: &Token) -> R;

    // Stmt 用

    /// 式文（式だけの文）を訪問します。
    fn visit_expression(&mut self, expr: &Expr) -> R;

    /// `print` 文を訪問します。
    fn visit_print(&mut self, expr: &Expr) -> R;

    /// 変数宣言文を訪問します。
    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) -> R;

    /// ブロック文を訪問します。
    fn visit_block(&mut self, statements: &[Stmt]) -> R;

    /// `while` 文を訪問します。
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> R;

    /// `for` 文を訪問します。
    fn visit_for(
        &mut self,
        initializer: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        increment: &Option<Expr>,
        body: &Stmt,
    ) -> R;

    /// `if` 文を訪問します。
    fn visit_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Stmt>) -> R;

    /// 関数宣言を訪問します。
    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]) -> R;

    /// `return` 文を訪問します。
    fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>) -> R;

    /// クラス宣言を訪問します。
    fn visit_class(&mut self, name: &Token, methods: &[(Token, Stmt)]) -> R;
}

/// 抽象構文木（AST）のノードを文字列形式で表現するプリンタ。
pub struct AstPrinter;

impl AstPrinter {
    /// 式を受け取り、その内容を文字列として返します。
    ///
    /// # 引数
    /// - `expr`: 評価する式。
    ///
    /// # 戻り値
    /// 式を表す文字列。
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor<String> for AstPrinter {
    /// バイナリ式（例: 加算や減算）。
    ///
    /// # 引数
    /// - `left`: 左辺の式。
    /// - `operator`: 使用される演算子。
    /// - `right`: 右辺の式。
    ///
    /// # 戻り値
    /// バイナリ式を文字列で表現した結果。
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        format!(
            "({} {} {})",
            operator.lexeme,
            left.accept(self),
            right.accept(self)
        )
    }

    /// リテラル値（例: 数値や文字列）。
    ///
    /// # 引数
    /// - `value`: リテラル値。
    ///
    /// # 戻り値
    /// リテラル値を文字列で表現した結果。
    fn visit_literal(&mut self, value: &LiteralValue) -> String {
        value.to_string()
    }

    /// グループ化された式（括弧内の式）。
    ///
    /// # 引数
    /// - `expression`: グループ化された式。
    ///
    /// # 戻り値
    /// グループ化された式を文字列で表現した結果。
    fn visit_grouping(&mut self, expression: &Expr) -> String {
        format!("(group {})", expression.accept(self))
    }

    /// 変数参照。
    ///
    /// # 引数
    /// - `name`: 変数名。
    ///
    /// # 戻り値
    /// 変数名を文字列で表現した結果。
    fn visit_variable(&mut self, name: &Token) -> String {
        format!("{}", name.lexeme)
    }

    /// 単項演算子（例: `-` や `!`）。
    ///
    /// # 引数
    /// - `operator`: 単項演算子。
    /// - `operand`: 演算対象の式。
    ///
    /// # 戻り値
    /// 単項式を文字列で表現した結果。
    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> String {
        format!("({} {})", operator.lexeme, operand.accept(self))
    }

    /// 変数の代入。
    ///
    /// # 引数
    /// - `name`: 変数名。
    /// - `value`: 代入する値。
    ///
    /// # 戻り値
    /// 代入式を文字列で表現した結果。
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> String {
        format!("(assign {} {})", name.lexeme, value.accept(self))
    }

    /// 関数呼び出し。
    ///
    /// # 引数
    /// - `callee`: 呼び出す関数。
    /// - `arguments`: 引数のリスト。
    ///
    /// # 戻り値
    /// 関数呼び出しを文字列で表現した結果。
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> String {
        let args = arguments
            .iter()
            .map(|arg| arg.accept(self))
            .collect::<Vec<_>>()
            .join(", ");
        format!("(call {} {})", callee.accept(self), args)
    }

    /// オブジェクトのプロパティ取得。
    ///
    /// # 引数
    /// - `object`: 対象のオブジェクト。
    /// - `name`: プロパティ名。
    ///
    /// # 戻り値
    /// プロパティ取得を文字列で表現した結果。
    fn visit_get(&mut self, object: &Expr, name: &Token) -> String {
        format!("(get {}.{})", object.accept(self), name.lexeme)
    }

    /// オブジェクトのプロパティ設定。
    ///
    /// # 引数
    /// - `object`: 対象のオブジェクト。
    /// - `name`: プロパティ名。
    /// - `value`: 設定する値。
    ///
    /// # 戻り値
    /// プロパティ設定を文字列で表現した結果。
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> String {
        format!(
            "(set {}.{} = {})",
            object.accept(self),
            name.lexeme,
            value.accept(self)
        )
    }

    /// `this` キーワード。
    ///
    /// # 引数
    /// - `keyword`: `this` キーワードのトークン。
    ///
    /// # 戻り値
    /// `this` を文字列で表現した結果。
    fn visit_this(&mut self, keyword: &Token) -> String {
        format!("(this {})", keyword.lexeme)
    }

    /// `super` キーワード。
    ///
    /// # 引数
    /// - `keyword`: `super` キーワードのトークン。
    /// - `method`: メソッド名。
    ///
    /// # 戻り値
    /// `super` を文字列で表現した結果。
    fn visit_super(&mut self, keyword: &Token, method: &Token) -> String {
        format!("(super {}.{})", keyword.lexeme, method.lexeme)
    }

    /// 式文。
    ///
    /// # 引数
    /// - `expr`: 評価する式。
    ///
    /// # 戻り値
    /// 式文を文字列で表現した結果。
    fn visit_expression(&mut self, expr: &Expr) -> String {
        self.print(expr)
    }

    /// `print` 文。
    ///
    /// # 引数
    /// - `expr`: 印刷する式。
    ///
    /// # 戻り値
    /// `print` 文を文字列で表現した結果。
    fn visit_print(&mut self, expr: &Expr) -> String {
        format!("(print {})", self.print(expr))
    }

    /// 変数宣言文。
    ///
    /// # 引数
    /// - `name`: 変数名。
    /// - `initializer`: 初期化式（オプション）。
    ///
    /// # 戻り値
    /// 変数宣言文を文字列で表現した結果。
    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) -> String {
        if let Some(init) = initializer {
            format!("(var {} = {})", name.lexeme, self.print(init))
        } else {
            format!("(var {})", name.lexeme)
        }
    }

    /// ブロック文。
    ///
    /// # 引数
    /// - `statements`: ブロック内のステートメント。
    ///
    /// # 戻り値
    /// ブロック文を文字列で表現した結果。
    fn visit_block(&mut self, statements: &[Stmt]) -> String {
        let stmts = statements
            .iter()
            .map(|stmt| stmt.accept(self))
            .collect::<Vec<_>>()
            .join(" ");
        format!("(block {})", stmts)
    }

    /// `while` 文。
    ///
    /// # 引数
    /// - `condition`: 条件式。
    /// - `body`: 繰り返し実行するステートメント。
    ///
    /// # 戻り値
    /// `while` 文を文字列で表現した結果。
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> String {
        format!("(while {} {})", self.print(condition), body.accept(self))
    }

    /// `for` 文。
    ///
    /// # 引数
    /// - `initializer`: 初期化式（オプション）。
    /// - `condition`: 条件式（オプション）。
    /// - `increment`: 増分式（オプション）。
    /// - `body`: 繰り返し実行するステートメント。
    ///
    /// # 戻り値
    /// `for` 文を文字列で表現した結果。
    fn visit_for(
        &mut self,
        initializer: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        increment: &Option<Expr>,
        body: &Stmt,
    ) -> String {
        let init = initializer
            .as_ref()
            .map(|stmt| stmt.accept(self))
            .unwrap_or_default();
        let cond = condition
            .as_ref()
            .map(|expr| self.print(expr))
            .unwrap_or_else(|| "true".to_string());
        let inc = increment
            .as_ref()
            .map(|expr| self.print(expr))
            .unwrap_or_default();
        format!("(for {} {} {} {})", init, cond, inc, body.accept(self))
    }

    /// `if` 文。
    ///
    /// # 引数
    /// - `condition`: 条件式。
    /// - `then_branch`: 条件が真の場合に実行されるステートメント。
    /// - `else_branch`: 条件が偽の場合に実行されるステートメント（オプション）。
    ///
    /// # 戻り値
    /// `if` 文を文字列で表現した結果。
    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> String {
        let else_part = if let Some(else_stmt) = else_branch {
            format!(" else {}", else_stmt.accept(self))
        } else {
            "".to_string()
        };
        format!(
            "(if {} {}{})",
            self.print(condition),
            then_branch.accept(self),
            else_part
        )
    }

    /// 関数宣言。
    ///
    /// # 引数
    /// - `name`: 関数名。
    /// - `params`: パラメータリスト。
    /// - `body`: 関数の本体となるステートメント。
    ///
    /// # 戻り値
    /// 関数宣言を文字列で表現した結果。
    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]) -> String {
        let params_str = params
            .iter()
            .map(|param| param.lexeme.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let body_str = body
            .iter()
            .map(|stmt| stmt.accept(self))
            .collect::<Vec<_>>()
            .join(" ");
        format!("(fun {} ({}) {})", name.lexeme, params_str, body_str)
    }

    /// `return` 文。
    ///
    /// # 引数
    /// - `keyword`: `return` キーワード。
    /// - `value`: 返す値（オプション）。
    ///
    /// # 戻り値
    /// `return` 文を文字列で表現した結果。
    fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>) -> String {
        if let Some(val) = value {
            format!("(return {})", self.print(val))
        } else {
            "(return)".to_string()
        }
    }

    /// クラス宣言。
    ///
    /// # 引数
    /// - `name`: クラス名。
    /// - `methods`: クラスのメソッドリスト。
    ///
    /// # 戻り値
    /// クラス宣言を文字列で表現した結果。
    fn visit_class(&mut self, name: &Token, methods: &[(Token, Stmt)]) -> String {
        let methods_str = methods
            .iter()
            .map(|(method_name, method_stmt)| {
                format!("{} {}", method_name.lexeme, method_stmt.accept(self))
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!("(class {} {{ {} }})", name.lexeme, methods_str)
    }
}
