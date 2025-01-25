use crate::lexer::CodePosition;

pub enum CompilerError {
    UnexpectedChar(CodePosition),
    UnknownChar(CodePosition),
    FileNotAccessible(String, bool),
    FileCorrupted(String)
}

pub type CompResult<T> = Result<T, CompilerError>;