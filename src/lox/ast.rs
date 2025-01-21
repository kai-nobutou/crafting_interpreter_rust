use crate::lox::printer::Visitor;
use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Variable {
        name: Token,
    },
    Unary {
        operator: Token,
        operand: Box<Expr>,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    While(Expr, Box<Stmt>),
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<(Token, Stmt)>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Assign {
        name: Token,
        value: Expr,
    },
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary(left, operator, right),
            Expr::Literal { value } => visitor.visit_literal(value),
            Expr::Grouping { expression } => visitor.visit_grouping(expression),
            Expr::Variable { name } => visitor.visit_variable(name),
            Expr::Unary { operator, operand } => visitor.visit_unary(operator, operand),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::Call { callee, arguments } => visitor.visit_call(callee, arguments),
            Expr::Get { object, name } => visitor.visit_get(object, name),
            Expr::Set {
                object,
                name,
                value,
            } => visitor.visit_set(object, name, value),
        }
    }
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression(expr),
            Stmt::Print(expr) => visitor.visit_print(expr),
            Stmt::Var { name, initializer } => visitor.visit_var(name, initializer),
            Stmt::Block(statements) => visitor.visit_block(statements),
            Stmt::While(condition, body) => visitor.visit_while(condition, body),
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => visitor.visit_for(initializer, condition, increment, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let temp_else_branch: Option<Stmt> =
                    else_branch.as_ref().map(|stmt| (**stmt).clone());
                visitor.visit_if(condition.as_ref(), then_branch.as_ref(), &temp_else_branch)
            }
            Stmt::Function { name, params, body } => visitor.visit_function(name, params, body),
            Stmt::Return { keyword, value } => visitor.visit_return(keyword, value),
            Stmt::Class { name, methods } => visitor.visit_class(name, methods),
            Stmt::Call { callee, arguments } => visitor.visit_call(callee, arguments),
            Stmt::Assign { name, value } => visitor.visit_assign(name, value),
        }
    }
}
