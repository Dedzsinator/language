// Standard library implementation for Matrix Language
// Contains essential math, physics, and utility functions

use crate::eval::interpreter::{RuntimeError, RuntimeResult, Value};
use crate::physics::math::*;
use std::collections::HashMap;

/// Standard library functions available in Matrix Language
pub struct StandardLibrary {
    functions: HashMap<String, fn(&[Value]) -> RuntimeResult<Value>>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = Self {
            functions: HashMap::new(),
        };

        stdlib.register_math_functions();
        stdlib.register_physics_functions();
        stdlib.register_utility_functions();
        stdlib.register_vector_functions();

        stdlib
    }

    pub fn get_function(&self, name: &str) -> Option<&fn(&[Value]) -> RuntimeResult<Value>> {
        self.functions.get(name)
    }

    pub fn call_function(&self, name: &str, args: &[Value]) -> RuntimeResult<Value> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(RuntimeError::FunctionCallError {
                message: format!("Undefined function: {}", name),
            })
        }
    }

    /// Register essential math functions
    fn register_math_functions(&mut self) {
        // Basic math
        self.functions.insert("abs".to_string(), stdlib_abs);
        self.functions.insert("sqrt".to_string(), stdlib_sqrt);
        self.functions.insert("cbrt".to_string(), stdlib_cbrt);
        self.functions.insert("pow".to_string(), stdlib_pow);
        self.functions.insert("exp".to_string(), stdlib_exp);
        self.functions.insert("ln".to_string(), stdlib_ln);
        self.functions.insert("log10".to_string(), stdlib_log10);
        self.functions.insert("log2".to_string(), stdlib_log2);

        // Trigonometric functions
        self.functions.insert("sin".to_string(), stdlib_sin);
        self.functions.insert("cos".to_string(), stdlib_cos);
        self.functions.insert("tan".to_string(), stdlib_tan);
        self.functions.insert("asin".to_string(), stdlib_asin);
        self.functions.insert("acos".to_string(), stdlib_acos);
        self.functions.insert("atan".to_string(), stdlib_atan);
        self.functions.insert("atan2".to_string(), stdlib_atan2);

        // Hyperbolic functions
        self.functions.insert("sinh".to_string(), stdlib_sinh);
        self.functions.insert("cosh".to_string(), stdlib_cosh);
        self.functions.insert("tanh".to_string(), stdlib_tanh);

        // Rounding and comparison
        self.functions.insert("floor".to_string(), stdlib_floor);
        self.functions.insert("ceil".to_string(), stdlib_ceil);
        self.functions.insert("round".to_string(), stdlib_round);
        self.functions.insert("min".to_string(), stdlib_min);
        self.functions.insert("max".to_string(), stdlib_max);
        self.functions.insert("clamp".to_string(), stdlib_clamp);

        // Constants
        self.functions
            .insert("pi".to_string(), |_| Ok(Value::Float(std::f64::consts::PI)));
        self.functions
            .insert("e".to_string(), |_| Ok(Value::Float(std::f64::consts::E)));
        self.functions.insert("tau".to_string(), |_| {
            Ok(Value::Float(std::f64::consts::TAU))
        });
    }

    /// Register physics-specific functions
    fn register_physics_functions(&mut self) {
        self.functions.insert("vec3".to_string(), stdlib_vec3);
        self.functions
            .insert("magnitude".to_string(), stdlib_magnitude);
        self.functions
            .insert("normalize".to_string(), stdlib_normalize);
        self.functions.insert("dot".to_string(), stdlib_dot);
        self.functions.insert("cross".to_string(), stdlib_cross);
        self.functions
            .insert("distance".to_string(), stdlib_distance);
        self.functions.insert("lerp".to_string(), stdlib_lerp);
        self.functions.insert("reflect".to_string(), stdlib_reflect);

        // Physics calculations
        self.functions
            .insert("kinetic_energy".to_string(), stdlib_kinetic_energy);
        self.functions
            .insert("potential_energy".to_string(), stdlib_potential_energy);
        self.functions
            .insert("momentum".to_string(), stdlib_momentum);
        self.functions.insert("force".to_string(), stdlib_force);
        self.functions
            .insert("acceleration".to_string(), stdlib_acceleration);
        self.functions.insert(
            "velocity_from_acceleration".to_string(),
            stdlib_velocity_from_acceleration,
        );

        // Gravitational physics
        self.functions.insert(
            "gravitational_force".to_string(),
            stdlib_gravitational_force,
        );
        self.functions
            .insert("escape_velocity".to_string(), stdlib_escape_velocity);
        self.functions
            .insert("orbital_velocity".to_string(), stdlib_orbital_velocity);
    }

    /// Register utility functions
    fn register_utility_functions(&mut self) {
        self.functions.insert("print".to_string(), stdlib_print);
        self.functions.insert("println".to_string(), stdlib_println);
        self.functions.insert("type_of".to_string(), stdlib_type_of);
        self.functions
            .insert("to_string".to_string(), stdlib_to_string);
        self.functions.insert("random".to_string(), stdlib_random);
        self.functions
            .insert("random_range".to_string(), stdlib_random_range);
        self.functions.insert("time".to_string(), stdlib_time);
    }

    /// Register vector-specific functions
    fn register_vector_functions(&mut self) {
        self.functions
            .insert("array_sum".to_string(), stdlib_array_sum);
        self.functions
            .insert("array_avg".to_string(), stdlib_array_avg);
        self.functions
            .insert("array_length".to_string(), stdlib_array_length);
        self.functions
            .insert("array_push".to_string(), stdlib_array_push);
        self.functions
            .insert("array_pop".to_string(), stdlib_array_pop);
        self.functions
            .insert("array_reverse".to_string(), stdlib_array_reverse);
        self.functions
            .insert("array_sort".to_string(), stdlib_array_sort);
    }
}

