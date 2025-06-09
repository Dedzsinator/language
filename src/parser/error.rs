use crate::ast::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error(
        "Unexpected token: expected {expected}, found {found} at line {line}, column {column}"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    fn create_test_span() -> Span {
        Span::new(0, 10, 1, 5)
    }

    #[test]
    fn test_unexpected_token_error() {
        let span = create_test_span();
        let error = ParseError::unexpected_token("identifier", "number", &span);

        match error {
            ParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => {
                assert_eq!(expected, "identifier");
                assert_eq!(found, "number");
                assert_eq!(line, 1);
                assert_eq!(column, 5);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }

    #[test]
    fn test_unexpected_eof_error() {
        let span = create_test_span();
        let error = ParseError::unexpected_eof(&span);

        match error {
            ParseError::UnexpectedEof { line, column } => {
                assert_eq!(line, 1);
                assert_eq!(column, 5);
            }
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_invalid_syntax_error() {
        let span = create_test_span();
        let error = ParseError::invalid_syntax("Missing semicolon", &span);

        match error {
            ParseError::InvalidSyntax {
                message,
                line,
                column,
            } => {
                assert_eq!(message, "Missing semicolon");
                assert_eq!(line, 1);
                assert_eq!(column, 5);
            }
            _ => panic!("Expected InvalidSyntax error"),
        }
    }

    #[test]
    fn test_lexical_error() {
        let error = ParseError::lexical_error("Invalid character");

        match error {
            ParseError::LexicalError { message } => {
                assert_eq!(message, "Invalid character");
            }
            _ => panic!("Expected LexicalError"),
        }
    }

    #[test]
    fn test_type_annotation_required_error() {
        let span = create_test_span();
        let error = ParseError::type_annotation_required("variable x", &span);

        match error {
            ParseError::TypeAnnotationRequired { item, line, column } => {
                assert_eq!(item, "variable x");
                assert_eq!(line, 1);
                assert_eq!(column, 5);
            }
            _ => panic!("Expected TypeAnnotationRequired error"),
        }
    }

    #[test]
    fn test_error_display() {
        let span = create_test_span();
        let error = ParseError::unexpected_token("identifier", "number", &span);
        let display_string = format!("{}", error);

        assert!(display_string.contains("Unexpected token"));
        assert!(display_string.contains("expected identifier"));
        assert!(display_string.contains("found number"));
        assert!(display_string.contains("line 1"));
        assert!(display_string.contains("column 5"));
    }

    #[test]
    fn test_error_equality() {
        let span = create_test_span();
        let error1 = ParseError::unexpected_token("identifier", "number", &span);
        let error2 = ParseError::unexpected_token("identifier", "number", &span);
        let error3 = ParseError::unexpected_token("string", "number", &span);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_cloning() {
        let span = create_test_span();
        let error = ParseError::invalid_syntax("Test message", &span);
        let cloned_error = error.clone();

        assert_eq!(error, cloned_error);
    }

    #[test]
    fn test_parse_result_type() {
        // Test that ParseResult is a proper type alias
        let success: ParseResult<i32> = Ok(42);
        let failure: ParseResult<i32> = Err(ParseError::lexical_error("test error"));

        assert!(success.is_ok());
        assert!(failure.is_err());

        if let Ok(value) = success {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn test_all_error_variants() {
        let span = create_test_span();

        // Test all error variants can be created
        let errors = vec![
            ParseError::unexpected_token("a", "b", &span),
            ParseError::unexpected_eof(&span),
            ParseError::invalid_syntax("test", &span),
            ParseError::lexical_error("test"),
            ParseError::type_annotation_required("test", &span),
        ];

        assert_eq!(errors.len(), 5);

        // Ensure each error has the correct type
        for error in errors {
            match error {
                ParseError::UnexpectedToken { .. }
                | ParseError::UnexpectedEof { .. }
                | ParseError::InvalidSyntax { .. }
                | ParseError::LexicalError { .. }
                | ParseError::TypeAnnotationRequired { .. } => {
                    // All variants accounted for
                }
            }
        }
    }
}
