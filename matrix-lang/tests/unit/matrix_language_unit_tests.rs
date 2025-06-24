//! Unit tests for Matrix Language functionality
//! These tests validate that the Matrix Language compiler and runtime work correctly

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the matrix-lang binary
fn get_matrix_lang_binary() -> PathBuf {
    // Get the project root (parent of matrix-lang)
    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap())
        .parent()
        .unwrap()
        .to_path_buf();

    let mut binary_path = project_root
        .join("target")
        .join("debug")
        .join("matrix-lang");

    // On Windows, add .exe extension
    if cfg!(windows) {
        binary_path.set_extension("exe");
    }

    binary_path
}

/// Run a Matrix Language source code string and return the output
fn run_matrix_code(code: &str) -> Result<String, String> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary file with the Matrix Language code
    let mut temp_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_file
        .write_all(code.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    let temp_path = temp_file.path();

    // Ensure matrix-lang binary exists
    let binary_path = get_matrix_lang_binary();
    if !binary_path.exists() {
        // Try to build it from the parent directory
        let project_root = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap())
            .parent()
            .unwrap()
            .to_path_buf();

        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "matrix-lang"])
            .current_dir(&project_root)
            .output()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;

        if !build_output.status.success() {
            return Err(format!(
                "Failed to build matrix-lang: {}",
                String::from_utf8_lossy(&build_output.stderr)
            ));
        }
    }

    // Run the Matrix Language code
    let output = Command::new(&binary_path)
        .arg(temp_path)
        .output()
        .map_err(|e| format!("Failed to run matrix-lang: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_matrix_language_constants() {
    let code = r#"
        let pi_test = println(pi)
        let e_test = println(e)
        let tau_test = println(tau)
    "#;

    let result = run_matrix_code(code).expect("Matrix Language constants test failed");

    assert!(result.contains("3.141592653589793"));
    assert!(result.contains("2.718281828459045"));
    assert!(result.contains("6.283185307179586"));
}

#[test]
fn test_matrix_language_math_functions() {
    let code = r#"
        let abs_result = println(abs(-42))
        let sqrt_result = println(sqrt(16.0))
        let sin_result = println(sin(0.0))
        let cos_result = println(cos(0.0))
    "#;

    let result = run_matrix_code(code).expect("Matrix Language math functions test failed");

    assert!(result.contains("42"));
    assert!(result.contains("4"));
    assert!(result.contains("0"));
    assert!(result.contains("1"));
}

#[test]
fn test_matrix_language_string_functions() {
    let code = r#"
        let str_int = println(str(42))
        let str_float = println(str(3.14))
        let str_bool = println(str(true))
    "#;

    let result = run_matrix_code(code).expect("Matrix Language string functions test failed");

    assert!(result.contains("42"));
    assert!(result.contains("3.14"));
    assert!(result.contains("true"));
}

#[test]
fn test_matrix_language_variables() {
    let code = r#"
        let x = 42
        let y = 3.14
        let z = true
        let w = "hello"
        let result1 = println(x)
        let result2 = println(y)
        let result3 = println(z)
        let result4 = println(w)
    "#;

    let result = run_matrix_code(code).expect("Matrix Language variables test failed");

    assert!(result.contains("42"));
    assert!(result.contains("3.14"));
    assert!(result.contains("true"));
    assert!(result.contains("hello"));
}

#[test]
fn test_matrix_language_arithmetic() {
    let code = r#"
        let add_result = println(10 + 5)
        let sub_result = println(10 - 5)
        let mul_result = println(10 * 5)
        let div_result = println(10.0 / 2.0)
    "#;

    let result = run_matrix_code(code).expect("Matrix Language arithmetic test failed");

    assert!(result.contains("15"));
    assert!(result.contains("5"));
    assert!(result.contains("50"));
    assert!(result.contains("5"));
}

#[test]
fn test_matrix_language_physics_functions() {
    let code = r#"
        let world = create_physics_world()
        let world_msg = println("Physics world created")
        let position = [0.0, 0.0, 0.0]
        let body = add_rigid_body(world, "sphere", 1.0, position)
        let body_msg = println("Rigid body added")
        let step_result = physics_step(world)
        let step_msg = println("Physics step completed")
    "#;

    let result = run_matrix_code(code).expect("Matrix Language physics test failed");

    assert!(result.contains("Physics world created"));
    assert!(result.contains("Rigid body added"));
    assert!(result.contains("Physics step completed"));
}

#[test]
fn test_matrix_language_quantum_functions() {
    let code = r#"
        let circuit = quantum_circuit(2)
        let circuit_msg = println("Quantum circuit created")
        let h_result = h(circuit, 0)
        let h_msg = println("Hadamard gate applied")
        let x_result = x(circuit, 1)
        let x_msg = println("Pauli-X gate applied")
        let cnot_result = cnot(circuit, 0, 1)
        let cnot_msg = println("CNOT gate applied")
    "#;

    let result = run_matrix_code(code).expect("Matrix Language quantum test failed");

    assert!(result.contains("Quantum circuit created"));
    assert!(result.contains("Hadamard gate applied"));
    assert!(result.contains("Pauli-X gate applied"));
    assert!(result.contains("CNOT gate applied"));
}

#[test]
fn test_matrix_language_type_checking() {
    let code = r#"
        let x = 42
        let y = 3.14
        let z = true
        let result = println("Type checking test")
    "#;

    let result = run_matrix_code(code).expect("Matrix Language type checking test failed");

    assert!(result.contains("Type checking test"));
}

#[test]
fn test_matrix_language_polymorphic_functions() {
    let code = r#"
        let int_print = println(42)
        let float_print = println(3.14)
        let bool_print = println(true)
        let string_print = println("hello")
    "#;

    let result = run_matrix_code(code).expect("Matrix Language polymorphic functions test failed");

    assert!(result.contains("42"));
    assert!(result.contains("3.14"));
    assert!(result.contains("true"));
    assert!(result.contains("hello"));
}

#[test]
fn test_matrix_language_error_handling() {
    let code = r#"
        let invalid_syntax =
    "#;

    let result = run_matrix_code(code);
    assert!(
        result.is_err(),
        "Matrix Language should fail with invalid syntax"
    );
}

/// Test that validates the Matrix Language compiler produces consistent output
#[test]
fn test_matrix_language_output_consistency() {
    let code = r#"
        let test = println(pi)
    "#;

    let result1 = run_matrix_code(code).expect("First run failed");
    let result2 = run_matrix_code(code).expect("Second run failed");

    // Extract just the meaningful output (excluding file paths)
    let extract_output = |s: &str| -> String {
        s.lines()
            .filter(|line| !line.starts_with("Compiling /tmp/"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let output1 = extract_output(&result1);
    let output2 = extract_output(&result2);

    assert_eq!(
        output1, output2,
        "Matrix Language output should be consistent"
    );
}

/// Performance test to ensure reasonable execution time
#[test]
fn test_matrix_language_performance() {
    use std::time::Instant;

    let code = r#"
        let x = 42
        let y = println(x)
    "#;

    let start = Instant::now();
    let _result = run_matrix_code(code).expect("Performance test failed");
    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 10,
        "Matrix Language execution took too long: {:?}",
        duration
    );
}