// Implementation of math functions
fn stdlib_abs(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("abs expects 1 argument, got {}", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(RuntimeError::TypeError {
            message: format!("Cannot get absolute value of {}", args[0].type_name()),
        }),
    }
}

fn stdlib_sqrt(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("sqrt expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!("Cannot get square root of {}", args[0].type_name()),
            })
        }
    };

    if num < 0.0 {
        return Err(RuntimeError::Generic {
            message: "sqrt of negative number".to_string(),
        });
    }

    Ok(Value::Float(num.sqrt()))
}

fn stdlib_cbrt(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("cbrt expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!("Cannot get cube root of {}", args[0].type_name()),
            })
        }
    };

    Ok(Value::Float(num.cbrt()))
}

fn stdlib_pow(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("pow expects 2 arguments, got {}", args.len()),
        });
    }

    let base = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid base type for pow, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let exp = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid exponent type for pow, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(base.powf(exp)))
}

fn stdlib_exp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("exp expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for exp, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.exp()))
}

fn stdlib_ln(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("ln expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for ln, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::Generic {
            message: "ln of non-positive number".to_string(),
        });
    }

    Ok(Value::Float(num.ln()))
}

fn stdlib_log10(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("log10 expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for log10, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::Generic {
            message: "log10 of non-positive number".to_string(),
        });
    }

    Ok(Value::Float(num.log10()))
}

fn stdlib_log2(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("log2 expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for log2, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::Generic {
            message: "log2 of non-positive number".to_string(),
        });
    }

    Ok(Value::Float(num.log2()))
}

// Trigonometric functions
fn stdlib_sin(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("sin expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for sin, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.sin()))
}

fn stdlib_cos(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("cos expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for cos, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.cos()))
}

fn stdlib_tan(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("tan expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for tan, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.tan()))
}

fn stdlib_asin(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("asin expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for asin, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    if num < -1.0 || num > 1.0 {
        return Err(RuntimeError::Generic {
            message: "asin input must be in [-1, 1]".to_string(),
        });
    }

    Ok(Value::Float(num.asin()))
}

fn stdlib_acos(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("acos expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for acos, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    if num < -1.0 || num > 1.0 {
        return Err(RuntimeError::Generic {
            message: "acos input must be in [-1, 1]".to_string(),
        });
    }

    Ok(Value::Float(num.acos()))
}

fn stdlib_atan(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("atan expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for atan, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.atan()))
}

fn stdlib_atan2(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("atan2 expects 2 arguments, got {}", args.len()),
        });
    }

    let y = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for atan2, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let x = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for atan2, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(y.atan2(x)))
}

// Hyperbolic functions
fn stdlib_sinh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("sinh expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for sinh, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.sinh()))
}

fn stdlib_cosh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("cosh expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for cosh, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.cosh()))
}

fn stdlib_tanh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("tanh expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for tanh, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.tanh()))
}

// Rounding and comparison functions
fn stdlib_floor(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("floor expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => return Ok(Value::Int(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for floor, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.floor()))
}

fn stdlib_ceil(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("ceil expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => return Ok(Value::Int(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for ceil, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.ceil()))
}

fn stdlib_round(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("round expects 1 argument, got {}", args.len()),
        });
    }

    let num = match &args[0] {
        Value::Int(n) => return Ok(Value::Int(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for round, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(num.round()))
}

fn stdlib_min(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("min expects 2 arguments, got {}", args.len()),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
        _ => Err(RuntimeError::TypeError {
            message: "Invalid argument types for min, expected numbers".to_string(),
        }),
    }
}

