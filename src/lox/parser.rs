use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::{Token};
use crate::lox::token_type::{TokenType, LiteralValue};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { 
            tokens, 
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?.clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration.")?;
        Some(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }


    fn while_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Some(Stmt::While(condition, Box::new(body)))
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        
        let condition = self.expression()?;
        
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
        
        let then_branch = self.statement()?;
        
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };
        
        Some(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn block(&mut self) -> Option<Stmt> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
    
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Some(Stmt::Block(statements))
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Some(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Some(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Some(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            eprintln!("Parsing error: Invalid assignment target.");
            return None;
        }

        Some(expr)
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison();

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term();

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor();

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary();

        while self.match_token(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            });
        }
        expr
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Some(Expr::Unary {
                operator,
                operand: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.match_token(&[TokenType::Number]) {
            if let Some(literal) = &self.previous().literal {
                return Some(Expr::Literal { value: literal.clone() });
            }
        }

        if self.match_token(&[TokenType::StringLit]) {
            if let Some(literal) = &self.previous().literal {
                if let LiteralValue::String(s) = literal {
                    return Some(Expr::Literal {
                        value: LiteralValue::String(s.clone()),
                    });
                }
            }
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Some(Expr::Variable {
                name: self.previous().clone(),
            });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Some(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        None
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Option<&Token> {
        if self.check(token_type) {
            return Some(self.advance());
        }
        eprintln!("Parsing error: {}", message);
        None
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn for_statement(&mut self) -> Option<Stmt> {
    
        if self.consume(TokenType::LeftParen, "Expect '(' after 'for'.").is_none() {
            return None;
        }
    
        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration().map(Box::new)
        } else {
            self.expression_statement().map(Box::new)
        };
    
        // Condition
        let condition = if !self.check(TokenType::Semicolon) {
            self.expression()
        } else {
            None
        };
    
        if self.consume(TokenType::Semicolon, "Expect ';' after loop condition.").is_none() {
            return None;
        }
    
        let increment = if !self.check(TokenType::RightParen) {
            match self.expression() {
                Some(expr) => {
                    Some(expr)
                }
                None => {
                    None
                }
            }
        } else {
            None
        };
    
        if self.consume(TokenType::RightParen, "Expect ')' after for clauses.").is_none() {
            return None;
        }
    
        let body = match self.statement() {
            Some(stmt) => {
                stmt
            },
            None => {
                return None;
            }
        };
    
        Some(Stmt::For {
            initializer,
            condition,
            increment,
            body: Box::new(body),
        })
    }
}