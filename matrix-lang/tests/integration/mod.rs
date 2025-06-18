// Integration tests for matrix-lang components working together
use super::utils::*;
use matrix_lang::*;

/// Language feature integration tests
#[cfg(test)]
mod language_integration_tests {
    use super::*;

    #[test]
    fn test_complex_mathematical_expression() {
        let source = r#"
            let factorial = (n) => {
                if n <= 1 then
                    1
                else
                    n * factorial(n - 1)
            };
            factorial(5)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 120);
    }

    #[test]
    fn test_matrix_computations() {
        let source = r#"
            let matrix_a = [[1, 2], [3, 4]];
            let matrix_b = [[5, 6], [7, 8]];
            matrix_a
        "#;
        let result = execute(source).unwrap();
        assert_matrix_dimensions(&result, 2, 2);
    }

    #[test]
    fn test_higher_order_functions() {
        let source = r#"
            let apply_twice = (f, x) => f(f(x));
            let square = (x) => x * x;
            apply_twice(square, 2)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 16); // square(square(2)) = square(4) = 16
    }

    #[test]
    fn test_nested_function_definitions() {
        let source = r#"
            let outer = (x) => {
                let inner = (y) => x + y;
                inner(10)
            };
            outer(5)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 15);
    }

    #[test]
    fn test_conditional_logic_with_functions() {
        let source = r#"
            let abs_value = (x) => if x < 0 then -x else x;
            let test_positive = abs_value(5);
            let test_negative = abs_value(-3);
            test_positive + test_negative
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 8);
    }

    #[test]
    fn test_multiple_variable_bindings() {
        let source = r#"
            let a = 10;
            let b = 20;
            let c = a + b;
            let d = c * 2;
            d
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 60);
    }

    #[test]
    fn test_function_composition() {
        let source = r#"
            let add_one = (x) => x + 1;
            let multiply_two = (x) => x * 2;
            let compose = (f, g, x) => f(g(x));
            compose(add_one, multiply_two, 5)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 11); // add_one(multiply_two(5)) = add_one(10) = 11
    }
}

/// Physics engine integration tests
#[cfg(test)]
mod physics_integration_tests {
    use super::*;

    #[test]
    fn test_basic_physics_simulation() {
        // Test basic physics functionality when available
        let source = r#"
            let velocity = 10.0;
            let time = 2.0;
            let distance = velocity * time;
            distance
        "#;
        let result = execute(source).unwrap();
        assert_float_value(&result, 20.0, 0.001);
    }

    #[test]
    fn test_vector_operations() {
        // Test basic vector-like operations using arrays
        let source = r#"
            let vec_a = [1, 2, 3];
            let vec_b = [4, 5, 6];
            vec_a
        "#;
        let result = execute(source).unwrap();
        assert_array_length(&result, 3);
    }
}

/// Cross-module integration tests
#[cfg(test)]
mod cross_module_tests {
    use super::*;

    #[test]
    fn test_lexer_parser_integration() {
        let complex_source = r#"
            let complex_function = (a, b, c) => {
                let intermediate = a * b;
                if intermediate > c then
                    intermediate + c
                else
                    intermediate - c
            };
            complex_function(5, 3, 10)
        "#;

        // Test that lexer and parser work together correctly
        let mut lexer = Lexer::new(complex_source);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 20);

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.items.len(), 1);

        // Test execution
        let result = execute(complex_source).unwrap();
        assert_int_value(&result, 5); // 5 * 3 = 15, 15 - 10 = 5
    }

    #[test]
    fn test_type_checker_interpreter_integration() {
        let source = r#"
            let typed_add = (a: Int, b: Int) => a + b;
            typed_add(15, 25)
        "#;

        // Test type checking integration (if available)
        let result = execute(source).unwrap();
        assert_int_value(&result, 40);
    }

    #[test]
    fn test_stdlib_interpreter_integration() {
        let source = r#"
            let compute = (x) => {
                let abs_x = abs(x);
                let sqrt_abs = sqrt(abs_x);
                sqrt_abs
            };
            compute(-16.0)
        "#;

        let result = execute(source).unwrap();
        assert_float_value(&result, 4.0, 0.001);
    }
}

