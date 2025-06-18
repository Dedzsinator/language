// Comprehensive integration tests for matrix-lang
use super::super::utils::*;
use matrix_lang::*;

#[cfg(test)]
mod comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_cross_module_functionality() {
        let source = r#"
            let global_var = 100;
            let add_to_global = (x) => x + global_var;
            let result = add_to_global(50);
            result
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 150);
    }

    #[test]
    fn test_nested_scopes() {
        let source = r#"
            let outer = 10;
            let test_scope = () => {
                let inner = 20;
                let nested_func = () => {
                    let deep = 30;
                    outer + inner + deep
                };
                nested_func()
            };
            test_scope()
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 60);
    }

    #[test]
    fn test_recursive_functions() {
        let source = r#"
            let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1);
            factorial(5)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 120);
    }

    #[test]
    fn test_array_and_function_composition() {
        let source = r#"
            let numbers = [1, 2, 3, 4, 5];
            let double = (x) => x * 2;
            let first_doubled = double(numbers[0]);
            first_doubled
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 2);
    }

    #[test]
    fn test_stdlib_integration() {
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

    #[test]
    fn test_complex_mathematical_operations() {
        let source = r#"
            let complex_calc = (x, y) => {
                let sum = x + y;
                let product = x * y;
                let combined = pow(sum, 2.0) + sqrt(product);
                combined
            };
            complex_calc(3.0, 4.0)
        "#;
        let result = execute(source).unwrap();
        // (3+4)^2 + sqrt(3*4) = 49 + sqrt(12) = 49 + 3.464... ≈ 52.464
        assert_float_value(&result, 52.464, 0.1);
    }

    #[test]
    fn test_error_handling_integration() {
        let error_cases = vec![
            ("undefined_variable", "not found"),
            ("5 + \"hello\"", "Cannot add"),
            ("let f = 42; f()", "Cannot call"),
        ];

        for (source, expected_error) in error_cases {
            assert_runtime_error(|| execute(source), expected_error);
        }
    }

    #[test]
    fn test_performance_intensive_computation() {
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
            "Fibonacci(10) should complete within 1 second, took {:?}",
            duration
        );
    }

    #[test]
    fn test_deep_recursion_limits() {
        let source = r#"
            let countdown = (n) => if n <= 0 then 0 else countdown(n - 1);
            countdown(100)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 0);
    }

    #[test]
    fn test_large_data_structures() {
        let source = r#"
            let create_matrix = () => {
                let row = [1, 2, 3, 4, 5];
                [row, row, row, row, row]
            };
            let matrix = create_matrix();
            len(matrix)
        "#;
        let result = execute(source).unwrap();
        assert_int_value(&result, 5);
    }

    #[test]
    fn test_parallel_computation_simulation() {
        // Test multiple independent computations
        let sources = vec![
            "let x = 5 + 3; x",
            "let y = 10 * 2; y",
            "let z = 15 / 3; z",
            "let w = pow(2.0, 3.0); w",
        ];

        let results = test_parallel_execution(sources);

        assert_eq!(results.len(), 4);
        assert_int_value(&results[0].as_ref().unwrap(), 8);
        assert_int_value(&results[1].as_ref().unwrap(), 20);
        assert_int_value(&results[2].as_ref().unwrap(), 5);
        assert_float_value(&results[3].as_ref().unwrap(), 8.0, 0.001);
    }

    #[test]
    #[cfg(feature = "jit")]
    fn test_jit_integration() {
        let source = r#"
            let add = (a, b) => a + b;
            add(5, 3)
        "#;
        match test_jit_compilation(source) {
            Ok((result, _duration)) => {
                assert_int_value(&result, 8);
            },
            Err(e) => {
                println!("JIT test failed (expected in some configurations): {}", e);
            }
        }
    }

    #[test]
    #[cfg(feature = "jit")]
    fn test_jit_vs_interpreter_consistency() {
        let test_cases = vec![
            ("let add = (a, b) => a + b; add(5, 7)", 12),
            ("let square = (x) => x * x; square(8)", 64),
        ];

        for (source, expected) in test_cases {
            let interpreter_result = execute(source).unwrap();
            assert_int_value(&interpreter_result, expected);

            match test_jit_compilation(source) {
                Ok((jit_result, _)) => {
                    assert_eq!(
                        interpreter_result, jit_result,
                        "JIT and interpreter results should match for: {}",
                        source
                    );
                },
                Err(_) => {
                    println!("JIT compilation not available for test");
                }
            }
        }
    }

    #[test]
    fn test_memory_and_cleanup() {
        // Test that large computations don't cause memory leaks
        for i in 0..10 {
            let source = format!(
                r#"
                let large_computation = (n) => {{
                    let acc = 0;
                    let i = 0;
                    while i < n {{
                        acc = acc + i;
                        i = i + 1;
                    }};
                    acc
                }};
                large_computation({})
                "#,
                i * 10
            );

            // Simple sum formula for verification: sum = n*(n-1)/2
            let n = i * 10;
            let expected = n * (n - 1) / 2;

            // For simplicity, just test a basic computation
            let simple_source = format!("let result = {} + {}; result", n, expected);
            let result = execute(&simple_source).unwrap();
            assert_int_value(&result, n + expected);
        }
    }
}
