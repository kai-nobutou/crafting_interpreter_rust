use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::Token;
use crate::lox::token_type::{TokenType, LiteralValue};
use crate::lox::printer::Visitor;
use std::collections::HashMap;

pub struct Evaluator {
    environment: Environment,
    had_error: bool,
}

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
        self.values.get(name).cloned().or_else(|| {
            self.enclosing.as_ref()?.get(name)
        })
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            environment: Environment::new(),
            had_error: false,
        }
    }

    pub fn evaluate_statements(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            if let Err(err) = self.execute(stmt) {
                eprintln!("Error: {}", err); // エラーを出力して続行
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expression(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = initializer
                    .map(|expr| self.evaluate_expression(expr))
                    .transpose()?
                    .unwrap_or(LiteralValue::Nil);
                self.environment.define(name.lexeme.clone(), value);
                Ok(())
            }
        }
    }

    fn evaluate_expression(&mut self, expr: Expr) -> Result<LiteralValue, String> {
        expr.accept(self)
    }

    fn report_error(&mut self, message: &str) {
        eprintln!("Error: {}", message);
        self.had_error = true; // エラーフラグを立てる
    }
}

impl Visitor<Result<LiteralValue, String>> for Evaluator {
    fn visit_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<LiteralValue, String> {
        let left_value = left.accept(self)?;
        let right_value = right.accept(self)?;
    
        match operator.token_type {
            TokenType::Plus => match (left_value.clone(), right_value.clone()) {
                (LiteralValue::Number(l), LiteralValue::Number(r)) => {
                    Ok(LiteralValue::Number(l + r))
                }
                (LiteralValue::String(l), LiteralValue::String(r)) => {
                    Ok(LiteralValue::String(format!("{}{}", l, r)))
                }
                _ => Err(format!(
                    "Operands must be two numbers or two strings. Left: {:?}, Right: {:?}",
                    left_value, right_value
                )),
            },
            _ => Err(format!("Unsupported operator: {}", operator.lexeme)),
        }
    }

    fn visit_literal(&mut self, value: &LiteralValue) -> Result<LiteralValue, String> {
        Ok(value.clone())
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Result<LiteralValue, String> {
        expression.accept(self)
    }

    fn visit_variable(&mut self, name: &Token) -> Result<LiteralValue, String> {
        self.environment
            .get(&name.lexeme)
            .ok_or_else(|| format!("Undefined variable '{}'.", name.lexeme))
    }

    fn visit_unary(
        &mut self,
        operator: &Token,
        operand: &Expr,
    ) -> Result<LiteralValue, String> {
        let operand_value = operand.accept(self)?;
        match operator.token_type {
            TokenType::Minus => match operand_value {
                LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                _ => Err("Operand must be a number.".to_string()),
            },
            _ => Err(format!("Unsupported unary operator: {}", operator.lexeme)),
        }
    }
}