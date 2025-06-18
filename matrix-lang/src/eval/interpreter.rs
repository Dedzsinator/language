use crate::ast::*;
#[cfg(feature = "jit")]
use crate::jit::{JitContext, JitError, JitStats}; // Add JIT import conditionally
use crate::types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

// Stub JitError when JIT feature is not enabled
#[cfg(not(feature = "jit"))]
#[derive(Debug, Clone)]
pub enum JitError {
    NotAvailable,
}

#[cfg(not(feature = "jit"))]
impl std::fmt::Display for JitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitError::NotAvailable => write!(f, "JIT compilation not available"),
        }
    }
}

#[cfg(not(feature = "jit"))]
impl std::error::Error for JitError {}

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

/// Async task handle for managing spawned computations
#[derive(Debug, Clone)]
pub struct AsyncTask {
    pub id: usize,
    pub result: Arc<Mutex<Option<RuntimeResult<Value>>>>,
    pub completed: Arc<Mutex<bool>>,
    pub start_time: Instant,
}

impl AsyncTask {
    fn new(id: usize) -> Self {
        Self {
            id,
            result: Arc::new(Mutex::new(None)),
            completed: Arc::new(Mutex::new(false)),
            start_time: Instant::now(),
        }
    }

    pub fn is_complete(&self) -> bool {
        *self.completed.lock().unwrap()
    }

    fn get_result(&self) -> Option<RuntimeResult<Value>> {
        self.result.lock().unwrap().take()
    }

    fn set_result(&self, result: RuntimeResult<Value>) {
        *self.result.lock().unwrap() = Some(result);
        *self.completed.lock().unwrap() = true;
    }
}

/// GPU computation mode simulation
#[derive(Debug, Clone, Copy)]
pub enum GpuMode {
    Cpu,      // Fallback to CPU
    Simd,     // Use SIMD instructions
    Parallel, // Use parallel threads
}

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
    AsyncHandle(AsyncTask), // Handle to async computation
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
            Value::AsyncHandle(_) => "AsyncHandle",
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

    /// Set a variable, creating it if it doesn't exist
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Get a variable, returning a default value if it doesn't exist
    pub fn get_variable(&self, name: &str, default: Value) -> Value {
        self.bindings.get(name).cloned().unwrap_or_else(|| {
            self.parent
                .as_ref()
                .map_or(default.clone(), |p| p.get_variable(name, default))
        })
    }
}

