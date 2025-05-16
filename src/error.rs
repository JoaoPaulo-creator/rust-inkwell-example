use inkwell::builder::BuilderError;
use std::error::Error;
use std::fmt;

/// CompileError represents errors from different stages of compilation.
#[derive(Debug)]
pub enum CompileError {
    Io(String),
    Lex(String),
    Parse(String),
    Codegen(String),
}

impl From<BuilderError> for CompileError {
    fn from(e: BuilderError) -> Self {
        CompileError::Codegen(format!("LLVM builder error: {:?}", e))
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Io(msg) => write!(f, "IO error: {}", msg),
            CompileError::Lex(msg) => write!(f, "Lexical error: {}", msg),
            CompileError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CompileError::Codegen(msg) => write!(f, "Codegen error: {}", msg),
        }
    }
}

impl Error for CompileError {}
