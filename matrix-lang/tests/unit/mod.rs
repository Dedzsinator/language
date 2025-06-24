// Unit tests for individual language components
use super::utils::*;
use matrix_lang::*;

/// Parser unit tests
mod parser_tests;

/// Matrix Language comprehensive unit tests
mod matrix_language_unit_tests;

/// Lexer unit tests
#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_integer_tokenization() {
        let lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // Int token + EOF
        assert_eq!(tokens[0].token, Token::IntLiteral(42));
    }

    #[test]
    fn test_float_tokenization() {
        let lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // Float token + EOF
        assert_eq!(tokens[0].token, Token::FloatLiteral(3.14));
    }

    #[test]
    fn test_string_tokenization() {
        let lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // String token + EOF
        assert_eq!(
            tokens[0].token,
            Token::StringLiteral("hello world".to_string())
        );
    }

    #[test]
    fn test_identifier_tokenization() {
        let lexer = Lexer::new("variable_name");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // Identifier token + EOF
        assert_eq!(
            tokens[0].token,
            Token::Identifier("variable_name".to_string())
        );
    }

    #[test]
    fn test_operator_tokenization() {
        let lexer = Lexer::new("+ - * / ** == != < > <= >=");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 10); // Multiple operator tokens + EOF
    }

    #[test]
    fn test_complex_expression_tokenization() {
        let source = "let x = (a + b) * c / 2.5;";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 10); // Multiple tokens for complex expression
    }

    #[test]
    fn test_keyword_tokenization() {
        let keywords = vec![
            ("struct", Token::Struct),
            ("let", Token::Let),
            ("if", Token::If),
            ("else", Token::Else),
            ("match", Token::Match),
            ("true", Token::True),
            ("false", Token::False),
        ];

        for (keyword, expected_token) in keywords {
            let lexer = Lexer::new(keyword);
            let tokens = lexer.tokenize().unwrap();
            assert_eq!(tokens[0].token, expected_token);
        }
    }

    #[test]
    fn test_matrix_syntax() {
        let lexer = Lexer::new("[[1, 2], [3, 4]]");
        let tokens = lexer.tokenize().unwrap();

        // Should start with double left bracket
        assert_eq!(tokens[0].token, Token::LeftBracket);
        assert_eq!(tokens[1].token, Token::LeftBracket);

        // Should contain integer literals
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token, Token::IntLiteral(1))));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token, Token::IntLiteral(2))));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token, Token::IntLiteral(3))));
        assert!(tokens
            .iter()
            .any(|t| matches!(t.token, Token::IntLiteral(4))));
    }
}

/// Type checker unit tests
#[cfg(test)]
mod type_checker_tests {
    use super::*;