fn stdlib_max(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("max expects 2 arguments, got {}", args.len()),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
        _ => Err(RuntimeError::TypeError {
            message: "Invalid argument types for max, expected numbers".to_string(),
        }),
    }
}

fn stdlib_clamp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("clamp expects 3 arguments, got {}", args.len()),
        });
    }

    let value = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for clamp, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let min_val = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for clamp, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    let max_val = match &args[2] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for clamp, expected number, got {}",
                    args[2].type_name()
                ),
            })
        }
    };

    Ok(Value::Float(value.clamp(min_val, max_val)))
}

// Physics and vector functions
fn stdlib_vec3(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("vec3 expects 3 arguments, got {}", args.len()),
        });
    }

    let x = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for vec3, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let y = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for vec3, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    let z = match &args[2] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for vec3, expected number, got {}",
                    args[2].type_name()
                ),
            })
        }
    };

    Ok(Value::Array(vec![
        Value::Float(x),
        Value::Float(y),
        Value::Float(z),
    ]))
}

// Placeholder implementations for physics functions
fn stdlib_magnitude(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("magnitude expects 1 argument, got {}", args.len()),
        });
    }

    let vec = extract_vec3(&args[0])?;
    Ok(Value::Float(vec.magnitude()))
}

fn stdlib_normalize(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("normalize expects 1 argument, got {}", args.len()),
        });
    }

    let vec = extract_vec3(&args[0])?;
    let normalized = vec.normalize();
    Ok(Value::Array(vec![
        Value::Float(normalized.x),
        Value::Float(normalized.y),
        Value::Float(normalized.z),
    ]))
}

fn stdlib_dot(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("dot expects 2 arguments, got {}", args.len()),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    Ok(Value::Float(vec1.dot(&vec2)))
}

fn stdlib_cross(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("cross expects 2 arguments, got {}", args.len()),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    let result = vec1.cross(&vec2);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_distance(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("distance expects 2 arguments, got {}", args.len()),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    let diff = vec1 - vec2;
    Ok(Value::Float(diff.magnitude()))
}

fn stdlib_lerp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("lerp expects 3 arguments, got {}", args.len()),
        });
    }

    let start = extract_vec3(&args[0])?;
    let end = extract_vec3(&args[1])?;
    let t = match &args[2] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for lerp, expected number, got {}",
                    args[2].type_name()
                ),
            })
        }
    };

    let result = start.lerp(&end, t);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_reflect(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("reflect expects 2 arguments, got {}", args.len()),
        });
    }

    let incident = extract_vec3(&args[0])?;
    let normal = extract_vec3(&args[1])?;
    let result = incident.reflect(&normal);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

// Physics calculation functions
fn stdlib_kinetic_energy(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("kinetic_energy expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for kinetic_energy, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let velocity = extract_vec3(&args[1])?;
    let speed_squared = velocity.magnitude_squared();
    Ok(Value::Float(0.5 * mass * speed_squared))
}

fn stdlib_potential_energy(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("potential_energy expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for potential_energy, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let height = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for potential_energy, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    const G: f64 = 9.81; // Earth gravity
    Ok(Value::Float(mass * G * height))
}

fn stdlib_momentum(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("momentum expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for momentum, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let velocity = extract_vec3(&args[1])?;
    let result = velocity * mass;
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_force(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("force expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for force, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let acceleration = extract_vec3(&args[1])?;
    let result = acceleration * mass;
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_acceleration(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("acceleration expects 2 arguments, got {}", args.len()),
        });
    }

    let force = extract_vec3(&args[0])?;
    let mass = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for acceleration, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    if mass == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    let result = force / mass;
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_velocity_from_acceleration(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::FunctionCallError {
            message: format!(
                "velocity_from_acceleration expects 3 arguments, got {}",
                args.len()
            ),
        });
    }

    let initial_velocity = extract_vec3(&args[0])?;
    let acceleration = extract_vec3(&args[1])?;
    let time =
        match &args[2] {
            Value::Int(n) => *n as f64,
            Value::Float(f) => *f,
            _ => return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for velocity_from_acceleration, expected number, got {}",
                    args[2].type_name()
                ),
            }),
        };

    let result = initial_velocity + (acceleration * time);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

// Gravitational functions
fn stdlib_gravitational_force(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::FunctionCallError {
            message: format!(
                "gravitational_force expects 3 arguments, got {}",
                args.len()
            ),
        });
    }

    let mass1 = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for gravitational_force, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let mass2 = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for gravitational_force, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    let distance = match &args[2] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for gravitational_force, expected number, got {}",
                    args[2].type_name()
                ),
            })
        }
    };

    if distance == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    const G: f64 = 6.67430e-11; // Gravitational constant
    let force = G * mass1 * mass2 / (distance * distance);
    Ok(Value::Float(force))
}

