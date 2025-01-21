use crate::lox::ast::{Expr, Stmt};
use crate::lox::error::LoxError;
use crate::lox::token::Token;
use crate::lox::token_type::{LiteralValue, TokenType};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    /// 新しい環境を作成
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    /// 指定された環境を囲む新しい環境を作成
    pub fn with_enclosing(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    /// 環境に新しい変数を定義
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// 子スコープの値を親スコープに統合する（現状では統合せずログ出力のみ）
    pub fn merge_to_parent(&mut self) {
        if let Some(parent) = &mut self.enclosing {
            // 子スコープの値をマージせずにログ出力
            for (key, value) in self.values.iter() {}
        }
    }

    /// 変数の値を取得（現在のスコープまたは親スコープを検索）
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    /// 変数の値を更新（存在しない場合はエラーを返す）
    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
        } else if let Some(enclosing) = &mut self.enclosing {
            self.values.insert(name.clone(), value.clone());
            enclosing.assign(name, value)?;
        } else {
            return Err(format!("Variable '{}' not defined.", name));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Return(Box<Value>),
    Function {
        name: String,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Class {
        name: String,
        methods: HashMap<String, Value>,
    },
    Instance {
        class: Box<Value>,
        fields: HashMap<String, Value>,
    },
    NativeFunction(fn(Vec<Value>) -> Value),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
            _ => write!(f, "Unsupported value"),
        }
    }
}
#[derive(Debug)]
pub enum EvalResult {
    Return(Value),
    Error(LoxError),
}

pub struct Evaluator {
    environment: Environment,
    output: Vec<String>,
}

impl Evaluator {
    /// `Evaluator` の新しいインスタンスを作成します。
    ///
    /// この初期化では、新しい環境を設定し、標準のネイティブ関数を登録します。
    ///
    /// # ネイティブ関数
    /// - `clock`: 現在のUNIXエポック時間を秒単位で返します。
    ///
    /// # 戻り値
    /// 新しい `Evaluator` インスタンス。
    pub fn new() -> Self {
        let mut environment = Environment::new();

        // ネイティブ関数の登録
        environment.define(
            "clock".to_string(),
            Value::NativeFunction(|_args| {
                let time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Value::Number(time as f64)
            }),
        );

        Self {
            environment,
            output: Vec::new(),
        }
    }

    /// ステートメントのリストを評価します。
    ///
    /// # 引数
    /// - `statements`: 評価するステートメントのリスト
    ///
    /// # 戻り値
    /// - 成功時: 最後に評価された値を含む `EvalResult::Return`
    /// - 失敗時: エラー `LoxError` を含む `EvalResult::Error`
    pub fn evaluate_statements(&mut self, statements: Vec<Stmt>) -> EvalResult {
        let mut last_value = Value::Nil; // 最後の評価結果を保持
        for stmt in statements {
            match self.execute(stmt) {
                EvalResult::Return(value) => last_value = value, // 処理を継続
                EvalResult::Error(err) => return EvalResult::Error(err), // エラー時は即終了
            }
        }
        EvalResult::Return(last_value) // 最後の値を返す
    }

