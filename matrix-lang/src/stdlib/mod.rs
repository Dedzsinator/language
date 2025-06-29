// Standard library implementation for Matrix Language
// Focus: Physics simulation and mathematical functions
use crate::eval::interpreter::{RuntimeError, Value};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub mod physics;
pub mod quantum;

// Physics engine integration
static PHYSICS_WORLDS: LazyLock<Mutex<HashMap<usize, PhysicsWorld>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_WORLD_ID: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));

#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    pub id: usize,
    pub objects: Vec<PhysicsObject>,
    pub gravity: Vec3,
    pub time: f64,
    pub dt: f64,
}

#[derive(Debug, Clone)]
pub struct PhysicsObject {
    pub id: usize,
    pub shape: String,
    pub mass: f64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let mut next_id = NEXT_WORLD_ID.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        Self {
            id,
            objects: Vec::new(),
            gravity: Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
            time: 0.0,
            dt: 1.0 / 60.0, // 60 FPS
        }
    }

    pub fn add_object(&mut self, shape: String, mass: f64, position: Vec3) -> usize {
        let id = self.objects.len();
        self.objects.push(PhysicsObject {
            id,
            shape,
            mass,
            position,
            velocity: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            is_static: mass == 0.0,
        });
        id
    }

    pub fn step(&mut self) {
        // Simple physics integration
        for obj in &mut self.objects {
            if !obj.is_static {
                // Apply gravity
                obj.velocity.y += self.gravity.y * self.dt;

                // Update position
                obj.position.x += obj.velocity.x * self.dt;
                obj.position.y += obj.velocity.y * self.dt;
                obj.position.z += obj.velocity.z * self.dt;

                // Simple ground collision
                if obj.position.y < 0.0 {
                    obj.position.y = 0.0;
                    obj.velocity.y = -obj.velocity.y * 0.8; // Bounce with damping
                }
            }
        }

        self.time += self.dt;
    }
}

/// Register all standard library functions with an interpreter
pub fn register_all(interpreter: &mut crate::eval::Interpreter) {
    register_math_functions(interpreter);
    physics::register_physics_functions(interpreter);
    quantum::register_quantum_functions(interpreter);
}

fn register_math_functions(interpreter: &mut crate::eval::Interpreter) {
    // Note: abs, sin, cos, sqrt, len are already registered in interpreter builtins
    // Only register functions that are NOT in builtins

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
        "max".to_string(),
        Value::BuiltinFunction {
            name: "max".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.max(*b as f64))),
                _ => Err(RuntimeError::TypeError {
                    message: "max requires numeric arguments".to_string(),
                }),
            },
        },
    );

    interpreter.environment.define(
        "min".to_string(),
        Value::BuiltinFunction {
            name: "min".to_string(),
            arity: 2,
            func: |args| match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.min(*b as f64))),
                _ => Err(RuntimeError::TypeError {
                    message: "min requires numeric arguments".to_string(),
                }),
            },
        },
    );

    // println is a stdlib-only function (print is in builtins with different arity)
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

    // str is a stdlib-only function
    interpreter.environment.define(
        "str".to_string(),
        Value::BuiltinFunction {
            name: "str".to_string(),
            arity: 1,
            func: |args| {
                let string_val = value_to_string(&args[0]);
                Ok(Value::String(string_val))
            },
        },
    );

    // Register physics computing functions
    physics::register_physics_functions(interpreter);
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
        Value::PhysicsWorld(world) => {
            format!(
                "PhysicsWorld(id:{}, objects:{})",
                world.id,
                world.objects.len()
            )
        }
        Value::PhysicsObject(obj) => {
            format!("PhysicsObject(id:{}, shape:{})", obj.id, obj.shape)
        }
    }
}