fn stdlib_escape_velocity(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("escape_velocity expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for escape_velocity, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let radius = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for escape_velocity, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    if radius == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    const G: f64 = 6.67430e-11;
    let escape_vel = (2.0 * G * mass / radius).sqrt();
    Ok(Value::Float(escape_vel))
}

fn stdlib_orbital_velocity(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("orbital_velocity expects 2 arguments, got {}", args.len()),
        });
    }

    let mass = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for orbital_velocity, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let radius = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for orbital_velocity, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    if radius == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }

    const G: f64 = 6.67430e-11;
    let orbital_vel = (G * mass / radius).sqrt();
    Ok(Value::Float(orbital_vel))
}

// Utility functions
fn stdlib_print(args: &[Value]) -> RuntimeResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg.to_string());
    }
    Ok(Value::String("".to_string()))
}

fn stdlib_println(args: &[Value]) -> RuntimeResult<Value> {
    stdlib_print(args)?;
    println!();
    Ok(Value::String("".to_string()))
}

fn stdlib_type_of(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("type_of expects 1 argument, got {}", args.len()),
        });
    }

    let type_name = match &args[0] {
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "array",
        Value::Function { .. } => "function",
        Value::Unit => "unit",
        _ => "unknown",
    };

    Ok(Value::String(type_name.to_string()))
}

fn stdlib_to_string(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("to_string expects 1 argument, got {}", args.len()),
        });
    }

    Ok(Value::String(args[0].to_string()))
}

fn stdlib_random(_args: &[Value]) -> RuntimeResult<Value> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Float(rng.gen::<f64>()))
}

fn stdlib_random_range(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("random_range expects 2 arguments, got {}", args.len()),
        });
    }

    let min = match &args[0] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for random_range, expected number, got {}",
                    args[0].type_name()
                ),
            })
        }
    };

    let max = match &args[1] {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                message: format!(
                    "Invalid argument type for random_range, expected number, got {}",
                    args[1].type_name()
                ),
            })
        }
    };

    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Float(rng.gen_range(min..max)))
}

fn stdlib_time(_args: &[Value]) -> RuntimeResult<Value> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Ok(Value::Float(now))
}

// Array functions
fn stdlib_array_sum(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_sum expects 1 argument, got {}", args.len()),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    let mut sum = 0.0;
    for item in arr {
        match item {
            Value::Int(n) => sum += *n as f64,
            Value::Float(f) => sum += f,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Expected number array".to_string(),
                })
            }
        }
    }

    Ok(Value::Float(sum))
}

fn stdlib_array_avg(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_avg expects 1 argument, got {}", args.len()),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    if arr.is_empty() {
        return Err(RuntimeError::Generic {
            message: "Cannot get average of empty array".to_string(),
        });
    }

    let sum = stdlib_array_sum(args)?;
    match sum {
        Value::Float(s) => Ok(Value::Float(s / arr.len() as f64)),
        _ => unreachable!(),
    }
}

fn stdlib_array_length(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_length expects 1 argument, got {}", args.len()),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    Ok(Value::Int(arr.len() as i64))
}

fn stdlib_array_push(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_push expects 2 arguments, got {}", args.len()),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    arr.push(args[1].clone());
    Ok(Value::Array(arr))
}

fn stdlib_array_pop(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_pop expects 1 argument, got {}", args.len()),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    if let Some(popped) = arr.pop() {
        Ok(popped)
    } else {
        Err(RuntimeError::IndexOutOfBounds {
            index: 0,
            length: 0,
        })
    }
}

fn stdlib_array_reverse(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_reverse expects 1 argument, got {}", args.len()),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    arr.reverse();
    Ok(Value::Array(arr))
}

fn stdlib_array_sort(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::FunctionCallError {
            message: format!("array_sort expects 1 argument, got {}", args.len()),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Expected array".to_string(),
            })
        }
    };

    // Simple sorting for numbers
    arr.sort_by(|a, b| match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => std::cmp::Ordering::Equal,
    });

    Ok(Value::Array(arr))
}

// Helper function to extract Vec3 from Value
fn extract_vec3(value: &Value) -> RuntimeResult<Vec3> {
    match value {
        Value::Array(arr) => {
            if arr.len() != 3 {
                return Err(RuntimeError::TypeError {
                    message: "Expected 3-element array (Vec3)".to_string(),
                });
            }

            let x = match &arr[0] {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Invalid array element type, expected number".to_string(),
                    })
                }
            };

            let y = match &arr[1] {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Invalid array element type, expected number".to_string(),
                    })
                }
            };

            let z = match &arr[2] {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: "Invalid array element type, expected number".to_string(),
                    })
                }
            };

            Ok(Vec3::new(x, y, z))
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected Vec3 (3-element array)".to_string(),
        }),
    }
}
