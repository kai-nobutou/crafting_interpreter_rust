use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::Token;
use crate::lox::token_type::{LiteralValue, TokenType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        println!("DEBUG: Creating new environment with enclosing scope");
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        println!("DEBUG: Defining variable '{}' with value {:?}", name, value);
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        if let Some(value) = self.values.get(name) {
            println!("DEBUG: Found variable '{}' in current scope with value {:?}", name, value);
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            println!("DEBUG: Looking for variable '{}' in enclosing scope", name);
            enclosing.get(name) // 無限再帰の可能性
        } else {
            println!("DEBUG: Variable '{}' not found", name);
            None
        }
    }

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<(), String> {
        if self.values.contains_key(&name) {
            println!("DEBUG: Updating variable '{}' in current scope to {:?}", name, value);
            self.values.insert(name, value);
            Ok(())
        } else if let Some(enclosing) = self.enclosing.as_mut() {
            println!("DEBUG: Assigning variable '{}' in enclosing scope to {:?}", name, value);
            enclosing.assign(name, value) // ここで外部スコープを再帰的に更新
        } else {
            println!("DEBUG: Variable '{}' not found in any scope.", name);
            Err(format!("Undefined variable '{}'.", name))
        }
    }
}

pub struct Evaluator {
    environment: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            environment: Environment::new(),
        }
    }

    pub fn evaluate_statements(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<LiteralValue, String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(&expr)?;
                Ok(LiteralValue::Nil)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr)?;
                println!("{:?}", value);
                Ok(LiteralValue::Nil)
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(&init)?
                } else {
                    LiteralValue::Nil
                };
                self.environment.define(name.lexeme.clone(), value);
                Ok(LiteralValue::Nil)
            }
            Stmt::Block(statements) => {
                println!("DEBUG: Entering block scope");

                // 現在のスコープを変更せず、ブロック内で実行
                for statement in statements {
                    self.execute(statement)?;
                }

                println!("DEBUG: Exiting block scope");
                Ok(LiteralValue::Nil)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if {
                    let condition_value = self.evaluate(&condition)?;
                    println!("DEBUG: If condition evaluated to {:?}", condition_value);
                    self.is_truthy(condition_value)
                } {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
                Ok(LiteralValue::Nil)
            }
            Stmt::While(condition, body) => {
                while {
                    let condition_value = self.evaluate(&condition)?;
                    println!("DEBUG: While condition evaluated to {:?}", condition_value);
                    self.is_truthy(condition_value)
                } {
                    self.execute(*body.clone())?;
                }
                Ok(LiteralValue::Nil)
            }
            Stmt::Return { value, .. } => {
                let result = if let Some(value) = value {
                    self.evaluate(&value)?
                } else {
                    LiteralValue::Nil
                };
                return Ok(LiteralValue::Return(Box::new(result))); // Return 値として包む
            }

            Stmt::Function { name, params, body } => {
                let func = LiteralValue::Function {
                    name: name.lexeme.clone(),
                    params,
                    body,
                };
                println!("DEBUG: Defining function '{}' in current scope", name.lexeme);
                self.environment.define(name.lexeme.clone(), func);
                Ok(LiteralValue::Nil)
            }

            Stmt::Call { callee, arguments } => {
                let callee_value = self.evaluate(&callee)?;
            
                if let LiteralValue::Function { name, params, body } = callee_value {
                    let args = arguments.iter().map(|arg| self.evaluate(arg)).collect::<Result<Vec<_>, _>>()?;
                    let result = self.call_function(&name, params, body, args)?; // 返り値を取得
                    Ok(result) // 関数の返り値を返す
                } else {
                    return Err("Attempted to call a non-function.".to_string());
                }
            }
            _ => Err("Unsupported statement.".to_string()),
        }
    }

    fn call_function(
        &mut self,
        name: &str,
        params: Vec<Token>,
        body: Vec<Stmt>,
        arguments: Vec<LiteralValue>,
    ) -> Result<LiteralValue, String> {
        println!("DEBUG: Calling function '{}' with args {:?}", name, arguments);
    
        // スコープの作成と引数の設定
        let mut new_env = Environment::with_enclosing(self.environment.clone());
        for (param, arg) in params.iter().zip(arguments.iter()) {
            new_env.define(param.lexeme.clone(), arg.clone());
        }
    
        // 関数自身の再定義（再帰呼び出しのため）
        if let Some(func) = self.environment.get(name) {
            new_env.define(name.to_string(), func);
        }
    
        // 関数の実行
        let result = self.evaluate_statements_in_new_scope(new_env, body)?;
    
        println!("DEBUG: Function '{}' returned with value {:?}", name, result);
    
        Ok(result)
    }
    
    fn evaluate_statements_in_new_scope(
        &mut self,
        new_env: Environment,
        body: Vec<Stmt>,
    ) -> Result<LiteralValue, String> {
        let mut evaluator = Evaluator {
            environment: new_env,
        };
    
        for stmt in body {
            match evaluator.execute(stmt) {
                Ok(value) => {
                    if let LiteralValue::Return(return_value) = value {
                        return Ok(*return_value); // Return 文の値を関数の結果として返す
                    }
                }
                Err(err) => return Err(err), // エラーが発生した場合は即座に返す
            }
        }
    
        Ok(LiteralValue::Nil) // Return 文がなかった場合は Nil を返す
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Unary { operator, operand } => self.visit_unary(operator, operand),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Variable { name } => self
                .environment
                .get(&name.lexeme)
                .ok_or_else(|| format!("Undefined variable '{}'.", name.lexeme)),
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment.assign(name.lexeme.clone(), val.clone())?;
                Ok(val)
            }
            Expr::Call { callee, arguments } => {
                let callee_value = self.evaluate(&callee)?;
            
                if let LiteralValue::Function { name, params, body } = callee_value {
                    // 引数の評価
                    let args = arguments
                        .iter()
                        .map(|arg| self.evaluate(arg))
                        .collect::<Result<Vec<_>, _>>()?;
            
                    // 関数を呼び出す
                    return self.call_function(&name, params, body, args);
                } else {
                    Err("Attempted to call a non-function.".to_string())
                }
            }
            _ => Err("Unsupported expression.".to_string()),
        }
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<LiteralValue, String> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        println!(
            "DEBUG: Binary operation - left: {:?}, operator: {:?}, right: {:?}",
            left_value, operator.token_type, right_value
        );

        match operator.token_type {
            TokenType::Plus => match (left_value.clone(), right_value.clone()) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    println!("DEBUG: Performing addition: {} + {}", l, r);
                    Ok(LiteralValue::Number(l + r))
                }
                (LiteralValue::String(l), LiteralValue::String(r)) => {
                    println!("DEBUG: Performing string concatenation: {} + {}", l, r);
                    Ok(LiteralValue::String(l + &r))
                }
                _ => Err("Operands for '+' must be two numbers or two strings.".to_string()),
            },
            TokenType::Minus => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l - r)),
                _ => Err("Operands for '-' must be numbers.".to_string()),
            },
            TokenType::Star => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l * r)),
                _ => Err("Operands for '*' must be numbers.".to_string()),
            },
            TokenType::Slash => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    if r == 0.0 {
                        Err("Division by zero.".to_string())
                    } else {
                        Ok(LiteralValue::Number(l / r))
                    }
                }
                _ => Err("Operands for '/' must be numbers.".to_string()),
            },
            TokenType::Percent => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    if r == 0.0 {
                        Err("Modulo by zero.".to_string())
                    } else {
                        Ok(LiteralValue::Number(l % r))
                    }
                }
                _ => Err("Operands for '%' must be numbers.".to_string()),
            },
            TokenType::Less => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l < r)),
                _ => Err("Operands for '<' must be numbers.".to_string()),
            },
            TokenType::LessEqual => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l <= r)),
                _ => Err("Operands for '<=' must be numbers.".to_string()),
            },
            TokenType::Greater => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l > r)),
                _ => Err("Operands for '>' must be numbers.".to_string()),
            },
            TokenType::GreaterEqual => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l >= r)),
                _ => Err("Operands for '>=' must be numbers.".to_string()),
            },
            TokenType::EqualEqual => Ok(LiteralValue::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(LiteralValue::Boolean(left_value != right_value)),
            TokenType::And => match (left_value, right_value) {
                (LiteralValue::Boolean(l), LiteralValue::Boolean(r)) => Ok(LiteralValue::Boolean(l && r)),
                _ => Err("Operands for 'and' must be booleans.".to_string()),
            },
            TokenType::Or => match (left_value, right_value) {
                (LiteralValue::Boolean(l), LiteralValue::Boolean(r)) => Ok(LiteralValue::Boolean(l || r)),
                _ => Err("Operands for 'or' must be booleans.".to_string()),
            },
            _ => Err(format!("Unsupported operator: {:?}", operator.token_type)),
        }
    }

    fn visit_unary(
        &mut self,
        operator: &Token,
        operand: &Expr,
    ) -> Result<LiteralValue, String> {
        let operand_value = self.evaluate(operand)?;

        println!(
            "DEBUG: Unary operation - operator: {:?}, operand: {:?}",
            operator.token_type, operand_value
        );

        match operator.token_type {
            TokenType::Minus => match operand_value {
                LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                _ => Err("Operand for '-' must be a number.".to_string()),
            },
            TokenType::Bang => match operand_value {
                LiteralValue::Boolean(b) => Ok(LiteralValue::Boolean(!b)),
                _ => Err("Operand for '!' must be a boolean.".to_string()),
            },
            _ => Err(format!(
                "Unsupported unary operator: {:?}",
                operator.token_type
            )),
        }
    }

    fn is_truthy(&self, value: LiteralValue) -> bool {
        match value {
            LiteralValue::Boolean(b) => b,
            LiteralValue::Nil => false,
            LiteralValue::Return(_) => true, 
            _ => true,
        }
    }
}