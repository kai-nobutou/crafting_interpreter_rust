use crate::lox::ast::{Expr, Stmt};
use crate::lox::error::LoxError;
use crate::lox::token::Token;
use crate::lox::token_type::{LiteralValue, TokenType};

const MAX_RECURSION_DEPTH: usize = 1000;

/// トークンのリストを解析して、抽象構文木（AST）を生成するための構造体。
/// この構造体は、スキャナーによって生成されたトークンを処理し、
/// ステートメントや式を構築します。
///
/// # フィールド
/// - `tokens`: 解析対象となるトークンのリスト。
/// - `current`: 現在解析中のトークンのインデックス。
/// - `recursion_depth`: 再帰の深さを追跡し、スタックオーバーフローを防止する。
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    recursion_depth: usize,
    in_function: bool,
}

impl Parser {
    /// 新しい `Parser` インスタンスを作成します。
    ///
    /// # 引数
    /// - `tokens`: 解析対象のトークンのリスト。
    ///
    /// # 戻り値
    /// - 新しい `Parser` インスタンス。
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            recursion_depth: 0,
            in_function: false,
        }
    }

    /// トークンのリストを解析し、ステートメントのリストを生成します。
    ///
    /// # 戻り値
    /// - 成功時: ステートメントのリスト。
    /// - 失敗時: `LoxError`。
    pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => return Err(err),
            }
        }
        Ok(statements)
    }

    /// トークンを解析し、ステートメントを生成します。
    ///
    /// この関数は、現在のトークンを確認し、それに基づいて適切なステートメントを解析します。
    /// サポートするステートメントの種類は以下の通りです:
    /// - `for` 文
    /// - `while` 文
    /// - `if` 文
    /// - `return` 文
    /// - `print` 文
    /// - ブロック `{ ... }`
    /// - 単一の式
    ///
    /// # 戻り値
    /// - 成功時: ステートメント（`Stmt`）。
    /// - 失敗時: `LoxError`。
    fn statement(&mut self) -> Result<Stmt, LoxError> {
        match self
            .peek()
            .ok_or_else(|| LoxError::ParseError("Unexpected end of input.".to_string()))?
            .token_type
        {
            TokenType::For => {
                self.advance();
                self.for_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::Return => {
                self.advance();
                self.return_statement()
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::LeftBrace => {
                self.advance();
                self.block()
            }
            _ => {
                let expr = self.assignment()?;
                self.consume(
                    TokenType::Semicolon,
                    "Expected ';' after expression or assignment.",
                )?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    /// トークンを解析して、ステートメントを生成します。
    ///
    /// # 戻り値
    /// - 成功時: ステートメント。
    /// - 失敗時: `LoxError`。
    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.match_token(&[TokenType::Fun]) {
            self.function("function")
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    /// 変数宣言を解析し、対応するステートメントを生成します。
    ///
    /// 例: `var x = 10;` のようなコードを解析します。
    ///
    /// # 処理の流れ
    /// 1. 変数名を取得します。トークンが `Identifier` である必要があります。
    /// 2. `=` が存在する場合、初期化式を解析します。
    /// 3. `;` を確認して文の終わりを検証します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Var` 型の変数宣言ステートメント。
    /// - 失敗時: `LoxError`。
    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self
            .consume(TokenType::Identifier, "Expected variable name.")?
            .clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    /// 現在のトークンを返し、次のトークンに進みます。
    ///
    /// # 戻り値
    /// - 現在のトークンへの参照。
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// `for` 文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `for (initializer; condition; increment) { body }`
    ///
    /// # 処理の流れ
    /// 1. 初期化式（`initializer`）を解析します。`var` 宣言または式文が許容されます。
    /// 2. 条件式（`condition`）を解析します。省略された場合、常に `true` を条件とします。
    /// 3. 増分式（`increment`）を解析します。
    /// 4. ループの本体（`body`）を解析します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::For` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after expression or assignment.",
        )?;
        // 初期化式の解析
        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(Box::new(self.var_declaration()?))
        } else {
            Some(Box::new(self.expression_statement()?))
        };

        // 条件式の解析
        let condition = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal {
                value: LiteralValue::Boolean(true),
            }
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        // 増分式の解析
        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        // ループ本体の解析
        let mut body = self.statement()?;

        // 増分式をループ本体の末尾に追加
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        // 条件式と本体を `while` 文としてラップ
        let body = Stmt::While(condition, Box::new(body));

        // 初期化式を全体の前に追加
        if let Some(initializer) = initializer {
            Ok(Stmt::Block(vec![*initializer, body]))
        } else {
            Ok(body)
        }
    }

    /// `while` 文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `while (condition) { ... }`
    ///
    /// # 処理の流れ
    /// 1. `(` の存在を確認し、条件式の解析を開始します。
    /// 2. 条件式を解析し、`)` の存在を確認します。
    /// 3. ループの本体（ステートメント）を解析します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::While` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    /// `if` 文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `if (condition) { ... } else { ... }`
    ///
    /// # 処理の流れ
    /// 1. `(` の存在を確認し、条件式を解析します。
    /// 2. `)` の存在を確認します。
    /// 3. `then_branch`（`if` 条件が真の場合のステートメント）を解析します。
    /// 4. `else_branch`（`if` 条件が偽の場合のステートメント）をオプションで解析します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::If` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    /// `return` 文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `return 42;` または `return;`
    ///
    /// # 処理の流れ
    /// 1. 現在のトークンをキーワードとして取得します。
    /// 2. セミコロンがない場合、返す値（式）を解析します。
    /// 3. `;` の存在を確認し、ステートメントの終わりを検証します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Return` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn return_statement(&mut self) -> Result<Stmt, LoxError> {
        // 関数内かどうかをチェック
        if !self.in_function {
            return Err(LoxError::ReturnOutsideFunction);
        }

        let keyword = self.previous().clone();

        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    /// `print` 文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `print value;`
    ///
    /// # 処理の流れ
    /// 1. `print` の後の式を解析します。
    /// 2. 式の後に `;` が続くことを確認します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Print` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;

        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;

        Ok(Stmt::Print(value))
    }

    /// ブロック文 `{ ... }` を解析し、対応するステートメントを生成します。
    ///
    /// 例: `{ stmt1; stmt2; ... }`
    ///
    /// # 処理の流れ
    /// 1. `{` の後に続く複数のステートメントを解析します。
    /// 2. `}` が現れるまで繰り返します。
    /// 3. `}` の存在を確認してブロックの終わりを検証します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Block` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn block(&mut self) -> Result<Stmt, LoxError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;

        Ok(Stmt::Block(statements))
    }

    /// 代入式を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `a = b`
    ///
    /// # 処理の流れ
    /// 1. 等価性の解析を行います。
    /// 2. `=` が現れた場合、右辺の式を解析します。
    /// 3. 代入対象が変数でない場合、エラーを返します。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Assign` またはその代わりの式。
    /// - 失敗時: `LoxError`。
    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;
            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else {
                return Err(LoxError::ParseError(
                    "Invalid assignment target.".to_string(),
                ));
            }
        }

        Ok(expr)
    }

    /// 指定されたトークンタイプが現在の位置に一致する場合にトークンを消費します。
    ///
    /// # 引数
    /// - `token_type`: 確認したいトークンタイプ。
    /// - `message`: エラーメッセージ。
    ///
    /// # 戻り値
    /// - 成功時: 一致したトークンの参照を返します。
    /// - 失敗時: `None` を返します。
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, LoxError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(LoxError::ParseError(message.to_string()))
        }
    }

    /// 式文を解析し、対応するステートメントを生成します。
    ///
    /// 例: `expression;`
    ///
    /// # 処理の流れ
    /// 1. 式を解析します。
    /// 2. 式の後に `;` が続くことを確認します。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Expression` 型のステートメント。
    /// - 失敗時: `LoxError`。
    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let expr = self.expression()?;

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after expression or assignment.",
        )?;

        Ok(Stmt::Expression(expr))
    }

    /// 式を解析し、対応する `Expr` を生成します。
    ///
    /// この関数は再帰の深さを追跡し、スタックオーバーフローを防止します。
    ///
    /// # 戻り値
    /// - 成功時: `Expr` 型の式。
    /// - 失敗時: `LoxError`。
    fn expression(&mut self) -> Result<Expr, LoxError> {
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            return Err(LoxError::ParseError(
                "Recursion depth exceeded.".to_string(),
            ));
        }
        self.recursion_depth += 1;
        let result = self.assignment();
        self.recursion_depth -= 1;
        result
    }

    /// 等価性の式を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `a == b` または `a != b`
    ///
    /// # 処理の流れ
    /// 1. 比較式を解析します。
    /// 2. `==` または `!=` が続く限りループします。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Binary` またはその代わりの式。
    /// - 失敗時: `LoxError`。
    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// 比較式を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `a > b`, `a >= b`, `a < b`, `a <= b`
    ///
    /// # 処理の流れ
    /// 1. 項を解析します。
    /// 2. 比較演算子が続く限りループします。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Binary` またはその代わりの式。
    /// - 失敗時: `LoxError`。
    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// 項（加減算）を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `a + b` または `a - b`
    ///
    /// # 処理の流れ
    /// 1. 因子（乗除算）を解析します。
    /// 2. `+` または `-` が続く限りループします。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Binary` またはその代わりの式。
    /// - 失敗時: `LoxError`。
    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// 因子（乗除算）を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `a * b` または `a / b`
    ///
    /// # 処理の流れ
    /// 1. 単項式を解析します。
    /// 2. `*`, `/`, `%` が続く限りループします。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Binary` またはその代わりの式。
    /// - 失敗時: `LoxError`。
    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;
        while self.match_token(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// 単項式を解析し、対応する `Expr` を生成します。
    ///
    /// 例: `!a` または `-a`
    ///
    /// # 処理の流れ
    /// 1. `!` または `-` があれば再帰的に解析します。
    /// 2. それ以外の場合は基本式（`primary`）を解析します。
    ///
    /// # 戻り値
    /// - 成功時: `Expr::Unary` または基本式。
    /// - 失敗時: `LoxError`。
    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                operand: Box::new(right),
            });
        }
        self.primary()
    }

    /// 基本式を解析し、対応する `Expr` を生成します。
    ///
    /// 例: 数値リテラル、文字列リテラル、変数、グループ化された式など。
    ///
    /// # 処理の流れ
    /// - 数値リテラル: `1`, `3.14`
    /// - 文字列リテラル: `"hello"`
    /// - 識別子（変数）
    /// - グループ化: `(expr)`
    ///
    /// # 戻り値
    /// - 成功時: `Expr` 型の基本式。
    /// - 失敗時: `LoxError`。
    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.match_token(&[TokenType::Number]) {
            if let Some(literal) = &self.previous().literal {
                return Ok(Expr::Literal {
                    value: literal.clone(),
                });
            }
        }

        if self.match_token(&[TokenType::StringLit]) {
            if let Some(literal) = &self.previous().literal {
                if let LiteralValue::String(s) = literal {
                    return Ok(Expr::Literal {
                        value: LiteralValue::String(s.clone()),
                    });
                }
            }
        }

        if self.match_token(&[TokenType::Identifier]) {
            let variable = self.previous().clone();

            if self.match_token(&[TokenType::LeftParen]) {
                let mut arguments = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
                return Ok(Expr::Call {
                    callee: Box::new(Expr::Variable { name: variable }),
                    arguments,
                });
            }

            return Ok(Expr::Variable { name: variable });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Boolean(true),
            });
        }

        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Boolean(false),
            });
        }

        Err(LoxError::ParseError("Unexpected token.".to_string()))
    }

    /// 指定されたトークンタイプが現在の位置にある場合にトークンを消費します。
    ///
    /// # 引数
    /// - `types`: チェックするトークンタイプのリスト。
    ///
    /// # 戻り値
    /// - 成功時: `true`（トークンが一致して消費された場合）。
    /// - 失敗時: `false`。
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// 現在のトークンが指定されたトークンタイプに一致するかを確認します。
    ///
    /// # 引数
    /// - `token_type`: 確認したいトークンタイプ。
    ///
    /// # 戻り値
    /// - `true`: トークンタイプが一致する場合。
    /// - `false`: 一致しない場合。
    fn check(&self, token_type: TokenType) -> bool {
        if let Some(token) = self.peek() {
            token.token_type == token_type
        } else {
            false
        }
    }

    /// パーサーがすべてのトークンを解析し終えたかを確認します。
    ///
    /// # 戻り値
    /// - `true`: 現在のトークンが `EOF` の場合。
    /// - `false`: それ以外の場合。
    fn is_at_end(&self) -> bool {
        if let Some(token) = self.peek() {
            token.token_type == TokenType::Eof
        } else {
            true
        }
    }

    /// 現在のトークンを返します。
    ///
    /// # 戻り値
    /// - 成功時: 現在のトークンへの参照。
    /// - 失敗時: `None`（トークンが存在しない場合）。
    fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }

    /// 現在のトークンの1つ前のトークンを返します。
    ///
    /// # 戻り値
    /// - 現在のトークンの1つ前のトークンの参照。
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// 関数定義を解析し、対応するステートメントを生成します。
    ///
    /// 例:
    /// ```lox
    /// fun add(a, b = 10) {
    ///     return a + b;
    /// }
    /// ```
    ///
    /// # 引数
    /// - `kind`: 関数の種類を示す文字列（例: "function"）。
    ///
    /// # 戻り値
    /// - 成功時: `Stmt::Function` 型の関数定義ステートメント。
    /// - 失敗時: `LoxError`。
    fn function(&mut self, kind: &str) -> Result<Stmt, LoxError> {
        self.enter_function();

        // 関数名を取得
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {} name.", kind))?
            .clone();

        // パラメータリストの開始確認
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut params: Vec<(Token, Option<Expr>)> = Vec::new();

        // パラメータの解析
        if !self.check(TokenType::RightParen) {
            loop {
                // パラメータ名を取得
                let param_name = self
                    .consume(TokenType::Identifier, "Expect parameter name.")?
                    .clone();

                // パラメータ名の重複チェック
                for (existing_param, _) in &params {
                    if existing_param.lexeme == param_name.lexeme {
                        return Err(LoxError::ParseError(format!(
                            "Duplicate parameter name '{}'.",
                            param_name.lexeme
                        )));
                    }
                }

                // デフォルト値が指定されている場合
                let default_value = if self.match_token(&[TokenType::Equal]) {
                    Some(self.expression().map_err(|e| {
                        LoxError::ParseError(format!(
                            "Invalid default value for parameter '{}': {}",
                            param_name.lexeme, e
                        ))
                    })?)
                } else {
                    None
                };

                params.push((param_name, default_value));

                // カンマがない場合は終了
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        // パラメータリストの終了確認
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        // 関数本体の開始確認
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;

        // 関数本体のブロックを解析
        let body = match self.block()? {
            Stmt::Block(statements) => statements,
            _ => {
                return Err(LoxError::ParseError(
                    "Expected a block as function body.".to_string(),
                ))
            }
        };

        // 関数の解析終了後に exit_function を呼び出す
        self.exit_function();

        // ステートメントを生成
        Ok(Stmt::Function {
            name,
            params: params.into_iter().map(|(token, _)| token).collect(),
            body,
        })
    }

    /// 関数の開始時に `in_function` を `true` に設定し、関数の終了時に `false` に設定するメソッドです。
    /// これにより、`return_statement` が関数内でのみ動作するようにします。
    ///
    /// # 処理の流れ
    /// 1. 関数の開始時に呼び出され、`in_function` を `true` に設定します。
    /// 2. 関数の終了時に呼び出され、`in_function` を `false` に戻します。
    fn enter_function(&mut self) {
        self.in_function = true;
    }

    fn exit_function(&mut self) {
        self.in_function = false;
    }
}
