use std::fs;
use std::path::PathBuf;
use annotate_snippets::Snippet;
use crate::comp_errors::{CompResult, CompilerError};
use crate::lexer::CodePosition;

pub fn pathbuf_to_string(p: PathBuf) -> String {
    p.into_os_string().into_string().expect("Failed to convert pathbuf to string").to_string()
}

pub fn full_path(p: &str) -> std::io::Result<PathBuf> {
    fs::canonicalize(PathBuf::from(p))
}

pub fn relative_path(p: &str) -> &str {
    // This *should* always work if compiler is accessing the nested files
    // Otherwise, we will return the full path
    p.strip_prefix(&std::env::current_dir().unwrap().to_str().unwrap().to_string()).or(Some(p)).expect("There is no reason")
}


pub struct FileManager {
    pub file_path: PathBuf,
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
    
    pub fn get_surrounding_slice(&self, line_index: usize) -> (String, usize) {
        let lines: Vec<&str> = self.content.lines().collect();
        let total_lines = lines.len();

        if total_lines == 0 {
            return (String::new(), 0);
        }

        let mut snippet = String::new();
        let mut offset = 0;

        if line_index > 0 {
            snippet.push_str(lines[line_index - 1]);
            snippet.push('\n');
            offset += lines[line_index - 1].len() + 1;
        }

        if line_index < total_lines {
            snippet.push_str(lines[line_index]);
            snippet.push('\n');
        }

        if line_index + 1 < total_lines {
            snippet.push_str(lines[line_index + 1]);
            snippet.push('\n');
        }

        (snippet, offset)
    }


    pub fn get_code_snippet(&self, code_position: &CodePosition) -> (Snippet, usize) {
        // TODO: Remove this super evil magic trick
        let sor_slc = self.get_surrounding_slice(code_position.line_start);
        // There is some weird stuff going on here
        let clean_path = &self.file_path.to_str().unwrap()[4..];
        (Snippet::source(sor_slc.0.leak())
            .line_start(if code_position.line_start == 0 {code_position.line_start+1} else {code_position.line_start})
            .origin(relative_path(clean_path).to_string().leak()), sor_slc.1)
    }
}