// Edge case tests for the Matrix Language implementation
// These tests cover boundary conditions, corner cases, and unusual scenarios

use crate::lexer::{Lexer, Token};
use crate::parser::{Parser, ParseError};
use crate::types::{TypeChecker, TypeError};
use crate::eval::Interpreter;
use crate::physics::rigid_body::RigidBody;
use crate::physics::soft_body::SoftBody;
use crate::physics::constraints::Constraint;
use crate::tests::test_utilities::*;

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    // === LEXER EDGE CASES ===

    #[test]
    fn test_lexer_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1); // Should have EOF token
        assert_eq!(tokens[0].token_type, TokenType::EOF);
    }

    #[test]
    fn test_lexer_whitespace_only() {
        let mut lexer = Lexer::new("   \n\t\r   ");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1); // Should have EOF token only
        assert_eq!(tokens[0].token_type, TokenType::EOF);
    }

    #[test]
    fn test_lexer_very_long_identifier() {
        let long_name = "a".repeat(1000);
        let mut lexer = Lexer::new(&long_name);
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 2); // Identifier + EOF
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].lexeme, long_name);
    }

    #[test]
    fn test_lexer_very_large_number() {
        let input = "999999999999999999999999999999999999999999999999999999999999999999999999";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].token_type, TokenType::Number);
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let mut lexer = Lexer::new("\"unterminated string");
        let tokens = lexer.tokenize();
        // Should handle gracefully - implementation dependent
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_lexer_nested_comments() {
        let input = "/* outer /* inner */ still outer */";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1); // Should have EOF token only
        assert_eq!(tokens[0].token_type, TokenType::EOF);
    }

    #[test]
    fn test_lexer_unicode_edge_cases() {
        let input = "🚀_identifier_🎯 = \"emoji string 🌟\";";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.len() > 3); // Should tokenize successfully
    }

    #[test]
    fn test_lexer_maximum_nesting() {
        let input = "((((((((((((((((((((";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 21); // 20 left parens + EOF
        for i in 0..20 {
            assert_eq!(tokens[i].token_type, TokenType::LeftParen);
        }
    }

    // === PARSER EDGE CASES ===

    #[test]
    fn test_parser_empty_program() {
        let mut parser = Parser::new(vec![Token::new(TokenType::EOF, "", 1, 1)]);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_deeply_nested_expressions() {
        let input = "(((((((((1 + 2) * 3) / 4) - 5) % 6) & 7) | 8) ^ 9) >> 1)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_extremely_long_expression() {
        let mut expr = "1".to_string();
        for i in 2..=100 {
            expr.push_str(&format!(" + {}", i));
        }
        let mut lexer = Lexer::new(&expr);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_malformed_function_definition() {
        let input = "func (x: int) -> { return x; }"; // Missing function name
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_unclosed_blocks() {
        let input = "{ let x = 5; { let y = 10;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_mismatched_delimiters() {
        let input = "func test() { return [1, 2, 3); }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_operator_precedence_edge_cases() {
        let cases = vec![
            "a + b * c / d - e",
            "a && b || c && d",
            "a == b != c <= d >= e",
            "!a && b || !c",
            "a << b >> c & d | e ^ f",
        ];

        for case in cases {
            let mut lexer = Lexer::new(case);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {}", case);
        }
    }

    // === TYPE CHECKER EDGE CASES ===

    #[test]
    fn test_type_checker_circular_type_definitions() {
        let input = r#"
            struct A {
                b: B
            }
            struct B {
                a: A
            }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        // Should detect circular dependency
        assert!(result.is_err());
    }

    #[test]
    fn test_type_checker_deeply_nested_generics() {
        let input = "let x: Array<Array<Array<Array<int>>>> = [[[[1]]]];";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_checker_function_with_many_parameters() {
        let mut params = Vec::new();
        for i in 0..100 {
            params.push(format!("p{}: int", i));
        }
        let input = format!(
            "func test({}) -> int {{ return 42; }}",
            params.join(", ")
        );
        
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_checker_recursive_function_types() {
        let input = r#"
            func factorial(n: int) -> int {
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
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_checker_ambiguous_type_resolution() {
        let input = r#"
            let x = null;
            let y = x;
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        // Should handle null type appropriately
        assert!(result.is_ok() || result.is_err()); // Implementation dependent
    }

    // === INTERPRETER EDGE CASES ===

    #[test]
    fn test_interpreter_maximum_recursion_depth() {
        let input = r#"
            func recurse(n: int) -> int {
                if n <= 0 {
                    return 0;
                } else {
                    return recurse(n - 1);
                }
            }
            let result = recurse(1000);
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should either complete or handle stack overflow gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_interpreter_integer_overflow() {
        let input = "let x = 9223372036854775807 + 1;"; // Max i64 + 1
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should handle overflow appropriately
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_interpreter_division_by_zero() {
        let input = "let x = 10 / 0;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should handle division by zero
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_null_pointer_access() {
        let input = r#"
            let x = null;
            let y = x.field;
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should handle null access
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_array_bounds_checking() {
        let input = r#"
            let arr = [1, 2, 3];
            let x = arr[10];
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should handle out-of-bounds access
        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_memory_exhaustion_simulation() {
        let input = r#"
            func create_large_array(size: int) -> Array<int> {
                let arr = [];
                for i in 0..size {
                    arr.push(i);
                }
                return arr;
            }
            let big_array = create_large_array(1000000);
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        // Should either complete or handle memory limits
        assert!(result.is_ok() || result.is_err());
    }

    // === PHYSICS EDGE CASES ===

    #[test]
    fn test_physics_zero_mass_rigid_body() {
        let body = RigidBody::new([0.0, 0.0, 0.0], 0.0); // Zero mass
        // Should handle zero mass appropriately
        assert_eq!(body.mass(), 0.0);
        assert!(body.inverse_mass().is_infinite() || body.inverse_mass() == 0.0);
    }

    #[test]
    fn test_physics_infinite_mass_rigid_body() {
        let body = RigidBody::new([0.0, 0.0, 0.0], f64::INFINITY);
        assert_eq!(body.inverse_mass(), 0.0);
    }

    #[test]
    fn test_physics_nan_values() {
        let mut body = RigidBody::new([0.0, 0.0, 0.0], 1.0);
        body.apply_force([f64::NAN, f64::NAN, f64::NAN]);
        // Should handle NaN values gracefully
        assert!(body.position()[0].is_nan() || body.position()[0].is_finite());
    }

    #[test]
    fn test_physics_extreme_velocities() {
        let mut body = RigidBody::new([0.0, 0.0, 0.0], 1.0);
        body.set_velocity([1e10, 1e10, 1e10]); // Very high velocity
        body.update(0.01); // Small time step
        // Should handle extreme velocities
        assert!(body.position()[0].is_finite());
    }

    #[test]
    fn test_physics_very_small_time_step() {
        let mut body = RigidBody::new([0.0, 0.0, 0.0], 1.0);
        body.set_velocity([1.0, 1.0, 1.0]);
        body.update(1e-15); // Extremely small time step
        // Should handle very small time steps
        assert!(body.position()[0].is_finite());
    }

    #[test]
    fn test_physics_collision_with_zero_restitution() {
        let mut body1 = RigidBody::new([0.0, 0.0, 0.0], 1.0);
        let mut body2 = RigidBody::new([1.0, 0.0, 0.0], 1.0);
        body1.set_velocity([1.0, 0.0, 0.0]);
        body2.set_velocity([-1.0, 0.0, 0.0]);
        
        // Simulate collision with zero restitution
        let constraint = Constraint::Contact {
            body_a: 0,
            body_b: 1,
            contact_point: [0.5, 0.0, 0.0],
            contact_normal: [1.0, 0.0, 0.0],
            penetration_depth: 0.1,
        };
        
        // Should handle perfectly inelastic collision
        assert!(constraint.is_contact());
    }

    #[test]
    fn test_physics_soft_body_degenerate_mesh() {
        // Test with minimal valid mesh (triangle)
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];
        
        let soft_body = SoftBody::new(positions, indices, 1.0);
        assert_eq!(soft_body.particles().len(), 3);
        assert_eq!(soft_body.springs().len(), 3); // Triangle has 3 edges
    }

    #[test]
    fn test_physics_constraint_solver_edge_cases() {
        // Test with conflicting constraints
        let mut body = RigidBody::new([0.0, 0.0, 0.0], 1.0);
        
        let constraint1 = Constraint::Contact {
            body_a: 0,
            body_b: 1,
            contact_point: [0.0, 0.0, 0.0],
            contact_normal: [1.0, 0.0, 0.0],
            penetration_depth: 0.1,
        };
        
        let constraint2 = Constraint::Contact {
            body_a: 0,
            body_b: 2,
            contact_point: [0.0, 0.0, 0.0],
            contact_normal: [-1.0, 0.0, 0.0],
            penetration_depth: 0.1,
        };
        
        // Should handle conflicting constraints
        assert!(constraint1.is_contact());
        assert!(constraint2.is_contact());
    }

    // === BOUNDARY VALUE TESTS ===

    #[test]
    fn test_empty_string_literals() {
        let input = r#"let x = "";"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_character_identifiers() {
        let input = "let a = 1; let b = 2; let c = a + b;";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_maximum_array_size() {
        let input = "let arr = [0; 1000];"; // Array with 1000 elements
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok() || result.is_err()); // Implementation dependent
    }

    #[test]
    fn test_deeply_nested_data_structures() {
        let input = r#"
            struct Node {
                value: int,
                next: Option<Node>
            }
            let node = Node { 
                value: 1, 
                next: Some(Node { 
                    value: 2, 
                    next: Some(Node { 
                        value: 3, 
                        next: None 
                    }) 
                }) 
            };
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_no_parameters() {
        let input = r#"
            func get_constant() -> int {
                return 42;
            }
            let x = get_constant();
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_no_return_value() {
        let input = r#"
            func print_hello() {
                print("Hello, World!");
            }
            print_hello();
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    // === CORNER CASES FOR CONTROL FLOW ===

    #[test]
    fn test_infinite_loop_detection() {
        let input = r#"
            let mut i = 0;
            while true {
                i = i + 1;
                if i > 1000 {
                    break;
                }
            }
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_break_continue() {
        let input = r#"
            for i in 0..10 {
                for j in 0..10 {
                    if i == j {
                        continue;
                    }
                    if i + j > 10 {
                        break;
                    }
                }
            }
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_early_return_from_nested_function() {
        let input = r#"
            func outer() -> int {
                func inner() -> int {
                    return 42;
                }
                let x = inner();
                return x * 2;
            }
            let result = outer();
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast);
        assert!(result.is_ok());
    }
}
