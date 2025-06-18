// Standard library implementation for Matrix Language (simplified)
use crate::eval::interpreter::{RuntimeError, Value};

/// Register basic standard library functions with an interpreter
pub fn register_all(interpreter: &mut crate::eval::Interpreter) {
    // Register basic math functions directly
    interpreter.environment.define(
        "abs".to_string(),
        Value::BuiltinFunction {
            name: "abs".to_string(),
            arity: 1,
            func: |args| match &args[0] {
                Value::Int(n) => Ok(Value::Int(n.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(RuntimeError::TypeError {
                    message: format!("Cannot get absolute value of {}", args[0].type_name()),
                }),
            },
        },
    );

    interpreter.environment.define(
        "sqrt".to_string(),
        Value::BuiltinFunction {
            name: "sqrt".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute sqrt of {}", args[0].type_name()),
                        })
                    }
                };

                if num < 0.0 {
                    return Err(RuntimeError::Generic {
                        message: "sqrt of negative number".to_string(),
                    });
                }

                Ok(Value::Float(num.sqrt()))
            },
        },
    );

    interpreter.environment.define(
        "sin".to_string(),
        Value::BuiltinFunction {
            name: "sin".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute sin of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.sin()))
            },
        },
    );

    interpreter.environment.define(
        "cos".to_string(),
        Value::BuiltinFunction {
            name: "cos".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute cos of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.cos()))
            },
        },
    );

    interpreter.environment.define(
        "tan".to_string(),
        Value::BuiltinFunction {
            name: "tan".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute tan of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.tan()))
            },
        },
    );

    interpreter.environment.define(
        "exp".to_string(),
        Value::BuiltinFunction {
            name: "exp".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute exp of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.exp()))
            },
        },
    );

    interpreter.environment.define(
        "log".to_string(),
        Value::BuiltinFunction {
            name: "log".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute log of {}", args[0].type_name()),
                        })
                    }
                };
                if num <= 0.0 {
                    return Err(RuntimeError::Generic {
                        message: "log of non-positive number".to_string(),
                    });
                }
                Ok(Value::Float(num.ln()))
            },
        },
    );

    interpreter.environment.define(
        "pow".to_string(),
        Value::BuiltinFunction {
            name: "pow".to_string(),
            arity: 2,
            func: |args| {
                let base = match &args[0] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute power of {}", args[0].type_name()),
                        })
                    }
                };
                let exponent = match &args[1] {
                    Value::Int(n) => *n as f64,
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Invalid exponent type {}", args[1].type_name()),
                        })
                    }
                };
                Ok(Value::Float(base.powf(exponent)))
            },
        },
    );

    interpreter.environment.define(
        "floor".to_string(),
        Value::BuiltinFunction {
            name: "floor".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => return Ok(Value::Int(*n)),
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute floor of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.floor()))
            },
        },
    );

    interpreter.environment.define(
        "ceil".to_string(),
        Value::BuiltinFunction {
            name: "ceil".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => return Ok(Value::Int(*n)),
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute ceil of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.ceil()))
            },
        },
    );

    interpreter.environment.define(
        "round".to_string(),
        Value::BuiltinFunction {
            name: "round".to_string(),
            arity: 1,
            func: |args| {
                let num = match &args[0] {
                    Value::Int(n) => return Ok(Value::Int(*n)),
                    Value::Float(f) => *f,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: format!("Cannot compute round of {}", args[0].type_name()),
                        })
                    }
                };
                Ok(Value::Float(num.round()))
            },
        },
    );

    interpreter.environment.define(
        "len".to_string(),
        Value::BuiltinFunction {
            name: "len".to_string(),
            arity: 1,
            func: |args| {
                match &args[0] {
                    Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::Matrix(mat) => Ok(Value::Int(mat.len() as i64)),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot get length of {}", args[0].type_name()),
                    }),
                }
            },
        },
    );

    interpreter.environment.define(
        "max".to_string(),
        Value::BuiltinFunction {
            name: "max".to_string(),
            arity: 2,
            func: |args| {
                match (&args[0], &args[1]) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
                    _ => Err(RuntimeError::TypeError {
                        message: "max requires numeric arguments".to_string(),
                    }),
                }
            },
        },
    );

    interpreter.environment.define(
        "min".to_string(),
        Value::BuiltinFunction {
            name: "min".to_string(),
            arity: 2,
            func: |args| {
                match (&args[0], &args[1]) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
                    _ => Err(RuntimeError::TypeError {
                        message: "min requires numeric arguments".to_string(),
                    }),
                }
            },
        },
    );

    interpreter.environment.define(
        "print".to_string(),
        Value::BuiltinFunction {
            name: "print".to_string(),
            arity: 1,
            func: |args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", value_to_string(arg));
                }
                Ok(Value::Unit)
            },
        },
    );

    interpreter.environment.define(
        "println".to_string(),
        Value::BuiltinFunction {
            name: "println".to_string(),
            arity: 1,
            func: |args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", value_to_string(arg));
                }
                println!();
                Ok(Value::Unit)
            },
        },
    );
}

// Helper function to convert Value to string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Unit => "()".to_string(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Matrix(mat) => {
            let rows: Vec<String> = mat
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row.iter().map(|f| f.to_string()).collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            format!("[{}]", rows.join(", "))
        }
        Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        Value::Function { .. } => "<function>".to_string(),
        Value::BuiltinFunction { name, .. } => format!("<builtin: {}>", name),
        Value::AsyncHandle(_) => "<async handle>".to_string(),
    }
}
