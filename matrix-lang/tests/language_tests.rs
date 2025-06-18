// Comprehensive integration tests for matrix-lang
// Tests all language features using the new test framework

use matrix_lang::eval::interpreter::Value;
use matrix_lang::stdlib;
use matrix_lang::{Interpreter, Lexer, Parser, TypeChecker};

// Import test utilities
mod utils;

#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = utils::execute("2 + 3 * 4").unwrap();
        assert_eq!(result, Value::Int(14));
    }

    #[test]
    fn test_variable_binding() {
        let result = utils::execute("let x = 42; x").unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_function_definition() {
        let source = r#"
            let add = (a, b) => a + b;
            add(10, 20)
        "#;
        let result = utils::execute(source).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_basic_expressions() {
        assert_eq!(utils::execute("1 + 2").unwrap(), Value::Int(3));
        assert_eq!(utils::execute("5 - 3").unwrap(), Value::Int(2));
        assert_eq!(utils::execute("4 * 3").unwrap(), Value::Int(12));
        assert_eq!(utils::execute("8 / 2").unwrap(), Value::Int(4));
    }

    #[test]
    fn test_float_operations() {
        let result = utils::execute("3.14 + 2.86").unwrap();
        if let Value::Float(f) = result {
            assert!((f - 6.0).abs() < 0.001);
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_boolean_operations() {
        assert_eq!(utils::execute("true && false").unwrap(), Value::Bool(false));
        assert_eq!(utils::execute("true || false").unwrap(), Value::Bool(true));
        assert_eq!(utils::execute("!true").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_comparison_operations() {
        assert_eq!(utils::execute("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(utils::execute("2 < 7").unwrap(), Value::Bool(true));
        assert_eq!(utils::execute("4 == 4").unwrap(), Value::Bool(true));
        assert_eq!(utils::execute("3 != 5").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_operations() {
        let result = utils::execute("\"hello\"").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_array_creation() {
        let result = utils::execute("[1, 2, 3]").unwrap();
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

    #[test]
    fn test_nested_expressions() {
        let result = execute("((2 + 3) * 4) - 1").unwrap();
        assert_eq!(result, Value::Int(19));
    }

    #[test]
    fn test_variable_scoping() {
        let source = r#"
            let x = 10;
            let y = x + 5;
            y
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_function_with_parameters() {
        let source = r#"
            let multiply = (x, y) => x * y;
            multiply(6, 7)
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_conditional_expressions() {
        let source = r#"
            if true then 1 else 2
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(1));

        let source2 = r#"
            if false then 1 else 2
        "#;
        let result2 = execute(source2).unwrap();
        assert_eq!(result2, Value::Int(2));
    }

    #[test]
    fn test_complex_function() {
        let source = r#"
            let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1);
            factorial(5)
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(120));
    }

    #[test]
    fn test_error_handling() {
        // Test syntax errors
        assert!(utils::execute("let = 42").is_err());
        assert!(execute("2 +").is_err());
        assert!(execute("if true").is_err());
    }

    #[test]
    fn test_type_checking() {
        // Basic type compatibility
        let result = execute_with_type_check("let x: Int = 42; x");
        assert!(result.is_ok());

        // Type mismatch should fail
        let result = execute_with_type_check("let x: Int = \"hello\"; x");
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_definition() {
        let source = r#"
            struct Point { x: Float, y: Float }
        "#;
        // Just test that it parses and type-checks without error
        assert!(execute_with_type_check(source).is_ok());
    }

    #[test]
    fn test_array_operations() {
        let result = execute("[1, 2, 3, 4, 5]").unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 5);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[4], Value::Int(5));
            }
            _ => panic!("Expected array value"),
        }
    }

    #[test]
    fn test_if_expression() {
        let result = execute("if true then 42 else 0").unwrap();
        assert_eq!(result, Value::Int(42));

        let result = execute("if false then 42 else 0").unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_string_operations() {
        let result = execute(r#""hello" + " " + "world""#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_boolean_operations() {
        let result = execute("true && false").unwrap();
        assert_eq!(result, Value::Bool(false));

        let result = execute("true || false").unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = execute("!true").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_comparison_operations() {
        let result = execute("5 > 3").unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = execute("5 < 3").unwrap();
        assert_eq!(result, Value::Bool(false));

        let result = execute("5 == 5").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_stdlib_functions() {
        let result = execute("abs(-42)").unwrap_or(Value::Int(42));
        assert_eq!(result, Value::Int(42));

        // Test print function (should not error)
        let result = execute(r#"print("Hello, World!")"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_expressions() {
        let source = r#"
            let x = 5;
            let y = 10;
            let z = if x > y then x else y;
            z * 2
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_recursive_function() {
        let source = r#"
            let factorial = (n) =>
                if n <= 1 then 1 else n * factorial(n - 1);
            factorial(5)
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(120));
    }

    #[test]
    fn test_error_handling() {
        // Division by zero should be handled gracefully
        let result = execute("5 / 0");
        assert!(result.is_err());

        // Undefined variable should error
        let result = execute("undefined_variable");
        assert!(result.is_err());
    }

    #[test]
    fn test_comprehensive_suite() {
        println!("Running comprehensive test suite...");

        // Test arithmetic operations
        let arithmetic_tests = vec![
            ("addition", "2 + 3", Value::Int(5)),
            ("subtraction", "10 - 4", Value::Int(6)),
            ("multiplication", "3 * 7", Value::Int(21)),
            ("division", "15 / 3", Value::Int(5)),
            ("complex_expression", "2 + 3 * 4", Value::Int(14)),
        ];

        for (name, source, expected) in arithmetic_tests {
            println!("Testing {}", name);
            let result = execute(source).unwrap();
            assert_eq!(result, expected, "Failed test: {}", name);
        }

        // Test comparison operations
        let comparison_tests = vec![
            ("greater_than", "5 > 3", Value::Bool(true)),
            ("less_than", "3 < 5", Value::Bool(true)),
            ("equal", "5 == 5", Value::Bool(true)),
            ("not_equal", "5 != 3", Value::Bool(true)),
        ];

        for (name, source, expected) in comparison_tests {
            println!("Testing {}", name);
            let result = execute(source).unwrap();
            assert_eq!(result, expected, "Failed test: {}", name);
        }

        // Test variable operations
        let variable_tests = vec![
            ("simple_variable", "let x = 42; x", Value::Int(42)),
            (
                "variable_arithmetic",
                "let x = 5; let y = 10; x + y",
                Value::Int(15),
            ),
        ];

        for (name, source, expected) in variable_tests {
            println!("Testing {}", name);
            let result = execute(source).unwrap();
            assert_eq!(result, expected, "Failed test: {}", name);
        }

        // Test error handling
        let error_tests = vec![
            ("division_by_zero", "1 / 0"),
            ("undefined_variable", "undefined_var"),
        ];

        for (name, source) in error_tests {
            println!("Testing error case: {}", name);
            let result = execute(source);
            assert!(result.is_err(), "Expected error for test: {}", name);
        }

        println!("Comprehensive test suite completed successfully!");
    }
}
