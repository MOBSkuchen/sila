use std::fs;
use std::path::PathBuf;
use crate::comp_errors::{CompResult, CompilerError};

pub fn pathbuf_to_string(p: PathBuf) -> String {
    p.into_os_string().into_string().expect("Failed to convert pathbuf to string").to_string()
}

pub fn full_path(p: &str) -> std::io::Result<PathBuf> {
    fs::canonicalize(PathBuf::from(p))
}


pub struct FileManager {
    file_path: PathBuf,
    content: String
}

impl FileManager {
    pub fn new(file_path: PathBuf) -> CompResult<Self> {
        if !file_path.exists() {
            Err(CompilerError::FileNotAccessible(pathbuf_to_string((&file_path).to_owned()),
                                             !file_path.parent().is_some_and(|t| {t.exists()})))
        } else {
            let content = fs::read_to_string(&file_path);
            if content.is_err() {
                Err(CompilerError::FileCorrupted(pathbuf_to_string(file_path)))
            } else {
                Ok(Self { file_path, content: content.unwrap() })
            }
        }
    }
    
    pub fn new_from(file: String) -> CompResult<Self> {
        let x = full_path(&file);
        if x.is_err() {
            Err(CompilerError::FileNotAccessible(file, true))
        } else {
            Self::new(x.unwrap())
        }
    }
    
    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}