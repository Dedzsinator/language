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
            Err(RuntimeError::UndefinedFunction(name.to_string()))
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(RuntimeError::TypeError {
            expected: "number".to_string(),
            got: format!("{:?}", args[0]),
        }),
    }
}

fn stdlib_sqrt(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num < 0.0 {
        return Err(RuntimeError::DomainError(
            "sqrt of negative number".to_string(),
        ));
    }

    Ok(Value::Float(num.sqrt()))
}

fn stdlib_cbrt(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.cbrt()))
}

fn stdlib_pow(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let base = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let exponent = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    Ok(Value::Float(base.powf(exponent)))
}

fn stdlib_exp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.exp()))
}

fn stdlib_ln(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::DomainError(
            "ln of non-positive number".to_string(),
        ));
    }

    Ok(Value::Float(num.ln()))
}

fn stdlib_log10(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::DomainError(
            "log10 of non-positive number".to_string(),
        ));
    }

    Ok(Value::Float(num.log10()))
}

fn stdlib_log2(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num <= 0.0 {
        return Err(RuntimeError::DomainError(
            "log2 of non-positive number".to_string(),
        ));
    }

    Ok(Value::Float(num.log2()))
}

// Trigonometric functions
fn stdlib_sin(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.sin()))
}

fn stdlib_cos(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.cos()))
}

fn stdlib_tan(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.tan()))
}

fn stdlib_asin(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num < -1.0 || num > 1.0 {
        return Err(RuntimeError::DomainError(
            "asin input must be in [-1, 1]".to_string(),
        ));
    }

    Ok(Value::Float(num.asin()))
}

fn stdlib_acos(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if num < -1.0 || num > 1.0 {
        return Err(RuntimeError::DomainError(
            "acos input must be in [-1, 1]".to_string(),
        ));
    }

    Ok(Value::Float(num.acos()))
}

fn stdlib_atan(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.atan()))
}

fn stdlib_atan2(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let y = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let x = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    Ok(Value::Float(y.atan2(x)))
}

// Hyperbolic functions
fn stdlib_sinh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.sinh()))
}

fn stdlib_cosh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.cosh()))
}

fn stdlib_tanh(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.tanh()))
}

// Rounding and comparison functions
fn stdlib_floor(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => return Ok(Value::Integer(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.floor()))
}

fn stdlib_ceil(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => return Ok(Value::Integer(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.ceil()))
}

fn stdlib_round(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let num = match &args[0] {
        Value::Integer(n) => return Ok(Value::Integer(*n)),
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Float(num.round()))
}

fn stdlib_min(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.min(*b as f64))),
        _ => Err(RuntimeError::TypeError {
            expected: "numbers".to_string(),
            got: format!("{:?}, {:?}", args[0], args[1]),
        }),
    }
}

fn stdlib_max(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.max(*b as f64))),
        _ => Err(RuntimeError::TypeError {
            expected: "numbers".to_string(),
            got: format!("{:?}, {:?}", args[0], args[1]),
        }),
    }
}

fn stdlib_clamp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 3,
            got: args.len(),
        });
    }

    let value = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let min_val = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let max_val = match &args[2] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[2]),
            })
        }
    };

    Ok(Value::Float(value.clamp(min_val, max_val)))
}

// Vector and physics functions
fn stdlib_vec3(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 3,
            got: args.len(),
        });
    }

    let x = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let y = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let z = match &args[2] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[2]),
            })
        }
    };

    Ok(Value::Array(vec![
        Value::Float(x),
        Value::Float(y),
        Value::Float(z),
    ]))
}

fn stdlib_magnitude(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let vec = extract_vec3(&args[0])?;
    Ok(Value::Float(vec.magnitude()))
}

fn stdlib_normalize(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let vec = extract_vec3(&args[0])?;
    let normalized = vec.normalized();
    Ok(Value::Array(vec![
        Value::Float(normalized.x),
        Value::Float(normalized.y),
        Value::Float(normalized.z),
    ]))
}

fn stdlib_dot(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    Ok(Value::Float(vec1.dot(vec2)))
}

fn stdlib_cross(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    let result = vec1.cross(vec2);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_distance(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    Ok(Value::Float(vec1.distance_to(vec2)))
}

fn stdlib_lerp(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 3,
            got: args.len(),
        });
    }

    let vec1 = extract_vec3(&args[0])?;
    let vec2 = extract_vec3(&args[1])?;
    let t = match &args[2] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[2]),
            })
        }
    };

    let result = vec1.lerp(vec2, t);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_reflect(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let incident = extract_vec3(&args[0])?;
    let normal = extract_vec3(&args[1])?;
    let result = incident.reflect(normal);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

// Physics calculation functions
fn stdlib_kinetic_energy(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let velocity = extract_vec3(&args[1])?;
    let speed_squared = velocity.magnitude_squared();
    Ok(Value::Float(0.5 * mass * speed_squared))
}

fn stdlib_potential_energy(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let height = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    const G: f64 = 9.81; // Earth gravity
    Ok(Value::Float(mass * G * height))
}

fn stdlib_momentum(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let acceleration = extract_vec3(&args[1])?;
    let result = acceleration.force_from_acceleration(mass);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_acceleration(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let force = extract_vec3(&args[0])?;
    let mass = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 3,
            got: args.len(),
        });
    }

    let acceleration = extract_vec3(&args[0])?;
    let mass = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let dt = match &args[2] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[2]),
            })
        }
    };

    let result = acceleration.velocity_from_force(mass, dt);
    Ok(Value::Array(vec![
        Value::Float(result.x),
        Value::Float(result.y),
        Value::Float(result.z),
    ]))
}