/// Main interpreter for the matrix language
pub struct Interpreter {
    pub environment: Environment,
    struct_registry: StructRegistry,
    module_cache: HashMap<String, Environment>, // Cache for loaded modules
    #[cfg(feature = "jit")]
    jit_context: Option<JitContext>, // JIT compilation context
    async_tasks: HashMap<usize, AsyncTask>,     // Track async tasks
    next_task_id: usize,                        // Counter for task IDs
    gpu_mode: GpuMode,                          // Current GPU computation mode
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            struct_registry: StructRegistry::new(),
            module_cache: HashMap::new(),
            #[cfg(feature = "jit")]
            jit_context: JitContext::new().ok(), // Initialize JIT if available
            async_tasks: HashMap::new(),
            next_task_id: 0,
            gpu_mode: GpuMode::Cpu,
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
            BinaryOperator::Pow => match (&left_val, &right_val) {
                (Value::Int(base), Value::Int(exp)) => {
                    if *exp >= 0 {
                        Ok(Value::Int((*base as f64).powf(*exp as f64) as i64))
                    } else {
                        Ok(Value::Float((*base as f64).powf(*exp as f64)))
                    }
                }
                (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
                (Value::Int(base), Value::Float(exp)) => {
                    Ok(Value::Float((*base as f64).powf(*exp)))
                }
                (Value::Float(base), Value::Int(exp)) => Ok(Value::Float(base.powf(*exp as f64))),
                _ => Err(RuntimeError::TypeError {
                    message: format!(
                        "Cannot compute power of {} and {}",
                        left_val.type_name(),
                        right_val.type_name()
                    ),
                }),
            },
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
                if let Expression::Identifier(_func_name, _) = func {
                    #[cfg(feature = "jit")]
                    if let Some(ref jit) = self.jit_context {
                        // Check if function is JIT compiled
                        if let Ok(result) = jit.execute_function(_func_name, &arg_values) {
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
        match import.module_path.as_str() {
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
        _generators: &[Generator],
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
                        return self.eval_expression(&arm.body);
                    }
                } else {
                    return self.eval_expression(&arm.body);
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

            // Check if this is a lambda function that can be JIT compiled
            if let Expression::Lambda {
                params,
                body: lambda_body,
                ..
            } = &binding.value
            {
                self.try_jit_compile_lambda(&binding.name, params, lambda_body);
            }

            new_env.define(binding.name.clone(), value);
        }

        // Swap environments and evaluate body
        let old_env = std::mem::replace(&mut self.environment, new_env);
        let result = self.eval_expression(body);
        self.environment = old_env;

        result
    }

    /// Try to JIT compile a lambda function
    #[cfg(feature = "jit")]
    fn try_jit_compile_lambda(&mut self, name: &str, params: &[Parameter], body: &Expression) {
        // Create a temporary FunctionDef from the lambda
        let func_def = FunctionDef {
            name: name.to_string(),
            params: params.to_vec(),
            return_type: None, // Type inference would handle this
            body: body.clone(),
            attributes: Vec::new(),
            span: Span::new(0, 0, 0, 0), // Dummy span
        };

        // Try to JIT compile if suitable
        if self.can_jit_compile(&func_def) {
            match self.jit_compile_function(&func_def) {
                Ok(compiled_name) => {
                    #[cfg(debug_assertions)]
                    eprintln!("✓ JIT compiled lambda function '{}'", compiled_name);
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("⚠ JIT compilation failed for lambda '{}': {}", name, e);
                }
            }
        }
    }

    #[cfg(not(feature = "jit"))]
    fn try_jit_compile_lambda(&mut self, _name: &str, _params: &[Parameter], _body: &Expression) {
        // JIT compilation not available
    }

    /// Evaluate a block with statements and optional result expression
    fn eval_block(
        &mut self,
        statements: &[Statement],
        result: &Option<Box<Expression>>,
    ) -> RuntimeResult<Value> {
        let mut last_value = Value::Unit;

        // Execute all statements in sequence
        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    last_value = self.eval_expression(expr)?;
                }
                Statement::LetBinding(let_binding) => {
                    let value = self.eval_expression(&let_binding.value)?;
                    self.environment
                        .define(let_binding.name.clone(), value.clone());
                    last_value = value;
                }
            }
        }

