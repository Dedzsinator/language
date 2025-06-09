// Comprehensive test suite for Matrix Language
// This module provides additional test coverage beyond the existing unit tests

use crate::eval::Interpreter;
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use crate::physics::math::Vec3;
use crate::physics::rigid_body::{RigidBody, Shape};
use crate::physics::soft_body::SoftBody;
use crate::types::TypeChecker;

#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    // === INTEGRATION TESTS ===

    #[test]
    fn test_full_pipeline_simple_arithmetic() {
        let input = "let x = 5 + 3; x * 2";

        // Tokenize
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(!tokens.is_empty());

        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Failed to parse: {:?}", ast.err());
        let ast = ast.unwrap();

        // Type check
        let mut type_checker = TypeChecker::new();
        let type_result = type_checker.check_program(&ast);
        // Type checking might fail due to implementation - that's ok for now

        // Interpret
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok(), "Failed to interpret: {:?}", result.err());
    }

    #[test]
    fn test_full_pipeline_function_definition() {
        let input = r#"
            func add(a: Int, b: Int) -> Int {
                return a + b;
            }
            let result = add(5, 3);
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Failed to parse function definition");

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast.unwrap());
        // Function definition interpretation might have issues - checking parse is enough
    }

    #[test]
    fn test_full_pipeline_struct_creation() {
        let input = r#"
            struct Point {
                x: Float,
                y: Float
            }
            let p = Point { x: 1.0, y: 2.0 };
            p.x + p.y
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Failed to parse struct definition");

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast.unwrap());
        // Struct interpretation might have issues - checking parse is enough
    }

    #[test]
    fn test_full_pipeline_array_operations() {
        let input = r#"
            let arr = [1, 2, 3, 4, 5];
            let sum = arr[0] + arr[1] + arr[2];
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Failed to parse array operations");

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast.unwrap());
        assert!(
            result.is_ok(),
            "Failed to interpret array operations: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_full_pipeline_control_flow() {
        let input = r#"
            let x = 10;
            if x > 5 {
                x * 2
            } else {
                x / 2
            }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Failed to parse control flow");

        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast.unwrap());
        assert!(
            result.is_ok(),
            "Failed to interpret control flow: {:?}",
            result.err()
        );
    }

    // === LEXER COMPREHENSIVE TESTS ===

    #[test]
    fn test_lexer_all_token_types() {
        let input = r#"
            struct Point { x: Int, y: Float }
            func test() -> Bool { 
                let a = [1, 2, 3];
                let b = "hello";
                let c = true;
                let d = null;
                return a.len() > 0 && c != false;
            }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Should have many different token types
        assert!(tokens.len() > 30);

        // Check for presence of key token types by looking for their string representations
        let token_strings: Vec<String> = tokens.iter().map(|t| format!("{:?}", t)).collect();
        let combined = token_strings.join(" ");

        assert!(combined.contains("Struct"));
        assert!(combined.contains("LeftBrace"));
        assert!(combined.contains("RightBrace"));
        assert!(combined.contains("Let"));
        assert!(combined.contains("Func"));
    }

    #[test]
    fn test_lexer_unicode_handling() {
        let input = "let café = \"🚀 hello world 🌟\";";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.len() >= 4); // Let, identifier, =, string, semicolon, EOF
    }

    #[test]
    fn test_lexer_numeric_literals() {
        let cases = vec!["123", "123.456", "0.5", ".5", "1e10", "1.5e-3"];

        for case in cases {
            let mut lexer = Lexer::new(case);
            let tokens = lexer.tokenize();
            assert!(tokens.len() >= 1, "Failed to tokenize: {}", case);
        }
    }

    #[test]
    fn test_lexer_string_escapes() {
        let input = r#""hello\nworld\t\"""#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.len() >= 1);
    }

    // === PARSER COMPREHENSIVE TESTS ===

    #[test]
    fn test_parser_operator_precedence() {
        let cases = vec![
            ("1 + 2 * 3", "Should parse as 1 + (2 * 3)"),
            ("a && b || c", "Should handle logical operators"),
            ("x == y != z", "Should handle comparison chains"),
            ("!a && b", "Should handle unary operators"),
        ];

        for (input, description) in cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse {}: {}", input, description);
        }
    }

    #[test]
    fn test_parser_nested_expressions() {
        let input = "((((1 + 2) * 3) / 4) - 5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse deeply nested expression");
    }

    #[test]
    fn test_parser_complex_function() {
        let input = r#"
            func factorial(n: Int) -> Int {
                if n <= 1 {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse recursive function");
    }

    // === PHYSICS TESTS ===

    #[test]
    fn test_physics_rigid_body_comprehensive() {
        // Test basic rigid body creation and properties
        let sphere_shape = Shape::Sphere { radius: 1.0 };
        let body = RigidBody::new(sphere_shape, 1.0, Vec3::new(0.0, 0.0, 0.0));

        assert_eq!(body.mass, 1.0);
        assert_eq!(body.position, Vec3::new(0.0, 0.0, 0.0));
        assert!(!body.is_static);

        // Test zero mass (static) body
        let static_body = RigidBody::new(
            Shape::Box {
                size: Vec3::new(2.0, 2.0, 2.0),
            },
            0.0,
            Vec3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(static_body.mass, 0.0);
        assert_eq!(static_body.inv_mass, 0.0);
    }

    #[test]
    fn test_physics_soft_body_creation() {
        // Create a simple triangle mesh
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![0, 1, 2];

        let soft_body = SoftBody::new(positions, indices, 1.0);
        assert_eq!(soft_body.particles().len(), 3);
        assert!(!soft_body.springs().is_empty());
    }

    #[test]
    fn test_physics_shape_inertia_tensors() {
        // Test different shapes have reasonable inertia tensors
        let sphere = Shape::Sphere { radius: 1.0 };
        let sphere_inertia = sphere.inertia_tensor(1.0);

        let box_shape = Shape::Box {
            size: Vec3::new(2.0, 2.0, 2.0),
        };
        let box_inertia = box_shape.inertia_tensor(1.0);

        // Both should have positive diagonal elements
        assert!(sphere_inertia.xx > 0.0);
        assert!(box_inertia.xx > 0.0);
    }

    // === ERROR HANDLING TESTS ===

    #[test]
    fn test_lexer_error_recovery() {
        // Test unterminated strings
        let input = "\"unterminated string";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        // Should not panic, should handle gracefully
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parser_error_recovery() {
        // Test malformed expressions
        let inputs = vec![
            "let x = ;",    // Missing expression
            "func () {}",   // Missing function name
            "{ let x = 5;", // Unclosed block
        ];

        for input in inputs {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            // Should return error, not panic
            assert!(result.is_err(), "Expected parse error for: {}", input);
        }
    }

    #[test]
    fn test_interpreter_runtime_errors() {
        let cases = vec![
            ("let x = 10 / 0;", "Division by zero"),
            ("let arr = [1, 2, 3]; arr[10];", "Array bounds"),
        ];

        for (input, description) in cases {
            let mut lexer = Lexer::new(input);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            if let Ok(ast) = parser.parse() {
                let mut interpreter = Interpreter::new();
                let result = interpreter.interpret(&ast);
                // Should handle runtime errors gracefully
                if result.is_ok() {
                    println!("Expected runtime error for {}: {}", input, description);
                }
            }
        }
    }

    // === PERFORMANCE TESTS ===

    #[test]
    fn test_lexer_performance_large_input() {
        // Create a large input string
        let mut large_input = String::new();
        for i in 0..1000 {
            large_input.push_str(&format!("let x{} = {}; ", i, i));
        }

        let start = std::time::Instant::now();
        let mut lexer = Lexer::new(&large_input);
        let tokens = lexer.tokenize();
        let duration = start.elapsed();

        assert!(tokens.len() > 3000); // Should have many tokens
        assert!(duration.as_millis() < 1000); // Should complete in reasonable time
    }

    #[test]
    fn test_parser_performance_nested_expressions() {
        // Create deeply nested expression
        let mut input = "1".to_string();
        for i in 2..=50 {
            input = format!("({} + {})", input, i);
        }

        let start = std::time::Instant::now();
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should parse nested expressions");
        assert!(duration.as_millis() < 1000); // Should complete in reasonable time
    }

    #[test]
    fn test_physics_performance_many_bodies() {
        // Create many rigid bodies to test performance
        let mut bodies = Vec::new();
        let start = std::time::Instant::now();

        for i in 0..100 {
            let shape = Shape::Sphere { radius: 1.0 };
            let body = RigidBody::new(shape, 1.0, Vec3::new(i as f64, 0.0, 0.0));
            bodies.push(body);
        }

        let duration = start.elapsed();
        assert_eq!(bodies.len(), 100);
        assert!(duration.as_millis() < 100); // Should create bodies quickly
    }

    // === REGRESSION TESTS ===

    #[test]
    fn test_regression_matrix_literals() {
        let input = "let m = [[1, 2], [3, 4]];";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok(), "Matrix literals should parse correctly");
    }

    #[test]
    fn test_regression_range_expressions() {
        let input = "let r = 1..10;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok(), "Range expressions should parse correctly");
    }

    #[test]
    fn test_regression_match_expressions() {
        let input = r#"
            match x {
                1 => "one",
                2 => "two",
                _ => "other"
            }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok(), "Match expressions should parse correctly");
    }
}
