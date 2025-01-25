use std::fmt;
use crate::codeviz::print_code_error;
use crate::filemanager::FileManager;
use crate::lexer::CodePosition;

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
}

#[derive(Debug)]
pub struct CodeError {
    pub position: CodePosition,
    pub code_error_type: CodeErrorType,
    pub title: String,
    pub footer: String,
    pub pointer: Option<String>,
}

impl CodeError {
    pub fn new(position: CodePosition, code_error_type: CodeErrorType, title: String, pointer: Option<String>, footer: String) -> Self {
        Self {position, code_error_type, title, footer, pointer }
    }

    pub fn new_unknown_char_error(position: CodePosition, c: char) -> Self {
        Self::new(position, CodeErrorType::LexerUnknownChar, "Unknown character".to_string(), 
                  Some("This one".to_string()), format!("Character '{}' is weird!", c))
    }

    pub fn new_unexpected_char_error(position: CodePosition, c: char) -> Self {
        Self::new(position, CodeErrorType::LexerUnexpectedChar, "Unexpected character".to_string(),
                  Some("This one".to_string()), format!("Character '{}' is invalid at this point!", c))
    }

    pub fn new_eof_error() -> Self {
        Self::new(CodePosition::eof(), CodeErrorType::LexerUnexpectedChar, "End of File".to_string(), None, "Premature end of file!".to_string())
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