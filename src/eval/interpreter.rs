use crate::ast::*;
use crate::physics;
use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RuntimeError {
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Index out of bounds: {index} for length {length}")]
    IndexOutOfBounds { index: usize, length: usize },

    #[error("Field not found: {field} in {type_name}")]
    FieldNotFound { field: String, type_name: String },

    #[error("Function call error: {message}")]
    FunctionCallError { message: String },

    #[error("Pattern match failed")]
    PatternMatchFailed,

    #[error("Runtime error: {message}")]
    Generic { message: String },

    #[error("Physics error: {message}")]
    PhysicsError { message: String },
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Runtime values
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
    Array(Vec<Value>),
    Matrix(Vec<Vec<Value>>),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    Function {
        params: Vec<Parameter>,
        body: Expression,
        closure: Environment,
    },
    BuiltinFunction {
        name: String,
        arity: usize,
        func: fn(&[Value]) -> RuntimeResult<Value>,
    },
    PhysicsWorldHandle(Rc<RefCell<physics::PhysicsWorld>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::Unit => "Unit",
            Value::Array(_) => "Array",
            Value::Matrix(_) => "Matrix",
            Value::Struct { .. } => "Struct",
            Value::Function { .. } => "Function",
            Value::BuiltinFunction { .. } => "BuiltinFunction",
            Value::PhysicsWorldHandle(_) => "PhysicsWorldHandle",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Unit => false,
            Value::Array(arr) => !arr.is_empty(),
            Value::Matrix(mat) => !mat.is_empty(),
            _ => true,
        }
    }

    pub fn add(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot add {} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    pub fn subtract(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot subtract {} and {}",
                    self.type_name(),
                    other.type_name()
                ),
            }),
        }
    }

    pub fn multiply(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot multiply {} and {}",
                    self.type_name(),
                    other.type_name()
                ),
            }),
        }
    }

    pub fn divide(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot divide {} and {}",
                    self.type_name(),
                    other.type_name()
                ),
            }),
        }
    }

    pub fn matrix_multiply(&self, other: &Value) -> RuntimeResult<Value> {
        match (self, other) {
            (Value::Matrix(a), Value::Matrix(b)) => {
                if a.is_empty() || b.is_empty() {
                    return Ok(Value::Matrix(vec![]));
                }

                let rows_a = a.len();
                let cols_a = a[0].len();
                let rows_b = b.len();
                let cols_b = b[0].len();

                if cols_a != rows_b {
                    return Err(RuntimeError::TypeError {
                        message: format!(
                            "Matrix dimensions incompatible: {}x{} and {}x{}",
                            rows_a, cols_a, rows_b, cols_b
                        ),
                    });
                }

                let mut result = vec![vec![Value::Int(0); cols_b]; rows_a];

                for i in 0..rows_a {
                    for j in 0..cols_b {
                        let mut sum = Value::Int(0);
                        for k in 0..cols_a {
                            let product = a[i][k].multiply(&b[k][j])?;
                            sum = sum.add(&product)?;
                        }
                        result[i][j] = sum;
                    }
                }

                Ok(Value::Matrix(result))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot matrix multiply {} and {}",
                    self.type_name(),
                    other.type_name()
                ),
            }),
        }
    }

    pub fn equals(&self, other: &Value) -> RuntimeResult<Value> {
        let result = match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            _ => false,
        };
        Ok(Value::Bool(result))
    }

    pub fn less_than(&self, other: &Value) -> RuntimeResult<Value> {
        let result = match (self, other) {
            (Value::Int(a), Value::Int(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) < *b,
            (Value::Float(a), Value::Int(b)) => *a < (*b as f64),
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!(
                        "Cannot compare {} and {}",
                        self.type_name(),
                        other.type_name()
                    ),
                })
            }
        };
        Ok(Value::Bool(result))
    }
}

/// Runtime environment for variable bindings
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: Environment) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    pub fn set(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable {
                name: name.to_string(),
            })
        }
    }
}

