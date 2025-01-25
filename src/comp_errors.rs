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
    pub details: String
}

impl CodeError {
    pub fn new(position: CodePosition, code_error_type: CodeErrorType, details: String) -> Self {
        Self {position, code_error_type, details}
    }
    
    pub fn new_unknown_char_error(position: CodePosition, c: char) -> Self {
        Self::new(position, CodeErrorType::LexerUnexpectedChar, format!("Character '{}' is weird!", c))
    }

    pub fn new_unexpected_char_error(position: CodePosition, c: char) -> Self {
        Self::new(position, CodeErrorType::LexerUnexpectedChar, format!("Character '{}' is invalid at this point!", c))
    }

    pub fn new_eof_error() -> Self {
        Self::new(CodePosition::eof(), CodeErrorType::LexerUnexpectedChar, "Premature end of file!".to_string())
    }
}

pub type CompResult<T> = Result<T, CompilerError>;
pub type CodeResult<T> = Result<T, CodeError>;