    /// ステートメントを評価します。
    ///
    /// # 引数
    /// - `stmt`: 評価するステートメント。
    ///
    /// # 戻り値
    /// - `EvalResult`: 評価結果（`Return` または `Error`）。
    fn execute(&mut self, stmt: Stmt) -> EvalResult {
        match stmt {
            Stmt::Expression(expr) => match self.evaluate(&expr) {
                Ok(value) => EvalResult::Return(Value::Nil),
                Err(err) => {
                    let context = format!("Error occurred during expression evaluation: {:?}", err);
                    EvalResult::Error(LoxError::InvalidTypeConversion(context))
                }
            },
            Stmt::Print(expr) => match self.evaluate(&expr) {
                Ok(value) => {
                    self.output.push(value.to_string());
                    EvalResult::Return(Value::Nil)
                }
                Err(err) => {
                    self.output.push(format!("[Error: {}]", err));
                    EvalResult::Error(err)
                }
            },
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    match self.evaluate(&init) {
                        Ok(val) => val,
                        Err(err) => {
                            return EvalResult::Error(err);
                        }
                    }
                } else {
                    Value::Nil
                };

                self.environment.define(name.lexeme.clone(), value);
                EvalResult::Return(Value::Nil)
            }
            Stmt::Block(statements) => {
                let enclosing = self.environment.clone();
                let new_env = Environment::with_enclosing(enclosing);
                let mut previous_env = std::mem::replace(&mut self.environment, new_env);

                let mut last_result = EvalResult::Return(Value::Nil);

                for stmt in statements {
                    let result = self.execute(stmt);

                    match result {
                        EvalResult::Error(err) => {
                            self.environment = previous_env;
                            return EvalResult::Error(err);
                        }
                        EvalResult::Return(_) => last_result = result,
                    }
                }

                self.environment.merge_to_parent();

                if let Some(enclosing) = &self.environment.enclosing {
                    previous_env.values.extend(enclosing.values.clone());
                }

                self.environment = previous_env;

                last_result
            }
            Stmt::While(condition, body) => {
                loop {
                    match self.evaluate(&condition) {
                        Ok(Value::Boolean(true)) => match self.execute(*body.clone()) {
                            EvalResult::Error(err) => {
                                return EvalResult::Error(err);
                            }
                            EvalResult::Return(_) => continue,
                            _ => println!("Body executed successfully, continuing loop."),
                        },
                        Ok(Value::Boolean(false)) => {
                            break;
                        }
                        Err(err) => {
                            return EvalResult::Error(err);
                        }
                        _ => {
                            return EvalResult::Error(LoxError::NonBooleanCondition(
                                "Condition must evaluate to a boolean.".to_string(),
                            ));
                        }
                    }
                }
                EvalResult::Return(Value::Nil)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => match self.evaluate(&condition) {
                Ok(Value::Boolean(true)) => self.execute(*then_branch),

                Ok(Value::Boolean(false)) => else_branch
                    .map_or(EvalResult::Return(Value::Nil), |branch| {
                        self.execute(*branch)
                    }),

                Err(err) => EvalResult::Error(err),
                Err(err) => EvalResult::Error(err),

                _ => EvalResult::Error(LoxError::NonBooleanCondition(
                    "Condition must evaluate to a boolean.".to_string(),
                )),
            },
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    if let EvalResult::Error(err) = self.execute(*initializer) {
                        return EvalResult::Error(err);
                    }
                }

                let condition_expr = condition.unwrap_or_else(|| Expr::Literal {
                    value: LiteralValue::Boolean(true),
                });

                while let Ok(Value::Boolean(true)) = self.evaluate(&condition_expr) {
                    match self.execute(*body.clone()) {
                        EvalResult::Error(err) => return EvalResult::Error(err),
                        _ => {}
                    }

                    if let Some(increment) = &increment {
                        if let Err(err) = self.evaluate(increment) {
                            return EvalResult::Error(err);
                        }
                    }
                }