/// Main interpreter for the matrix language
pub struct Interpreter {
    environment: Environment,
    struct_registry: StructRegistry,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            struct_registry: StructRegistry::new(),
        };

        interpreter.register_builtins();
        interpreter
    }

    fn register_builtins(&mut self) {
        // Mathematical constants
        self.environment
            .define("pi".to_string(), Value::Float(std::f64::consts::PI));

        self.environment
            .define("e".to_string(), Value::Float(std::f64::consts::E));

        self.environment
            .define("tau".to_string(), Value::Float(std::f64::consts::TAU));

        // Built-in functions
        self.environment.define(
            "print".to_string(),
            Value::BuiltinFunction {
                name: "print".to_string(),
                arity: 1,
                func: |args| {
                    if let Some(arg) = args.first() {
                        println!("{}", format_value(arg));
                    }
                    Ok(Value::Unit)
                },
            },
        );

        self.environment.define(
            "len".to_string(),
            Value::BuiltinFunction {
                name: "len".to_string(),
                arity: 1,
                func: |args| match &args[0] {
                    Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                    Value::Matrix(mat) => Ok(Value::Int(mat.len() as i64)),
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot get length of {}", args[0].type_name()),
                    }),
                },
            },
        );

        self.environment.define(
            "abs".to_string(),
            Value::BuiltinFunction {
                name: "abs".to_string(),
                arity: 1,
                func: |args| match &args[0] {
                    Value::Int(i) => Ok(Value::Int(i.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot get absolute value of {}", args[0].type_name()),
                    }),
                },
            },
        );

        // Mathematical functions
        self.environment.define(
            "sin".to_string(),
            Value::BuiltinFunction {
                name: "sin".to_string(),
                arity: 1,
                func: |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sin())),
                    Value::Int(i) => Ok(Value::Float((*i as f64).sin())),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot compute sin of {}", args[0].type_name()),
                    }),
                },
            },
        );

        self.environment.define(
            "cos".to_string(),
            Value::BuiltinFunction {
                name: "cos".to_string(),
                arity: 1,
                func: |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.cos())),
                    Value::Int(i) => Ok(Value::Float((*i as f64).cos())),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot compute cos of {}", args[0].type_name()),
                    }),
                },
            },
        );

        self.environment.define(
            "sqrt".to_string(),
            Value::BuiltinFunction {
                name: "sqrt".to_string(),
                arity: 1,
                func: |args| match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sqrt())),
                    Value::Int(i) => Ok(Value::Float((*i as f64).sqrt())),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot compute sqrt of {}", args[0].type_name()),
                    }),
                },
            },
        );

        // Physics engine functions
        self.register_physics_functions();
    }

    fn register_physics_functions(&mut self) {
        // Create physics world
        self.environment.define(
            "create_physics_world".to_string(),
            Value::BuiltinFunction {
                name: "create_physics_world".to_string(),
                arity: 0,
                func: |_args| {
                    let world = physics::PhysicsWorld::new();
                    Ok(Value::PhysicsWorldHandle(Rc::new(RefCell::new(world))))
                },
            },
        );

        // Add rigid body to physics world
        self.environment.define(
            "add_rigid_body".to_string(),
            Value::BuiltinFunction {
                name: "add_rigid_body".to_string(),
                arity: 5, // world, shape_type, size, mass, position
                func: |args| {
                    if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                        let shape_type = match &args[1] {
                            Value::String(s) => s.as_str(),
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Shape type must be a string".to_string(),
                                })
                            }
                        };

                        let size = match &args[2] {
                            Value::Array(arr) => {
                                if arr.len() != 3 {
                                    return Err(RuntimeError::TypeError {
                                        message: "Size must be a 3-element array [x, y, z]"
                                            .to_string(),
                                    });
                                }
                                let x = match &arr[0] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Size components must be numbers".to_string(),
                                        })
                                    }
                                };
                                let y = match &arr[1] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Size components must be numbers".to_string(),
                                        })
                                    }
                                };
                                let z = match &arr[2] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Size components must be numbers".to_string(),
                                        })
                                    }
                                };
                                physics::math::Vec3::new(x, y, z)
                            }
                            Value::Float(f) => physics::math::Vec3::new(*f, *f, *f), // Uniform size
                            Value::Int(i) => {
                                physics::math::Vec3::new(*i as f64, *i as f64, *i as f64)
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Size must be a number or 3-element array".to_string(),
                                })
                            }
                        };

                        let mass = match &args[3] {
                            Value::Float(f) => *f,
                            Value::Int(i) => *i as f64,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Mass must be a number".to_string(),
                                })
                            }
                        };

                        let position = match &args[4] {
                            Value::Array(arr) => {
                                if arr.len() != 3 {
                                    return Err(RuntimeError::TypeError {
                                        message: "Position must be a 3-element array [x, y, z]"
                                            .to_string(),
                                    });
                                }
                                let x = match &arr[0] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Position components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                let y = match &arr[1] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Position components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                let z = match &arr[2] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Position components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                physics::math::Vec3::new(x, y, z)
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Position must be a 3-element array".to_string(),
                                })
                            }
                        };

                        let shape = match shape_type {
                            "sphere" => physics::rigid_body::Shape::Sphere { radius: size.x },
                            "box" => physics::rigid_body::Shape::Box { size },
                            "capsule" => physics::rigid_body::Shape::Capsule {
                                radius: size.x,
                                height: size.y,
                            },
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: format!("Unknown shape type: {}", shape_type),
                                })
                            }
                        };

                        let mut world = world_ref.borrow_mut();
                        let body_id = world.add_rigid_body(shape, mass, position);
                        Ok(Value::Int(body_id as i64))
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "First argument must be a physics world".to_string(),
                        })
                    }
                },
            },
        );

        // Add soft body to physics world
        self.environment.define(
            "add_soft_body".to_string(),
            Value::BuiltinFunction {
                name: "add_soft_body".to_string(),
                arity: 3, // world, type, stiffness
                func: |args| {
                    if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                        let body_type = match &args[1] {
                            Value::String(s) => s.as_str(),
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Body type must be a string".to_string(),
                                })
                            }
                        };

                        let _stiffness = match &args[2] {
                            Value::Float(f) => *f,
                            Value::Int(i) => *i as f64,
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Stiffness must be a number".to_string(),
                                })
                            }
                        };

                        let soft_body = match body_type {
                            "cloth" => physics::soft_body::SoftBody::create_cloth(10, 10, 1.0, 1.0),
                            "sphere" => physics::soft_body::SoftBody::create_sphere(
                                physics::math::Vec3::zero(),
                                1.0,
                                10,
                                1.0,
                            ),
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: format!("Unknown soft body type: {}", body_type),
                                })
                            }
                        };

                        let mut world = world_ref.borrow_mut();
                        let body_id = world.add_soft_body(soft_body);
                        Ok(Value::Int(body_id as i64))
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "First argument must be a physics world".to_string(),
                        })
                    }
                },
            },
        );

        // Add fluid system to physics world
        self.environment.define("add_fluid_system".to_string(), Value::BuiltinFunction {
            name: "add_fluid_system".to_string(),
            arity: 3, // world, particles, rest_density
            func: |args| {
                if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                    let particles = match &args[1] {
                        Value::Array(arr) => {
                            let mut positions = Vec::new();
                            for particle in arr {
                                if let Value::Array(pos_arr) = particle {
                                    if pos_arr.len() != 3 {
                                        return Err(RuntimeError::TypeError {
                                            message: "Each particle position must be a 3-element array".to_string(),
                                        });
                                    }
                                    let x = match &pos_arr[0] { Value::Float(f) => *f, Value::Int(i) => *i as f64, _ => return Err(RuntimeError::TypeError { message: "Position components must be numbers".to_string() }) };
                                    let y = match &pos_arr[1] { Value::Float(f) => *f, Value::Int(i) => *i as f64, _ => return Err(RuntimeError::TypeError { message: "Position components must be numbers".to_string() }) };
                                    let z = match &pos_arr[2] { Value::Float(f) => *f, Value::Int(i) => *i as f64, _ => return Err(RuntimeError::TypeError { message: "Position components must be numbers".to_string() }) };
                                    positions.push(physics::math::Vec3::new(x, y, z));
                                } else {
                                    return Err(RuntimeError::TypeError {
                                        message: "Each particle must be a position array".to_string(),
                                    });
                                }
                            }
                            positions
                        },
                        _ => return Err(RuntimeError::TypeError {
                            message: "Particles must be an array of position arrays".to_string(),
                        }),
                    };

                    let rest_density = match &args[2] {
                        Value::Float(f) => *f,
                        Value::Int(i) => *i as f64,
                        _ => return Err(RuntimeError::TypeError {
                            message: "Rest density must be a number".to_string(),
                        }),
                    };

                    let mut world = world_ref.borrow_mut();
                    let fluid_id = world.add_fluid_system(particles, rest_density);
                    Ok(Value::Int(fluid_id as i64))
                } else {
                    Err(RuntimeError::TypeError {
                        message: "First argument must be a physics world".to_string(),
                    })
                }
            },
        });

        // Step physics simulation
        self.environment.define(
            "physics_step".to_string(),
            Value::BuiltinFunction {
                name: "physics_step".to_string(),
                arity: 1, // world
                func: |args| {
                    if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                        let mut world = world_ref.borrow_mut();
                        world.step();
                        Ok(Value::Unit)
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "Argument must be a physics world".to_string(),
                        })
                    }
                },
            },
        );

        // Get physics simulation state for visualization
        self.environment.define(
            "get_physics_state".to_string(),
            Value::BuiltinFunction {
                name: "get_physics_state".to_string(),
                arity: 1, // world
                func: |args| {
                    if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                        let world = world_ref.borrow();
                        world.to_simulation_state()
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "Argument must be a physics world".to_string(),
                        })
                    }
                },
            },
        );

        // Set physics world properties
        self.environment.define(
            "set_gravity".to_string(),
            Value::BuiltinFunction {
                name: "set_gravity".to_string(),
                arity: 2, // world, gravity_vector
                func: |args| {
                    if let Value::PhysicsWorldHandle(world_ref) = &args[0] {
                        let gravity = match &args[1] {
                            Value::Array(arr) => {
                                if arr.len() != 3 {
                                    return Err(RuntimeError::TypeError {
                                        message: "Gravity must be a 3-element array [x, y, z]"
                                            .to_string(),
                                    });
                                }
                                let x = match &arr[0] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Gravity components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                let y = match &arr[1] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Gravity components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                let z = match &arr[2] {
                                    Value::Float(f) => *f,
                                    Value::Int(i) => *i as f64,
                                    _ => {
                                        return Err(RuntimeError::TypeError {
                                            message: "Gravity components must be numbers"
                                                .to_string(),
                                        })
                                    }
                                };
                                physics::math::Vec3::new(x, y, z)
                            }
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    message: "Gravity must be a 3-element array".to_string(),
                                })
                            }
                        };

                        let mut world = world_ref.borrow_mut();
                        world.gravity = gravity;
                        Ok(Value::Unit)
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "First argument must be a physics world".to_string(),
                        })
                    }
                },
            },
        );
    }

    pub fn eval_program(&mut self, program: &Program) -> RuntimeResult<Value> {
        let mut last_value = Value::Unit;

        for item in &program.items {
            last_value = self.eval_item(item)?;
        }

        Ok(last_value)
    }

    pub fn eval_item(&mut self, item: &Item) -> RuntimeResult<Value> {
        match item {
            Item::StructDef(struct_def) => {
                self.struct_registry.register(struct_def.clone());
                Ok(Value::Unit)
            }

            Item::TypeclassDef(_) => {
                // Typeclasses are handled at compile time
                Ok(Value::Unit)
            }

            Item::InstanceDef(_) => {
                // Instances are handled at compile time
                Ok(Value::Unit)
            }

            Item::FunctionDef(func_def) => {
                let function_value = Value::Function {
                    params: func_def.params.clone(),
                    body: func_def.body.clone(),
                    closure: self.environment.clone(),
                };

                self.environment
                    .define(func_def.name.clone(), function_value);
                Ok(Value::Unit)
            }

            Item::LetBinding(let_binding) => {
                let value = self.eval_expression(&let_binding.value)?;
                self.environment
                    .define(let_binding.name.clone(), value.clone());
                Ok(value)
            }

            Item::Import(_) => {
                // TODO: Implement imports
                Ok(Value::Unit)
            }
        }
    }
    pub fn eval_expression(&mut self, expr: &Expression) -> RuntimeResult<Value> {
        match expr {
            Expression::IntLiteral(value, _) => Ok(Value::Int(*value)),
            Expression::FloatLiteral(value, _) => Ok(Value::Float(*value)),
            Expression::BoolLiteral(value, _) => Ok(Value::Bool(*value)),
            Expression::StringLiteral(value, _) => Ok(Value::String(value.clone())),

            Expression::Identifier(name, _) => {
                if let Some(value) = self.environment.get(name) {
                    Ok(value.clone())
                } else {
                    Err(RuntimeError::UndefinedVariable { name: name.clone() })
                }
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
                ..
            } => self.eval_binary_op(left, operator, right),

            Expression::UnaryOp {
                operator, operand, ..
            } => self.eval_unary_op(operator, operand),

            Expression::FunctionCall { function, args, .. } => {
                self.eval_function_call(function, args)
            }

            Expression::FieldAccess { object, field, .. } => self.eval_field_access(object, field),

            Expression::StructCreation { name, fields, .. } => {
                self.eval_struct_creation(name, fields)
            }

            Expression::ArrayLiteral(elements, _) => self.eval_array_literal(elements),

            Expression::MatrixLiteral(rows, _) => self.eval_matrix_literal(rows),

            Expression::MatrixComprehension {
                element,
                generators,
                ..
            } => self.eval_matrix_comprehension(element, generators),

            Expression::IfExpression {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.eval_if_expression(condition, then_branch, else_branch),

            Expression::Match {
                expression, arms, ..
            } => self.eval_match_expression(expression, arms),

            Expression::Let { bindings, body, .. } => self.eval_let_expression(bindings, body),

            Expression::Lambda { params, body, .. } => Ok(Value::Function {
                params: params.clone(),
                body: (**body).clone(),
                closure: self.environment.clone(),
            }),

            Expression::Block {
                statements, result, ..
            } => self.eval_block(statements, result),

            Expression::Parallel { expressions, .. } => {
                // For now, evaluate serially
                // TODO: Implement parallel execution
                self.eval_parallel_block(expressions)
            }

            Expression::Spawn { expression, .. } => {
                // For now, evaluate directly
                // TODO: Implement async spawn
                self.eval_expression(expression)
            }

            Expression::Wait { expression, .. } => {
                // For now, evaluate directly
                // TODO: Implement async wait
                self.eval_expression(expression)
            }

            Expression::GpuDirective { expression, .. } => {
                // For now, evaluate on CPU
                // TODO: Implement GPU execution
                self.eval_expression(expression)
            }
            Expression::OptionalAccess {
                object,
                field,
                fallback,
                ..
            } => self.eval_optional_access(object, field, &Some(fallback.clone())),

            Expression::Range {
                start,
                end,
                inclusive,
                ..
            } => self.eval_range(start, end, *inclusive),
        }
    }

    fn eval_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> RuntimeResult<Value> {
        let left_val = self.eval_expression(left)?;
        let right_val = self.eval_expression(right)?;

        match op {
            BinaryOperator::Add => left_val.add(&right_val),
            BinaryOperator::Sub => left_val.subtract(&right_val),
            BinaryOperator::Mul => left_val.multiply(&right_val),
            BinaryOperator::Div => left_val.divide(&right_val),
            BinaryOperator::MatMul => left_val.matrix_multiply(&right_val),
            BinaryOperator::Eq => left_val.equals(&right_val),
            BinaryOperator::Ne => {
                let eq_result = left_val.equals(&right_val)?;
                match eq_result {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => unreachable!(),
                }
            }
            BinaryOperator::Lt => left_val.less_than(&right_val),
            BinaryOperator::Le => {
                let lt = left_val.less_than(&right_val)?;
                let eq = left_val.equals(&right_val)?;
                match (lt, eq) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => unreachable!(),
                }
            }
            BinaryOperator::Gt => {
                let lt = left_val.less_than(&right_val)?;
                let eq = left_val.equals(&right_val)?;
                match (lt, eq) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(!a && !b)),
                    _ => unreachable!(),
                }
            }
            BinaryOperator::Ge => {
                let lt = left_val.less_than(&right_val)?;
                match lt {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => unreachable!(),
                }
            }
            BinaryOperator::And => {
                if left_val.is_truthy() {
                    Ok(right_val)
                } else {
                    Ok(left_val)
                }
            }
            BinaryOperator::Or => {
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            }
            BinaryOperator::OptionalOr => {
                // TODO: Implement proper Option handling
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Unimplemented binary operator: {:?}", op),
            }),
        }
    }
    fn eval_unary_op(&mut self, op: &UnaryOperator, expr: &Expression) -> RuntimeResult<Value> {
        let value = self.eval_expression(expr)?;

        match op {
            UnaryOperator::Not => Ok(Value::Bool(!value.is_truthy())),
            UnaryOperator::Neg => match value {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::TypeError {
                    message: format!("Cannot negate {}", value.type_name()),
                }),
            },
            UnaryOperator::Transpose => {
                // TODO: Implement matrix transpose
                match value {
                    Value::Matrix(mat) => {
                        // Simple transpose
                        if mat.is_empty() {
                            Ok(Value::Matrix(vec![]))
                        } else {
                            let rows = mat.len();
                            let cols = mat[0].len();
                            let mut transposed = vec![vec![Value::Int(0); rows]; cols];

                            for i in 0..rows {
                                for j in 0..cols {
                                    transposed[j][i] = mat[i][j].clone();
                                }
                            }

                            Ok(Value::Matrix(transposed))
                        }
                    }
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot transpose {}", value.type_name()),
                    }),
                }
            }
        }
    }

    fn eval_function_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
    ) -> RuntimeResult<Value> {
        let func_value = self.eval_expression(func)?;
        let arg_values: Result<Vec<_>, _> =
            args.iter().map(|arg| self.eval_expression(arg)).collect();
        let arg_values = arg_values?;

        match func_value {
            Value::Function {
                params,
                body,
                closure,
            } => {
                if params.len() != arg_values.len() {
                    return Err(RuntimeError::FunctionCallError {
                        message: format!(
                            "Expected {} arguments, got {}",
                            params.len(),
                            arg_values.len()
                        ),
                    });
                }

                // Create new environment with closure as parent
                let mut new_env = Environment::with_parent(closure);

                // Bind parameters
                for (param, arg_value) in params.iter().zip(arg_values.iter()) {
                    new_env.define(param.name.clone(), arg_value.clone());
                }

                // Swap environments and evaluate body
                let old_env = std::mem::replace(&mut self.environment, new_env);
                let result = self.eval_expression(&body);
                self.environment = old_env;

                result
            }

            Value::BuiltinFunction { func, arity, .. } => {
                if arg_values.len() != arity {
                    return Err(RuntimeError::FunctionCallError {
                        message: format!("Expected {} arguments, got {}", arity, arg_values.len()),
                    });
                }

                func(&arg_values)
            }

            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot call {}", func_value.type_name()),
            }),
        }
    }

    fn eval_field_access(&mut self, expr: &Expression, field: &str) -> RuntimeResult<Value> {
        let value = self.eval_expression(expr)?;

        match value {
            Value::Struct { fields, .. } => {
                if let Some(field_value) = fields.get(field) {
                    Ok(field_value.clone())
                } else {
                    Err(RuntimeError::FieldNotFound {
                        field: field.to_string(),
                        type_name: "Struct".to_string(),
                    })
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot access field of {}", value.type_name()),
            }),
        }
    }

    fn eval_array_literal(&mut self, elements: &[Expression]) -> RuntimeResult<Value> {
        let values: Result<Vec<_>, _> = elements.iter().map(|e| self.eval_expression(e)).collect();
        Ok(Value::Array(values?))
    }

    fn eval_matrix_literal(&mut self, rows: &[Vec<Expression>]) -> RuntimeResult<Value> {
        let mut matrix_rows = Vec::new();

        for row in rows {
            let row_values: Result<Vec<_>, _> =
                row.iter().map(|e| self.eval_expression(e)).collect();
            matrix_rows.push(row_values?);
        }

        Ok(Value::Matrix(matrix_rows))
    }

    fn eval_if_expression(
        &mut self,
        condition: &Expression,
        then_expr: &Expression,
        else_expr: &Option<Box<Expression>>,
    ) -> RuntimeResult<Value> {
        let cond_value = self.eval_expression(condition)?;

        if cond_value.is_truthy() {
            self.eval_expression(then_expr)
        } else if let Some(else_expr) = else_expr {
            self.eval_expression(else_expr)
        } else {
            Ok(Value::Unit)
        }
    }

    #[allow(dead_code)]
    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> RuntimeResult<bool> {
        match pattern {
            Pattern::Wildcard(_) => Ok(true),

            Pattern::Identifier(name, _) => {
                self.environment.define(name.clone(), value.clone());
                Ok(true)
            }

            Pattern::IntLiteral(expected, _) => match value {
                Value::Int(actual) => Ok(actual == expected),
                _ => Ok(false),
            },

            Pattern::FloatLiteral(expected, _) => match value {
                Value::Float(actual) => Ok((actual - expected).abs() < f64::EPSILON),
                _ => Ok(false),
            },

            Pattern::BoolLiteral(expected, _) => match value {
                Value::Bool(actual) => Ok(actual == expected),
                _ => Ok(false),
            },

            Pattern::StringLiteral(expected, _) => match value {
                Value::String(actual) => Ok(actual == expected),
                _ => Ok(false),
            },

            Pattern::Some(inner_pattern, _) => {
                // TODO: Implement proper Option handling
                self.match_pattern(inner_pattern, value)
            }

            Pattern::None(_) => {
                // TODO: Implement proper Option handling
                Ok(matches!(value, Value::Unit))
            }

            Pattern::Struct { name, fields, .. } => match value {
                Value::Struct {
                    name: struct_name,
                    fields: struct_fields,
                } if struct_name == name => {
                    for (field_name, field_pattern) in fields {
                        if let Some(field_value) = struct_fields.get(field_name) {
                            if !self.match_pattern(field_pattern, field_value)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },

            Pattern::Array(patterns, _) => match value {
                Value::Array(arr) => {
                    if patterns.len() != arr.len() {
                        return Ok(false);
                    }

                    for (pattern, value) in patterns.iter().zip(arr.iter()) {
                        if !self.match_pattern(pattern, value)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
        }
    }

    fn eval_let_expression(
        &mut self,
        bindings: &[LetBinding],
        body: &Expression,
    ) -> RuntimeResult<Value> {
        let old_env = self.environment.clone();

        // Evaluate all bindings in order
        for binding in bindings {
            let value = self.eval_expression(&binding.value)?;
            self.environment.define(binding.name.clone(), value);
        }

        let result = self.eval_expression(body);

        self.environment = old_env;

        result
    }

    fn eval_block(
        &mut self,
        statements: &[Statement],
        result: &Option<Box<Expression>>,
    ) -> RuntimeResult<Value> {
        let old_env = self.environment.clone();

        // Execute statements
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    self.eval_expression(expr)?;
                }
                Statement::LetBinding(binding) => {
                    let value = self.eval_expression(&binding.value)?;
                    self.environment.define(binding.name.clone(), value);
                }
            }
        }

        let final_result = if let Some(result_expr) = result {
            self.eval_expression(result_expr)?
        } else {
            Value::Unit
        };

        self.environment = old_env;

        Ok(final_result)
    }

    fn eval_parallel_block(&mut self, expressions: &[Expression]) -> RuntimeResult<Value> {
        // For now, evaluate serially
        // TODO: Implement parallel execution
        if expressions.is_empty() {
            return Ok(Value::Unit);
        }

        let mut last_value = Value::Unit;
        for expr in expressions {
            last_value = self.eval_expression(expr)?;
        }

        Ok(last_value)
    }

    fn eval_struct_creation(
        &mut self,
        name: &str,
        fields: &HashMap<String, Expression>,
    ) -> RuntimeResult<Value> {
        let mut field_values = std::collections::HashMap::new();

        for (field_name, field_expr) in fields {
            let field_value = self.eval_expression(field_expr)?;
            field_values.insert(field_name.clone(), field_value);
        }

        Ok(Value::Struct {
            name: name.to_string(),
            fields: field_values,
        })
    }
    fn eval_matrix_comprehension(
        &mut self,
        element: &Expression,
        generators: &Vec<Generator>,
    ) -> RuntimeResult<Value> {
        // For now, implement a simple case - single generator
        if generators.is_empty() {
            return Err(RuntimeError::Generic {
                message: "Matrix comprehension requires at least one generator".to_string(),
            });
        }

        // Evaluate first generator to get range/array to iterate over
        let generator_value = self.eval_expression(&generators[0].iterable)?;

        let mut result_rows = Vec::new();
        match generator_value {
            Value::Array(arr) => {
                for _item in arr {
                    // TODO: Bind iterator variable to environment
                    let elem_value = self.eval_expression(element)?;
                    if let Value::Array(row) = elem_value {
                        result_rows.push(row);
                    } else {
                        result_rows.push(vec![elem_value]);
                    }
                }
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Generator must be an array".to_string(),
                })
            }
        }

        Ok(Value::Matrix(result_rows))
    }
    fn eval_match_expression(
        &mut self,
        expression: &Expression,
        arms: &Vec<MatchArm>,
    ) -> RuntimeResult<Value> {
        let _expr_value = self.eval_expression(expression)?;

        // For now, implement simple pattern matching
        // TODO: Implement proper pattern matching with guards
        for arm in arms {
            // Each arm should be a pattern => expression
            // For simplicity, just evaluate the first arm for now
            return self.eval_expression(&arm.body);
        }

        Err(RuntimeError::Generic {
            message: "No matching pattern found".to_string(),
        })
    }

    fn eval_optional_access(
        &mut self,
        object: &Expression,
        field: &str,
        fallback: &Option<Box<Expression>>,
    ) -> RuntimeResult<Value> {
        let obj_value = self.eval_expression(object)?;

        match obj_value {
            Value::Struct { fields, .. } => {
                if let Some(field_value) = fields.get(field) {
                    Ok(field_value.clone())
                } else if let Some(fallback_expr) = fallback {
                    self.eval_expression(fallback_expr)
                } else {
                    Ok(Value::Unit)
                }
            }
            _ => {
                if let Some(fallback_expr) = fallback {
                    self.eval_expression(fallback_expr)
                } else {
                    Err(RuntimeError::TypeError {
                        message: "Optional access on non-struct type".to_string(),
                    })
                }
            }
        }
    }

    fn eval_range(
        &mut self,
        start: &Expression,
        end: &Expression,
        inclusive: bool,
    ) -> RuntimeResult<Value> {
        let start_val = self.eval_expression(start)?;
        let end_val = self.eval_expression(end)?;

        match (start_val, end_val) {
            (Value::Int(s), Value::Int(e)) => {
                let range: Vec<Value> = if inclusive {
                    (s..=e).map(Value::Int).collect()
                } else {
                    (s..e).map(Value::Int).collect()
                };
                Ok(Value::Array(range))
            }
            (Value::Float(s), Value::Float(e)) => {
                // For float ranges, create a simple array with step of 1.0
                let mut range = Vec::new();
                let mut current = s;
                let step = if s < e { 1.0 } else { -1.0 };

                if inclusive {
                    while (step > 0.0 && current <= e) || (step < 0.0 && current >= e) {
                        range.push(Value::Float(current));
                        current += step;
                    }
                } else {
                    while (step > 0.0 && current < e) || (step < 0.0 && current > e) {
                        range.push(Value::Float(current));
                        current += step;
                    }
                }
                Ok(Value::Array(range))
            }
            _ => Err(RuntimeError::TypeError {
                message: "Range bounds must be numbers".to_string(),
            }),
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Unit => "()".to_string(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Matrix(mat) => {
            let rows: Vec<String> = mat
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row.iter().map(format_value).collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            format!("[{}]", rows.join(", "))
        }
        Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        Value::Function { .. } => "<function>".to_string(),
        Value::BuiltinFunction { name, .. } => format!("<builtin: {}>", name),
        Value::PhysicsWorldHandle(_) => "<physics world handle>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;
    use std::collections::HashMap;

    fn create_test_span() -> Span {
        Span::new(0, 0, 0, 0)
    }

    #[test]
    fn test_runtime_error_variants() {
        let err1 = RuntimeError::UndefinedVariable {
            name: "x".to_string(),
        };
        assert_eq!(err1.to_string(), "Undefined variable: x");

        let err2 = RuntimeError::TypeError {
            message: "type mismatch".to_string(),
        };
        assert_eq!(err2.to_string(), "Type error: type mismatch");

        let err3 = RuntimeError::DivisionByZero;
        assert_eq!(err3.to_string(), "Division by zero");

        let err4 = RuntimeError::IndexOutOfBounds {
            index: 5,
            length: 3,
        };
        assert_eq!(err4.to_string(), "Index out of bounds: 5 for length 3");

        let err5 = RuntimeError::FunctionCallError {
            message: "arity mismatch".to_string(),
        };
        assert_eq!(err5.to_string(), "Function call error: arity mismatch");
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(Value::Int(42).type_name(), "Int");
        assert_eq!(Value::Float(3.14).type_name(), "Float");
        assert_eq!(Value::Bool(true).type_name(), "Bool");
        assert_eq!(Value::String("hello".to_string()).type_name(), "String");
        assert_eq!(Value::Unit.type_name(), "Unit");
        assert_eq!(Value::Array(vec![]).type_name(), "Array");
        assert_eq!(Value::Matrix(vec![]).type_name(), "Matrix");
    }

    #[test]
    fn test_value_truthiness() {
        // Truthy values
        assert!(Value::Bool(true).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Float(0.1).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(Value::Array(vec![Value::Int(1)]).is_truthy());

        // Falsy values
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
        assert!(!Value::Unit.is_truthy());
        assert!(!Value::Array(vec![]).is_truthy());
        assert!(!Value::Matrix(vec![]).is_truthy());
    }

    #[test]
    fn test_value_arithmetic_operations() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        let c = Value::Float(3.5);
        let d = Value::Float(2.0);

        // Addition
        assert_eq!(a.add(&b).unwrap(), Value::Int(15));
        assert_eq!(a.add(&c).unwrap(), Value::Float(13.5));
        assert_eq!(c.add(&d).unwrap(), Value::Float(5.5));

        // String concatenation
        let s1 = Value::String("Hello".to_string());
        let s2 = Value::String(" World".to_string());
        assert_eq!(
            s1.add(&s2).unwrap(),
            Value::String("Hello World".to_string())
        );

        // Subtraction
        assert_eq!(a.subtract(&b).unwrap(), Value::Int(5));
        assert_eq!(c.subtract(&d).unwrap(), Value::Float(1.5));

        // Multiplication
        assert_eq!(a.multiply(&b).unwrap(), Value::Int(50));
        assert_eq!(c.multiply(&d).unwrap(), Value::Float(7.0));

        // Division
        assert_eq!(a.divide(&b).unwrap(), Value::Int(2));
        assert_eq!(c.divide(&d).unwrap(), Value::Float(1.75));

        // Division by zero
        assert!(matches!(
            a.divide(&Value::Int(0)),
            Err(RuntimeError::DivisionByZero)
        ));
        assert!(matches!(
            c.divide(&Value::Float(0.0)),
            Err(RuntimeError::DivisionByZero)
        ));
    }

    #[test]
    fn test_value_comparison_operations() {
        let a = Value::Int(10);
        let b = Value::Int(5);
        let c = Value::Float(10.0);

        // Equality
        assert_eq!(a.equals(&Value::Int(10)).unwrap(), Value::Bool(true));
        assert_eq!(a.equals(&b).unwrap(), Value::Bool(false));
        assert_eq!(a.equals(&c).unwrap(), Value::Bool(false)); // Different types

        // Less than
        assert_eq!(b.less_than(&a).unwrap(), Value::Bool(true));
        assert_eq!(a.less_than(&b).unwrap(), Value::Bool(false));
        assert_eq!(
            Value::Float(5.5).less_than(&Value::Int(6)).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_matrix_multiplication() {
        let mat1 = Value::Matrix(vec![
            vec![Value::Int(1), Value::Int(2)],
            vec![Value::Int(3), Value::Int(4)],
        ]);
        let mat2 = Value::Matrix(vec![
            vec![Value::Int(5), Value::Int(6)],
            vec![Value::Int(7), Value::Int(8)],
        ]);

        let result = mat1.matrix_multiply(&mat2).unwrap();
        if let Value::Matrix(res_mat) = result {
            assert_eq!(res_mat[0][0], Value::Int(19));
            assert_eq!(res_mat[0][1], Value::Int(22));
            assert_eq!(res_mat[1][0], Value::Int(43));
            assert_eq!(res_mat[1][1], Value::Int(50));
        } else {
            panic!("Expected matrix result");
        }

        // Incompatible dimensions
        let mat3 = Value::Matrix(vec![vec![Value::Int(1)]]);
        assert!(mat1.matrix_multiply(&mat3).is_err());
    }

    #[test]
    fn test_environment_operations() {
        let mut env = Environment::new();

        // Define and get variables
        env.define("x".to_string(), Value::Int(42));
        assert_eq!(env.get("x"), Some(&Value::Int(42)));
        assert_eq!(env.get("y"), None);

        // Environment with parent
        let mut child_env = Environment::with_parent(env.clone());
        child_env.define("y".to_string(), Value::String("hello".to_string()));

        // Child can access parent's bindings
        assert_eq!(child_env.get("x"), Some(&Value::Int(42)));
        assert_eq!(
            child_env.get("y"),
            Some(&Value::String("hello".to_string()))
        );

        // Set existing variable
        assert!(child_env.set("x", Value::Int(100)).is_ok());
        assert_eq!(child_env.get("x"), Some(&Value::Int(100)));

        // Set non-existent variable
        let mut isolated_env = Environment::new();
        assert!(matches!(
            isolated_env.set("nonexistent", Value::Unit),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }

    #[test]
    fn test_interpreter_literal_evaluation() {
        let mut interpreter = Interpreter::new();

        // Integer literal
        let int_expr = Expression::IntLiteral(42, create_test_span());
        assert_eq!(
            interpreter.eval_expression(&int_expr).unwrap(),
            Value::Int(42)
        );

        // Float literal
        let float_expr = Expression::FloatLiteral(3.14, create_test_span());
        assert_eq!(
            interpreter.eval_expression(&float_expr).unwrap(),
            Value::Float(3.14)
        );

        // Boolean literal
        let bool_expr = Expression::BoolLiteral(true, create_test_span());
        assert_eq!(
            interpreter.eval_expression(&bool_expr).unwrap(),
            Value::Bool(true)
        );

        // String literal
        let string_expr = Expression::StringLiteral("hello".to_string(), create_test_span());
        assert_eq!(
            interpreter.eval_expression(&string_expr).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_interpreter_binary_operations() {
        let mut interpreter = Interpreter::new();

        let left = Expression::IntLiteral(10, create_test_span());
        let right = Expression::IntLiteral(5, create_test_span());

        // Addition
        let add_expr = Expression::BinaryOp {
            left: Box::new(left.clone()),
            operator: BinaryOperator::Add,
            right: Box::new(right.clone()),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&add_expr).unwrap(),
            Value::Int(15)
        );

        // Subtraction
        let sub_expr = Expression::BinaryOp {
            left: Box::new(left.clone()),
            operator: BinaryOperator::Sub,
            right: Box::new(right.clone()),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&sub_expr).unwrap(),
            Value::Int(5)
        );

        // Logical AND
        let true_expr = Expression::BoolLiteral(true, create_test_span());
        let false_expr = Expression::BoolLiteral(false, create_test_span());
        let and_expr = Expression::BinaryOp {
            left: Box::new(true_expr),
            operator: BinaryOperator::And,
            right: Box::new(false_expr.clone()),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&and_expr).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_interpreter_unary_operations() {
        let mut interpreter = Interpreter::new();

        // Negation
        let int_expr = Expression::IntLiteral(42, create_test_span());
        let neg_expr = Expression::UnaryOp {
            operator: UnaryOperator::Neg,
            operand: Box::new(int_expr),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&neg_expr).unwrap(),
            Value::Int(-42)
        );

        // Logical NOT
        let bool_expr = Expression::BoolLiteral(true, create_test_span());
        let not_expr = Expression::UnaryOp {
            operator: UnaryOperator::Not,
            operand: Box::new(bool_expr),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&not_expr).unwrap(),
            Value::Bool(false)
        );

        // Matrix transpose
        let matrix_expr = Expression::MatrixLiteral(
            vec![
                vec![
                    Expression::IntLiteral(1, create_test_span()),
                    Expression::IntLiteral(2, create_test_span()),
                ],
                vec![
                    Expression::IntLiteral(3, create_test_span()),
                    Expression::IntLiteral(4, create_test_span()),
                ],
            ],
            create_test_span(),
        );
        let transpose_expr = Expression::UnaryOp {
            operator: UnaryOperator::Transpose,
            operand: Box::new(matrix_expr),
            span: create_test_span(),
        };

        if let Value::Matrix(result) = interpreter.eval_expression(&transpose_expr).unwrap() {
            assert_eq!(result[0][0], Value::Int(1));
            assert_eq!(result[0][1], Value::Int(3));
            assert_eq!(result[1][0], Value::Int(2));
            assert_eq!(result[1][1], Value::Int(4));
        } else {
            panic!("Expected matrix result");
        }
    }

    #[test]
    fn test_interpreter_variables() {
        let mut interpreter = Interpreter::new();

        // Define a variable via let binding
        let let_binding = LetBinding {
            name: "x".to_string(),
            type_annotation: None,
            value: Expression::IntLiteral(42, create_test_span()),
            span: create_test_span(),
        };
        let let_item = Item::LetBinding(let_binding);
        assert_eq!(interpreter.eval_item(&let_item).unwrap(), Value::Int(42));

        // Access the variable
        let var_expr = Expression::Identifier("x".to_string(), create_test_span());
        assert_eq!(
            interpreter.eval_expression(&var_expr).unwrap(),
            Value::Int(42)
        );

        // Access undefined variable
        let undef_expr = Expression::Identifier("undefined".to_string(), create_test_span());
        assert!(matches!(
            interpreter.eval_expression(&undef_expr),
            Err(RuntimeError::UndefinedVariable { .. })
        ));
    }

    #[test]
    fn test_interpreter_arrays() {
        let mut interpreter = Interpreter::new();

        // Array literal
        let array_expr = Expression::ArrayLiteral(
            vec![
                Expression::IntLiteral(1, create_test_span()),
                Expression::IntLiteral(2, create_test_span()),
                Expression::IntLiteral(3, create_test_span()),
            ],
            create_test_span(),
        );

        if let Value::Array(arr) = interpreter.eval_expression(&array_expr).unwrap() {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Int(1));
            assert_eq!(arr[1], Value::Int(2));
            assert_eq!(arr[2], Value::Int(3));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_interpreter_matrices() {
        let mut interpreter = Interpreter::new();

        // Matrix literal
        let matrix_expr = Expression::MatrixLiteral(
            vec![
                vec![
                    Expression::IntLiteral(1, create_test_span()),
                    Expression::IntLiteral(2, create_test_span()),
                ],
                vec![
                    Expression::IntLiteral(3, create_test_span()),
                    Expression::IntLiteral(4, create_test_span()),
                ],
            ],
            create_test_span(),
        );

        if let Value::Matrix(mat) = interpreter.eval_expression(&matrix_expr).unwrap() {
            assert_eq!(mat.len(), 2);
            assert_eq!(mat[0].len(), 2);
            assert_eq!(mat[0][0], Value::Int(1));
            assert_eq!(mat[0][1], Value::Int(2));
            assert_eq!(mat[1][0], Value::Int(3));
            assert_eq!(mat[1][1], Value::Int(4));
        } else {
            panic!("Expected matrix result");
        }
    }

    #[test]
    fn test_interpreter_if_expressions() {
        let mut interpreter = Interpreter::new();

        // If with true condition
        let if_expr = Expression::IfExpression {
            condition: Box::new(Expression::BoolLiteral(true, create_test_span())),
            then_branch: Box::new(Expression::IntLiteral(42, create_test_span())),
            else_branch: Some(Box::new(Expression::IntLiteral(0, create_test_span()))),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&if_expr).unwrap(),
            Value::Int(42)
        );

        // If with false condition
        let if_expr = Expression::IfExpression {
            condition: Box::new(Expression::BoolLiteral(false, create_test_span())),
            then_branch: Box::new(Expression::IntLiteral(42, create_test_span())),
            else_branch: Some(Box::new(Expression::IntLiteral(0, create_test_span()))),
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&if_expr).unwrap(),
            Value::Int(0)
        );

        // If without else branch
        let if_expr = Expression::IfExpression {
            condition: Box::new(Expression::BoolLiteral(false, create_test_span())),
            then_branch: Box::new(Expression::IntLiteral(42, create_test_span())),
            else_branch: None,
            span: create_test_span(),
        };
        assert_eq!(interpreter.eval_expression(&if_expr).unwrap(), Value::Unit);
    }

    #[test]
    fn test_interpreter_builtin_functions() {
        let mut interpreter = Interpreter::new();

        // Test print function exists
        let print_var = Expression::Identifier("print".to_string(), create_test_span());
        let print_value = interpreter.eval_expression(&print_var).unwrap();
        assert!(matches!(print_value, Value::BuiltinFunction { .. }));

        // Test len function
        let len_var = Expression::Identifier("len".to_string(), create_test_span());
        let array_expr = Expression::ArrayLiteral(
            vec![
                Expression::IntLiteral(1, create_test_span()),
                Expression::IntLiteral(2, create_test_span()),
            ],
            create_test_span(),
        );
        let len_call = Expression::FunctionCall {
            function: Box::new(len_var),
            args: vec![array_expr],
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&len_call).unwrap(),
            Value::Int(2)
        );

        // Test abs function
        let abs_var = Expression::Identifier("abs".to_string(), create_test_span());
        let neg_expr = Expression::IntLiteral(-42, create_test_span());
        let abs_call = Expression::FunctionCall {
            function: Box::new(abs_var),
            args: vec![neg_expr],
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&abs_call).unwrap(),
            Value::Int(42)
        );
    }

    #[test]
    fn test_interpreter_function_definition_and_call() {
        let mut interpreter = Interpreter::new();

        // Define a simple function
        let func_def = FunctionDef {
            name: "add_one".to_string(),
            params: vec![Parameter {
                name: "x".to_string(),
                type_annotation: Type::Int,
                span: create_test_span(),
            }],
            return_type: None,
            body: Expression::BinaryOp {
                left: Box::new(Expression::Identifier("x".to_string(), create_test_span())),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::IntLiteral(1, create_test_span())),
                span: create_test_span(),
            },
            attributes: vec![],
            span: create_test_span(),
        };

        let func_item = Item::FunctionDef(func_def);
        interpreter.eval_item(&func_item).unwrap();

        // Call the function
        let func_call = Expression::FunctionCall {
            function: Box::new(Expression::Identifier(
                "add_one".to_string(),
                create_test_span(),
            )),
            args: vec![Expression::IntLiteral(41, create_test_span())],
            span: create_test_span(),
        };
        assert_eq!(
            interpreter.eval_expression(&func_call).unwrap(),
            Value::Int(42)
        );
    }

    #[test]
    fn test_interpreter_struct_creation_and_access() {
        let mut interpreter = Interpreter::new();

        // Define a struct
        let struct_def = StructDef {
            name: "Point".to_string(),
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    type_annotation: Type::Int,
                    optional: false,
                    default_value: None,
                    span: create_test_span(),
                },
                StructField {
                    name: "y".to_string(),
                    type_annotation: Type::Int,
                    optional: false,
                    default_value: None,
                    span: create_test_span(),
                },
            ],
            span: create_test_span(),
        };
        let struct_item = Item::StructDef(struct_def);
        interpreter.eval_item(&struct_item).unwrap();

        // Create struct instance
        let mut fields = HashMap::new();
        fields.insert(
            "x".to_string(),
            Expression::IntLiteral(10, create_test_span()),
        );
        fields.insert(
            "y".to_string(),
            Expression::IntLiteral(20, create_test_span()),
        );

        let struct_expr = Expression::StructCreation {
            name: "Point".to_string(),
            fields,
            span: create_test_span(),
        };

        let struct_value = interpreter.eval_expression(&struct_expr).unwrap();
        if let Value::Struct { name, fields } = struct_value {
            assert_eq!(name, "Point");
            assert_eq!(fields.get("x"), Some(&Value::Int(10)));
            assert_eq!(fields.get("y"), Some(&Value::Int(20)));
        } else {
            panic!("Expected struct value");
        }
    }

    #[test]
    fn test_interpreter_range_expressions() {
        let mut interpreter = Interpreter::new();

        // Inclusive integer range
        let range_expr = Expression::Range {
            start: Box::new(Expression::IntLiteral(1, create_test_span())),
            end: Box::new(Expression::IntLiteral(3, create_test_span())),
            inclusive: true,
            span: create_test_span(),
        };

        if let Value::Array(arr) = interpreter.eval_expression(&range_expr).unwrap() {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Int(1));
            assert_eq!(arr[1], Value::Int(2));
            assert_eq!(arr[2], Value::Int(3));
        } else {
            panic!("Expected array result");
        }

        // Exclusive integer range
        let range_expr = Expression::Range {
            start: Box::new(Expression::IntLiteral(1, create_test_span())),
            end: Box::new(Expression::IntLiteral(3, create_test_span())),
            inclusive: false,
            span: create_test_span(),
        };

        if let Value::Array(arr) = interpreter.eval_expression(&range_expr).unwrap() {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Int(1));
            assert_eq!(arr[1], Value::Int(2));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_pattern_matching() {
        let mut interpreter = Interpreter::new();

        // Wildcard pattern
        let wildcard_pattern = Pattern::Wildcard(create_test_span());
        assert!(interpreter
            .match_pattern(&wildcard_pattern, &Value::Int(42))
            .unwrap());

        // Integer literal pattern
        let int_pattern = Pattern::IntLiteral(42, create_test_span());
        assert!(interpreter
            .match_pattern(&int_pattern, &Value::Int(42))
            .unwrap());
        assert!(!interpreter
            .match_pattern(&int_pattern, &Value::Int(41))
            .unwrap());

        // Boolean pattern
        let bool_pattern = Pattern::BoolLiteral(true, create_test_span());
        assert!(interpreter
            .match_pattern(&bool_pattern, &Value::Bool(true))
            .unwrap());
        assert!(!interpreter
            .match_pattern(&bool_pattern, &Value::Bool(false))
            .unwrap());

        // String pattern
        let string_pattern = Pattern::StringLiteral("hello".to_string(), create_test_span());
        assert!(interpreter
            .match_pattern(&string_pattern, &Value::String("hello".to_string()))
            .unwrap());
        assert!(!interpreter
            .match_pattern(&string_pattern, &Value::String("world".to_string()))
            .unwrap());
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&Value::Int(42)), "42");
        assert_eq!(format_value(&Value::Float(3.14)), "3.14");
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::String("hello".to_string())), "hello");
        assert_eq!(format_value(&Value::Unit), "()");

        let array = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(format_value(&array), "[1, 2, 3]");

        let matrix = Value::Matrix(vec![
            vec![Value::Int(1), Value::Int(2)],
            vec![Value::Int(3), Value::Int(4)],
        ]);
        assert_eq!(format_value(&matrix), "[[1, 2], [3, 4]]");

        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Int(10));
        fields.insert("y".to_string(), Value::Int(20));
        let struct_val = Value::Struct {
            name: "Point".to_string(),
            fields,
        };
        let formatted = format_value(&struct_val);
        assert!(formatted.contains("Point"));
        assert!(formatted.contains("x: 10"));
        assert!(formatted.contains("y: 20"));
    }

    #[test]
    fn test_interpreter_error_cases() {
        let mut interpreter = Interpreter::new();

        // Type error in binary operation
        let type_error_expr = Expression::BinaryOp {
            left: Box::new(Expression::StringLiteral(
                "hello".to_string(),
                create_test_span(),
            )),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::IntLiteral(42, create_test_span())),
            span: create_test_span(),
        };
        assert!(matches!(
            interpreter.eval_expression(&type_error_expr),
            Err(RuntimeError::TypeError { .. })
        ));

        // Function call with wrong arity
        let wrong_arity_call = Expression::FunctionCall {
            function: Box::new(Expression::Identifier(
                "len".to_string(),
                create_test_span(),
            )),
            args: vec![
                Expression::IntLiteral(1, create_test_span()),
                Expression::IntLiteral(2, create_test_span()),
            ],
            span: create_test_span(),
        };
        assert!(matches!(
            interpreter.eval_expression(&wrong_arity_call),
            Err(RuntimeError::FunctionCallError { .. })
        ));

        // Call non-function
        let non_func_call = Expression::FunctionCall {
            function: Box::new(Expression::IntLiteral(42, create_test_span())),
            args: vec![],
            span: create_test_span(),
        };
        assert!(matches!(
            interpreter.eval_expression(&non_func_call),
            Err(RuntimeError::TypeError { .. })
        ));
    }

    #[test]
    fn test_let_expression_scoping() {
        let mut interpreter = Interpreter::new();

        // Define a variable in outer scope
        interpreter
            .environment
            .define("x".to_string(), Value::Int(10));

        // Create let expression that shadows the variable
        let let_expr = Expression::Let {
            bindings: vec![LetBinding {
                name: "x".to_string(),
                type_annotation: None,
                value: Expression::IntLiteral(42, create_test_span()),
                span: create_test_span(),
            }],
            body: Box::new(Expression::Identifier("x".to_string(), create_test_span())),
            span: create_test_span(),
        };

        // Inside let expression, x should be 42
        assert_eq!(
            interpreter.eval_expression(&let_expr).unwrap(),
            Value::Int(42)
        );

        // Outside let expression, x should still be 10
        let outer_x = Expression::Identifier("x".to_string(), create_test_span());
        assert_eq!(
            interpreter.eval_expression(&outer_x).unwrap(),
            Value::Int(10)
        );
    }

    #[test]
    fn test_interpreter_program_evaluation() {
        let mut interpreter = Interpreter::new();

        let program = Program {
            items: vec![
                Item::LetBinding(LetBinding {
                    name: "x".to_string(),
                    type_annotation: None,
                    value: Expression::IntLiteral(10, create_test_span()),
                    span: create_test_span(),
                }),
                Item::LetBinding(LetBinding {
                    name: "y".to_string(),
                    type_annotation: None,
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("x".to_string(), create_test_span())),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::IntLiteral(5, create_test_span())),
                        span: create_test_span(),
                    },
                    span: create_test_span(),
                }),
            ],
            span: create_test_span(),
        };

        let result = interpreter.eval_program(&program).unwrap();
        assert_eq!(result, Value::Int(15)); // Last expression result
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Matrix(a), Value::Matrix(b)) => a == b,
            (
                Value::Struct {
                    name: n1,
                    fields: f1,
                },
                Value::Struct {
                    name: n2,
                    fields: f2,
                },
            ) => n1 == n2 && f1 == f2,
            (Value::Function { .. }, Value::Function { .. }) => false, // Functions can't be compared
            (Value::BuiltinFunction { name: n1, .. }, Value::BuiltinFunction { name: n2, .. }) => {
                n1 == n2
            }
            (Value::PhysicsWorldHandle(_), Value::PhysicsWorldHandle(_)) => false, // Physics worlds can't be compared
            _ => false,
        }
    }
}
