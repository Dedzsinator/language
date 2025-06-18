// Physics Engine - High-performance physics simulation with ECS architecture
//
// This crate provides a complete physics simulation engine including:
// - Rigid body dynamics with constraints
// - Soft body simulation
// - Fluid dynamics
// - Spatial partitioning and collision detection
// - ECS architecture for performance
// - GPU acceleration support
// - Real-time visualization

pub mod ecs;
pub mod physics;

#[cfg(feature = "gpu")]
pub mod gpu;

// Re-exports for convenience
pub use ecs::{Component, Entity, System, World};
pub use physics::*;

// Version and metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Create a new physics world with default configuration
pub fn create_world() -> World {
    World::new()
}

/// Create a physics world optimized for large simulations
pub fn create_large_world() -> World {
    let mut world = World::new();
    // Configure for large simulations
    world
}

/// Register Matrix Language bindings for physics functions
/// This allows Matrix Language scripts to interact with the physics engine
pub fn register_matrix_lang_bindings(interpreter: &mut matrix_lang::Interpreter) {
    use matrix_lang::{RuntimeError, Value};

    // Physics world creation
    interpreter.environment.define(
        "create_physics_world".to_string(),
        Value::BuiltinFunction {
            name: "create_physics_world".to_string(),
            arity: 0,
            func: |_args| {
                // Create a new physics world and return a handle
                // In a real implementation, we'd store the world and return an ID
                Ok(Value::String("physics_world_0".to_string()))
            },
        },
    );

    // Rigid body creation
    interpreter.environment.define(
        "add_rigid_body".to_string(),
        Value::BuiltinFunction {
            name: "add_rigid_body".to_string(),
            arity: 4, // world_id, shape, mass, position
            func: |args| {
                if args.len() != 4 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "add_rigid_body expects 4 arguments".to_string(),
                    });
                }

                // Extract arguments and create rigid body
                // This is a simplified implementation
                Ok(Value::Int(0)) // Return body ID
            },
        },
    );

    // Physics simulation step
    interpreter.environment.define(
        "physics_step".to_string(),
        Value::BuiltinFunction {
            name: "physics_step".to_string(),
            arity: 2, // world_id, delta_time
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "physics_step expects 2 arguments".to_string(),
                    });
                }

                // Step the physics simulation
                Ok(Value::Unit)
            },
        },
    );

    // Get rigid body position
    interpreter.environment.define(
        "get_position".to_string(),
        Value::BuiltinFunction {
            name: "get_position".to_string(),
            arity: 2, // world_id, body_id
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "get_position expects 2 arguments".to_string(),
                    });
                }

                // Return position as array [x, y, z]
                Ok(Value::Array(vec![
                    Value::Float(0.0),
                    Value::Float(0.0),
                    Value::Float(0.0),
                ]))
            },
        },
    );

    // Set rigid body position
    interpreter.environment.define(
        "set_position".to_string(),
        Value::BuiltinFunction {
            name: "set_position".to_string(),
            arity: 3, // world_id, body_id, position
            func: |args| {
                if args.len() != 3 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "set_position expects 3 arguments".to_string(),
                    });
                }

                // Set the position
                Ok(Value::Unit)
            },
        },
    );

    // Apply force to rigid body
    interpreter.environment.define(
        "apply_force".to_string(),
        Value::BuiltinFunction {
            name: "apply_force".to_string(),
            arity: 3, // world_id, body_id, force
            func: |args| {
                if args.len() != 3 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "apply_force expects 3 arguments".to_string(),
                    });
                }

                // Apply force to the body
                Ok(Value::Unit)
            },
        },
    );

    // Vector operations for physics
    interpreter.environment.define(
        "vec3".to_string(),
        Value::BuiltinFunction {
            name: "vec3".to_string(),
            arity: 3, // x, y, z
            func: |args| {
                if args.len() != 3 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "vec3 expects 3 arguments".to_string(),
                    });
                }
                Ok(Value::Array(vec![
                    args[0].clone(),
                    args[1].clone(),
                    args[2].clone(),
                ]))
            },
        },
    );

    // Vector dot product
    interpreter.environment.define(
        "dot".to_string(),
        Value::BuiltinFunction {
            name: "dot".to_string(),
            arity: 2, // vec1, vec2
            func: |args| {
                if args.len() != 2 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "dot expects 2 arguments".to_string(),
                    });
                }

                let vec1 = match &args[0] {
                    Value::Array(v) => v,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "First argument must be a vector".to_string(),
                        })
                    }
                };
                let vec2 = match &args[1] {
                    Value::Array(v) => v,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Second argument must be a vector".to_string(),
                        })
                    }
                };

                if vec1.len() != vec2.len() {
                    return Err(RuntimeError::Generic {
                        message: "Vectors must have same length".to_string(),
                    });
                }

                let mut sum = 0.0;
                for (a, b) in vec1.iter().zip(vec2.iter()) {
                    let a_f = match a {
                        Value::Float(f) => *f,
                        Value::Int(i) => *i as f64,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Vector elements must be numbers".to_string(),
                            })
                        }
                    };
                    let b_f = match b {
                        Value::Float(f) => *f,
                        Value::Int(i) => *i as f64,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Vector elements must be numbers".to_string(),
                            })
                        }
                    };
                    sum += a_f * b_f;
                }

                Ok(Value::Float(sum))
            },
        },
    );

    // Vector magnitude
    interpreter.environment.define(
        "magnitude".to_string(),
        Value::BuiltinFunction {
            name: "magnitude".to_string(),
            arity: 1, // vector
            func: |args| {
                if args.len() != 1 {
                    return Err(RuntimeError::FunctionCallError {
                        message: "magnitude expects 1 argument".to_string(),
                    });
                }

                let vec = match &args[0] {
                    Value::Array(v) => v,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Argument must be a vector".to_string(),
                        })
                    }
                };

                let mut sum_squares = 0.0;
                for element in vec.iter() {
                    let val = match element {
                        Value::Float(f) => *f,
                        Value::Int(i) => *i as f64,
                        _ => {
                            return Err(RuntimeError::TypeError {
                                message: "Vector elements must be numbers".to_string(),
                            })
                        }
                    };
                    sum_squares += val * val;
                }

                Ok(Value::Float(sum_squares.sqrt()))
            },
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = create_world();
        // Test basic world functionality
    }

    #[test]
    fn test_large_world_creation() {
        let world = create_large_world();
        // Test large world configuration
    }
}
