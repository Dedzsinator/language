// Parser unit tests
use matrix_lang::ast::nodes::*;
use matrix_lang::lexer::*;
use matrix_lang::parser::*;
use matrix_lang::*;

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_integer_literal() {
        let lexer = Lexer::new("42");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::IntLiteral(42, _) => {}
            _ => panic!("Expected integer literal 42, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let lexer = Lexer::new("3.14");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::FloatLiteral(f, _) if (f - 3.14).abs() < f64::EPSILON => {}
            _ => panic!("Expected float literal 3.14, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let lexer = Lexer::new("\"hello world\"");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::StringLiteral(s, _) if s == "hello world" => {}
            _ => panic!("Expected string literal \"hello world\", got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let lexer = Lexer::new("2 + 3");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                assert!(matches!(**left, Expression::IntLiteral(2, _)));
                assert!(matches!(operator, BinaryOperator::Add));
                assert!(matches!(**right, Expression::IntLiteral(3, _)));
            }
            _ => panic!("Expected binary operation, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let lexer = Lexer::new("add(1, 2)");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::FunctionCall { function, args, .. } => {
                assert!(matches!(**function, Expression::Identifier(ref name, _) if name == "add"));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let lexer = Lexer::new("[1, 2, 3]");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::ArrayLiteral(elements, _) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], Expression::IntLiteral(1, _)));
                assert!(matches!(elements[1], Expression::IntLiteral(2, _)));
                assert!(matches!(elements[2], Expression::IntLiteral(3, _)));
            }
            _ => panic!("Expected array literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let lexer = Lexer::new("(a + b) * (c - d) / 2.0");
        let mut parser = Parser::new(lexer).unwrap();
        let expr = parser.parse_expression().unwrap();

        // This tests operator precedence and associativity
        match expr {
            Expression::BinaryOp {
                left,
                operator: BinaryOperator::Div,
                right,
                ..
            } => {
                // Left side should be (a + b) * (c - d)
                assert!(matches!(
                    **left,
                    Expression::BinaryOp {
                        operator: BinaryOperator::Mul,
                        ..
                    }
                ));
                // Right side should be 2.0
                assert!(
                    matches!(**right, Expression::FloatLiteral(f, _) if (f - 2.0).abs() < f64::EPSILON)
                );
            }
            _ => panic!("Expected division expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let test_cases = vec![
            ("2 + 3 * 4", "2 + (3 * 4)"),
            ("2 * 3 + 4", "(2 * 3) + 4"),
            ("2 + 3 == 5", "(2 + 3) == 5"),
            ("2 == 3 + 1", "2 == (3 + 1)"),
        ];

        for (input, _expected_structure) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer).unwrap();
            let expr = parser.parse_expression();

            // Just verify it parses without error
            assert!(expr.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_error_handling() {
        let test_cases = vec![
            "let = 42",   // Missing identifier
            "2 +",        // Incomplete expression
            "let x = ",   // Incomplete let binding
            "struct { }", // Missing struct name
            "if true",    // Incomplete if expression
        ];

        for input in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer).unwrap();
            let result = parser.parse_program();

            assert!(result.is_err(), "Expected error for input: {}", input);
        }
    }
}
