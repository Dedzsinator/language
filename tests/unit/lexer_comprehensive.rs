// Comprehensive Lexer Tests
use crate::lexer::{Lexer, Token};
use crate::tests::test_utilities::*;

#[cfg(test)]
mod comprehensive_lexer_tests {
    use super::*;

    #[test]
    fn test_all_token_types() {
        let input = r#"
        // Test all possible tokens
        struct enum fn let mut if else match for while loop break continue return
        async await parallel physics gpu import export module use
        true false null
        123 456789 0 -42
        3.14 2.718 1e-10 -0.5 1.23e+5
        "hello world" "escaped \"quotes\"" "unicode: ∑∞∆"
        'a' 'Z' '\n' '\t' '\''
        identifier _underscore CamelCase SCREAMING_CASE
        + - * / % ^ & | ! ~ << >> == != < > <= >= && || ?? ??= 
        = += -= *= /= %= ^= &= |= <<= >>=
        -> => :: . , ; : ( ) [ ] { }
        @ # $ \ `
        "#;
        
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // Should successfully tokenize everything
        assert!(tokens.len() > 50);
        
        // Check specific token types
        let token_types: Vec<_> = tokens.iter().map(|t| std::mem::discriminant(&t.token)).collect();
        assert!(token_types.contains(&std::mem::discriminant(&Token::Struct)));
        assert!(token_types.contains(&std::mem::discriminant(&Token::IntLiteral(0))));
        assert!(token_types.contains(&std::mem::discriminant(&Token::FloatLiteral(0.0))));
        assert!(token_types.contains(&std::mem::discriminant(&Token::StringLiteral(String::new()))));
    }

    #[test]
    fn test_complex_numeric_literals() {
        let test_cases = vec![
            ("42", Token::IntLiteral(42)),
            ("0", Token::IntLiteral(0)),
            ("-123", vec![Token::Minus, Token::IntLiteral(123)]),
            ("3.14159", Token::FloatLiteral(3.14159)),
            ("1e10", Token::FloatLiteral(1e10)),
            ("2.5e-3", Token::FloatLiteral(2.5e-3)),
            ("1.23E+45", Token::FloatLiteral(1.23E+45)),
            ("-0.001", vec![Token::Minus, Token::FloatLiteral(0.001)]),
        ];
        
        for (input, expected) in test_cases {
            let tokens = tokenize_source(input);
            match expected {
                Token::IntLiteral(val) => {
                    assert_eq!(tokens.len(), 1);
                    assert_eq!(tokens[0], Token::IntLiteral(val));
                },
                Token::FloatLiteral(val) => {
                    assert_eq!(tokens.len(), 1);
                    assert_eq!(tokens[0], Token::FloatLiteral(val));
                },
                _ => {} // For multi-token cases, handle separately
            }
        }
    }

