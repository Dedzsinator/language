use std::path::Path;
use std::process::Command;

/// Integration tests for Matrix Language comprehensive test suite
/// This integrates our Matrix Language tests into Rust's testing framework

#[test]
fn test_matrix_language_comprehensive_suite() {
    let result = run_matrix_test("tests/matrix-files/final_matrix_language_test_suite.matrix");
    assert!(
        result.success,
        "Matrix Language comprehensive test suite failed"
    );
}

#[test]
fn test_matrix_constants() {
    let result = run_matrix_test("tests/matrix-files/test_constants.matrix");
    assert!(result.success, "Matrix Language constants test failed");
}

#[test]
fn test_matrix_math_functions() {
    let result = run_matrix_test("tests/matrix-files/test_math_float.matrix");
    assert!(result.success, "Matrix Language math functions test failed");
}

#[test]
fn test_matrix_physics_system() {
    let result = run_matrix_test("tests/matrix-files/test_physics.matrix");
    assert!(result.success, "Matrix Language physics system test failed");
}

#[test]
fn test_matrix_quantum_computing() {
    let result = run_matrix_test("tests/matrix-files/test_quantum_fixed.matrix");
    assert!(
        result.success,
        "Matrix Language quantum computing test failed"
    );
}

#[test]
fn test_matrix_simple_features() {
    let result = run_matrix_test("tests/matrix-files/test_simple.matrix");
    assert!(
        result.success,
        "Matrix Language simple features test failed"
    );
}

#[test]
fn test_matrix_working_comprehensive() {
    let result = run_matrix_test("tests/matrix-files/working_comprehensive_test.matrix");
    assert!(
        result.success,
        "Matrix Language working comprehensive test failed"
    );
}

/// Test result structure
struct TestResult {
    success: bool,
    output: String,
}

/// Run a Matrix Language test file
fn run_matrix_test(test_file: &str) -> TestResult {
    // Get the project root directory (parent of matrix-lang)
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    // Build the matrix-lang binary path
    let binary_path = project_root
        .join("target")
        .join("debug")
        .join("matrix-lang");

    // Ensure the binary exists by building it first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "matrix-lang"])
        .current_dir(project_root)
        .output()
        .expect("Failed to build matrix-lang binary");

    if !build_output.status.success() {
        return TestResult {
            success: false,
            output: format!(
                "Failed to build matrix-lang: {}",
                String::from_utf8_lossy(&build_output.stderr)
            ),
        };
    }

    // Run the Matrix Language test
    let test_path = project_root.join(test_file);
    let output = Command::new(&binary_path)
        .arg(&test_path)
        .current_dir(project_root)
        .output()
        .expect("Failed to run matrix-lang test");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let _stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check if the test succeeded
    let success = output.status.success()
        && stdout.contains("✓ Type checking passed")
        && stdout.contains("✓ Execution completed successfully");

    TestResult {
        success,
        output: stdout,
    }
}

/// Test that all Matrix Language test files exist
#[test]
fn test_matrix_test_files_exist() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let test_files = vec![
        "tests/matrix-files/final_matrix_language_test_suite.matrix",
        "tests/matrix-files/test_constants.matrix",
        "tests/matrix-files/test_math_float.matrix",
        "tests/matrix-files/test_physics.matrix",
        "tests/matrix-files/test_quantum_fixed.matrix",
        "tests/matrix-files/test_simple.matrix",
        "tests/matrix-files/working_comprehensive_test.matrix",
    ];

    for test_file in test_files {
        let file_path = project_root.join(test_file);
        assert!(
            file_path.exists(),
            "Matrix Language test file does not exist: {}",
            test_file
        );
    }
}

/// Test that the test runner script exists and is executable
#[test]
fn test_matrix_test_runner_exists() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let runner_path = project_root.join("tests/run_all_tests.sh");

    assert!(
        runner_path.exists(),
        "Matrix Language test runner script does not exist"
    );

    // Check if it's executable (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&runner_path).expect("Failed to get file metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Test runner script is not executable"
        );
    }
}

/// Benchmark test to ensure Matrix Language tests complete in reasonable time
#[test]
fn test_matrix_performance_benchmark() {
    use std::time::Instant;

    let start = Instant::now();
    let result = run_matrix_test("tests/matrix-files/test_simple.matrix");
    let duration = start.elapsed();

    assert!(result.success, "Matrix Language simple test failed");
    assert!(
        duration.as_secs() < 30,
        "Matrix Language test took too long: {:?}",
        duration
    );

    println!("Matrix Language simple test completed in: {:?}", duration);
}

/// Test that validates the Matrix Language compiler produces expected output
#[test]
fn test_matrix_language_output_validation() {
    let result = run_matrix_test("tests/matrix-files/test_constants.matrix");

    assert!(result.success, "Matrix Language constants test failed");

    // Validate that constants are output correctly
    assert!(
        result.output.contains("3.141592653589793"),
        "PI constant not found in output"
    );
    assert!(
        result.output.contains("2.718281828459045"),
        "E constant not found in output"
    );
    assert!(
        result.output.contains("6.283185307179586"),
        "TAU constant not found in output"
    );
}

/// Test Matrix Language error handling
#[test]
fn test_matrix_language_error_handling() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let binary_path = project_root
        .join("target")
        .join("debug")
        .join("matrix-lang");

    // Build the binary first
    let _build_output = Command::new("cargo")
        .args(&["build", "--bin", "matrix-lang"])
        .current_dir(project_root)
        .output()
        .expect("Failed to build matrix-lang binary");

    // Test with non-existent file
    let output = Command::new(&binary_path)
        .arg("non_existent_file.matrix")
        .current_dir(project_root)
        .output()
        .expect("Failed to run matrix-lang with invalid file");

    // Should fail gracefully
    assert!(
        !output.status.success(),
        "Matrix Language should fail with non-existent file"
    );
}

/// Test that demonstrates Matrix Language features work correctly
#[test]
fn test_matrix_language_feature_validation() {
    let result = run_matrix_test("tests/matrix-files/final_matrix_language_test_suite.matrix");

    assert!(result.success, "Matrix Language comprehensive test failed");

    // Validate that all major features are tested
    assert!(
        result.output.contains("CONSTANTS AND BASIC MATH"),
        "Constants section not found"
    );
    assert!(
        result.output.contains("STRING FUNCTIONS"),
        "String functions section not found"
    );
    assert!(
        result.output.contains("PHYSICS SYSTEM"),
        "Physics system section not found"
    );
    assert!(
        result.output.contains("QUANTUM COMPUTING"),
        "Quantum computing section not found"
    );
    assert!(
        result.output.contains("LANGUAGE FEATURES"),
        "Language features section not found"
    );
    assert!(
        result
            .output
            .contains("Matrix Language comprehensive test suite PASSED!"),
        "Final success message not found"
    );
}