                EvalResult::Return(Value::Nil)
            }
            Stmt::Call { callee, arguments } => {
                let function = match self.evaluate(&callee) {
                    Ok(value) => value,
                    Err(err) => return EvalResult::Error(err),
                };

                let argument_values: Result<Vec<_>, _> =
                    arguments.iter().map(|arg| self.evaluate(arg)).collect();

                match argument_values {
                    Ok(values) => match self.evaluate_call(function, values) {
                        Ok(value) => EvalResult::Return(value),
                        Err(err) => EvalResult::Error(err),
                    },
                    Err(err) => EvalResult::Error(err),
                }
            }
            Stmt::Function { name, params, body } => {
                let function = Value::Function {
                    name: name.lexeme.clone(),
                    params,
                    body,
                };
                self.environment.define(name.lexeme.clone(), function);
                EvalResult::Return(Value::Nil)
            }
            Stmt::Return { value, .. } => {
                let return_value = match value {
                    Some(expr) => match self.evaluate(&expr) {
                        Ok(val) => val,
                        Err(err) => return EvalResult::Error(err),
                    },
                    None => Value::Nil,
                };
                EvalResult::Return(Value::Return(Box::new(return_value)))
            }
            Stmt::Assign { name, value } => match self.evaluate(&value) {
                Ok(val) => {
                    if let Err(_) = self.environment.assign(name.lexeme.clone(), val.clone()) {
                        return EvalResult::Error(LoxError::UndefinedVariable(name.lexeme.clone()));
                    }
                    EvalResult::Return(Value::Nil)
                }
                Err(err) => EvalResult::Error(err),
            },
            _ => EvalResult::Error(LoxError::InvalidTypeConversion(
                "Unsupported statement.".to_string(),
            )),
        }
    }

    /// 式を評価します。
    ///
    /// # 引数
    /// - `expr`: 評価対象の式。
    ///
    /// # 戻り値
    /// - 成功時: 評価結果 `Value` を含む `Ok`。
    /// - 失敗時: エラー `LoxError` を含む `Err`。
    fn evaluate(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        match expr {
            Expr::Literal { value } => self.literal_to_value(value.clone()),

            Expr::Unary { operator, operand } => {
                let right = self.evaluate(operand)?;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operand must be a number for unary minus.".to_string(),
                        )),
                    },
                    TokenType::Bang => Ok(Value::Boolean(!self.is_truthy(right))),
                    _ => Err(LoxError::InvalidTypeConversion(
                        "Invalid unary operator.".to_string(),
                    )),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let right_value = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Plus => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operands must be two numbers or two strings for '+'.".to_string(),
                        )),
                    },
                    TokenType::Minus => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operands must be numbers for '-'.".to_string(),
                        )),
                    },
                    TokenType::Star => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operands must be numbers for '*'.".to_string(),
                        )),
                    },
                    TokenType::Slash => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => {
                            if r == 0.0 {
                                Err(LoxError::DivisionByZero)
                            } else {
                                Ok(Value::Number(l / r))
                            }
                        }
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operands must be numbers for '/'.".to_string(),
                        )),
                    },
                    TokenType::Less
                    | TokenType::LessEqual
                    | TokenType::Greater
                    | TokenType::GreaterEqual => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => match operator.token_type {
                            TokenType::Less => Ok(Value::Boolean(l < r)),
                            TokenType::LessEqual => Ok(Value::Boolean(l <= r)),
                            TokenType::Greater => Ok(Value::Boolean(l > r)),
                            TokenType::GreaterEqual => Ok(Value::Boolean(l >= r)),
                            _ => unreachable!(),
                        },
                        _ => Err(LoxError::InvalidTypeConversion(
                            "Operands must be numbers for comparison.".to_string(),
                        )),
                    },
                    _ => Err(LoxError::InvalidTypeConversion(
                        "Invalid binary operator.".to_string(),
                    )),
                }
            }

            Expr::Variable { name } => self
                .environment
                .get(&name.lexeme)
                .ok_or_else(|| LoxError::UndefinedVariable(name.lexeme.clone())),

            Expr::Grouping { expression } => self.evaluate(expression),

            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment
                    .assign(name.lexeme.clone(), val.clone())
                    .map_err(|_| LoxError::UndefinedVariable(name.lexeme.clone()))?;
                Ok(val)
            }

            Expr::Call { callee, arguments } => {
                let function = self.evaluate(callee)?;
                let argument_values: Result<Vec<_>, _> =
                    arguments.iter().map(|arg| self.evaluate(arg)).collect();
                self.evaluate_call(function, argument_values?)
            }

            _ => Err(LoxError::InvalidTypeConversion(
                "Unsupported expression.".to_string(),
            )),
        }
    }

    /// ブロックを実行します。
    ///
    /// # 引数
    /// - `statements`: 実行するステートメントのリスト。
    /// - `new_env`: ブロック専用の新しい環境。
    ///
    /// # 戻り値
    /// - 成功時: 最後に評価された値を含む `Ok`。
    /// - 失敗時: エラー `LoxError` を含む `Err`。
    fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        new_env: Environment,
    ) -> Result<Value, LoxError> {
        let previous_env = std::mem::replace(&mut self.environment, new_env);
        let mut last_result = Value::Nil;

        for stmt in statements {
            match self.execute(stmt) {
                EvalResult::Return(Value::Return(inner_value)) => {
                    self.environment = previous_env;
                    return Ok(*inner_value);
                }
                EvalResult::Return(value) => {
                    last_result = value;
                }
                EvalResult::Error(err) => {
                    self.environment = previous_env;
                    return Err(err);
                }
                _ => {}
            }
        }
        // ブロック終了後、元の環境を復元
        self.environment = previous_env;

        Ok(last_result)
    }

    /// `LiteralValue` を `Value` に変換します。
    ///
    /// # 引数
    /// - `literal`: 変換対象のリテラル値。
    ///
    /// # 戻り値
    /// - 成功時: 対応する `Value` を含む `Ok`。
    /// - 失敗時: 未知のリテラル型に対するエラー `LoxError` を含む `Err`。
    fn literal_to_value(&self, literal: LiteralValue) -> Result<Value, LoxError> {
        match literal {
            LiteralValue::Boolean(b) => Ok(Value::Boolean(b)),
            LiteralValue::Number(n) => Ok(Value::Number(n)),
            LiteralValue::String(s) => Ok(Value::String(s)),
            LiteralValue::Nil => Ok(Value::Nil),
            _ => {
                // 未対応の型の場合、エラーを返す
                let err_message = format!("Unsupported literal value: {:?}", literal);
                Err(LoxError::InvalidTypeConversion(err_message))
            }
        }
    }

    /// 値が「真」とみなされるかを判定します。
    ///
    /// # 引数
    /// - `value`: 判定対象の値。
    ///
    /// # 戻り値
    /// - `true`: 値が「真」とみなされる場合。
    /// - `false`: 値が「偽」とみなされる場合。
    ///
    /// # 備考
    /// - `Nil` および `Boolean(false)` は偽とみなされます。
    /// - その他の値はすべて真とみなされます。
    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Boolean(b) => b,
            Value::Nil => false,
            _ => true,
        }
    }

    /// 関数呼び出しを評価します。
    ///
    /// # 引数
    /// - `function`: 呼び出される関数の値。
    /// - `arguments`: 関数に渡される引数。
    ///
    /// # 戻り値
    /// - 成功時: 評価結果 `Value` を含む `Ok`。
    /// - 失敗時: エラー `LoxError` を含む `Err`。
    fn evaluate_call(&mut self, function: Value, arguments: Vec<Value>) -> Result<Value, LoxError> {
        // 関数として扱えるかを確認
        if let Value::Function { params, body, .. } = function {
            // 引数の数を検証
            if params.len() != arguments.len() {
                return Err(LoxError::InvalidTypeConversion(format!(
                    "Expected {} arguments but got {}.",
                    params.len(),
                    arguments.len()
                )));
            }
            // 新しい環境を作成し、引数をバインド
            let mut new_env = Environment::with_enclosing(self.environment.clone());
            for (param, arg) in params.iter().zip(arguments.iter()) {
                new_env.define(param.lexeme.clone(), arg.clone());
            }
            // 関数のブロックを実行
            match self.execute_block(body, new_env) {
                Ok(Value::Return(value)) => Ok(*value),
                Ok(value) => Ok(value),
                Err(err) => Err(err), // ここで既に LoxError を返しているのでそのまま渡す
            }
        } else {
            Err(LoxError::InvalidTypeConversion(
                "Can only call functions.".to_string(),
            ))
        }
    }

    /// 実行結果を取得します。
    ///
    /// # 戻り値
    /// 出力された文字列を結合した結果を返します。
    pub fn get_output(&self) -> String {
        self.output.join("\n")
    }
}