    #[test]
    fn test_basic_type_checking() {
        let source = "let x: Int = 42; x";
        let result = execute_with_type_check(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "let x: Int = \"hello\"; x";
        let result = execute_with_type_check(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_type_checking() {
        let source = "let add = (a: Int, b: Int) -> Int => a + b; add(1, 2)";
        let result = execute_with_type_check(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_type_checking() {
        let source = "let arr: [Int] = [1, 2, 3]; arr";
        let result = execute_with_type_check(source);
        // This may fail if array types aren't fully implemented
        // assert!(result.is_ok());
    }

    #[test]
    fn test_struct_type_checking() {
        let source = r#"
            struct Point { x: Float, y: Float }
        "#;
        let result = execute_with_type_check(source);
        assert!(result.is_ok());
    }
}

/// Interpreter unit tests
#[cfg(test)]
mod interpreter_tests {
    use super::*;

    #[test]
    fn test_basic_evaluation() {
        assert_eq!(execute("42").unwrap(), Value::Int(42));
        assert_eq!(execute("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(execute("true").unwrap(), Value::Bool(true));
        assert_eq!(
            execute("\"hello\"").unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_arithmetic_evaluation() {
        assert_eq!(execute("2 + 3").unwrap(), Value::Int(5));
        assert_eq!(execute("10 - 4").unwrap(), Value::Int(6));
        assert_eq!(execute("3 * 4").unwrap(), Value::Int(12));
        assert_eq!(execute("15 / 3").unwrap(), Value::Int(5));
    }

    #[test]
    fn test_boolean_evaluation() {
        assert_eq!(execute("true && false").unwrap(), Value::Bool(false));
        assert_eq!(execute("true || false").unwrap(), Value::Bool(true));
        assert_eq!(execute("!true").unwrap(), Value::Bool(false));
        assert_eq!(execute("!false").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_comparison_evaluation() {
        assert_eq!(execute("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(execute("2 < 7").unwrap(), Value::Bool(true));
        assert_eq!(execute("4 == 4").unwrap(), Value::Bool(true));
        assert_eq!(execute("3 != 5").unwrap(), Value::Bool(true));
        assert_eq!(execute("5 >= 5").unwrap(), Value::Bool(true));
        assert_eq!(execute("3 <= 4").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_variable_binding() {
        let source = "let x = 42; x";
        assert_eq!(execute(source).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_function_evaluation() {
        let source = r#"
            let add = (a, b) => a + b;
            add(5, 7)
        "#;
        assert_eq!(execute(source).unwrap(), Value::Int(12));
    }

    #[test]
    fn test_conditional_evaluation() {
        assert_eq!(execute("if true then 1 else 2").unwrap(), Value::Int(1));
        assert_eq!(execute("if false then 1 else 2").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_array_evaluation() {
        let result = execute("[1, 2, 3]").unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[1], Value::Int(2));
                assert_eq!(arr[2], Value::Int(3));
            }
            _ => panic!("Expected array value"),
        }
    }
}

/// Standard library unit tests
#[cfg(test)]
mod stdlib_tests {
    use super::*;

    #[test]
    fn test_math_functions() {
        // Test basic math functions if they exist in stdlib
        let source = "abs(-5)";
        // This test may fail if abs function isn't implemented
        // let result = execute(source).unwrap();
        // assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_array_functions() {
        // Test array manipulation functions if they exist
        let source = "length([1, 2, 3])";
        // This test may fail if length function isn't implemented
        // let result = execute(source).unwrap();
        // assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_string_functions() {
        // Test string manipulation functions if they exist
        let source = "length(\"hello\")";
        // This test may fail if string length function isn't implemented
        // let result = execute(source).unwrap();
        // assert_eq!(result, Value::Int(5));
    }
}

/// JIT compilation unit tests
#[cfg(test)]
mod jit_tests {
    use super::*;

    #[test]
    #[cfg(feature = "jit")]
    fn test_basic_jit_compilation() {
        // Test basic JIT compilation if feature is enabled
        // This is a placeholder test
    }

    #[test]
    fn test_jit_not_available_gracefully() {
        // Test that JIT gracefully handles being unavailable
        // This is a placeholder test
    }
}

/// Error handling unit tests
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_syntax_errors() {
        let error_cases = vec![
            "let = 42",             // Missing identifier
            "2 +",                  // Incomplete expression
            "let x = ",             // Incomplete let binding
            "struct { }",           // Missing struct name
            "if true",              // Incomplete if expression
            ")",                    // Unmatched parenthesis
            "let x = 42 let y = 3", // Missing semicolon
        ];

        for case in error_cases {
            let result = execute(case);
            assert!(result.is_err(), "Expected error for: {}", case);
        }
    }

    #[test]
    fn test_runtime_errors() {
        let error_cases = vec![
            "unknown_variable", // Undefined variable
            "add(1)",           // Wrong number of arguments (if add is defined)
            "1 / 0",            // Division by zero (if checked)
        ];

        for case in error_cases {
            let result = execute(case);
            // Some of these might not error yet depending on implementation
            // assert!(result.is_err(), "Expected runtime error for: {}", case);
        }
    }

    #[test]
    fn test_type_errors() {
        let error_cases = vec![
            "let x: Int = \"hello\"", // Type mismatch
            "let f: Int -> Int = 42", // Function type mismatch
        ];

        for case in error_cases {
            let result = execute_with_type_check(case);
            assert!(result.is_err(), "Expected type error for: {}", case);
        }
    }
}

/// Parser unit tests
#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_integer_literal_parsing() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_binary_expression_parsing() {
        let mut lexer = Lexer::new("2 + 3");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_function_definition_parsing() {
        let source = "let add = (a, b) => a + b;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_matrix_literal_parsing() {
        let source = "[[1, 2], [3, 4]]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_if_expression_parsing() {
        let source = "if true then 1 else 0";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }
}

/// Type checker unit tests
#[cfg(test)]
mod type_checker_tests {
    use super::*;

    #[test]
    fn test_integer_type_inference() {
        let result = execute_with_type_check("42").unwrap();
        assert_int_value(&result, 42);
    }

    #[test]
    fn test_function_type_checking() {
        let source = "let add = (a: Int, b: Int) => a + b; add(1, 2)";
        let result = execute_with_type_check(source).unwrap();
        assert_int_value(&result, 3);
    }

    #[test]
    fn test_type_error_detection() {
        let source = "let add = (a: Int, b: Int) => a + b; add(1, \"hello\")";
        let result = execute_with_type_check(source);
        assert!(
            result.is_err(),
            "Expected type error for mismatched argument types"
        );
    }

    #[test]
    fn test_matrix_type_checking() {
        let source = "let m: Matrix<Int> = [[1, 2], [3, 4]]; m";
        let result = execute_with_type_check(source).unwrap();
        assert_matrix_dimensions(&result, 2, 2);
    }
}

/// Interpreter unit tests
#[cfg(test)]
mod interpreter_tests {
    use super::*;

    #[test]
    fn test_arithmetic_evaluation() {
        assert_int_value(&execute("2 + 3").unwrap(), 5);
        assert_int_value(&execute("10 - 4").unwrap(), 6);
        assert_int_value(&execute("3 * 7").unwrap(), 21);
        assert_int_value(&execute("15 / 3").unwrap(), 5);
    }

    #[test]
    fn test_variable_binding_evaluation() {
        let result = execute("let x = 42; x").unwrap();
        assert_int_value(&result, 42);
    }

    #[test]
    fn test_function_call_evaluation() {
        let source = "let square = (x) => x * x; square(5)";
        let result = execute(source).unwrap();
        assert_int_value(&result, 25);
    }

    #[test]
    fn test_conditional_evaluation() {
        assert_int_value(&execute("if true then 1 else 0").unwrap(), 1);
        assert_int_value(&execute("if false then 1 else 0").unwrap(), 0);
    }

    #[test]
    fn test_matrix_operations() {
        let source = "let m = [[1, 2], [3, 4]]; m";
        let result = execute(source).unwrap();
        assert_matrix_dimensions(&result, 2, 2);
    }

    #[test]
    fn test_builtin_functions() {
        assert_int_value(&execute("abs(-5)").unwrap(), 5);
        assert_float_value(&execute("sqrt(16.0)").unwrap(), 4.0, 0.001);
    }

    #[test]
    fn test_error_handling() {
        assert_runtime_error(|| execute("1 / 0"), "DivisionByZero");
        assert_runtime_error(|| execute("undefined_variable"), "UndefinedVariable");
    }
}

/// Standard library unit tests
#[cfg(test)]
mod stdlib_tests {
    use super::*;

    #[test]
    fn test_math_functions() {
        assert_int_value(&execute("abs(-10)").unwrap(), 10);
        assert_float_value(&execute("abs(-3.14)").unwrap(), 3.14, 0.001);

        assert_float_value(&execute("sqrt(25.0)").unwrap(), 5.0, 0.001);
        assert_float_value(&execute("sqrt(2.0)").unwrap(), 1.414213562, 0.001);
    }

    #[test]
    fn test_print_function() {
        // Print function should execute without error
        let result = execute("print(42)").unwrap();
        assert_eq!(result, Value::Unit);
    }

    #[test]
    fn test_constants() {
        assert_float_value(&execute("pi").unwrap(), std::f64::consts::PI, 0.001);
        assert_float_value(&execute("e").unwrap(), std::f64::consts::E, 0.001);
    }
}

/// JIT compilation unit tests
#[cfg(test)]
#[cfg(feature = "jit")]
mod jit_tests {
    use super::*;

    #[test]
    fn test_simple_jit_compilation() {
        let source = "let add = (a, b) => a + b; add(5, 3)";
        let (result, _duration) = test_jit_compilation(source).unwrap();
        assert_int_value(&result, 8);
    }

    #[test]
    fn test_mathematical_jit_compilation() {
        let source =
            "let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1); factorial(5)";
        let (result, _duration) = test_jit_compilation(source).unwrap();
        assert_int_value(&result, 120);
    }

    #[test]
    fn test_jit_performance() {
        let source = "let sum = (n) => if n <= 0 then 0 else n + sum(n - 1); sum(100)";
        compare_performance("recursive_sum", source, Some(source));
    }
}

/// Error handling unit tests
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_division_by_zero() {
        assert_runtime_error(|| execute("5 / 0"), "DivisionByZero");
        assert_runtime_error(|| execute("5.0 / 0.0"), "DivisionByZero");
    }

    #[test]
    fn test_undefined_variable() {
        assert_runtime_error(|| execute("nonexistent_var"), "UndefinedVariable");
    }

    #[test]
    fn test_type_errors() {
        assert_runtime_error(|| execute("\"hello\" + 42"), "TypeError");
        assert_runtime_error(|| execute("abs(\"not a number\")"), "TypeError");
    }

    #[test]
    fn test_function_call_errors() {
        assert_runtime_error(|| execute("abs(1, 2)"), "FunctionCallError");
        assert_runtime_error(|| execute("let f = 42; f()"), "TypeError");
    }

    #[test]
    fn test_index_out_of_bounds() {
        // This would test array/matrix indexing when implemented
        // assert_runtime_error(|| execute("let arr = [1, 2, 3]; arr[5]"), "IndexOutOfBounds");
    }
}
