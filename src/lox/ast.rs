use crate::lox::token::Token;
use crate::lox::token_type::LiteralValue;
use crate::lox::printer::Visitor;

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
}

impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Expr::Binary { left, operator, right } => visitor.visit_binary(left, operator, right),
            Expr::Literal { value } => visitor.visit_literal(value),
            Expr::Grouping { expression } => visitor.visit_grouping(expression),
            Expr::Variable { name } => visitor.visit_variable(name),
            Expr::Unary { operator, operand } => visitor.visit_unary(operator, operand),
            Expr::Assign { name, value } => visitor.visit_assign(name, value),
            Expr::Call { callee, arguments } => visitor.visit_call(callee, arguments),
        }
    }
}