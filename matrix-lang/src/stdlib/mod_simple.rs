// Simplified stdlib for debugging
use crate::eval::{Interpreter, Value};

/// Register basic standard library functions with an interpreter
pub fn register_all(interpreter: &mut Interpreter) {
    // Simple function for testing
    interpreter.environment.define(
        "test".to_string(),
        Value::String("stdlib loaded".to_string()),
    );
}
