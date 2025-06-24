// Matrix Language - JIT-compiled physics simulation language
//
// This crate provides the Matrix Language implementation including:
// - Lexical analysis and parsing
// - Type checking and inference
// - AST-based interpretation
// - LLVM-based JIT compilation
// - Standard library with mathematical functions
// - Physics simulation integration

pub mod ast;
pub mod eval;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod types;

#[cfg(feature = "jit")]
pub mod jit;

#[cfg(test)]
pub mod debug_tests;

// Re-exports for convenience
pub use ast::*;
pub use eval::{Interpreter, RuntimeError, RuntimeResult, Value};
pub use lexer::Lexer;
pub use parser::Parser;
pub use types::TypeChecker;

#[cfg(feature = "jit")]
pub use jit::{JitContext, JitError, JitStats};

// Version and metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Initialize the Matrix Language runtime with default configuration
pub fn init() -> Interpreter {
    let mut interpreter = Interpreter::new();

    // Register standard library functions
    stdlib::register_all(&mut interpreter);

    interpreter
}

/// Convenience function to execute Matrix Language source code
pub fn execute(source: &str) -> RuntimeResult<Value> {
    let mut interpreter = init();

    // Lexical analysis
    let lexer = Lexer::new(source);

    // Parsing
    let mut parser = Parser::new(lexer).map_err(|e| RuntimeError::Generic {
        message: format!("Parser initialization failed: {}", e),
    })?;

    let ast = parser.parse_program().map_err(|e| RuntimeError::Generic {
        message: format!("Parse error: {}", e),
    })?;

    // Type checking (optional, can fail gracefully)
    let mut type_checker = TypeChecker::new();
    let _ = type_checker.check_program(&ast); // Don't fail on type errors for now

    // Execution
    interpreter.eval_program(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let result = execute("let x = 5 + 3; x").unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_function_definition() {
        let source = r#"
            let add = (a: Int, b: Int) -> Int => a + b;
            add(10, 20)
        "#;
        let result = execute(source).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_matrix_operations() {
        let source = r#"
            let m = [[1, 2], [3, 4]];
            m
        "#;
        let result = execute(source);
        assert!(result.is_ok());
    }
}