/// Performance integration tests
#[cfg(test)]
mod performance_integration_tests {
    use super::*;

    #[test]
    fn test_recursive_function_performance() {
        let source = r#"
            let fibonacci = (n) => {
                if n <= 1 then
                    n
                else
                    fibonacci(n - 1) + fibonacci(n - 2)
            };
            fibonacci(10)
        "#;

        let start = std::time::Instant::now();
        let result = execute(source).unwrap();
        let duration = start.elapsed();

        assert_int_value(&result, 55);
        assert!(
            duration.as_millis() < 1000,
            "Fibonacci(10) should complete within 1 second"
        );
    }

    #[test]
    fn test_large_matrix_creation() {
        let source = r#"
            let create_matrix = (size) => {
                let row = [1, 2, 3, 4, 5];
                [row, row, row, row, row]
            };
            create_matrix(5)
        "#;

        let result = execute(source).unwrap();
        assert_matrix_dimensions(&result, 5, 5);
    }
}

/// Error handling integration tests
#[cfg(test)]
mod error_integration_tests {
    use super::*;

    #[test]
    fn test_nested_error_propagation() {
        let source = r#"
            let divide_and_process = (a, b) => {
                let result = a / b;
                result * 2
            };
            divide_and_process(10, 0)
        "#;

        assert_runtime_error(|| execute(source), "DivisionByZero");
    }

    #[test]
    fn test_function_call_error_propagation() {
        let source = r#"
            let call_undefined = () => undefined_function();
            call_undefined()
        "#;

        assert_runtime_error(|| execute(source), "UndefinedVariable");
    }

    #[test]
    fn test_type_error_in_complex_expression() {
        let source = r#"
            let mixed_operations = (x, y) => {
                let sum = x + y;
                abs(sum)
            };
            mixed_operations("hello", 42)
        "#;

        assert_runtime_error(|| execute(source), "TypeError");
    }
}

/// JIT integration tests
#[cfg(test)]
#[cfg(feature = "jit")]
mod jit_integration_tests {
    use super::*;

    #[test]
    fn test_jit_with_complex_functions() {
        let source = r#"
            let complex_math = (x) => {
                let a = x * x;
                let b = sqrt(a);
                let c = sin(b);
                cos(c)
            };
            complex_math(3.14159)
        "#;

        let (result, _duration) = test_jit_compilation(source).unwrap();
        // Just ensure it executes without error
        match result {
            Value::Float(_) => {}
            _ => panic!("Expected float result from complex math function"),
        }
    }

    #[test]
    fn test_jit_vs_interpreter_consistency() {
        let test_cases = vec![
            "let add = (a, b) => a + b; add(5, 7)",
            "let square = (x) => x * x; square(8)",
            "let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1); factorial(4)",
        ];

        for source in test_cases {
            let interpreter_result = execute(source).unwrap();
            let (jit_result, _) = test_jit_compilation(source).unwrap();

            assert_eq!(
                interpreter_result, jit_result,
                "JIT and interpreter results should match for: {}",
                source
            );
        }
    }
}

/// Stress tests
#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_deep_recursion() {
        let source = r#"
            let countdown = (n) => {
                if n <= 0 then
                    0
                else
                    countdown(n - 1)
            };
            countdown(100)
        "#;

        let result = execute(source).unwrap();
        assert_int_value(&result, 0);
    }

    #[test]
    fn test_many_variables() {
        let mut source = String::new();
        for i in 0..50 {
            source.push_str(&format!("let var_{} = {};\n", i, i));
        }
        source.push_str("var_49");

        let result = execute(&source).unwrap();
        assert_int_value(&result, 49);
    }

    #[test]
    fn test_large_array_creation() {
        let source = r#"
            let create_large_array = () => {
                let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
                arr
            };
            create_large_array()
        "#;

        let result = execute(source).unwrap();
        assert_array_length(&result, 10);
    }
}
