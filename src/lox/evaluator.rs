use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::Token;
use crate::lox::token_type::{LiteralValue, TokenType};
use std::collections::HashMap;
use std::process::Output;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else if let Some(enclosing) = self.enclosing.as_mut() {
            enclosing.assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
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

pub enum EvalResult {
    Return(Value),
    Error(String),
}

pub struct Evaluator {
    environment: Environment,
    output: Vec<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut environment = Environment::new();

        // ネイティブ関数の登録
        environment.define("clock".to_string(), Value::NativeFunction(|_args| {
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Value::Number(time as f64)
        }));

        Self {
            environment,
            output: Vec::new(),
        }
    }

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

    pub fn get_output(&self) -> String {
        self.output.join("\n")
    }

    fn execute(&mut self, stmt: Stmt) -> EvalResult {
        match stmt {
            Stmt::Expression(expr) => {
                if let Ok(value) = self.evaluate(&expr) {
                    if value != Value::Nil {
                        self.output.push(value.to_string());
                    }
                }
                EvalResult::Return(Value::Nil)
            }
            Stmt::Print(expr) => {
                match self.evaluate(&expr) {
                    Ok(value) => {
                        self.output.push(value.to_string());
                        print!("{:?}", self.output);
                        EvalResult::Return(Value::Nil)
                    }
                    Err(err) => EvalResult::Error(err),
                }
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    match self.evaluate(&init) {
                        Ok(val) => val,
                        Err(err) => return EvalResult::Error(err),
                    }
                } else {
                    Value::Nil
                };
                self.environment.define(name.lexeme.clone(), value);
                EvalResult::Return(Value::Nil)
            }
            Stmt::Block(statements) => {
                let enclosing = self.environment.clone();
            let previous_env = std::mem::replace(
                &mut self.environment,
                Environment::with_enclosing(enclosing),
            );
                let mut result = EvalResult::Return(Value::Nil);
    
                for stmt in statements {
                    result = self.execute(stmt);
                    if matches!(result, EvalResult::Error(_) | EvalResult::Return(_)) {
                        break;
                    }
                }
    
                self.environment = previous_env;
                result
            }
            Stmt::While(condition, body) => {
                while let Ok(Value::Boolean(true)) = self.evaluate(&condition) {
                    let result = self.execute(*body.clone());
                    if matches!(result, EvalResult::Error(_) | EvalResult::Return(_)) {
                        return result;
                    }
                }
                EvalResult::Return(Value::Nil)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                match self.evaluate(&condition) {
                    Ok(Value::Boolean(true)) => self.execute(*then_branch),
                    Ok(Value::Boolean(false)) => {
                        if let Some(else_branch) = else_branch {
                            self.execute(*else_branch)
                        } else {
                            EvalResult::Return(Value::Nil)
                        }
                    }
                    _ => EvalResult::Error("Condition must evaluate to a boolean.".to_string()),
                }
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    match self.execute(*initializer) {
                        EvalResult::Error(err) => return EvalResult::Error(err),
                        _ => {}
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
                        match self.evaluate(increment) {
                            Ok(_) => {}
                            Err(err) => return EvalResult::Error(err),
                        }
                    }
                }
                EvalResult::Return(Value::Nil)
            }
            Stmt::Call {
                callee,
                arguments,
            } => {
                let function = match self.evaluate(&callee) {
                    Ok(value) => value,
                    Err(err) => return EvalResult::Error(err),
                };
    
                if let Value::Function { params, body, .. } = function {
                    if params.len() != arguments.len() {
                        return EvalResult::Error("Incorrect number of arguments.".to_string());
                    }
    
                    let mut new_env = Environment::with_enclosing(self.environment.clone());
                    for (param, arg) in params.iter().zip(arguments.iter()) {
                        let value = match self.evaluate(arg) {
                            Ok(val) => val,
                            Err(err) => return EvalResult::Error(err),
                        };
                        new_env.define(param.lexeme.clone(), value);
                    }
    
                    match self.execute_block(body, new_env) {
                        Ok(val) => EvalResult::Return(val),
                        Err(err) => EvalResult::Error(err),
                    }
                } else {
                    EvalResult::Error("Can only call functions.".to_string())
                }
            }
            _ => EvalResult::Error("Unsupported statement.".to_string()),
        }
    }

    fn execute_block(&mut self, statements: Vec<Stmt>, mut new_env: Environment) -> Result<Value, String> {
    let previous_env = std::mem::replace(&mut self.environment, new_env);

    let mut result = Ok(Value::Nil);
    for stmt in statements {
        match self.execute(stmt) {
            EvalResult::Return(value) => {
                result = Ok(value);
                break;
            }
            EvalResult::Error(err) => {
                result = Err(err);
                break;
            }
            _ => {}
        }
    }

    self.environment = previous_env;
    result
}

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal { value } => Ok(self.literal_to_value(value.clone())),
            Expr::Unary { operator, operand } => {
                let right = self.evaluate(operand)?;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err("Operand must be a number.".to_string()),
                    },
                    TokenType::Bang => Ok(Value::Boolean(!self.is_truthy(right))),
                    _ => Err("Invalid unary operator.".to_string()),
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
                        _ => Err("Operands must be two numbers or two strings.".to_string()),
                    },
                    TokenType::Minus => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    TokenType::Star => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    TokenType::Slash => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => {
                            if r == 0.0 {
                                Err("Division by zero.".to_string())
                            } else {
                                Ok(Value::Number(l / r))
                            }
                        }
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    _ => Err("Invalid binary operator.".to_string()),
                }
            }
            Expr::Variable { name } => self.environment.get(&name.lexeme).ok_or_else(|| {
                format!("Undefined variable '{}'.", name.lexeme)
            }),
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment.assign(name.lexeme.clone(), val.clone())?;
                Ok(val)
            }
            _ => Err("Unsupported expression.".to_string()),
        }
    }

    fn literal_to_value(&self, literal: LiteralValue) -> Value {
        match literal {
            LiteralValue::Boolean(b) => Value::Boolean(b),
            LiteralValue::Number(n) => Value::Number(n),
            LiteralValue::String(s) => Value::String(s),
            LiteralValue::Nil => Value::Nil,
            _ => Value::Nil, // 他のリテラルタイプを追加
        }
    }

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Boolean(b) => b,
            Value::Nil => false,
            _ => true,
        }
    }
}