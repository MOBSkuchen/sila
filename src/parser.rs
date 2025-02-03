use crate::codeviz::print_code_warn;
use crate::comp_errors::{CodeError, CodeResult, CodeWarning};
use crate::filemanager::FileManager;
use crate::lexer::{CodePosition, Token, TokenType};
use crate::parser::ASTNode::FunctionCall;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    file_manager: &'a FileManager,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, file_manager: &'a FileManager) -> Self {
        Self {
            tokens,
            file_manager,
        }
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

    fn match_token(&self, pointer: &mut usize, token_type: TokenType) -> CodeResult<bool> {
        self.is_done_err(pointer)?;
        if let Some(token) = self.peek(pointer) {
            if token.token_type == token_type {
                self.advance(pointer);
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    fn multi_match_token(&self, pointer: &mut usize, token_types: Vec<TokenType>) -> CodeResult<bool> {
        self.is_done_err(pointer)?;
        if let Some(token) = self.peek(pointer) {
            if token_types.contains(&token.token_type) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn match_next_token(&self, pointer: &mut usize, token_type: TokenType) -> CodeResult<bool> {
        self.is_done_err(pointer)?;
        if let Some(token) = self.tokens.get(*pointer + 1) {
            if token.token_type == token_type {
                self.advance(pointer);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn consume(
        &self,
        pointer: &mut usize,
        expected: TokenType,
        note: Option<String>,
    ) -> CodeResult<&Token> {
        self.is_done_err(pointer)?;
        if self.match_token(pointer, expected)? {
            Ok(self.previous(pointer).unwrap())
        } else {
            Err(CodeError::new_unexpected_token_error(
                self.current(pointer).or(self.previous(pointer)).unwrap(),
                expected,
                note,
            ))
        }
    }

    fn previous(&self, pointer: &usize) -> Option<&Token> {
        self.tokens.get(*pointer - 1)
    }

    fn current(&self, pointer: &usize) -> Option<&Token> {
        self.tokens.get(*pointer)
    }

    fn warning(&self, code_warning: CodeWarning) {
        print_code_warn(code_warning, self.file_manager)
    }

    fn codepos_from_space(&self, s: usize, e: &usize, sub_off: usize) -> CodePosition {
        let start = self.tokens.get(s).unwrap().code_position;
        let end = self.tokens.get(*e - sub_off).unwrap().code_position;
        CodePosition {
            idx_start: start.idx_start,
            idx_end: end.idx_end,
            line_start: start.line_start,
            line_end: end.line_end,
            line_idx_start: start.line_idx_start,
            line_idx_end: end.line_idx_end,
        }
    }

    pub fn parse(&self, pointer: &mut usize) -> CodeResult<Vec<ASTNode>> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek(pointer) {
            match token.token_type {
                // Parse function definitions
                TokenType::Define => {
                    self.advance(pointer);
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
        self.consume(pointer, TokenType::Import, None)?;

        // Expect an identifier for the import (e.g., module name)
        let module_name = self.consume(pointer, TokenType::Identifier, None)?;

        // Optionally, handle import paths or other structures here if needed
        Ok(ASTNode::Import(module_name))
    }

    pub fn parse_function(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let fmod = if self.match_token(pointer, TokenType::Export)? { FunctionMode::Export }
        else if self.match_token(pointer, TokenType::Private)? { FunctionMode::Private }
        else if self.match_token(pointer, TokenType::Extern)? { FunctionMode::Extern } 
        else { FunctionMode::Default };
        
        if self.multi_match_token(pointer, vec![TokenType::Extern, TokenType::Export, TokenType::Private])? {
            return Err(CodeError::function_overloaded(self.previous(pointer).unwrap()))
        }
        
        let name = self.consume(pointer, TokenType::Identifier, None)?;

        self.consume(pointer, TokenType::LParen, None)?;

        let args = self.parse_arguments(pointer)?;

        self.consume(pointer, TokenType::RParen, None)?;

        self.consume(pointer, TokenType::Colon, None)?;
        let return_type = self.parse_type(pointer)?;

        let body = self.parse_block(pointer)?;

        Ok(ASTNode::FunctionDef(
            name,
            fmod,
            Box::new(return_type),
            args,
            body,
        ))
    }

    fn parse_block(&self, pointer: &mut usize) -> CodeResult<Vec<Box<ASTNode>>> {
        self.consume(pointer, TokenType::LBrace, None)?;

        let mut statements = Vec::new();

        while let Some(token) = self.peek(pointer) {
            if token.token_type == TokenType::RBrace {
                break;
            }

            let stmt = self.parse_statement(pointer)?;
            statements.push(Box::new(stmt));

            if !self.match_token(pointer, TokenType::SemiColon)? {
                break;
            }
        }

        self.consume(
            pointer,
            TokenType::RBrace,
            Some("You may be missing a semi colon".to_string()),
        )?;

        Ok(statements)
    }

    fn parse_function_call(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let name = self.previous(pointer).unwrap();
        self.consume(pointer, TokenType::LParen, None)?;
        let mut paras = vec![];
        while let Some(tok) = self.peek(pointer) {
            paras.push(Box::new(self.parse_expression(pointer)?));
            if self.match_token(pointer, TokenType::RParen)? {
                break;
            }
            self.consume(pointer, TokenType::Comma, Some("Add a comma".to_string()))?;
        }
        Ok(ASTNode::FunctionCall(name, paras))
    }

    fn parse_return(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        self.consume(pointer, TokenType::Return, None)?;
        Ok(ASTNode::Return(Box::new(self.parse_expression(pointer)?)))
    }

    fn parse_statement(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let token = self.peek(pointer);

        if let Some(token) = token {
            match token.token_type {
                TokenType::Identifier => {
                    if self.match_next_token(pointer, TokenType::LParen)? {
                        self.parse_function_call(pointer)
                    } else {
                        let a = *pointer;
                        let res = self.parse_expression(pointer);
                        self.warning(CodeWarning::new_unnecessary_code(
                            self.codepos_from_space(a, pointer, 1),
                            None,
                        ));
                        res
                    }
                }
                TokenType::NumberInt | TokenType::NumberFloat => {
                    let a = *pointer;
                    let res = self.parse_expression(pointer);
                    self.warning(CodeWarning::new_unnecessary_code(
                        self.codepos_from_space(a, pointer, 1),
                        None,
                    ));
                    res
                }
                TokenType::Return => self.parse_return(pointer),
                o => Err(CodeError::new_unexpected_token_error(
                    token,
                    TokenType::Statement,
                    Some("Expected some sort of statement".to_string()),
                )),
            }
        } else {
            Err(CodeError::missing_token_error(
                self.previous(pointer).unwrap(),
            ))
        }
    }

    fn is_done(&self, pointer: &usize) -> bool {
        (*pointer - 1) == self.tokens.len()
    }

    fn is_done_err(&self, pointer: &usize) -> CodeResult<()> {
        if self.is_done(pointer) {
            Err(CodeError::missing_token_error(
                self.previous(pointer).unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    fn parse_arguments(&self, pointer: &mut usize) -> CodeResult<Vec<(&Token, Box<ASTNode>)>> {
        let mut arguments = Vec::new();

        while let Some(token) = self.peek(pointer) {
            if token.token_type == TokenType::RParen {
                break;
            }

            let name = self.consume(pointer, TokenType::Identifier, None)?;
            self.consume(pointer, TokenType::Colon, None)?;
            let arg_type = self.parse_type(pointer)?;

            arguments.push((name, Box::new(arg_type)));

            if !self.match_token(pointer, TokenType::Comma)? {
                break;
            }
        }

        Ok(arguments)
    }

    fn parse_expression(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        let term = self.parse_term(pointer)?;
        if self.match_token(pointer, TokenType::As)? {
            Ok(ASTNode::CastExpr(
                Box::new(term),
                Box::new(self.parse_type(pointer)?),
            ))
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
                TokenType::NumberInt | TokenType::NumberFloat => Ok(ASTNode::Literal(token)),
                TokenType::Identifier => {
                    if self.match_next_token(pointer, TokenType::LParen)? {
                        self.parse_function_call(pointer)
                    } else {
                        Ok(ASTNode::Identifier(token))
                    }
                }
                TokenType::String => Ok(ASTNode::String(token)),
                TokenType::LParen => {
                    let expr = self.parse_expression(pointer)?;
                    if self.match_token(pointer, TokenType::RParen)? {
                        Ok(expr)
                    } else {
                        println!("LParen");
                        Err(CodeError::placeholder())
                    }
                }
                _ => Err(CodeError::new_unexpected_token_error(
                    self.previous(pointer).unwrap(),
                    TokenType::Expression,
                    Some(
                        "You may add a literal (number), string, variable, or a term here"
                            .to_string(),
                    ),
                )),
            }
        } else {
            Err(CodeError::missing_token_error(
                self.previous(pointer).unwrap(),
            ))
        }
    }

    fn parse_type(&self, pointer: &mut usize) -> CodeResult<ASTNode> {
        Ok(ASTNode::Type(self.consume(
            pointer,
            TokenType::Identifier,
            None,
        )?))
    }
}

#[derive(Debug)]
pub enum FunctionMode {
    Private,
    Export,
    Extern,
    Default,
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
    FunctionDef(
        &'a Token,
        FunctionMode,
        Box<ASTNode<'a>>,
        Vec<(&'a Token, Box<ASTNode<'a>>)>,
        Vec<Box<ASTNode<'a>>>,
    ),
    // Name, Expr, Type annotation (opt)
    VariableSet(&'a Token, Box<ASTNode<'a>>, Option<Box<ASTNode<'a>>>),
    // Lib name
    Import(&'a Token),
    // Name, Arguments (expr)
    FunctionCall(&'a Token, Vec<Box<ASTNode<'a>>>),
    // Expr
    Return(Box<ASTNode<'a>>),
}
