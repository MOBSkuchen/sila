use std::ops::Range;
use crate::comp_errors::{CodeError, CodeResult};

#[derive(Debug)]
pub enum TokenType {
    // Keywords
    Define,
    Export,
    Import,
    Extern,
    Mut,

    Identifier,

    String,
    NumberInt,
    NumberFloat,
    
    LParen,
    RParen,
    Comma,
    Dot,
    Plus,
    Minus,
    Slash,
    Star,
    Colon,
    SemiColon,
    Greater,
    Lesser,
    Pipe,
    And,
    Exclamation,
    Equals,
    DoubleEquals,
    NotEquals,
    GreaterEquals,
    LesserEquals
}

#[derive(Debug)]
pub struct CodePosition {
    pub idx: usize,
    pub line: usize,
    pub line_idx_start: usize,
    pub line_idx_end: usize,
}

impl CodePosition {
    pub fn one_char(idx: usize, line: usize, line_idx: usize) -> Self {
        CodePosition {idx, line, line_idx_start: line_idx, line_idx_end: line_idx + 1}
    }
    
    pub fn eof() -> Self {
        CodePosition {idx: 0, line: 0, line_idx_start: 0, line_idx_end: 0}
    }
    
    pub fn is_eof(&self) -> bool {
        [self.idx, self.line, self.line_idx_start, self.line_idx_end].iter().all(|t| {*t==0})
    }
}

impl CodePosition {
    pub fn range(&self) -> Range<usize> {
        self.line_idx_start..self.line_idx_end
    }
}

#[derive(Debug)]
pub struct Token {
    pub content: String,
    pub token_type: TokenType,
    pub code_position: CodePosition
}

impl Token {
    pub fn from_one(idx: usize, line: usize, line_idx: usize, content: char, token_type: TokenType) -> Self {
        Self {content: content.to_string(), token_type, code_position: CodePosition::one_char(idx, line, line_idx)}
    }
}

pub struct Scanner {
    pub cursor: usize,
    pub line: usize,
    pub line_idx: usize,
    pub characters: Vec<char>,
}

impl Scanner {
    pub fn new(string: &str) -> Self {
        Self {
            cursor: 0,
            line: 0,
            line_idx: 0,
            characters: string.chars().collect(),
        }
    }
    
    /// Returns the next character without advancing the cursor.
    /// AKA "lookahead"
    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    /// Returns true if further progress is not possible.
    pub fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    /// Returns the next character (if available) and advances the cursor.
    pub fn pop(&mut self) -> Option<&char> {
        match self.characters.get(self.cursor) {
            Some(character) => {
                self.cursor += 1;
                if *character == '\n' {
                    self.line += 1;
                    self.line_idx = 0;
                }

                Some(character)
            }
            None => None,
        }
    }
    
    pub fn current(&self) -> Option<&char> {
        match self.characters.get(self.cursor) {
            Some(character) => { Some(character) }
            None => None,
        }
    }
    
    pub fn previous(&self) -> Option<&char> {
        match self.characters.get(self.cursor - 1) {
            Some(character) => { Some(character) }
            None => None,
        }
    }
    
    pub fn this_as_token(&self, token_type: TokenType) -> Option<Token> {
        let c = self.previous();
        if c.is_none() {
            None
        } else {
            Some(Token::from_one(self.cursor, self.line, self.line_idx, *c.unwrap(), token_type))
        }
    }

    pub fn this_as_codepos(&self) -> Option<CodePosition> {
        if self.is_done() {
            None
        } else {
            Some(CodePosition::one_char(self.cursor, self.line, self.line_idx))
        }
    }
    
    pub fn this_as_codepos2(&self) -> CodePosition {
        self.this_as_codepos().expect("This should not happen -> constructing code pos")
    }
}

