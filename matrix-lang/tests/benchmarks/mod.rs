// Performance benchmarks for matrix-lang
use super::utils::*;
use matrix_lang::*;
use std::time::{Duration, Instant};

/// Arithmetic operation benchmarks
#[cfg(test)]
mod arithmetic_benchmarks {
    use super::*;

    #[test]
    fn benchmark_simple_arithmetic() {
        let test_cases = vec![
            ("addition", "1000 + 2000"),
            ("subtraction", "5000 - 1000"),
            ("multiplication", "123 * 456"),
            ("division", "10000 / 25"),
            ("power", "2 ** 10"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 1000, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_complex_arithmetic() {
        let source = "((123 + 456) * 789) / (234 - 56) + (78 ** 2)";
        benchmark_execution("complex_arithmetic", 1000, || {
            execute(source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }

    #[test]
    fn benchmark_floating_point_operations() {
        let test_cases = vec![
            ("float_addition", "3.14159 + 2.71828"),
            ("float_multiplication", "1.41421 * 1.73205"),
            ("float_division", "22.0 / 7.0"),
            ("sqrt_operation", "sqrt(1024.0)"),
            ("sin_operation", "sin(3.14159 / 2.0)"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 1000, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }
}

/// Function call benchmarks
#[cfg(test)]
mod function_benchmarks {
    use super::*;

    #[test]
    fn benchmark_simple_function_calls() {
        let source = r#"
            let add = (a, b) => a + b;
            add(42, 58)
        "#;

        benchmark_execution("simple_function_call", 1000, || {
            execute(source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }

    #[test]
    fn benchmark_recursive_functions() {
        let test_cases = vec![
            ("factorial_5", r#"
                let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1);
                factorial(5)
            "#),
            ("fibonacci_10", r#"
                let fib = (n) => if n <= 1 then n else fib(n - 1) + fib(n - 2);
                fib(10)
            "#),
            ("countdown_50", r#"
                let countdown = (n) => if n <= 0 then 0 else countdown(n - 1);
                countdown(50)
            "#),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 100, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_higher_order_functions() {
        let source = r#"
            let apply_twice = (f, x) => f(f(x));
            let increment = (x) => x + 1;
            apply_twice(increment, 0)
        "#;

        benchmark_execution("higher_order_function", 1000, || {
            execute(source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }
}

/// Data structure benchmarks
#[cfg(test)]
mod data_structure_benchmarks {
    use super::*;

    #[test]
    fn benchmark_array_operations() {
        let test_cases = vec![
            ("small_array", "[1, 2, 3, 4, 5]"),
            ("medium_array", "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]"),
            ("array_with_expressions", "[1 + 1, 2 * 2, 3 ** 2, 4 / 2, 5 - 1]"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 1000, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_matrix_operations() {
        let test_cases = vec![
            ("small_matrix", "[[1, 2], [3, 4]]"),
            ("medium_matrix", "[[1, 2, 3], [4, 5, 6], [7, 8, 9]]"),
            ("large_matrix", "[[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15], [16, 17, 18, 19, 20], [21, 22, 23, 24, 25]]"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 500, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }
}

/// Variable binding benchmarks
#[cfg(test)]
mod variable_benchmarks {
    use super::*;

    #[test]
    fn benchmark_variable_access() {
        let source = r#"
            let x = 42;
            let y = x + 1;
            let z = y * 2;
            z
        "#;

        benchmark_execution("variable_access", 1000, || {
            execute(source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }

    #[test]
    fn benchmark_many_variables() {
        let mut source = String::new();
        source.push_str("let x0 = 0;\n");
        for i in 1..=20 {
            source.push_str(&format!("let x{} = x{} + 1;\n", i, i - 1));
        }
        source.push_str("x20");

        benchmark_execution("many_variables", 500, || {
            execute(&source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }

    #[test]
    fn benchmark_nested_scopes() {
        let source = r#"
            let outer = (x) => {
                let middle = (y) => {
                    let inner = (z) => x + y + z;
                    inner(3)
                };
                middle(2)
            };
            outer(1)
        "#;

        benchmark_execution("nested_scopes", 1000, || {
            execute(source).map_err(|e| format!("{:?}", e))?;
            Ok(())
        });
    }
}

/// Standard library benchmarks
#[cfg(test)]
mod stdlib_benchmarks {
    use super::*;

    #[test]
    fn benchmark_math_functions() {
        let test_cases = vec![
            ("abs_int", "abs(-42)"),
            ("abs_float", "abs(-3.14159)"),
            ("sqrt_function", "sqrt(256.0)"),
            ("sin_function", "sin(1.5708)"), // π/2
            ("cos_function", "cos(0.0)"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 1000, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_builtin_constants() {
        let test_cases = vec![
            ("pi_constant", "pi"),
            ("e_constant", "e"),
            ("tau_constant", "tau"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(name, 1000, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }
}

/// Parser and lexer benchmarks
#[cfg(test)]
mod parsing_benchmarks {
    use super::*;

    #[test]
    fn benchmark_lexing() {
        let test_cases = vec![
            ("simple_expression", "2 + 3 * 4"),
            ("complex_expression", "let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1); factorial(5)"),
            ("matrix_literal", "[[1, 2, 3], [4, 5, 6], [7, 8, 9]]"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(&format!("lexing_{}", name), 1000, || {
                let mut lexer = Lexer::new(source);
                lexer.tokenize().map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_parsing() {
        let test_cases = vec![
            ("simple_expression", "2 + 3 * 4"),
            ("function_definition", "let add = (a, b) => a + b; add(1, 2)"),
            ("conditional_expression", "if true then 42 else 0"),
        ];

        for (name, source) in test_cases {
            benchmark_execution(&format!("parsing_{}", name), 1000, || {
                let mut lexer = Lexer::new(source);
                let tokens = lexer.tokenize().map_err(|e| format!("{:?}", e))?;
                let mut parser = Parser::new(tokens);
                parser.parse().map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }
}

/// Memory usage benchmarks
#[cfg(test)]
mod memory_benchmarks {
    use super::*;

    #[test]
    fn benchmark_memory_usage() {
        let test_cases = vec![
            ("small_computation", "2 + 2"),
            ("recursive_function", r#"
                let fib = (n) => if n <= 1 then n else fib(n - 1) + fib(n - 2);
                fib(8)
            "#),
            ("large_array", "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"),
            ("matrix_creation", "[[1, 2, 3], [4, 5, 6], [7, 8, 9]]"),
        ];

        for (name, source) in test_cases {
            let (memory_estimate, duration) = measure_memory_usage(|| execute(source));
            println!("Memory benchmark '{}': ~{} bytes, {:?}", name, memory_estimate, duration);
        }
    }
}

/// JIT compilation benchmarks
#[cfg(test)]
#[cfg(feature = "jit")]
mod jit_benchmarks {
    use super::*;

    #[test]
    fn benchmark_jit_compilation_time() {
        let test_cases = vec![
            ("simple_add", "let add = (a, b) => a + b; add(5, 3)"),
            ("recursive_factorial", r#"
                let factorial = (n) => if n <= 1 then 1 else n * factorial(n - 1);
                factorial(5)
            "#),
            ("mathematical_operations", r#"
                let compute = (x) => {
                    let a = x * x;
                    let b = sqrt(a);
                    sin(b) + cos(b)
                };
                compute(2.0)
            "#),
        ];

        for (name, source) in test_cases {
            benchmark_execution(&format!("jit_{}", name), 100, || {
                test_jit_compilation(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
        }
    }

    #[test]
    fn benchmark_jit_vs_interpreter_performance() {
        let performance_tests = vec![
            ("arithmetic_heavy", r#"
                let compute = (n) => {
                    let result = 0;
                    // Simulate arithmetic-heavy computation
                    result + n * n + n / 2 + n - 1
                };
                compute(1000)
            "#),
            ("function_call_heavy", r#"
                let helper = (x) => x + 1;
                let compute = (n) => helper(helper(helper(n)));
                compute(100)
            "#),
        ];

        for (name, source) in performance_tests {
            compare_performance(name, source, Some(source));
        }
    }
}

/// Comprehensive performance suite
#[cfg(test)]
mod comprehensive_benchmarks {
    use super::*;

    #[test]
    fn full_language_performance_suite() {
        println!("\n=== Matrix Language Performance Benchmark Suite ===");

        let comprehensive_tests = vec![
            ("fibonacci_recursive", r#"
                let fib = (n) => if n <= 1 then n else fib(n - 1) + fib(n - 2);
                fib(12)
            "#),
            ("matrix_computation", r#"
                let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
                matrix
            "#),
            ("nested_function_calls", r#"
                let f1 = (x) => x + 1;
                let f2 = (x) => f1(f1(x));
                let f3 = (x) => f2(f2(x));
                f3(0)
            "#),
            ("mathematical_computation", r#"
                let compute = (x) => {
                    let a = sqrt(x);
                    let b = sin(a);
                    let c = cos(b);
                    abs(c)
                };
                compute(100.0)
            "#),
        ];

        let mut total_duration = Duration::from_nanos(0);

        for (name, source) in comprehensive_tests {
            let duration = benchmark_execution(name, 100, || {
                execute(source).map_err(|e| format!("{:?}", e))?;
                Ok(())
            });
            total_duration += duration;
        }

        println!("Total benchmark suite duration: {:?}", total_duration);
        println!("Average per test: {:?}", total_duration / comprehensive_tests.len() as u32);
    }
}