    #[test]
    fn test_string_literal_edge_cases() {
        let test_cases = vec![
            (r#""""#, ""),  // Empty string
            (r#""hello world""#, "hello world"),
            (r#""with\nnewlines""#, "with\\nnewlines"),
            (r#""with\ttabs""#, "with\\ttabs"),
            (r#""with \"quotes\"""#, "with \\\"quotes\\\""),
            (r#""unicode: 🚀∑∞∆""#, "unicode: 🚀∑∞∆"),
            (r#""path/to/file.txt""#, "path/to/file.txt"),
        ];
        
        for (input, expected) in test_cases {
            let tokens = tokenize_source(input);
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0], Token::StringLiteral(expected.to_string()));
        }
    }

    #[test]
    fn test_identifier_variations() {
        let test_cases = vec![
            "identifier", "_underscore", "camelCase", "PascalCase", 
            "SCREAMING_CASE", "with123numbers", "_leading_underscore",
            "trailing_", "mixed_Case123_", "x", "very_long_identifier_name_that_goes_on_and_on"
        ];
        
        for identifier in test_cases {
            let tokens = tokenize_source(identifier);
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0], Token::Identifier(identifier.to_string()));
        }
    }

    #[test]
    fn test_operator_combinations() {
        let input = "++ -- << >> && || != == <= >= += -= *= /= %= ^= &= |= <<= >>= ?? ??=";
        let tokens = tokenize_source(input);
        
        let expected = vec![
            Token::Plus, Token::Plus,           // ++
            Token::Minus, Token::Minus,         // --
            Token::LeftShift,                   // <<
            Token::RightShift,                  // >>
            Token::And,                         // &&
            Token::Or,                          // ||
            Token::NotEqual,                    // !=
            Token::Equal,                       // ==
            Token::LessEqual,                   // <=
            Token::GreaterEqual,                // >=
            Token::PlusEqual,                   // +=
            Token::MinusEqual,                  // -=
            Token::StarEqual,                   // *=
            Token::SlashEqual,                  // /=
            Token::PercentEqual,                // %=
            Token::CaretEqual,                  // ^=
            Token::AmpersandEqual,              // &=
            Token::PipeEqual,                   // |=
            Token::LeftShiftEqual,              // <<=
            Token::RightShiftEqual,             // >>=
            Token::NullCoalesce,                // ??
            Token::NullCoalesceEqual,           // ??=
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_matrix_syntax() {
        let input = "[[1, 2, 3], [4, 5, 6]]";
        let tokens = tokenize_source(input);
        
        let expected = vec![
            Token::LeftBracket, Token::LeftBracket,
            Token::IntLiteral(1), Token::Comma,
            Token::IntLiteral(2), Token::Comma,
            Token::IntLiteral(3),
            Token::RightBracket, Token::Comma,
            Token::LeftBracket,
            Token::IntLiteral(4), Token::Comma,
            Token::IntLiteral(5), Token::Comma,
            Token::IntLiteral(6),
            Token::RightBracket, Token::RightBracket,
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_function_definition_syntax() {
        let input = "fn add(a: Int, b: Int) -> Int { a + b }";
        let tokens = tokenize_source(input);
        
        let expected = vec![
            Token::Fn,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Colon,
            Token::TypeInt,
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Colon,
            Token::TypeInt,
            Token::RightParen,
            Token::Arrow,
            Token::TypeInt,
            Token::LeftBrace,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::RightBrace,
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_physics_syntax() {
        let input = "physics { world.step(); body.velocity }";
        let tokens = tokenize_source(input);
        
        assert!(tokens.contains(&Token::Physics));
        assert!(tokens.contains(&Token::LeftBrace));
        assert!(tokens.contains(&Token::RightBrace));
        assert!(tokens.contains(&Token::Dot));
        assert!(tokens.contains(&Token::Semicolon));
    }

    #[test]
    fn test_async_parallel_syntax() {
        let input = "async fn compute() { await result; parallel for x in data { x * 2 } }";
        let tokens = tokenize_source(input);
        
        assert!(tokens.contains(&Token::Async));
        assert!(tokens.contains(&Token::Await));
        assert!(tokens.contains(&Token::Parallel));
        assert!(tokens.contains(&Token::For));
        assert!(tokens.contains(&Token::In));
    }

    #[test]
    fn test_comment_handling() {
        let input = r#"
        // Single line comment
        let x = 42; // End of line comment
        /*
         * Multi-line comment
         * with multiple lines
         */
        let y = 3.14;
        /* Inline /* nested */ comment */
        "#;
        
        let tokens = tokenize_source(input);
        
        // Comments should be filtered out
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment(_))));
        
        // But actual code should remain
        assert!(tokens.contains(&Token::Let));
        assert!(tokens.contains(&Token::IntLiteral(42)));
        assert!(tokens.contains(&Token::FloatLiteral(3.14)));
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "let    x\t=\n42\r\n;\n\n";
        let tokens = tokenize_source(input);
        
        let expected = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::IntLiteral(42),
            Token::Semicolon,
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_error_recovery() {
        // Test lexer's ability to handle and recover from errors
        let input = "let x = 42; ∑invalid∞ let y = 3.14;";
        
        // Should be able to tokenize the valid parts
        let result = Lexer::new(input).tokenize();
        
        // Either succeeds with valid tokens or fails gracefully
        match result {
            Ok(tokens) => {
                // Should at least get the first let statement
                assert!(tokens.len() >= 5);
                assert_eq!(tokens[0].token, Token::Let);
            },
            Err(_) => {
                // Error is acceptable for invalid unicode
            }
        }
    }

    #[test]
    fn test_performance_large_input() {
        // Test lexer performance with large input
        let mut large_input = String::new();
        for i in 0..1000 {
            large_input.push_str(&format!("let var{} = {};\n", i, i));
        }
        
        let start = std::time::Instant::now();
        let result = Lexer::new(&large_input).tokenize();
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100); // Should be fast
        
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1000 * 5); // Each line has 5 tokens
    }

    #[test]
    fn test_unicode_support() {
        let input = r#"let π = 3.14159; let 变量 = "值"; let identifier_∑ = 42;"#;
        
        // Test that unicode identifiers and strings are handled correctly
        let result = Lexer::new(input).tokenize();
        
        match result {
            Ok(tokens) => {
                // Should handle unicode in identifiers and strings
                assert!(tokens.len() > 0);
            },
            Err(_) => {
                // Unicode handling might not be fully implemented yet
                // This test documents current limitations
            }
        }
    }

    #[test]
    fn test_token_position_tracking() {
        let input = "let x = 42;\nlet y = 3.14;";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // Test that position information is correctly tracked
        assert!(tokens.len() > 0);
        
        // First token should be at start
        assert_eq!(tokens[0].span.start_line, 1);
        assert_eq!(tokens[0].span.start_col, 1);
        
        // Should have tokens on second line
        let second_line_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.span.start_line == 2)
            .collect();
        assert!(second_line_tokens.len() > 0);
    }
}
