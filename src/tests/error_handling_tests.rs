use crate::lexer::{Lexer, LexError};
use crate::parser::{Parser, ParseError};
use crate::types::{TypeChecker, TypeError};
use crate::interpreter::{Interpreter, RuntimeError};
use super::test_utilities::*;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    // Helper function to expect lexer error
    fn expect_lex_error(source: &str) -> LexError {
        let mut lexer = Lexer::new(source);
        lexer.tokenize().expect_err("Expected lexer error")
    }

    // Helper function to expect parser error
    fn expect_parse_error(source: &str) -> ParseError {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().expect_err("Expected parser error")
    }

    // Helper function to expect type error
    fn expect_type_error(source: &str) -> TypeError {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast).expect_err("Expected type error")
    }

    // Helper function to expect runtime error
    fn expect_runtime_error(source: &str) -> RuntimeError {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast).unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&ast).expect_err("Expected runtime error")
    }

    #[test]
    fn test_lexer_error_invalid_characters() {
        // Invalid character
        let error = expect_lex_error("let x = 5@;");
        assert!(matches!(error, LexError::InvalidCharacter { .. }));
        
        // Invalid escape sequence
        let error = expect_lex_error("\"invalid \\q escape\"");
        assert!(matches!(error, LexError::InvalidEscapeSequence { .. }));
        
        // Unterminated string
        let error = expect_lex_error("\"unterminated string");
        assert!(matches!(error, LexError::UnterminatedString { .. }));
        
        // Unterminated comment
        let error = expect_lex_error("/* unterminated comment");
        assert!(matches!(error, LexError::UnterminatedComment { .. }));
    }

    #[test]
    fn test_lexer_error_malformed_numbers() {
        // Multiple decimal points
        let error = expect_lex_error("let x = 3.14.15;");
        assert!(matches!(error, LexError::MalformedNumber { .. }));
        
        // Invalid number format
        let error = expect_lex_error("let x = 3.;");
        assert!(matches!(error, LexError::MalformedNumber { .. }));
        
        // Number too large
        let error = expect_lex_error("let x = 999999999999999999999999999999999999999999;");
        assert!(matches!(error, LexError::NumberTooLarge { .. }));
    }

    #[test]
    fn test_lexer_error_positions() {
        let source = "let x = 5;\nlet y = @invalid;";
        let error = expect_lex_error(source);
        
        if let LexError::InvalidCharacter { line, column, .. } = error {
            assert_eq!(line, 2);
            assert_eq!(column, 9); // Position of '@'
        } else {
            panic!("Expected InvalidCharacter error with position info");
        }
    }

    #[test]
    fn test_parser_error_unexpected_tokens() {
        // Missing semicolon
        let error = expect_parse_error("let x = 5 let y = 3;");
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
        
        // Missing closing brace
        let error = expect_parse_error("fn test() { let x = 5;");
        assert!(matches!(error, ParseError::UnexpectedEndOfInput { .. }));
        
        // Invalid expression
        let error = expect_parse_error("let x = + 5;");
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn test_parser_error_malformed_structures() {
        // Invalid function definition
        let error = expect_parse_error("fn (x: int) -> int { x }");
        assert!(matches!(error, ParseError::ExpectedIdentifier { .. }));
        
        // Invalid struct definition
        let error = expect_parse_error("struct { x: int }");
        assert!(matches!(error, ParseError::ExpectedIdentifier { .. }));
        
        // Invalid if statement
        let error = expect_parse_error("if { true } else { false }");
        assert!(matches!(error, ParseError::ExpectedExpression { .. }));
    }

    #[test]
    fn test_parser_error_mismatched_brackets() {
        // Mismatched parentheses
        let error = expect_parse_error("let x = (5 + 3];");
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
        
        // Mismatched braces
        let error = expect_parse_error("fn test() [ let x = 5; }");
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
        
        // Missing closing bracket
        let error = expect_parse_error("let arr = [1, 2, 3;");
        assert!(matches!(error, ParseError::UnexpectedEndOfInput { .. }));
    }

    #[test]
    fn test_type_error_mismatched_types() {
        // Integer + String
        let error = expect_type_error("let x = 5 + \"hello\";");
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
        
        // Boolean in arithmetic
        let error = expect_type_error("let x = true * 5;");
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
        
        // Wrong function argument type
        let error = expect_type_error(r#"
            fn add(x: int, y: int) -> int { x + y }
            add(5.0, 3)
        "#);
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn test_type_error_undefined_variables() {
        // Undefined variable
        let error = expect_type_error("let x = undefined_var + 5;");
        assert!(matches!(error, TypeError::UndefinedVariable { .. }));
        
        // Undefined function
        let error = expect_type_error("let x = undefined_func(5);");
        assert!(matches!(error, TypeError::UndefinedFunction { .. }));
        
        // Undefined struct field
        let error = expect_type_error(r#"
            struct Point { x: int, y: int }
            let p = Point { x: 1, y: 2 };
            p.z
        "#);
        assert!(matches!(error, TypeError::UndefinedField { .. }));
    }

    #[test]
    fn test_type_error_function_signatures() {
        // Wrong number of arguments
        let error = expect_type_error(r#"
            fn add(x: int, y: int) -> int { x + y }
            add(5)
        "#);
        assert!(matches!(error, TypeError::WrongArgumentCount { .. }));
        
        // Return type mismatch
        let error = expect_type_error(r#"
            fn get_number() -> int {
                "not a number"
            }
        "#);
        assert!(matches!(error, TypeError::ReturnTypeMismatch { .. }));
        
        // Recursive type inference failure
        let error = expect_type_error(r#"
            fn recursive_error(x) {
                recursive_error(x)
            }
        "#);
        assert!(matches!(error, TypeError::CannotInferType { .. }));
    }

    #[test]
    fn test_type_error_array_operations() {
        // Index with non-integer
        let error = expect_type_error(r#"
            let arr = [1, 2, 3];
            arr["invalid"]
        "#);
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
        
        // Index non-array
        let error = expect_type_error(r#"
            let x = 42;
            x[0]
        "#);
        assert!(matches!(error, TypeError::NotIndexable { .. }));
        
        // Inconsistent array element types
        let error = expect_type_error("let arr = [1, \"two\", 3.0];");
        assert!(matches!(error, TypeError::InconsistentTypes { .. }));
    }

    #[test]
    fn test_type_error_control_flow() {
        // Non-boolean condition in if
        let error = expect_type_error(r#"
            if 42 {
                "true branch"
            } else {
                "false branch"
            }
        "#);
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
        
        // Different types in if-else branches
        let error = expect_type_error(r#"
            if true {
                42
            } else {
                "string"
            }
        "#);
        assert!(matches!(error, TypeError::BranchTypeMismatch { .. }));
        
        // Non-boolean condition in while
        let error = expect_type_error(r#"
            while "not boolean" {
                break;
            }
        "#);
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn test_runtime_error_division_by_zero() {
        // Integer division by zero
        let error = expect_runtime_error("let x = 10 / 0;");
        assert!(matches!(error, RuntimeError::DivisionByZero { .. }));
        
        // Float division by zero
        let error = expect_runtime_error("let x = 10.0 / 0.0;");
        assert!(matches!(error, RuntimeError::DivisionByZero { .. }));
        
        // Modulo by zero
        let error = expect_runtime_error("let x = 10 % 0;");
        assert!(matches!(error, RuntimeError::DivisionByZero { .. }));
    }

    #[test]
    fn test_runtime_error_array_bounds() {
        // Index out of bounds
        let error = expect_runtime_error(r#"
            let arr = [1, 2, 3];
            arr[5]
        "#);
        assert!(matches!(error, RuntimeError::IndexOutOfBounds { .. }));
        
        // Negative index
        let error = expect_runtime_error(r#"
            let arr = [1, 2, 3];
            arr[-1]
        "#);
        assert!(matches!(error, RuntimeError::IndexOutOfBounds { .. }));
        
        // Empty array access
        let error = expect_runtime_error(r#"
            let arr = [];
            arr[0]
        "#);
        assert!(matches!(error, RuntimeError::IndexOutOfBounds { .. }));
    }

    #[test]
    fn test_runtime_error_null_pointer() {
        // Accessing field of null
        let error = expect_runtime_error(r#"
            enum Option<T> {
                Some(T),
                None
            }
            
            let opt = Option::None;
            match opt {
                Option::Some(value) => value.field, // This shouldn't happen
                Option::None => panic("null access")
            }
        "#);
        assert!(matches!(error, RuntimeError::NullPointerAccess { .. }));
    }

    #[test]
    fn test_runtime_error_stack_overflow() {
        // Infinite recursion
        let error = expect_runtime_error(r#"
            fn infinite_recursion() -> int {
                infinite_recursion()
            }
            infinite_recursion()
        "#);
        assert!(matches!(error, RuntimeError::StackOverflow { .. }));
        
        // Mutual infinite recursion
        let error = expect_runtime_error(r#"
            fn func_a() -> int { func_b() }
            fn func_b() -> int { func_a() }
            func_a()
        "#);
        assert!(matches!(error, RuntimeError::StackOverflow { .. }));
    }

    #[test]
    fn test_runtime_error_type_cast_failure() {
        // Invalid type cast
        let error = expect_runtime_error(r#"
            let x = "not a number";
            x as int
        "#);
        assert!(matches!(error, RuntimeError::InvalidCast { .. }));
        
        // Overflow in cast
        let error = expect_runtime_error(r#"
            let x = 99999999999999999999.0;
            x as int
        "#);
        assert!(matches!(error, RuntimeError::CastOverflow { .. }));
    }

    #[test]
    fn test_error_recovery_in_parsing() {
        // Parser should be able to continue after certain errors
        let source = r#"
            let x = 5
            let y = 3; // Missing semicolon above, but this should still parse
            y
        "#;
        
        // This test depends on error recovery implementation
        // For now, we just check that the error is detected
        let error = expect_parse_error(source);
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn test_error_cascading_prevention() {
        // Type checker should not generate cascading errors
        let source = r#"
            let undefined_var = some_undefined_function();
            let x = undefined_var + 5; // Should not generate additional error
            let y = x * 2; // Should not generate additional error
        "#;
        
        let error = expect_type_error(source);
        // Should get one primary error, not multiple cascading errors
        assert!(matches!(error, TypeError::UndefinedFunction { .. }));
    }

    #[test]
    fn test_error_message_quality() {
        // Test that error messages contain helpful information
        
        // Lexer error with position
        let error = expect_lex_error("let x = 5@;");
        let error_msg = format!("{:?}", error);
        assert!(error_msg.contains("line") || error_msg.contains("position"));
        
        // Parser error with context
        let error = expect_parse_error("fn test() { let x = + 5; }");
        let error_msg = format!("{:?}", error);
        assert!(error_msg.contains("unexpected") || error_msg.contains("expected"));
        
        // Type error with type information
        let error = expect_type_error("let x = 5 + \"hello\";");
        let error_msg = format!("{:?}", error);
        assert!(error_msg.contains("int") || error_msg.contains("string") || error_msg.contains("type"));
    }

    #[test]
    fn test_nested_error_contexts() {
        // Errors in nested contexts should provide full context
        let error = expect_type_error(r#"
            fn outer() -> int {
                fn inner() -> string {
                    42 // Wrong return type
                }
                inner()
            }
        "#);
        
        let error_msg = format!("{:?}", error);
        assert!(error_msg.contains("inner") || error_msg.contains("return"));
    }

    #[test]
    fn test_macro_expansion_errors() {
        // If macros are supported, test macro expansion errors
        let error = expect_parse_error(r#"
            macro_rules! bad_macro {
                ($x:expr) => { $x + }; // Invalid macro body
            }
            bad_macro!(5)
        "#);
        
        // This test depends on macro implementation
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn test_unicode_error_handling() {
        // Unicode characters in error contexts
        let error = expect_type_error(r#"
            let 变量 = 5;
            let résult = 变量 + "文字列";
        "#);
        
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
        
        // Error messages should handle unicode properly
        let error_msg = format!("{:?}", error);
        assert!(!error_msg.is_empty());
    }

    #[test]
    fn test_very_long_error_contexts() {
        // Test error handling with very long identifiers/expressions
        let long_var_name = "very_very_very_long_variable_name_".repeat(10);
        let source = format!("let x = {} + \"string\";", long_var_name);
        
        let error = expect_type_error(&source);
        assert!(matches!(error, TypeError::UndefinedVariable { .. } | TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn test_error_in_complex_expressions() {
        // Error in deeply nested expression
        let error = expect_type_error(r#"
            let result = ((((5 + 3) * 2) - 1) / (true + 2)); // Boolean in arithmetic
        "#);
        
        assert!(matches!(error, TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn test_multiple_errors_in_sequence() {
        // Multiple independent errors (if supported by implementation)
        let sources = vec![
            "let x = 5 + \"hello\";",
            "let y = undefined_var;",
            "fn test() -> int { \"string\" }",
        ];
        
        for source in sources {
            // Each should produce an error
            let error = expect_type_error(source);
            assert!(matches!(error, TypeError::TypeMismatch { .. } | TypeError::UndefinedVariable { .. } | TypeError::ReturnTypeMismatch { .. }));
        }
    }

    #[test]
    fn test_error_recovery_strategies() {
        // Test different error recovery strategies
        
        // Panic mode recovery
        let error = expect_parse_error("let x = ; let y = 5;");
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
        
        // Phrase level recovery
        let error = expect_parse_error("fn test( -> int { 5 }"); // Missing parameter
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
        
        // Error productions
        let error = expect_parse_error("if (true { 5 } else { 3 }"); // Missing closing parenthesis
        assert!(matches!(error, ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn test_semantic_error_detection() {
        // Semantic errors that require analysis beyond syntax
        
        // Using variable before declaration (in same scope)
        let error = expect_type_error(r#"
            let x = y;
            let y = 5;
        "#);
        assert!(matches!(error, TypeError::UndefinedVariable { .. }));
        
        // Break outside loop
        let error = expect_type_error(r#"
            fn test() {
                break;
            }
        "#);
        assert!(matches!(error, TypeError::BreakOutsideLoop { .. }));
        
        // Return outside function
        let error = expect_type_error("return 5;");
        assert!(matches!(error, TypeError::ReturnOutsideFunction { .. }));
    }

    #[test]
    fn test_error_source_tracking() {
        // Test that errors maintain source location information
        let source = r#"
            fn test() {
                let x = 5;
                let y = x + "string"; // Error on this line
                y
            }
        "#;
        
        let error = expect_type_error(source);
        if let TypeError::TypeMismatch { line, column, .. } = error {
            assert_eq!(line, 4); // Line with the error
            assert!(column > 0); // Should have column information
        } else {
            // If the error variant doesn't include position, that's also valid
            // Just ensure we get the expected error type
            assert!(matches!(error, TypeError::TypeMismatch { .. }));
        }
    }
}
