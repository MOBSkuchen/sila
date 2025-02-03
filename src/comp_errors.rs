use std::fmt;
use crate::codeviz::print_code_error;
use crate::filemanager::FileManager;
use crate::lexer::{CodePosition, Token, TokenType};

#[derive(Debug)]
pub enum CompilerError {
    FileNotAccessible(String, bool),
    FileCorrupted(String)
}

#[derive(Debug)]
pub enum CodeErrorType {
    LexerUnknownChar,
    LexerUnexpectedChar,
    LexerEndOfFile,
    ParserUnexpectedToken,
    MissingTokenError
}

#[derive(Debug)]
pub enum CodeWarningType {
    DeadCode,
    UnnecessaryCode,
    DiscouragedPractice
}

#[derive(Debug)]
pub struct CodeError {
    pub position: CodePosition,
    pub code_error_type: CodeErrorType,
    pub title: String,
    pub footer: String,
    pub pointer: Option<String>,
    pub notes: Vec<String>,
}

impl CodeError {
    pub fn new(position: CodePosition, code_error_type: CodeErrorType, title: String, pointer: Option<String>, footer: String, notes: Vec<String>) -> Self {
        Self {position, code_error_type, title, footer, pointer, notes }
    }
    
    pub fn placeholder() -> Self {
        panic!("Please remove this placeholder!");
    }

    pub fn new_unexpected_token_error(token: &Token, expected: TokenType, extra: Option<String>) -> Self {
        Self::new(token.code_position, CodeErrorType::ParserUnexpectedToken, "Unexpected Token".to_string(),
                  Some(format!("Should be followed by `{}`", expected)), 
                  format!("Expected another token `{}`, but got `{}`", expected, token.token_type),
                  if extra.is_some() {vec![extra.unwrap()]} else {vec![]})
    }

    pub fn new_unknown_char_error(position: CodePosition, c: char) -> Self {
        Self::new(position, CodeErrorType::LexerUnknownChar, "Unknown character".to_string(), 
                  Some("This one".to_string()), format!("Character `{}` is weird!", c), vec![])
    }

    pub fn new_eof_error() -> Self {
        Self::new(CodePosition::eof(), CodeErrorType::LexerEndOfFile, "End of File".to_string(), None, "Premature end of file!".to_string(), vec![])
    }

    pub fn missing_token_error(last_token: &Token) -> Self {
        Self::new(last_token.code_position, CodeErrorType::MissingTokenError, "Missing token".to_string(), Some("After this".to_string()),
                  "Premature end of file!".to_string(), vec![])
    }
    
    pub fn visualize_error(self, file_manager: &FileManager) {
        print_code_error(self, file_manager)
    }
}

pub type CompResult<T> = Result<T, CompilerError>;
pub type CodeResult<T> = Result<T, CodeError>;

impl fmt::Display for CodeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct CodeWarning {
    pub position: CodePosition,
    pub code_warn_type: CodeWarningType,
    pub title: String,
    pub footer: String,
    pub pointer: Option<String>,
    pub notes: Vec<String>
}

impl CodeWarning {
    pub fn new(position: CodePosition, code_warn_type: CodeWarningType, title: String, footer: String, pointer: Option<String>, notes: Vec<String>) -> Self {
        Self {position, code_warn_type, title, footer, pointer, notes }
    }
    
    pub fn new_unnecessary_code(position: CodePosition, extra: Option<String>) -> Self {
        Self::new(position, CodeWarningType::UnnecessaryCode, "Unnecessary code".to_string(),
                  "This code does not change the outcome".to_string(), None, if extra.is_some() {vec![extra.unwrap()]} else {vec![]})
    }
}