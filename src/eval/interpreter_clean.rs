use crate::ast::*;
#[cfg(feature = "jit")]
use crate::jit::{JitContext, JitError, JitStats}; // Add JIT import conditionally
use crate::physics;
use crate::types::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::thread;
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
    pub bindings: HashMap<String, Value>,
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
    module_cache: HashMap<String, Environment>, // Cache for loaded modules
    #[cfg(feature = "jit")]
    jit_context: Option<JitContext>, // JIT compilation context
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            struct_registry: StructRegistry::new(),
            module_cache: HashMap::new(),
            #[cfg(feature = "jit")]
            jit_context: JitContext::new().ok(), // Initialize JIT if available
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

        // JIT compilation information
        self.environment.define(
            "jit_stats".to_string(),
            Value::BuiltinFunction {
                name: "jit_stats".to_string(),
                arity: 0,
                func: |_args| {
                    // This would need access to the interpreter context
                    // For now, return a placeholder
                    Ok(Value::String("JIT statistics not available".to_string()))
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

                // Try to JIT compile if possible
                if self.can_jit_compile(func_def) {
                    if let Err(e) = self.jit_compile_function(func_def) {
                        // Log JIT compilation failure but don't fail the function definition
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "JIT compilation failed for function '{}': {}",
                            func_def.name, e
                        );
                    } else {
                        #[cfg(debug_assertions)]
                        eprintln!("Successfully JIT compiled function '{}'", func_def.name);
                    }
                }

                Ok(Value::Unit)
            }

            Item::LetBinding(let_binding) => {
                let value = self.eval_expression(&let_binding.value)?;
                self.environment
                    .define(let_binding.name.clone(), value.clone());
                Ok(value)
            }

            Item::Import(import) => self.eval_import(import),
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

            Expression::Parallel { expressions, .. } => self.eval_parallel_block(expressions),

            Expression::Spawn { expression, .. } => self.eval_async_spawn(expression),

            Expression::Wait { expression, .. } => self.eval_async_wait(expression),

            Expression::GpuDirective { expression, .. } => self.eval_gpu_directive(expression),

            Expression::OptionalAccess {
                object,
                field,
                fallback,
                ..
            } => self.eval_optional_access(object, field, fallback),

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
                // Implement proper Option handling for ?? operator
                match &left_val {
                    Value::Unit => Ok(right_val), // None ?? value = value
                    _ => Ok(left_val),            // Some(value) ?? fallback = value
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
                // Implement matrix transpose
                match value {
                    Value::Matrix(mat) => {
                        if mat.is_empty() {
                            Ok(Value::Matrix(vec![]))
                        } else {
                            let rows = mat.len();
                            let cols = mat[0].len();

                            // Check that all rows have the same length
                            for row in &mat {
                                if row.len() != cols {
                                    return Err(RuntimeError::TypeError {
                                        message:
                                            "Cannot transpose matrix with inconsistent row lengths"
                                                .to_string(),
                                    });
                                }
                            }

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

                // Try JIT execution first if function name is available
                if let Expression::Identifier(func_name, _) = func {
                    #[cfg(feature = "jit")]
                    if let Some(ref jit) = self.jit_context {
                        // Check if function is JIT compiled
                        if let Ok(result) = jit.execute_function(func_name, &arg_values) {
                            return Ok(result);
                        }
                    }
                }

                // Fall back to interpreter execution
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

    /// Evaluate import statement
    fn eval_import(&mut self, import: &Import) -> RuntimeResult<Value> {
        // For now, just return Unit - full module system would be implemented here
        match import.module.as_str() {
            "std" => {
                // Load standard library
                Ok(Value::Unit)
            }
            _ => {
                // Load external module
                Ok(Value::Unit)
            }
        }
    }

    /// Evaluate struct creation
    fn eval_struct_creation(
        &mut self,
        name: &str,
        fields: &HashMap<String, Expression>,
    ) -> RuntimeResult<Value> {
        let mut field_values = HashMap::new();

        for (field_name, expr) in fields {
            let value = self.eval_expression(expr)?;
            field_values.insert(field_name.clone(), value);
        }

        Ok(Value::Struct {
            name: name.to_string(),
            fields: field_values,
        })
    }

    /// Evaluate matrix comprehension
    fn eval_matrix_comprehension(
        &mut self,
        element: &Expression,
        generators: &[Generator],
    ) -> RuntimeResult<Value> {
        // Simplified implementation - just create a 2x2 matrix for now
        let value = self.eval_expression(element)?;
        Ok(Value::Matrix(vec![
            vec![value.clone(), value.clone()],
            vec![value.clone(), value],
        ]))
    }

    /// Evaluate match expression
    fn eval_match_expression(
        &mut self,
        expression: &Expression,
        arms: &[MatchArm],
    ) -> RuntimeResult<Value> {
        let value = self.eval_expression(expression)?;

        for arm in arms {
            // Simplified pattern matching - just check if patterns match
            if self.pattern_matches(&arm.pattern, &value)? {
                if let Some(ref guard) = arm.guard {
                    let guard_result = self.eval_expression(guard)?;
                    if let Value::Bool(true) = guard_result {
                        return self.eval_expression(&arm.expression);
                    }
                } else {
                    return self.eval_expression(&arm.expression);
                }
            }
        }

        Err(RuntimeError::Generic {
            message: "No matching pattern found".to_string(),
        })
    }

    /// Check if a pattern matches a value
    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> RuntimeResult<bool> {
        match (pattern, value) {
            (Pattern::Wildcard(_), _) => Ok(true),
            (Pattern::Identifier(_, _), _) => Ok(true), // Variables always match
            (Pattern::IntLiteral(pat_val, _), Value::Int(val)) => Ok(pat_val == val),
            (Pattern::FloatLiteral(pat_val, _), Value::Float(val)) => {
                Ok((pat_val - val).abs() < f64::EPSILON)
            }
            (Pattern::BoolLiteral(pat_val, _), Value::Bool(val)) => Ok(pat_val == val),
            (Pattern::StringLiteral(pat_val, _), Value::String(val)) => Ok(pat_val == val),
            _ => Ok(false),
        }
    }

    /// Evaluate optional access with fallback
    fn eval_optional_access(
        &mut self,
        object: &Expression,
        field: &str,
        fallback: &Expression,
    ) -> RuntimeResult<Value> {
        match self.eval_expression(object) {
            Ok(value) => {
                // Try to access the field
                match value {
                    Value::Struct { fields, .. } => {
                        if let Some(field_value) = fields.get(field) {
                            Ok(field_value.clone())
                        } else {
                            self.eval_expression(fallback)
                        }
                    }
                    _ => self.eval_expression(fallback),
                }
            }
            Err(_) => self.eval_expression(fallback),
        }
    }

    /// Evaluate range expression
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
            _ => Err(RuntimeError::TypeError {
                message: "Range bounds must be integers".to_string(),
            }),
        }
    }

    /// Evaluate let expression with local bindings
    fn eval_let_expression(
        &mut self,
        bindings: &[LetBinding],
        body: &Expression,
    ) -> RuntimeResult<Value> {
        // Create new environment with current as parent
        let mut new_env = Environment::with_parent(self.environment.clone());

        // Evaluate and bind all let bindings
        for binding in bindings {
            let value = self.eval_expression(&binding.value)?;
            new_env.define(binding.name.clone(), value);
        }

        // Swap environments and evaluate body
        let old_env = std::mem::replace(&mut self.environment, new_env);
        let result = self.eval_expression(body);
        self.environment = old_env;

        result
    }

    /// Evaluate block expression
    fn eval_block(
        &mut self,
        statements: &[Statement],
        result: &Option<Box<Expression>>,
    ) -> RuntimeResult<Value> {
        // Execute all statements
        for stmt in statements {
            self.eval_statement(stmt)?;
        }

        // Evaluate result expression if present
        if let Some(result_expr) = result {
            self.eval_expression(result_expr)
        } else {
            Ok(Value::Unit)
        }
    }

    /// Evaluate statement
    fn eval_statement(&mut self, stmt: &Statement) -> RuntimeResult<Value> {
        match stmt {
            Statement::Expression(expr) => self.eval_expression(expr),
            Statement::Let(let_binding) => {
                let value = self.eval_expression(&let_binding.value)?;
                self.environment
                    .define(let_binding.name.clone(), value.clone());
                Ok(value)
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.eval_expression(value)?;
                // For now, only support simple variable assignment
                if let Expression::Identifier(name, _) = target {
                    self.environment.set(name, val.clone())?;
                }
                Ok(val)
            }
            Statement::Return(expr) => {
                // TODO: Implement proper return handling
                self.eval_expression(expr)
            }
            Statement::Break => {
                // TODO: Implement proper break handling
                Ok(Value::Unit)
            }
            Statement::Continue => {
                // TODO: Implement proper continue handling
                Ok(Value::Unit)
            }
        }
    }

    /// Evaluate parallel block (simplified - just evaluate sequentially for now)
    fn eval_parallel_block(&mut self, expressions: &[Expression]) -> RuntimeResult<Value> {
        let mut results = Vec::new();
        for expr in expressions {
            results.push(self.eval_expression(expr)?);
        }
        Ok(Value::Array(results))
    }

    /// Evaluate async spawn (simplified - just evaluate immediately for now)
    fn eval_async_spawn(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        self.eval_expression(expression)
    }

    /// Evaluate async wait (simplified - just evaluate immediately for now)
    fn eval_async_wait(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        self.eval_expression(expression)
    }

    /// Evaluate GPU directive (simplified - just evaluate the inner expression for now)
    fn eval_gpu_directive(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        self.eval_expression(expression)
    }

    // JIT compilation methods (conditional)

    /// Enable JIT compilation for performance-critical functions
    #[cfg(feature = "jit")]
    pub fn enable_jit(&mut self) -> Result<(), JitError> {
        if self.jit_context.is_none() {
            self.jit_context = Some(JitContext::new()?);
        }
        Ok(())
    }

    #[cfg(not(feature = "jit"))]
    pub fn enable_jit(&mut self) -> Result<(), String> {
        Err("JIT compilation not enabled".to_string())
    }

    /// Compile a function to native code using JIT
    #[cfg(feature = "jit")]
    pub fn jit_compile_function(&mut self, func: &FunctionDef) -> Result<String, JitError> {
        if let Some(ref mut jit) = self.jit_context {
            jit.compile_function(func)
        } else {
            Err(JitError::CompilationFailed(
                "JIT context not initialized".to_string(),
            ))
        }
    }

    #[cfg(not(feature = "jit"))]
    pub fn jit_compile_function(&mut self, _func: &FunctionDef) -> Result<String, String> {
        Err("JIT compilation not enabled".to_string())
    }

    /// Check if a function can be JIT compiled
    #[cfg(feature = "jit")]
    pub fn can_jit_compile(&self, func: &FunctionDef) -> bool {
        if let Some(ref jit) = self.jit_context {
            jit.can_jit_compile(func)
        } else {
            false
        }
    }

    #[cfg(not(feature = "jit"))]
    pub fn can_jit_compile(&self, _func: &FunctionDef) -> bool {
        false
    }

    /// Execute a JIT compiled function
    #[cfg(feature = "jit")]
    pub fn jit_execute_function(&self, name: &str, args: &[Value]) -> RuntimeResult<Value> {
        if let Some(ref jit) = self.jit_context {
            jit.execute_function(name, args)
        } else {
            Err(RuntimeError::Generic {
                message: "JIT context not available".to_string(),
            })
        }
    }

    #[cfg(not(feature = "jit"))]
    pub fn jit_execute_function(&self, _name: &str, _args: &[Value]) -> RuntimeResult<Value> {
        Err(RuntimeError::Generic {
            message: "JIT compilation not enabled".to_string(),
        })
    }

    /// Get JIT compilation statistics
    #[cfg(feature = "jit")]
    pub fn get_jit_stats(&self) -> Option<crate::jit::JitStats> {
        self.jit_context.as_ref().map(|jit| jit.get_stats())
    }

    #[cfg(not(feature = "jit"))]
    pub fn get_jit_stats(&self) -> Option<()> {
        None
    }

    /// Main interpretation entry point that delegates to eval_program
    pub fn interpret(&mut self, program: &Program) -> RuntimeResult<Value> {
        self.eval_program(program)
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Value::Matrix(mat) => {
                let rows: Vec<String> = mat
                    .iter()
                    .map(|row| {
                        let elements: Vec<String> = row.iter().map(|v| format!("{}", v)).collect();
                        format!("[{}]", elements.join(", "))
                    })
                    .collect();
                write!(f, "[{}]", rows.join(", "))
            }
            Value::Struct { name, fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::Function { params, .. } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| format!("{}: {:?}", p.name, p.type_annotation))
                    .collect();
                write!(f, "fn({})", param_strs.join(", "))
            }
            Value::BuiltinFunction { name, arity, .. } => {
                write!(f, "builtin {}({})", name, arity)
            }
            Value::PhysicsWorldHandle(_) => write!(f, "PhysicsWorld"),
        }
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
