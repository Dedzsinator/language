use crate::ast::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at line {line}, column {column}")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    
    #[error("Unexpected end of file at line {line}, column {column}")]
    UnexpectedEof { line: usize, column: usize },
    
    #[error("Invalid syntax: {message} at line {line}, column {column}")]
    InvalidSyntax {
        message: String,
        line: usize,
        column: usize,
    },
    
    #[error("Lexical error: {message}")]
    LexicalError { message: String },
    
    #[error("Type annotation required for {item} at line {line}, column {column}")]
    TypeAnnotationRequired {
        item: String,
        line: usize,
        column: usize,
    },
}

impl ParseError {
    pub fn unexpected_token(expected: &str, found: &str, span: &Span) -> Self {
        Self::UnexpectedToken {
            expected: expected.to_string(),
            found: found.to_string(),
            line: span.line,
            column: span.column,
        }
    }
    
    pub fn unexpected_eof(span: &Span) -> Self {
        Self::UnexpectedEof {
            line: span.line,
            column: span.column,
        }
    }
    
    pub fn invalid_syntax(message: &str, span: &Span) -> Self {
        Self::InvalidSyntax {
            message: message.to_string(),
            line: span.line,
            column: span.column,
        }
    }
    
    pub fn lexical_error(message: &str) -> Self {
        Self::LexicalError {
            message: message.to_string(),
        }
    }
    
    pub fn type_annotation_required(item: &str, span: &Span) -> Self {
        Self::TypeAnnotationRequired {
            item: item.to_string(),
            line: span.line,
            column: span.column,
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
