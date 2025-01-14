use crate::lox::ast::{Expr, Stmt};
use crate::lox::token::{Token};
use crate::lox::token_type::{TokenType, LiteralValue};


const MAX_RECURSION_DEPTH: usize = 1000;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    recursion_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { 
            tokens, 
            current: 0,
            recursion_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                break;
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(&[TokenType::Fun]) {
            return self.function("function");
        } else if self.match_token(&[TokenType::Var]) {
            return self.var_declaration();
        }
    
        self.statement()
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
        match self.peek()?.token_type {
            TokenType::For => {
                println!("Matched For");
                self.advance();
                self.for_statement()
            }
            TokenType::While => {
                println!("Matched While");
                self.advance();
                self.while_statement()
            }
            TokenType::If => {
                println!("Matched If");
                self.advance();
                self.if_statement()
            }
            TokenType::Return => {
                println!("Matched Return");
                self.advance();
                self.return_statement()
            }
            TokenType::Print => {
                println!("Matched Print");
                self.advance();
                self.print_statement()
            }
            TokenType::LeftBrace => {
                println!("Matched Block");
                self.advance();
                self.block()
            }
            _ => {
                println!("Matched Expression or Assignment");
                if let Some(expr) = self.assignment() {
                    // セミコロンを消費
                    self.consume(TokenType::Semicolon, "Expected ';' after expression or assignment.")?;
                    Some(Stmt::Expression(expr))
                } else {
                    self.expression_statement()
                }
            }
        }
    }

    fn return_statement(&mut self) -> Option<Stmt> {
        let keyword = self.previous().clone();
    
        let value = if !self.check(TokenType::Semicolon) {
            self.expression()
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expected ';' after return value.")?;
        Some(Stmt::Return { keyword, value })
    }


    fn while_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Some(Stmt::While(condition, Box::new(body)))
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        println!("Entering if_statement");
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        
        // 条件式のパース
        let condition = match self.expression() {
            Some(cond) => {
                println!("Parsed condition: {:?}", cond);
                cond
            }
            None => {
                eprintln!("Failed to parse condition in if_statement");
                return None;
            }
        };
    
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
    
        // then_branch のパース
        let then_branch = match self.statement() {
            Some(stmt) => {
                println!("Parsed then_branch: {:?}", stmt);
                stmt
            }
            None => {
                eprintln!("Failed to parse then_branch in if_statement");
                return None;
            }
        };
    
        // else_branch のパース
        let else_branch = if self.match_token(&[TokenType::Else]) {
            match self.statement() {
                Some(stmt) => {
                    println!("Parsed else_branch: {:?}", stmt);
                    Some(stmt)
                }
                None => {
                    eprintln!("Failed to parse else_branch in if_statement");
                    return None;
                }
            }
        } else {
            println!("No else_branch found");
            None
        };
    
        // Stmt::If の生成
        let stmt = Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        };
        println!("Generated If statement: {:?}", stmt);
    
        Some(stmt)
    }

    fn block(&mut self) -> Option<Stmt> {
        println!("block: Entering block");
        let mut statements = Vec::new();
    
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                println!("block: Successfully parsed statement: {:?}", stmt);
                statements.push(stmt);
            } else {
                println!("block: Error in declaration inside block. Current token: {:?}", self.peek());
                return None;
            }
        }
    
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        println!("block: Exiting block with statements: {:?}", statements);
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
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            eprintln!("Error: Maximum recursion depth exceeded.");
            return None;
        }
        self.recursion_depth += 1;
        let result = self.assignment();
        self.recursion_depth -= 1;
        result
    }

    fn assignment(&mut self) -> Option<Expr> {
        let mut expr = self.equality()?;
    
        while self.match_token(&[TokenType::Equal]) {
            let value = self.equality()?; // 再帰をループに変更
            if let Expr::Variable { name } = expr {
                expr = Expr::Assign {
                    name,
                    value: Box::new(value),
                };
            } else {
                return None; // 不正な代入の場合
            }
        }
    
        // println!("Exiting assignment with result: {:?}", expr);
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
            let variable = self.previous().clone();
    
            if self.match_token(&[TokenType::LeftParen]) {
                let mut arguments = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
                return Some(Expr::Call {
                    callee: Box::new(Expr::Variable { name: variable }),
                    arguments,
                });
            }
    
            return Some(Expr::Variable { name: variable });
        }
    
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Some(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.match_token(&[TokenType::True]) {
            return Some(Expr::Literal {
                value: LiteralValue::Boolean(true),
            });
        }

        if self.match_token(&[TokenType::False]) {
            return Some(Expr::Literal {
                value: LiteralValue::Boolean(false),
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
            println!("Consumed token: {:?}", self.peek());
            return Some(self.advance());
        }
        eprintln!("Parsing error in consume: {}", message);
        None
    }

    fn check(&self, token_type: TokenType) -> bool {
        if let Some(token) = self.peek() {
            token.token_type == token_type
        } else {
            false
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        if let Some(token) = self.peek() {
            token.token_type == TokenType::Eof
        } else {
            true
        }
    }

    fn peek(&self) -> Option<&Token> {
        let result = if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        };
        result
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
    
    fn function(&mut self, kind: &str) -> Option<Stmt> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?.clone();
        self.consume(TokenType::LeftParen, &format!("Expect '(' after {} name.", kind))?;
    
        let mut params: Vec<(Token, Option<Expr>)> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let param_name = self.consume(TokenType::Identifier, "Expect parameter name.")?.clone();
                let default_value = if self.match_token(&[TokenType::Equal]) {
                    Some(self.expression()?)
                } else {
                    None
                };
                params.push((param_name, default_value));
    
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
    
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, &format!("Expect '{{' before {} body.", kind))?;
    
        let body = match self.block()? {
            Stmt::Block(statements) => statements,
            _ => return None,
        };
    
        Some(Stmt::Function {
            name,
            params: params.into_iter().map(|(token, _)| token).collect(),
            body,
        })
    }
}