        // If there's a result expression, evaluate it; otherwise return last statement value
        if let Some(result_expr) = result {
            self.eval_expression(result_expr)
        } else {
            Ok(last_value)
        }
    }

    /// Evaluate parallel expressions using simplified approach
    fn eval_parallel_block(&mut self, expressions: &[Expression]) -> RuntimeResult<Value> {
        if expressions.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        // For single expression, just evaluate normally
        if expressions.len() == 1 {
            return Ok(Value::Array(vec![self.eval_expression(&expressions[0])?]));
        }

        // For multiple expressions, evaluate sequentially but simulate parallel execution
        // This avoids the Send trait issues while maintaining the interface
        let mut results = Vec::new();

        for expr in expressions {
            let result = self.eval_expression(expr)?;
            results.push(result);
        }

        #[cfg(debug_assertions)]
        eprintln!("⚡ Parallel block evaluation completed (simulated)");

        Ok(Value::Array(results))
    }

    /// Evaluate async spawn expression with simplified approach
    fn eval_async_spawn(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = AsyncTask::new(task_id);

        // For now, evaluate synchronously to avoid Send trait issues
        let result = self.eval_expression(expression);
        task.set_result(result);

        // Store the task for later retrieval
        self.async_tasks.insert(task_id, task.clone());

        #[cfg(debug_assertions)]
        eprintln!("🚀 Async spawn completed (synchronous fallback)");

        Ok(Value::AsyncHandle(task))
    }

    /// Evaluate async wait expression with proper synchronization
    fn eval_async_wait(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        let handle_value = self.eval_expression(expression)?;

        match handle_value {
            Value::AsyncHandle(task) => {
                // Polling-based wait with timeout
                let timeout = Duration::from_secs(30); // 30 second timeout
                let start_time = Instant::now();

                while !task.is_complete() {
                    if start_time.elapsed() > timeout {
                        return Err(RuntimeError::Generic {
                            message: "Async operation timed out".to_string(),
                        });
                    }

                    // Small sleep to avoid busy waiting
                    thread::sleep(Duration::from_millis(10));
                }

                // Retrieve and return the result
                match task.get_result() {
                    Some(Ok(value)) => Ok(value),
                    Some(Err(e)) => Err(e),
                    None => Err(RuntimeError::Generic {
                        message: "Async task completed but no result available".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Expected AsyncHandle, got {}", handle_value.type_name()),
            }),
        }
    }

    /// Evaluate GPU directive expression with computation mode optimization
    fn eval_gpu_directive(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        // Determine optimal computation mode based on expression complexity
        let old_mode = self.gpu_mode;
        self.gpu_mode = self.determine_optimal_gpu_mode(expression);

        let result = match self.gpu_mode {
            GpuMode::Cpu => {
                // Standard CPU evaluation
                self.eval_expression(expression)
            }
            GpuMode::Simd => {
                // Use SIMD-optimized evaluation for mathematical operations
                self.eval_with_simd_optimization(expression)
            }
            GpuMode::Parallel => {
                // Use parallel evaluation for array/matrix operations
                self.eval_with_parallel_optimization(expression)
            }
        };

        // Restore previous GPU mode
        self.gpu_mode = old_mode;

        result
    }

    /// Determine optimal GPU computation mode based on expression analysis
    fn determine_optimal_gpu_mode(&self, expr: &Expression) -> GpuMode {
        match expr {
            // Matrix operations benefit from parallel computation
            Expression::MatrixLiteral(rows, _) if rows.len() > 4 => GpuMode::Parallel,
            Expression::BinaryOp {
                operator: BinaryOperator::MatMul,
                ..
            } => GpuMode::Parallel,

            // Array operations with mathematical computations can use SIMD
            Expression::ArrayLiteral(elements, _) if elements.len() > 8 => GpuMode::Simd,

            // Complex mathematical expressions benefit from SIMD
            Expression::BinaryOp {
                operator:
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Pow,
                ..
            } => GpuMode::Simd,

            // Parallel blocks should use parallel computation
            Expression::Parallel { .. } => GpuMode::Parallel,

            // Default to CPU for other operations
            _ => GpuMode::Cpu,
        }
    }

    /// Evaluate expression with SIMD optimization simulation
    fn eval_with_simd_optimization(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::ArrayLiteral(elements, _) => {
                // Simulate SIMD processing by evaluating in batches
                let batch_size = 4; // Simulate 128-bit SIMD (4 x 32-bit values)
                let mut results = Vec::with_capacity(elements.len());

                for chunk in elements.chunks(batch_size) {
                    // Simulate parallel evaluation of chunk
                    let chunk_results: Result<Vec<_>, _> = chunk
                        .iter()
                        .map(|expr| self.eval_expression(expr))
                        .collect();

                    results.extend(chunk_results?);
                }

                Ok(Value::Array(results))
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                // For mathematical operations, evaluate normally but simulate SIMD speedup
                let start = Instant::now();
                let result = self.eval_binary_op(left, operator, right)?;
                let _elapsed = start.elapsed();

                // In a real implementation, this would use actual SIMD instructions
                #[cfg(debug_assertions)]
                eprintln!("🚀 SIMD-optimized operation completed");

                Ok(result)
            }

            _ => self.eval_expression(expression),
        }
    }

    /// Evaluate expression with parallel optimization (simplified)
    fn eval_with_parallel_optimization(&mut self, expression: &Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::MatrixLiteral(rows, _) => {
                // For now, use standard evaluation to avoid thread issues
                #[cfg(debug_assertions)]
                eprintln!("⚡ Matrix evaluation (CPU fallback)");

                self.eval_matrix_literal(rows)
            }

            Expression::BinaryOp {
                left,
                operator: BinaryOperator::MatMul,
                right,
                ..
            } => {
                // Use standard matrix multiplication
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;

                #[cfg(debug_assertions)]
                eprintln!("⚡ Matrix multiplication (CPU)");

                left_val.matrix_multiply(&right_val)
            }

            _ => self.eval_expression(expression),
        }
    }

    /// Simplified matrix multiplication (removed parallel version to avoid Send issues)
    fn _simple_matrix_multiply(&self, a: &[Vec<Value>], b: &[Vec<Value>]) -> RuntimeResult<Value> {
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

    /// Clean up completed async tasks
    pub fn cleanup_async_tasks(&mut self) {
        self.async_tasks.retain(|_, task| !task.is_complete());
    }

    /// Get statistics about async tasks
    pub fn get_async_stats(&self) -> (usize, usize) {
        let total = self.async_tasks.len();
        let completed = self
            .async_tasks
            .values()
            .filter(|t| t.is_complete())
            .count();
        (total, completed)
    }

    /// Check if a function can be JIT compiled
    #[cfg(feature = "jit")]
    fn can_jit_compile(&self, func_def: &FunctionDef) -> bool {
        // Only JIT compile if we have a JIT context
        if self.jit_context.is_none() {
            return false;
        }

        // Simple heuristics for JIT compilation suitability
        // For now, JIT compile functions with mathematical operations
        self.is_jit_suitable_function(func_def)
    }

    #[cfg(not(feature = "jit"))]
    fn can_jit_compile(&self, _func_def: &FunctionDef) -> bool {
        false
    }

    /// Check if function is suitable for JIT compilation
    #[cfg(feature = "jit")]
    fn is_jit_suitable_function(&self, func_def: &FunctionDef) -> bool {
        // Check if function contains operations that benefit from JIT compilation
        self.contains_mathematical_operations(&func_def.body)
    }

    /// Recursively check if expression tree contains mathematical operations
    #[cfg(feature = "jit")]
    fn contains_mathematical_operations(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Binary {
                op, left, right, ..
            } => {
                matches!(
                    op,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Modulo
                        | BinaryOperator::Power
                ) || self.contains_mathematical_operations(left)
                    || self.contains_mathematical_operations(right)
            }
            Expression::Unary { op, expr, .. } => {
                matches!(op, UnaryOperator::Minus | UnaryOperator::Plus)
                    || self.contains_mathematical_operations(expr)
            }
            Expression::FunctionCall { .. } => true, // Function calls can benefit from JIT
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.contains_mathematical_operations(condition)
                    || self.contains_mathematical_operations(then_branch)
                    || else_branch
                        .as_ref()
                        .map_or(false, |e| self.contains_mathematical_operations(e))
            }
            Expression::Block {
                statements, result, ..
            } => {
                statements
                    .iter()
                    .any(|s| self.contains_mathematical_operations(s))
                    || result
                        .as_ref()
                        .map_or(false, |r| self.contains_mathematical_operations(r))
            }
            _ => false,
        }
    }

    /// JIT compile a function
    #[cfg(feature = "jit")]
    fn jit_compile_function(&mut self, func_def: &FunctionDef) -> Result<String, JitError> {
        if let Some(ref mut jit_context) = self.jit_context {
            jit_context.compile_function(func_def)
        } else {
            Err(JitError::NotInitialized)
        }
    }

    #[cfg(not(feature = "jit"))]
    fn jit_compile_function(&mut self, _func_def: &FunctionDef) -> Result<String, JitError> {
        Err(JitError::NotAvailable)
    }

    /// Load a module and cache it
    pub fn load_module(&mut self, module_name: &str) -> Result<(), RuntimeError> {
        // Check if module is already cached
        if self.module_cache.contains_key(module_name) {
            return Ok(());
        }

        // For now, create a simple mock module environment
        let mut module_env = Environment::new();
        module_env.set_variable(format!("{}_loaded", module_name), Value::Bool(true));

        // Cache the module
        self.module_cache
            .insert(module_name.to_string(), module_env);
        Ok(())
    }

    /// Get a cached module environment
    pub fn get_module(&self, module_name: &str) -> Option<&Environment> {
        self.module_cache.get(module_name)
    }

    /// Clear module cache
    pub fn clear_module_cache(&mut self) {
        self.module_cache.clear();
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
        Value::AsyncHandle(_) => "<async handle>".to_string(),
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
            Value::AsyncHandle(task) => {
                if task.is_complete() {
                    write!(f, "AsyncHandle(completed:{})", task.id)
                } else {
                    write!(f, "AsyncHandle(pending:{})", task.id)
                }
            }
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
            (Value::AsyncHandle(a), Value::AsyncHandle(b)) => a.id == b.id,
            _ => false,
        }
    }
}
