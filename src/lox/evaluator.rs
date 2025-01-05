use crate::lox::ast::{Expr, Stmt};
use crate::lox::printer::Visitor;
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

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }
}

pub struct Evaluator {
    environment: Environment,
    had_error: bool,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            environment: Environment::new(),
            had_error: false,
        }
    }
    
    fn execute_block(&mut self, statements: Vec<Stmt>) -> Result<LiteralValue, String> {
        for stmt in statements {
            if let Err(err) = self.execute(stmt) {
                if err.starts_with("Return: ") {
                    let value_str = err.trim_start_matches("Return: ");
                    let result = if value_str == "nil" {
                        LiteralValue::Nil
                    } else if let Ok(number) = value_str.parse::<f64>() {
                        LiteralValue::Number(number)
                    } else {
                        LiteralValue::String(value_str.to_string())
                    };
                    return Ok(result);
                } else {
                    return Err(err);
                }
            }
        }
        Ok(LiteralValue::Nil)
    }

    pub fn evaluate_statements(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            if let Err(err) = self.execute(stmt) {
                eprintln!("Error: {}", err);
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(&expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr)?;
                println!("{:?}", value);
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(initializer) = initializer {
                    self.evaluate(&initializer)?
                } else {
                    LiteralValue::Nil
                };
                self.environment.define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block(statements) => {
                let mut new_environment = Environment {
                    enclosing: Some(Box::new(self.environment.clone())),
                    values: HashMap::new(),
                };
                std::mem::swap(&mut self.environment, &mut new_environment);

                for statement in statements {
                    self.execute(statement)?;
                }

                std::mem::swap(&mut self.environment, &mut new_environment);
                Ok(())
            }
            Stmt::While(condition, body) => {
                while self.evaluate(&condition).map_or(false, |v| self.is_truthy(v)) {
                    
                    // Execute the loop body
                    self.execute(*body.clone())?;
                    
                    // インクリメント処理を追加
                    if let Stmt::Block(statements) = *body.clone() {
                        for stmt in statements {
                            if let Stmt::Expression(expr) = stmt {
                                if let Expr::Assign { name, value } = expr {
                                    // 評価して値を更新
                                    let new_value = self.evaluate(&value)?;
                                    self.environment.assign(name.lexeme.clone(), new_value.clone());
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Stmt::For { initializer, condition, increment, body } => {
                if let Some(init_stmt) = initializer {
                    self.execute(*init_stmt)?;
                }

                loop {
                    if let Some(cond_expr) = &condition {
                        let cond_value = self.evaluate(cond_expr)?;
                        if !self.is_truthy(cond_value) {
                            break;
                        }
                    }

                    self.execute(*body.clone())?;

                    if let Some(incr_expr) = &increment {
                        self.evaluate(incr_expr)?;
                    }
                }

                Ok(())
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_value = self.evaluate(&condition)?;
                if self.is_truthy(cond_value) {
                    self.execute(*then_branch.clone())?;
                } else if let Some(else_stmt) = else_branch {
                    self.execute(*else_stmt.clone())?;
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let function = LiteralValue::Function {
                    name: name.lexeme.clone(),
                    params,
                    body,
                    closure: self.environment.clone(),
                };
                self.environment.define(name.lexeme.clone(), function);
                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let result = if let Some(expr) = value {
                    self.evaluate(&expr)?
                } else {
                    LiteralValue::Nil
                };
            
                Err(format!("Return: {:?}", result))
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        expr.accept(self)
    }

    fn is_truthy(&self, value: LiteralValue) -> bool {
        match value {
            LiteralValue::Boolean(b) => b,
            LiteralValue::Nil => false,
            _ => true,
        }
    }
}


impl Visitor<Result<LiteralValue, String>> for Evaluator {
    fn visit_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<LiteralValue, String> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Plus => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l + r)),
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Minus => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l - r)),
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Star => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Number(l * r)),
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Slash => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    if r == 0.0 {
                        Err("Division by zero.".to_string())
                    } else {
                        Ok(LiteralValue::Number(l / r))
                    }
                }
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Percent => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    if r == 0.0 {
                        Err("Modulo by zero.".to_string())
                    } else {
                        Ok(LiteralValue::Number(l % r))
                    }
                }
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Greater => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l > r)),
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::Less => match (left_value, right_value) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => Ok(LiteralValue::Boolean(l < r)),
                _ => Err("Operands must be numbers.".to_string()),
            },
            TokenType::EqualEqual => Ok(LiteralValue::Boolean(left_value == right_value)),
            TokenType::BangEqual => Ok(LiteralValue::Boolean(left_value != right_value)),
            _ => Err(format!("Unsupported operator: {:?}", operator.token_type)),
        }
    }

    fn visit_literal(&mut self, value: &LiteralValue) -> Result<LiteralValue, String> {
        Ok(value.clone())
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Result<LiteralValue, String> {
        self.evaluate(expression)
    }

    fn visit_variable(&mut self, name: &crate::lox::token::Token) -> Result<LiteralValue, String> {
        self.environment
            .get(&name.lexeme)
            .ok_or_else(|| format!("Undefined variable '{}'.", name.lexeme))
    }

    fn visit_unary(
        &mut self,
        operator: &crate::lox::token::Token,
        operand: &Expr,
    ) -> Result<LiteralValue, String> {
        let operand_value = self.evaluate(operand)?;
        match operator.token_type {
            crate::lox::token_type::TokenType::Minus => match operand_value {
                LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                _ => Err("Operand must be a number.".to_string()),
            },
            _ => Err("Unsupported unary operator.".to_string()),
        }
    }

    fn visit_assign(&mut self, name: &crate::lox::token::Token, value: &Expr) -> Result<LiteralValue, String> {
        let value = self.evaluate(value)?;
        self.environment.assign(name.lexeme.clone(), value.clone());
        Ok(value)
    }

    fn visit_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
    ) -> Result<LiteralValue, String> {
        let callee_value = self.evaluate(callee)?;
    
        if let LiteralValue::Function { params, body, closure, .. } = callee_value {
            if arguments.len() != params.len() {
                return Err(format!(
                    "Expected {} arguments but got {}.",
                    params.len(),
                    arguments.len()
                ));
            }
    
            let mut function_environment = Environment {
                enclosing: Some(Box::new(closure)),
                values: HashMap::new(),
            };
    
            for (param, arg) in params.iter().zip(arguments.iter()) {
                let value = self.evaluate(arg)?;
                function_environment.define(param.lexeme.clone(), value);
            }
    
            // スコープの切り替え
            let mut previous_environment = self.environment.clone();
            self.environment = function_environment;
    
            let result = match self.execute_block(body) {
                Ok(value) => value,
                Err(err) if err.starts_with("Return: ") => {
                    let return_value = err.trim_start_matches("Return: ").to_string();
                    self.evaluate(&Expr::Literal {
                        value: LiteralValue::String(return_value),
                    })?
                }
                Err(err) => return Err(err),
            };
    
            // スコープを元に戻す
            self.environment = previous_environment;
    
            Ok(result)
        } else {
            Err("Can only call functions.".to_string())
        }
    }
}