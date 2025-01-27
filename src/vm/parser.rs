use crate::vm::ast_node::{ASTNode, BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    Identifier(String),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    If,
    Else,
    While,
    Function,
    Return,
    EOF, // ファイルの終端を示す
}

pub struct Parser {
    tokens: Vec<Token>, 
    current: usize,  
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        self.parse_program()
    }

    fn parse_program(&mut self) -> Result<ASTNode, String> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            nodes.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(nodes))
    }

    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        if self.match_token(&[Token::If]) {
            self.parse_if_statement()
        } else if self.match_token(&[Token::While]) {
            self.parse_while_statement()
        } else if self.match_token(&[Token::Function]) {
            self.parse_function_declaration()
        } else if self.match_token(&[Token::Return]) {
            self.parse_return_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_expression_statement(&mut self) -> Result<ASTNode, String> {
        let expr = self.parse_expression()?;
        self.consume(&Token::Semicolon, "Expect ';' after expression.")?;
        Ok(ASTNode::ExpressionStatement(Box::new(expr)))
    }

    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<ASTNode, String> {
        let mut expr = self.parse_term()?;
    
        while self.match_token(&[Token::Equals]) {
            let operator = match self.previous() {
                Token::Equals => BinaryOperator::Equals,
                _ => return Err("Invalid binary operator.".to_string()),
            };
    
            let right = self.parse_term()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator, 
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ASTNode, String> {
        let mut expr = self.parse_factor()?;
    
        while self.match_token(&[Token::Plus, Token::Minus]) {
            let operator = match self.previous() {
                Token::Plus => BinaryOperator::Plus,
                Token::Minus => BinaryOperator::Minus,
                _ => return Err("Invalid binary operator.".to_string()),
            };
    
            let right = self.parse_factor()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ASTNode, String> {
        let mut expr = self.parse_unary()?;
    
        while self.match_token(&[Token::Star, Token::Slash]) {
            let operator = match self.previous() {
                Token::Star => BinaryOperator::Star,
                Token::Slash => BinaryOperator::Slash,
                _ => return Err("Invalid binary operator.".to_string()),
            };
    
            let right = self.parse_unary()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, String> {
        if self.match_token(&[Token::Minus]) {
            let operator = match self.previous() {
                Token::Minus => UnaryOperator::Minus,
                _ => return Err("Invalid unary operator.".to_string()),
            };
    
            let right = self.parse_primary()?;
            return Ok(ASTNode::UnaryExpression {
                operator, 
                right: Box::new(right),
            });
        }
    
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<ASTNode, String> {
        if self.match_token(&[Token::Number(0.0)]) {
            if let Token::Number(value) = self.previous() {
                return Ok(ASTNode::NumberLiteral(*value));
            }
        }

        if self.match_token(&[Token::String(String::new())]) {
            if let Token::String(value) = self.previous() {
                return Ok(ASTNode::StringLiteral(value.clone()));
            }
        }

        if self.match_token(&[Token::Identifier(String::new())]) {
            if let Token::Identifier(name) = self.previous() {
                return Ok(ASTNode::VariableReference(name.clone()));
            }
        }

        if self.match_token(&[Token::LeftParen]) {
            let expr = self.parse_expression()?;
            self.consume(&Token::RightParen, "Expect ')' after expression.")?;
            return Ok(ASTNode::Grouping(Box::new(expr)));
        }

        Err("Expect expression.".to_string())
    }

    fn parse_return_statement(&mut self) -> Result<ASTNode, String> {
        let value = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(&Token::Semicolon, "Expect ';' after return value.")?;
        Ok(ASTNode::ReturnStatement(value.map(Box::new)))
    }

    fn parse_if_statement(&mut self) -> Result<ASTNode, String> {
        self.consume(&Token::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expect ')' after condition.")?;

        self.consume(&Token::LeftBrace, "Expect '{' before 'if' body.")?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.match_token(&[Token::Else]) {
            self.consume(&Token::LeftBrace, "Expect '{' before 'else' body.")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(ASTNode::IfStatement {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<ASTNode, String> {
        self.consume(&Token::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.parse_expression()?;
        self.consume(&Token::RightParen, "Expect ')' after condition.")?;

        self.consume(&Token::LeftBrace, "Expect '{' before 'while' body.")?;
        let body = self.parse_block()?;

        Ok(ASTNode::WhileStatement {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_function_declaration(&mut self) -> Result<ASTNode, String> {
        let name_token = self.consume(&Token::Identifier(String::new()), "Expect function name.")?;
        let name = if let Token::Identifier(name) = name_token {
            name.clone()
        } else {
            return Err("Invalid function name.".to_string());
        };
    
        self.consume(&Token::LeftParen, "Expect '(' after function name.")?;
    
        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_token = self.consume(&Token::Identifier(String::new()), "Expect parameter name.")?;
                if let Token::Identifier(param) = param_token {
                    parameters.push(param.clone());
                }
    
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
    
        self.consume(&Token::RightParen, "Expect ')' after parameters.")?;
        self.consume(&Token::LeftBrace, "Expect '{' before function body.")?;
        let body = self.parse_block()?;
    
        Ok(ASTNode::FunctionDeclaration {
            name,
            parameters,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.consume(&Token::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn match_token(&mut self, types: &[Token]) -> bool {
        if self.is_at_end() {
            return false;
        }
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        match (token, &self.tokens[self.current]) {
            (Token::Number(_), Token::Number(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Identifier(_), Token::Identifier(_)) => true,
            _ => token == &self.tokens[self.current],
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<&Token, String> {
        if self.check(token) {
            return Ok(self.advance());
        }
        Err(message.to_string())
    }
}
