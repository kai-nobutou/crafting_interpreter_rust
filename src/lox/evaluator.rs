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
        println!("Creating new environment with enclosing: {:?}", enclosing.values);
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn merge_to_parent(&mut self) {
        if let Some(parent) = &mut self.enclosing {
            println!("Before merging: Child values: {:?}", self.values);
            println!("Parent values before merge: {:?}", parent.values);
            
            for (key, value) in self.values.iter() {
                println!("Merging key: {}, value: {:?}", key, value);
                parent.values.insert(key.clone(), value.clone());
            }
            
            println!("Parent values after merge: {:?}", parent.values);
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        println!("Defining variable: {} = {:?} in current scope", name, value);
        self.values.insert(name, value);
        println!("Current scope after define: {:?}", self.values);
    }
    
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            println!("Variable {} found in current scope: {:?}", name, value);
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            println!("Variable {} not found in current scope, checking enclosing scope.", name);
            enclosing.get(name)
        } else {
            println!("Variable {} not found in any scope.", name);
            None
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        println!(
            "Assigning variable: {}. Current scope: {:?}, Enclosing scope: {:?}",
            name,
            self.values,
            self.enclosing.as_ref().map(|env| &env.values)
        );
    
        // 現在のスコープで変数を探し、存在する場合は更新
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
        } else if let Some(enclosing) = &mut self.enclosing {
            // 親スコープに代入する前に現在のスコープに保存
            println!("Variable '{}' not found in current scope. Adding to current environment.", name);
            self.values.insert(name.clone(), value.clone());
            enclosing.assign(name, value)?;
        } else {
            println!("Variable '{}' not defined in any scope.", name);
            return Err(format!("Variable '{}' not defined.", name));
        }
    
        println!("Current scope after assign: {:?}", self.values);
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
                }
                EvalResult::Return(Value::Nil)
            }
            Stmt::Print(expr) => {
                match self.evaluate(&expr) {
                    Ok(value) => {
                        self.output.push(value.to_string());
                        println!("Print statement executed with value: {}", value);
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
                println!("Creating a new block. Current environment: {:?}", self.environment.values);
                let enclosing = self.environment.clone();
                let new_env = Environment::with_enclosing(enclosing);
            
                let mut previous_env = std::mem::replace(&mut self.environment, new_env);
            
                let mut last_result = EvalResult::Return(Value::Nil);
            
                for stmt in statements {
                    let result = self.execute(stmt);
            
                    if let EvalResult::Error(_) = result {
                        self.environment = previous_env;
                        return result;
                    }
            
                    if let EvalResult::Return(_) = result {
                        last_result = result;
                    }
                }
            
                // 子環境を親環境にマージ
                // 子環境を親環境にマージ
                println!(
                    "Before merging: Child environment: {:?}, Enclosing environment: {:?}",
                    self.environment.values,
                    self.environment.enclosing.as_ref().map(|env| &env.values),
                );
                self.environment.merge_to_parent();

                // 元の環境に戻す際に、マージされた状態を反映させる
                if let Some(enclosing) = &self.environment.enclosing {
                    previous_env.values.extend(enclosing.values.clone());
                }

                self.environment = previous_env;
                println!("Restored previous environment: {:?}", self.environment.values);
                
                last_result
            }
            Stmt::While(condition, body) => {
                println!("Entering while loop with condition: {:?}", condition);
                loop {
                    match self.evaluate(&condition) {
                        Ok(Value::Boolean(true)) => {
                            println!("Condition evaluated to true, entering loop body.");
                            
                            match self.execute(*body.clone()) {
                                EvalResult::Error(err) => {
                                    println!("Exiting loop due to error: {}", err);
                                    return EvalResult::Error(err);
                                }
                                EvalResult::Return(_) => {
                                    println!("Loop body executed with a return statement. Continuing loop.");
                                    // `Return` を受け取ってもループは継続
                                    continue;
                                }
                                _ => {
                                    println!("Body executed successfully, continuing loop.");
                                }
                            }
                        }
                        Ok(Value::Boolean(false)) => {
                            println!("Condition evaluated to false, exiting loop.");
                            break;
                        }
                        Err(err) => {
                            println!("Error evaluating while condition: {}", err);
                            return EvalResult::Error(err);
                        }
                        _ => {
                            return EvalResult::Error("Condition must evaluate to a boolean.".to_string());
                        }
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
            Stmt::Call { callee, arguments } => {
                let function = match self.evaluate(&callee) {
                    Ok(value) => value,
                    Err(err) => return EvalResult::Error(err),
                };
            
                let argument_values: Result<Vec<_>, _> = arguments.iter().map(|arg| self.evaluate(arg)).collect();
                let argument_values = match argument_values {
                    Ok(values) => values,
                    Err(err) => return EvalResult::Error(err),
                };
            
                match self.evaluate_call(function, argument_values) {
                    Ok(value) => EvalResult::Return(value),
                    Err(err) => EvalResult::Error(err),
                }
            },
            Stmt::Function { name, params, body } => {
                let function = Value::Function {
                    name: name.lexeme.clone(),
                    params,
                    body,
                };
                self.environment.define(name.lexeme.clone(), function);
                EvalResult::Return(Value::Nil)
            }
            Stmt::Return { keyword, value } => {
                // Return の値を評価
                let return_value = if let Some(expr) = value {
                    match self.evaluate(&expr) {
                        Ok(val) => {
                            println!("Evaluating return value: {:?}", val);
                            val
                        }
                        Err(err) => return EvalResult::Error(err),
                    }
                } else {
                    println!("Returning Nil");
                    Value::Nil
                };
            
                println!("Return statement executed with value: {:?}", return_value);
            
                // Return を特別な値として扱い、後続の処理をスキップ
                EvalResult::Return(Value::Return(Box::new(return_value)))
            }
            Stmt::Assign { name, value } => {
                println!("Executing assignment statement for: {}", name.lexeme);
            
                // 値を評価
                let val = match self.evaluate(&value) {
                    Ok(v) => v,
                    Err(err) => return EvalResult::Error(err),
                };
            
                // 現在の環境で代入を実行
                match self.environment.assign(name.lexeme.clone(), val.clone()) {
                    Ok(_) => {
                        println!("Successfully assigned value: {:?} to variable: {}", val, name.lexeme);
                        EvalResult::Return(Value::Nil)
                    }
                    Err(err) => {
                        println!("Assignment failed: {}", err);
                        EvalResult::Error(err)
                    }
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
                    _  => {}
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
                println!(
                    "Binary operation: {:?} {:?} {:?}",
                    left_value, operator.token_type, right_value
                );
                match operator.token_type {
                    TokenType::Plus => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => {
                            println!("Adding: {} + {}", l, r);
                            Ok(Value::Number(l + r))
                        }
                        (Value::String(l), Value::String(r)) => {
                            println!("Concatenating: {} + {}", l, r);
                            Ok(Value::String(l + &r))
                        }
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
                    TokenType::Less => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    TokenType::LessEqual => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    TokenType::Greater => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    TokenType::GreaterEqual => match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                        _ => Err("Operands must be numbers.".to_string()),
                    },
                    _ => Err("Invalid binary operator.".to_string()),
                }
            }
            Expr::Variable { name } => {
                println!("Evaluating variable: {}", name.lexeme);
                self.environment
                    .get(&name.lexeme)
                    .ok_or_else(|| format!("Undefined variable '{}'.", name.lexeme))
            }
            Expr::Grouping { expression } => {
                self.evaluate(expression)
            },
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                println!("Assigning value: {:?} to variable: {}", val, name.lexeme);
                match self.environment.assign(name.lexeme.clone(), val.clone()) {
                    Ok(_) => println!("Environment updated successfully for {}", name.lexeme),
                    Err(err) => println!("Failed to update environment for {}: {}", name.lexeme, err),
                }
                Ok(val)
            }
            Expr::Call { callee, arguments } => {
                let function = self.evaluate(&callee)?;

                // 引数を評価
                let argument_values: Result<Vec<_>, _> = arguments.iter().map(|arg| self.evaluate(arg)).collect();
                let argument_values = argument_values?;

                // 関数呼び出しを処理
                self.evaluate_call(function, argument_values)
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

    fn evaluate_call(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        println!("Evaluating call to function: {:?} with arguments: {:?}", function, arguments);
        
        if let Value::Function { params, body, .. } = function {
            // 引数の数が一致するかを確認
            if params.len() != arguments.len() {
                return Err(format!(
                    "Expected {} arguments but got {}.",
                    params.len(),
                    arguments.len()
                ));
            }
    
            // 新しい環境を作成
            let mut new_env = Environment::with_enclosing(self.environment.clone());
    
            // 引数を環境に追加
            for (param, arg) in params.iter().zip(arguments.iter()) {
                println!("Defining argument: {} = {:?}", param.lexeme, arg);
                new_env.define(param.lexeme.clone(), arg.clone());
            }
    
            // 関数本体を評価
            match self.execute_block(body, new_env) {
                Ok(Value::Return(value)) => {
                    println!("Returning value from function: {:?}", value);
                    Ok(*value)
                }
                Ok(value) => {
                    println!("Function executed with value: {:?}", value);
                    Ok(value)
                }
                Err(err) => Err(err),
            }
        } else {
            Err("Can only call functions.".to_string())
        }
    }
}