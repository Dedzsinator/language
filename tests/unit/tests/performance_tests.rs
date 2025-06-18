use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::TypeChecker;
use crate::interpreter::Interpreter;
use super::test_utilities::*;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_tests {
    use super::*;

    // Helper function to measure execution time
    fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    // Helper function to run complete pipeline with timing
    fn run_timed_pipeline(source: &str) -> (Result<crate::interpreter::Value, Box<dyn std::error::Error>>, Duration) {
        measure_time(|| {
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize()?;
            
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            
            let mut type_checker = TypeChecker::new();
            let _type_result = type_checker.check(&ast)?;
            
            let mut interpreter = Interpreter::new();
            let result = interpreter.interpret(&ast)?;
            
            Ok(result)
        })
    }

    #[test]
    fn test_lexer_performance_large_input() {
        // Generate large input
        let mut large_source = String::new();
        for i in 0..1000 {
            large_source.push_str(&format!("let var{} = {};\n", i, i));
        }
        
        let (result, duration) = measure_time(|| {
            let mut lexer = Lexer::new(&large_source);
            lexer.tokenize()
        });
        
        assert!(result.is_ok());
        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration < Duration::from_millis(100));
        
        // Verify token count is reasonable
        let tokens = result.unwrap();
        assert!(tokens.len() > 3000); // Each variable declaration produces multiple tokens
    }

    #[test]
    fn test_parser_performance_deep_nesting() {
        // Generate deeply nested expressions
        let mut nested_expr = String::from("1");
        for i in 2..=100 {
            nested_expr = format!("({} + {})", nested_expr, i);
        }
        
        let source = format!("let result = {}; result", nested_expr);
        
        let (result, duration) = measure_time(|| {
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse()
        });
        
        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(50));
    }

    #[test]
    fn test_type_checker_performance_complex_types() {
        let source = r#"
            struct Point<T> {
                x: T,
                y: T
            }
            
            struct Line<T> {
                start: Point<T>,
                end: Point<T>
            }
            
            struct Polygon<T> {
                vertices: [Point<T>]
            }
            
            fn create_complex_structure() -> Polygon<float> {
                let points = [
                    Point { x: 1.0, y: 2.0 },
                    Point { x: 3.0, y: 4.0 },
                    Point { x: 5.0, y: 6.0 },
                    Point { x: 7.0, y: 8.0 },
                    Point { x: 9.0, y: 10.0 }
                ];
                
                Polygon { vertices: points }
            }
            
            let polygon = create_complex_structure();
            polygon.vertices[0].x
        "#;
        
        let (result, duration) = measure_time(|| {
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse().unwrap();
            let mut type_checker = TypeChecker::new();
            type_checker.check(&ast)
        });
        
        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    fn test_interpreter_performance_recursive_functions() {
        let source = r#"
            fn fibonacci(n: int) -> int {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
            
            fibonacci(20)
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::interpreter::Value::Int(6765)); // fib(20)
        // Recursive fibonacci is inherently slow, but should complete within reasonable time
        assert!(duration < Duration::from_secs(1));
    }

    #[test]
    fn test_interpreter_performance_iterative_algorithms() {
        let source = r#"
            fn factorial_iterative(n: int) -> int {
                let mut result = 1;
                for i in 1..=n {
                    result = result * i;
                }
                result
            }
            
            fn sum_of_squares(n: int) -> int {
                let mut sum = 0;
                for i in 1..=n {
                    sum = sum + i * i;
                }
                sum
            }
            
            let fact = factorial_iterative(20);
            let squares = sum_of_squares(1000);
            
            fact % 1000000 + squares % 1000000
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Should complete quickly for iterative algorithms
        assert!(duration < Duration::from_millis(50));
    }

    #[test]
    fn test_large_array_operations_performance() {
        let source = r#"
            fn create_large_array(size: int) -> [int] {
                let mut arr = [];
                for i in 0..size {
                    arr = push(arr, i);
                }
                arr
            }
            
            fn sum_array(arr: [int]) -> int {
                let mut sum = 0;
                for item in arr {
                    sum = sum + item;
                }
                sum
            }
            
            fn map_array(arr: [int], f: fn(int) -> int) -> [int] {
                let mut result = [];
                for item in arr {
                    result = push(result, f(item));
                }
                result
            }
            
            let large_array = create_large_array(1000);
            let doubled = map_array(large_array, |x| x * 2);
            let sum = sum_array(doubled);
            
            sum % 1000000
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Large array operations should still complete in reasonable time
        assert!(duration < Duration::from_millis(500));
    }

    #[test]
    fn test_matrix_operations_performance() {
        let source = r#"
            fn create_matrix(rows: int, cols: int, init_val: float) -> [[float]] {
                let mut matrix = [];
                for i in 0..rows {
                    let mut row = [];
                    for j in 0..cols {
                        row = push(row, init_val + i as float + j as float);
                    }
                    matrix = push(matrix, row);
                }
                matrix
            }
            
            fn matrix_multiply(a: [[float]], b: [[float]]) -> [[float]] {
                let rows_a = len(a);
                let cols_a = len(a[0]);
                let cols_b = len(b[0]);
                
                let mut result = [];
                for i in 0..rows_a {
                    let mut row = [];
                    for j in 0..cols_b {
                        let mut sum = 0.0;
                        for k in 0..cols_a {
                            sum = sum + a[i][k] * b[k][j];
                        }
                        row = push(row, sum);
                    }
                    result = push(result, row);
                }
                result
            }
            
            let matrix_a = create_matrix(10, 10, 1.0);
            let matrix_b = create_matrix(10, 10, 2.0);
            let product = matrix_multiply(matrix_a, matrix_b);
            
            product[0][0]
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Matrix multiplication should complete within reasonable time
        assert!(duration < Duration::from_millis(200));
    }

    #[test]
    fn test_string_operations_performance() {
        let source = r#"
            fn create_long_string(n: int) -> string {
                let mut result = "";
                for i in 0..n {
                    result = result + "x";
                }
                result
            }
            
            fn string_processing(s: string) -> string {
                let mut result = s;
                for _ in 0..10 {
                    result = result + result;
                }
                result
            }
            
            let base_string = create_long_string(100);
            let processed = string_processing(base_string);
            
            len(processed)
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // String operations can be expensive, but should still complete
        assert!(duration < Duration::from_millis(300));
    }

    #[test]
    fn test_complex_control_flow_performance() {
        let source = r#"
            fn sieve_of_eratosthenes(limit: int) -> [int] {
                let mut is_prime = [];
                for i in 0..=limit {
                    is_prime = push(is_prime, true);
                }
                is_prime[0] = false;
                is_prime[1] = false;
                
                for i in 2..=limit {
                    if is_prime[i] {
                        let mut j = i * i;
                        while j <= limit {
                            is_prime[j] = false;
                            j = j + i;
                        }
                    }
                }
                
                let mut primes = [];
                for i in 2..=limit {
                    if is_prime[i] {
                        primes = push(primes, i);
                    }
                }
                
                primes
            }
            
            let primes = sieve_of_eratosthenes(1000);
            len(primes)
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Sieve algorithm should complete in reasonable time
        assert!(duration < Duration::from_millis(300));
        
        // Should find 168 primes up to 1000
        if let Ok(crate::interpreter::Value::Int(count)) = result {
            assert_eq!(count, 168);
        }
    }

    #[test]
    fn test_nested_function_calls_performance() {
        let source = r#"
            fn level_5(n: int) -> int { n + 5 }
            fn level_4(n: int) -> int { level_5(n) + 4 }
            fn level_3(n: int) -> int { level_4(n) + 3 }
            fn level_2(n: int) -> int { level_3(n) + 2 }
            fn level_1(n: int) -> int { level_2(n) + 1 }
            
            fn deep_recursion_test() -> int {
                let mut sum = 0;
                for i in 0..1000 {
                    sum = sum + level_1(i);
                }
                sum
            }
            
            deep_recursion_test()
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Nested function calls should not cause significant performance degradation
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    fn test_memory_intensive_operations_performance() {
        let source = r#"
            struct LargeStruct {
                data: [float],
                metadata: string,
                id: int
            }
            
            fn create_large_structures(count: int) -> [LargeStruct] {
                let mut structures = [];
                for i in 0..count {
                    let mut data = [];
                    for j in 0..100 {
                        data = push(data, i as float + j as float);
                    }
                    
                    let structure = LargeStruct {
                        data: data,
                        metadata: "Structure " + str(i),
                        id: i
                    };
                    
                    structures = push(structures, structure);
                }
                structures
            }
            
            fn process_structures(structures: [LargeStruct]) -> float {
                let mut total = 0.0;
                for structure in structures {
                    for value in structure.data {
                        total = total + value;
                    }
                }
                total
            }
            
            let structures = create_large_structures(10);
            let result = process_structures(structures);
            
            result % 10000.0
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        // Memory-intensive operations should complete without excessive time
        assert!(duration < Duration::from_millis(400));
    }

    #[test]
    fn test_compilation_phases_individual_performance() {
        let complex_source = generate_complex_program();
        
        // Test lexing performance
        let (tokens, lex_duration) = measure_time(|| {
            let mut lexer = Lexer::new(&complex_source);
            lexer.tokenize().unwrap()
        });
        
        // Test parsing performance
        let (ast, parse_duration) = measure_time(|| {
            let mut parser = Parser::new(tokens.clone());
            parser.parse().unwrap()
        });
        
        // Test type checking performance
        let (_, type_check_duration) = measure_time(|| {
            let mut type_checker = TypeChecker::new();
            type_checker.check(&ast).unwrap()
        });
        
        // Test interpretation performance
        let (_, interpret_duration) = measure_time(|| {
            let mut interpreter = Interpreter::new();
            interpreter.interpret(&ast).unwrap()
        });
        
        // Individual phases should complete within reasonable times
        assert!(lex_duration < Duration::from_millis(50));
        assert!(parse_duration < Duration::from_millis(100));
        assert!(type_check_duration < Duration::from_millis(150));
        assert!(interpret_duration < Duration::from_millis(200));
        
        let total_duration = lex_duration + parse_duration + type_check_duration + interpret_duration;
        assert!(total_duration < Duration::from_millis(500));
    }

    #[test]
    fn test_scalability_with_program_size() {
        // Test how performance scales with program size
        let sizes = vec![100, 500, 1000];
        let mut durations = Vec::new();
        
        for size in sizes {
            let source = generate_program_of_size(size);
            let (_, duration) = run_timed_pipeline(&source);
            durations.push(duration);
        }
        
        // Performance should scale reasonably (not exponentially)
        // This is a loose check - in practice, you might want more sophisticated analysis
        for duration in durations {
            assert!(duration < Duration::from_secs(2));
        }
    }

    #[test]
    fn test_physics_simulation_performance() {
        let source = r#"
            struct Particle {
                position: vec3,
                velocity: vec3,
                mass: float
            }
            
            fn update_particle(p: Particle, dt: float, forces: vec3) -> Particle {
                let acceleration = forces / p.mass;
                let new_velocity = p.velocity + acceleration * dt;
                let new_position = p.position + new_velocity * dt;
                
                Particle {
                    position: new_position,
                    velocity: new_velocity,
                    mass: p.mass
                }
            }
            
            fn simulate_system(particles: [Particle], steps: int, dt: float) -> [Particle] {
                let mut current_particles = particles;
                let gravity = [0.0, -9.8, 0.0];
                
                for _ in 0..steps {
                    let mut updated_particles = [];
                    for particle in current_particles {
                        let updated = update_particle(particle, dt, gravity);
                        updated_particles = push(updated_particles, updated);
                    }
                    current_particles = updated_particles;
                }
                
                current_particles
            }
            
            // Create 50 particles
            let mut particles = [];
            for i in 0..50 {
                let particle = Particle {
                    position: [i as float, 10.0, 0.0],
                    velocity: [0.0, 0.0, 0.0],
                    mass: 1.0
                };
                particles = push(particles, particle);
            }
            
            // Simulate for 100 steps
            let final_particles = simulate_system(particles, 100, 0.01);
            
            len(final_particles)
        "#;
        
        let (result, duration) = run_timed_pipeline(source);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), crate::interpreter::Value::Int(50));
        // Physics simulation should complete in reasonable time
        assert!(duration < Duration::from_millis(300));
    }

    // Helper function to generate a complex program for testing
    fn generate_complex_program() -> String {
        let mut program = String::new();
        
        // Add struct definitions
        program.push_str(r#"
            struct Vector3 {
                x: float,
                y: float,
                z: float
            }
            
            struct Matrix3 {
                data: [[float]]
            }
        "#);
        
        // Add function definitions
        program.push_str(r#"
            fn vector_add(a: Vector3, b: Vector3) -> Vector3 {
                Vector3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
            }
            
            fn vector_scale(v: Vector3, s: float) -> Vector3 {
                Vector3 { x: v.x * s, y: v.y * s, z: v.z * s }
            }
            
            fn complex_calculation(n: int) -> float {
                let mut sum = 0.0;
                for i in 0..n {
                    sum = sum + sqrt(i as float);
                }
                sum
            }
        "#);
        
        // Add main computation
        program.push_str(r#"
            let v1 = Vector3 { x: 1.0, y: 2.0, z: 3.0 };
            let v2 = Vector3 { x: 4.0, y: 5.0, z: 6.0 };
            let result = vector_add(v1, vector_scale(v2, 2.0));
            let calc_result = complex_calculation(100);
            
            result.x + calc_result % 100.0
        "#);
        
        program
    }

    // Helper function to generate a program of specific size
    fn generate_program_of_size(size: usize) -> String {
        let mut program = String::new();
        
        // Generate variable declarations
        for i in 0..size {
            program.push_str(&format!("let var{} = {};\n", i, i % 100));
        }
        
        // Generate a function that uses all variables
        program.push_str("fn sum_all() -> int {\n    let mut sum = 0;\n");
        for i in 0..size {
            program.push_str(&format!("    sum = sum + var{};\n", i));
        }
        program.push_str("    sum\n}\n");
        
        // Call the function
        program.push_str("sum_all()");
        
        program
    }
}