fn tokenizer(scanner: &mut Scanner) -> CodeResult<Option<Token>> {
    while let Some(current) = scanner.peek() {
        match current {
            ' ' | '\t' | '\n' | '\r' => {
                scanner.pop();
            }

            '#' => {
                if let Some('#') = scanner.peek() {
                    // Multi-line comment
                    scanner.pop(); // Consume the second '#'
                    while let Some(c) = scanner.pop() {
                        if *c == '#' && scanner.peek().is_some_and(|t| *t == '#') {
                            scanner.pop(); // Consume the closing '#'
                            break;
                        }
                    }
                } else {
                    // Single-line comment
                    while let Some(c) = scanner.pop() {
                        if *c == '\n' {
                            break;
                        }
                    }
                }
            }

            '(' | ')' | ',' | '.' | '+' | '-' | '/' | '*' | ':' | ';' => {
                let token_type = match current {
                    '(' => TokenType::LParen,
                    ')' => TokenType::RParen,
                    ',' => TokenType::Comma,
                    '.' => TokenType::Dot,
                    '+' => TokenType::Plus,
                    '-' => TokenType::Minus,
                    '/' => TokenType::Slash,
                    '*' => TokenType::Star,
                    '|' => TokenType::Pipe,
                    '&' => TokenType::And,
                    ':' => TokenType::Colon,
                    ';' => TokenType::SemiColon,
                    _ => unreachable!(),
                };
                scanner.pop();
                return Ok(scanner.this_as_token(token_type));
            }
            '>' => {
                scanner.pop();
                if let Some('=') = scanner.peek() {
                    scanner.pop();
                    return Ok(scanner.this_as_token(TokenType::GreaterEquals));
                }
                return Ok(scanner.this_as_token(TokenType::Greater));
            }
            '<' => {
                scanner.pop();
                if let Some('=') = scanner.peek() {
                    scanner.pop();
                    return Ok(scanner.this_as_token(TokenType::LesserEquals));
                }
                return Ok(scanner.this_as_token(TokenType::Lesser));
            }
            '!' => {
                scanner.pop();
                if let Some('=') = scanner.peek() {
                    scanner.pop();
                    return Ok(scanner.this_as_token(TokenType::NotEquals));
                }
                return Ok(scanner.this_as_token(TokenType::Exclamation));
            }
            '=' => {
                scanner.pop();
                if let Some('=') = scanner.peek() {
                    scanner.pop();
                    return Ok(scanner.this_as_token(TokenType::DoubleEquals));
                }
                return Ok(scanner.this_as_token(TokenType::Equals));
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || *c == '_' => {
                let start_pos = scanner.cursor;
                while let Some(next) = scanner.peek() {
                    if next.is_alphanumeric() || *next == '_' {
                        scanner.pop();
                    } else {
                        break;
                    }
                }
                let identifier: String = scanner.characters[start_pos..scanner.cursor].iter().collect();
                let token_type = match identifier.as_str() {
                    "def" => TokenType::Define,
                    "export" => TokenType::Export,
                    "import" => TokenType::Import,
                    "extern" => TokenType::Extern,
                    "mut" => TokenType::Mut,
                    _ => TokenType::Identifier,
                };
                return Ok(Some(Token {
                    content: identifier.clone(),
                    token_type,
                    code_position: CodePosition {
                        idx: start_pos,
                        line: scanner.line,
                        line_idx_start: scanner.line_idx,
                        line_idx_end: scanner.line_idx + identifier.len(),
                    },
                }));
            }

            // Numbers
            c if c.is_digit(10) => {
                let start_pos = scanner.cursor;
                let mut is_float = false;
                while let Some(next) = scanner.peek() {
                    if next.is_digit(10) {
                        scanner.pop();
                    } else if *next == '.' && !is_float {
                        is_float = true;
                        scanner.pop();
                    } else {
                        break;
                    }
                }
                let number: String = scanner.characters[start_pos..scanner.cursor].iter().collect();
                let token_type = if is_float {
                    TokenType::NumberFloat
                } else {
                    TokenType::NumberInt
                };
                return Ok(Some(Token {
                    content: number.clone(),
                    token_type,
                    code_position: CodePosition {
                        idx: start_pos,
                        line: scanner.line,
                        line_idx_start: scanner.line_idx,
                        line_idx_end: scanner.line_idx + number.len(),
                    },
                }));
            }

            // Strings
            '"' => {
                scanner.pop(); // Consume opening quote
                let start_pos = scanner.cursor;
                while let Some(next) = scanner.peek() {
                    if *next == '"' {
                        let string: String = scanner.characters[start_pos..scanner.cursor].iter().collect();
                        scanner.pop(); // Consume closing quote
                        return Ok(Some(Token {
                            content: string.clone(),
                            token_type: TokenType::String,
                            code_position: CodePosition {
                                idx: start_pos,
                                line: scanner.line,
                                line_idx_start: scanner.line_idx,
                                line_idx_end: scanner.line_idx + string.len(),
                            },
                        }));
                    } else {
                        scanner.pop();
                    }
                }
                return Err(CodeError::new_eof_error());
            }
            _ => {
                return Err(CodeError::new_unknown_char_error(scanner.this_as_codepos2(), *current));
            }
        }
    }
    Ok(None)
}

pub fn tokenize(content: String) -> CodeResult<Vec<Token>> {
    let mut scanner = Scanner::new(content.as_str());
    let mut tokens: Vec<Token> = vec![];
    loop {
        let token = tokenizer(&mut scanner)?;
        if token.is_some() {
            tokens.push(token.unwrap())
        } else {
            return Ok(tokens)
        }
    }
}