fn stdlib_gravitational_force(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 3,
            got: args.len(),
        });
    }

    let mass1 = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let mass2 = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let distance = match &args[2] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[2]),
            })
        }
    };

    const G: f64 = 6.67430e-11; // Gravitational constant
    let force = G * mass1 * mass2 / (distance * distance);
    Ok(Value::Float(force))
}

fn stdlib_escape_velocity(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let radius = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    const G: f64 = 6.67430e-11;
    let escape_vel = (2.0 * G * mass / radius).sqrt();
    Ok(Value::Float(escape_vel))
}

fn stdlib_orbital_velocity(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mass = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let radius = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

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
        match arg {
            Value::Integer(n) => print!("{}", n),
            Value::Float(f) => print!("{}", f),
            Value::String(s) => print!("{}", s),
            Value::Boolean(b) => print!("{}", b),
            Value::Array(arr) => {
                print!("[");
                for (j, item) in arr.iter().enumerate() {
                    if j > 0 {
                        print!(", ");
                    }
                    match item {
                        Value::Integer(n) => print!("{}", n),
                        Value::Float(f) => print!("{}", f),
                        Value::String(s) => print!("\"{}\"", s),
                        Value::Boolean(b) => print!("{}", b),
                        _ => print!("{:?}", item),
                    }
                }
                print!("]");
            }
            _ => print!("{:?}", arg),
        }
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let type_name = match &args[0] {
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Boolean(_) => "boolean",
        Value::Array(_) => "array",
        Value::Function { .. } => "function",
        Value::Unit => "unit",
    };

    Ok(Value::String(type_name.to_string()))
}

fn stdlib_to_string(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let string_repr = match &args[0] {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => format!("{:?}", arr),
        Value::Function { .. } => "<function>".to_string(),
        Value::Unit => "()".to_string(),
    };

    Ok(Value::String(string_repr))
}

fn stdlib_random(_args: &[Value]) -> RuntimeResult<Value> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple pseudo-random number generator
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let hash = hasher.finish();

    let random_val = (hash as f64) / (u64::MAX as f64);
    Ok(Value::Float(random_val))
}

fn stdlib_random_range(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let min_val = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let max_val = match &args[1] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "number".to_string(),
                got: format!("{:?}", args[1]),
            })
        }
    };

    let random_val = stdlib_random(&[])?;
    if let Value::Float(r) = random_val {
        Ok(Value::Float(min_val + r * (max_val - min_val)))
    } else {
        unreachable!()
    }
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    let mut sum = 0.0;
    for item in arr {
        match item {
            Value::Integer(n) => sum += *n as f64,
            Value::Float(f) => sum += f,
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "number array".to_string(),
                    got: format!("{:?}", item),
                })
            }
        }
    }

    Ok(Value::Float(sum))
}

fn stdlib_array_avg(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    if arr.is_empty() {
        return Err(RuntimeError::DivisionByZero);
    }

    let sum = stdlib_array_sum(args)?;
    if let Value::Float(s) = sum {
        Ok(Value::Float(s / arr.len() as f64))
    } else {
        unreachable!()
    }
}

fn stdlib_array_length(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let arr = match &args[0] {
        Value::Array(a) => a,
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    Ok(Value::Integer(arr.len() as i64))
}

fn stdlib_array_push(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    arr.push(args[1].clone());
    Ok(Value::Array(arr))
}

fn stdlib_array_pop(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
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
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    arr.reverse();
    Ok(Value::Array(arr))
}

fn stdlib_array_sort(args: &[Value]) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    let mut arr = match &args[0] {
        Value::Array(a) => a.clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                expected: "array".to_string(),
                got: format!("{:?}", args[0]),
            })
        }
    };

    arr.sort_by(|a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Integer(x), Value::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(x), Value::Integer(y)) => x
            .partial_cmp(&(*y as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(x), Value::String(y)) => x.cmp(y),
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
                    expected: "3-element array (Vec3)".to_string(),
                    got: format!("{}-element array", arr.len()),
                });
            }

            let x = match &arr[0] {
                Value::Integer(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        expected: "number".to_string(),
                        got: format!("{:?}", arr[0]),
                    })
                }
            };

            let y = match &arr[1] {
                Value::Integer(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        expected: "number".to_string(),
                        got: format!("{:?}", arr[1]),
                    })
                }
            };

            let z = match &arr[2] {
                Value::Integer(n) => *n as f64,
                Value::Float(f) => *f,
                _ => {
                    return Err(RuntimeError::TypeError {
                        expected: "number".to_string(),
                        got: format!("{:?}", arr[2]),
                    })
                }
            };

            Ok(Vec3::new(x, y, z))
        }
        _ => Err(RuntimeError::TypeError {
            expected: "Vec3 (3-element array)".to_string(),
            got: format!("{:?}", value),
        }),
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}
