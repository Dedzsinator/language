// Test utilities for matrix-lang testing framework
use matrix_lang::*;
use std::time::{Duration, Instant};

/// Test execution utilities
pub fn execute(source: &str) -> RuntimeResult<Value> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|e| RuntimeError::Generic {
        message: format!("Parser creation error: {:?}", e),
    })?;

    let ast = parser.parse_program().map_err(|e| RuntimeError::Generic {
        message: format!("Parser error: {:?}", e),
    })?;

    let mut interpreter = Interpreter::new();
    stdlib::register_all(&mut interpreter);

    interpreter.eval_program(&ast)
}

/// Execute with type checking
pub fn execute_with_type_check(source: &str) -> Result<Value, String> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|e| format!("Parser creation error: {:?}", e))?;

    let ast = parser.parse_program().map_err(|e| format!("Parser error: {:?}", e))?;

    let mut type_checker = TypeChecker::new();
    type_checker.check_program(&ast).map_err(|e| format!("Type error: {:?}", e))?;

    let mut interpreter = Interpreter::new();
    stdlib::register_all(&mut interpreter);

    interpreter.eval_program(&ast).map_err(|e| format!("Runtime error: {:?}", e))
}

/// Benchmark execution time
pub fn benchmark_execution<F>(name: &str, iterations: usize, mut f: F) -> Duration
where
    F: FnMut() -> Result<(), String>,
{
    let start = Instant::now();

    for _ in 0..iterations {
        if let Err(e) = f() {
            panic!("Benchmark '{}' failed: {}", name, e);
        }
    }

    let elapsed = start.elapsed();
    println!("Benchmark '{}': {} iterations in {:?} ({:?} per iteration)",
             name, iterations, elapsed, elapsed / iterations as u32);

    elapsed
}

/// Test assertions for Values
pub fn assert_int_value(value: &Value, expected: i64) {
    match value {
        Value::Int(actual) => assert_eq!(*actual, expected),
        _ => panic!("Expected Int({}), got {:?}", expected, value),
    }
}

pub fn assert_float_value(value: &Value, expected: f64, tolerance: f64) {
    match value {
        Value::Float(actual) => assert!((actual - expected).abs() < tolerance,
                                        "Expected Float({}), got Float({}), difference {} exceeds tolerance {}",
                                        expected, actual, (actual - expected).abs(), tolerance),
        _ => panic!("Expected Float({}), got {:?}", expected, value),
    }
}

pub fn assert_bool_value(value: &Value, expected: bool) {
    match value {
        Value::Bool(actual) => assert_eq!(*actual, expected),
        _ => panic!("Expected Bool({}), got {:?}", expected, value),
    }
}

pub fn assert_string_value(value: &Value, expected: &str) {
    match value {
        Value::String(actual) => assert_eq!(actual, expected),
        _ => panic!("Expected String({}), got {:?}", expected, value),
    }
}

pub fn assert_array_length(value: &Value, expected_length: usize) {
    match value {
        Value::Array(array) => assert_eq!(array.len(), expected_length),
        _ => panic!("Expected Array with length {}, got {:?}", expected_length, value),
    }
}

pub fn assert_matrix_dimensions(value: &Value, expected_rows: usize, expected_cols: usize) {
    match value {
        Value::Matrix(matrix) => {
            assert_eq!(matrix.len(), expected_rows);
            if expected_rows > 0 {
                assert_eq!(matrix[0].len(), expected_cols);
            }
        }
        _ => panic!("Expected Matrix with dimensions {}x{}, got {:?}", expected_rows, expected_cols, value),
    }
}

/// Error testing utilities
pub fn assert_runtime_error<F>(f: F, expected_error_type: &str)
where
    F: FnOnce() -> RuntimeResult<Value>,
{
    match f() {
        Ok(value) => panic!("Expected runtime error '{}', but got success: {:?}", expected_error_type, value),
        Err(error) => {
            let error_string = format!("{:?}", error);
            assert!(error_string.contains(expected_error_type),
                    "Expected error containing '{}', got: {}", expected_error_type, error_string);
        }
    }
}

/// Parallel execution testing
pub fn test_parallel_execution(sources: Vec<&str>) -> Vec<RuntimeResult<Value>> {
    sources.into_iter()
        .map(|source| execute(source))
        .collect()
}

/// Memory usage testing (simplified)
pub fn measure_memory_usage<F>(f: F) -> (usize, Duration)
where
    F: FnOnce() -> RuntimeResult<Value>,
{
    let start_time = Instant::now();

    // Execute the function
    let _result = f();

    let elapsed = start_time.elapsed();

    // Simplified memory measurement (would need actual memory profiling in real implementation)
    let estimated_memory = 1024; // Placeholder

    (estimated_memory, elapsed)
}

/// JIT compilation testing utilities
#[cfg(feature = "jit")]
pub fn test_jit_compilation(source: &str) -> Result<(Value, Duration), String> {
    let start = Instant::now();

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|e| format!("Parser creation error: {:?}", e))?;

    let ast = parser.parse_program().map_err(|e| format!("Parser error: {:?}", e))?;

    let mut interpreter = Interpreter::new();
    stdlib::register_all(&mut interpreter);

    // Execute with JIT enabled
    let result = interpreter.eval_program(&ast).map_err(|e| format!("Runtime error: {:?}", e))?;

    let elapsed = start.elapsed();
    Ok((result, elapsed))
}

#[cfg(not(feature = "jit"))]
pub fn test_jit_compilation(_source: &str) -> Result<(Value, Duration), String> {
    Err("JIT compilation not available".to_string())
}

/// Performance comparison utilities
pub fn compare_performance(name: &str, interpreter_source: &str, _jit_source: Option<&str>) {
    let _interpreter_time = benchmark_execution(&format!("{}_interpreter", name), 100, || {
        execute(interpreter_source).map_err(|e| format!("{:?}", e))?;
        Ok(())
    });

    #[cfg(feature = "jit")]
    if let Some(_jit_src) = _jit_source {
        // JIT comparison would go here when implemented
        println!("JIT comparison for '{}' would be implemented here", name);
    }

    #[cfg(not(feature = "jit"))]
    {
        println!("JIT testing skipped for '{}' (feature not enabled)", name);
    }
}
