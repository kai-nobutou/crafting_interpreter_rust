use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue; // LiteralValueをインポート

pub trait Visitor<R> {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_literal(&mut self, value: &LiteralValue) -> R; // ExprではなくLiteralValue
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_variable(&mut self, name: &Token) -> R;
    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> R;
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self) // acceptでVisitorに委譲
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        format!(
            "({} {} {})",
            operator.lexeme,
            self.print(left),
            self.print(right)
        )
    }

    fn visit_literal(&mut self, value: &LiteralValue) -> String { // LiteralValueを直接処理
        match value {
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::String(s) => s.clone(),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_grouping(&mut self, expression: &Expr) -> String {
        format!("(group {})", self.print(expression))
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        name.lexeme.clone()
    }

    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> String {
        format!("({} {})", operator.lexeme, self.print(operand))
    }
}
