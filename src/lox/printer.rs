use crate::lox::ast::Expr;
use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue;
use crate::lox::ast::Stmt;

pub trait Visitor<R> {
    // Expr 用
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> R;
    fn visit_literal(&mut self, value: &LiteralValue) -> R;
    fn visit_grouping(&mut self, expression: &Expr) -> R;
    fn visit_variable(&mut self, name: &Token) -> R;
    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> R;
    fn visit_assign(&mut self, name: &Token, value: &Expr) -> R;
    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> R;

    // 新しい Expr 用
    fn visit_get(&mut self, object: &Expr, name: &Token) -> R;
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> R;
    fn visit_this(&mut self, keyword: &Token) -> R;
    fn visit_super(&mut self, keyword: &Token, method: &Token) -> R;

    // Stmt 用
    fn visit_expression(&mut self, expr: &Expr) -> R;
    fn visit_print(&mut self, expr: &Expr) -> R;
    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) -> R;
    fn visit_block(&mut self, statements: &[Stmt]) -> R;
    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> R;
    fn visit_for(
        &mut self,
        initializer: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        increment: &Option<Expr>,
        body: &Stmt,
    ) -> R;
    fn visit_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Stmt>,
    ) -> R;
    fn visit_function(&mut self, name: &Token, params: &[Token], body: &[Stmt]) -> R;
    fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>) -> R;
    fn visit_class(&mut self, name: &Token, methods: &[(Token, Stmt)]) -> R;
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }
}


impl Visitor<String> for AstPrinter {
    // 既存のメソッド
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        format!("({} {} {})", operator.lexeme, left.accept(self), right.accept(self))
    }

    fn visit_literal(&mut self, value: &LiteralValue) -> String {
        value.to_string()
    }

    fn visit_grouping(&mut self, expression: &Expr) -> String {
        format!("(group {})", expression.accept(self))
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        format!("{}", name.lexeme)
    }

    fn visit_unary(&mut self, operator: &Token, operand: &Expr) -> String {
        format!("({} {})", operator.lexeme, operand.accept(self))
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> String {
        format!("(assign {} {})", name.lexeme, value.accept(self))
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> String {
        let args = arguments
            .iter()
            .map(|arg| arg.accept(self))
            .collect::<Vec<_>>()
            .join(", ");
        format!("(call {} {})", callee.accept(self), args)
    }

    // 新しい Expr 用
    fn visit_get(&mut self, object: &Expr, name: &Token) -> String {
        format!("(get {}.{})", object.accept(self), name.lexeme)
    }

    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> String {
        format!("(set {}.{} = {})", object.accept(self), name.lexeme, value.accept(self))
    }

    fn visit_this(&mut self, keyword: &Token) -> String {
        format!("(this {})", keyword.lexeme)
    }

    fn visit_super(&mut self, keyword: &Token, method: &Token) -> String {
        format!("(super {}.{})", keyword.lexeme, method.lexeme)
    }

    // Stmt 用
    fn visit_expression(&mut self, expr: &Expr) -> String {
        self.print(expr)
    }

    fn visit_print(&mut self, expr: &Expr) -> String {
        format!("(print {})", self.print(expr))
    }

    fn visit_var(&mut self, name: &Token, initializer: &Option<Expr>) -> String {
        if let Some(init) = initializer {
            format!("(var {} = {})", name.lexeme, self.print(init))
        } else {
            format!("(var {})", name.lexeme)
        }
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> String {
        let stmts = statements
            .iter()
            .map(|stmt| stmt.accept(self))
            .collect::<Vec<_>>()
            .join(" ");
        format!("(block {})", stmts)
    }

    fn visit_while(&mut self, condition: &Expr, body: &Stmt) -> String {
        format!("(while {} {})", self.print(condition), body.accept(self))
    }

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

    fn visit_return(&mut self, keyword: &Token, value: &Option<Expr>) -> String {
        if let Some(val) = value {
            format!("(return {})", self.print(val))
        } else {
            "(return)".to_string()
        }
    }

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