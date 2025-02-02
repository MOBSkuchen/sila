use crate::comp_errors::{CodeError, CodeResult};
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self, pointer: &usize) -> Option<&Token> {
        self.tokens.get(*pointer)
    }

    fn advance(&self, pointer: &mut usize) -> Option<&Token> {
        let token = self.tokens.get(*pointer);
        if token.is_some() {
            *pointer += 1;
        }
        token
    }

    fn match_token(&self, pointer: &mut usize, token_type: TokenType) -> bool {
        if let Some(token) = self.peek(pointer) {
            if token.token_type == token_type {
                self.advance(pointer);
                return true;
            }
        }
        false
    }
    
    fn consume(&self, pointer: &mut usize, expected: TokenType) -> CodeResult<&Token> {
        if self.match_token(pointer, expected) {
            Ok(self.previous(pointer).unwrap())
        } else {
            Err(CodeError::placeholder())
        }
    }
    
    fn previous(&self, pointer: &usize) -> Option<&Token> {
        self.tokens.get(*pointer)
    }


    pub fn parse(&self, pointer: &mut usize) -> CodeResult<Vec<ASTNode>> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek(pointer) {
            match token.token_type {
                // Parse function definitions
                TokenType::Define => {
                    let func = self.parse_function(pointer)?;
                    statements.push(func);
                }

                // Parse import statements
                TokenType::Import => {
                    let import_stmt = self.parse_import(pointer)?;
                    statements.push(import_stmt);
                }

                _ => {
                    return Err(CodeError::placeholder());
                }
            }
        }

        Ok(statements)
    }

    // Parse import statement (assuming a simple import structure)
    fn parse_import(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        // Consume 'import' keyword
        self.consume(pointer, TokenType::Import)?;

        // Expect an identifier for the import (e.g., module name)
        let module_name = self.consume(pointer, TokenType::Identifier)?;

        // Optionally, handle import paths or other structures here if needed
        Ok(ASTNode::Import(module_name))
    }

    pub fn parse_function(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let name = self.consume(pointer, TokenType::Identifier)?;

        self.consume(pointer, TokenType::LParen)?;

        let args = self.parse_arguments(pointer)?;

        self.consume(pointer, TokenType::RParen)?;

        self.consume(pointer, TokenType::Colon)?;
        let return_type = self.parse_type(pointer)?;

        let body = self.parse_block(pointer)?;

        Ok(ASTNode::FunctionDef(
            name,
            FunctionMode::Default, // TODO: Add modifiers
            Box::new(return_type),
            args,
            body,
        ))
    }

    fn parse_block(&self, pointer: &mut usize) -> CodeResult<Vec<Box<ASTNode>>> {
        self.consume(pointer, TokenType::LBrace)?;

        let mut statements = Vec::new();

        // Parse statements inside the block
        while let Some(token) = self.peek(pointer) {
            if token.token_type == TokenType::RBrace {
                break;
            }

            let stmt = self.parse_statement(pointer)?;
            statements.push(Box::new(stmt));

            if !self.match_token(pointer, TokenType::SemiColon) {
                break;
            }
        }

        self.consume(pointer, TokenType::RBrace)?;

        Ok(statements)
    }
    fn parse_statement(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let token = self.peek(pointer);

        if let Some(token) = token {
            match token.token_type {
                TokenType::Identifier | TokenType::NumberInt | TokenType::NumberFloat => {
                    self.parse_expression(pointer)
                }
                _ => {
                    Err(CodeError::placeholder())
                }
            }
        } else {
            Err(CodeError::placeholder())
        }
    }

    fn parse_arguments(&self, pointer: &mut usize) -> CodeResult<Vec<(&Token, Box<ASTNode>)>> {
        let mut arguments = Vec::new();

        while let Some(token) = self.peek(pointer) {
            if token.token_type == TokenType::RParen {
                break;
            }

            let name = self.consume(pointer, TokenType::Identifier)?;
            self.consume(pointer, TokenType::Colon)?;
            let arg_type = self.parse_type(pointer)?;

            arguments.push((name, Box::new(arg_type)));

            if !self.match_token(pointer, TokenType::Comma) {
                break;
            }
        }

        Ok(arguments)
    }

    fn parse_expression(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let term = self.parse_term(pointer)?;
        if self.match_token(pointer, TokenType::As) {
            Ok(ASTNode::CastExpr(Box::new(term), Box::new(self.parse_type(pointer)?)))
        } else {
            Ok(term)
        }
    }

    fn parse_term(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let mut node = self.parse_factor(pointer)?;

        while let Some(token) = self.peek(pointer) {
            match token.token_type {
                TokenType::Plus | TokenType::Minus => {
                    let op = self.advance(pointer).unwrap();
                    let right = self.parse_factor(pointer)?;
                    node = ASTNode::BinaryOp(Box::new(node), op, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_factor(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let mut node = self.parse_primary(pointer)?;

        while let Some(token) = self.peek(pointer) {
            match token.token_type {
                TokenType::Star | TokenType::Slash => {
                    let op = self.advance(pointer).unwrap();
                    let right = self.parse_primary(pointer)?;
                    node = ASTNode::BinaryOp(Box::new(node), op, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_primary(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        if let Some(token) = self.advance(pointer) {
            match token.token_type {
                TokenType::NumberInt | TokenType::NumberFloat => {
                    return Ok(ASTNode::Literal(token))
                }
                TokenType::Identifier => {
                    return Ok(ASTNode::Identifier(token))
                }
                TokenType::String => {
                    return Ok(ASTNode::String(token))
                }
                TokenType::LParen => {
                    let expr = self.parse_expression(pointer)?;
                    return if self.match_token(pointer, TokenType::RParen) {
                        Ok(expr)
                    } else {
                        Err(CodeError::placeholder())
                    }
                }
                _ => {}
            }
        }
        Err(CodeError::placeholder())
    }
    
    fn parse_type(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        Ok(ASTNode::Type(self.consume(pointer, TokenType::Identifier)?))
    }
}

#[derive(Debug)]
pub enum FunctionMode {
    Private,
    Export,
    Extern,
    Default
}

#[derive(Debug)]
pub enum ASTNode<'a> {
    // Literal (a number)
    Literal(&'a Token),
    // Name
    Identifier(&'a Token),
    // Content
    String(&'a Token),
    // Currently just an identifier
    Type(&'a Token),
    // LHS, Opcode, RHS
    BinaryOp(Box<ASTNode<'a>>, &'a Token, Box<ASTNode<'a>>),
    // Expr, Type
    CastExpr(Box<ASTNode<'a>>, Box<ASTNode<'a>>),
    // Name, Function mode (private / export / extern), Return-type, Arguments (name, type), Content (Node)
    FunctionDef(&'a Token, FunctionMode, Box<ASTNode<'a>>, Vec<(&'a Token, Box<ASTNode<'a>>)>, Vec<Box<ASTNode<'a>>>),
    // Name, Expr, Type annotation (opt)
    VariableSet(&'a Token, Box<ASTNode<'a>>, Option<Box<ASTNode<'a>>>),
    // Lib name
    Import(&'a Token),
}
