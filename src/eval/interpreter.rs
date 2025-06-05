use crate::ast::*;
use crate::types::*;
use std::collections::HashMap;
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
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Runtime values
#[derive(Debug, Clone, PartialEq)]
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
                message: format!("Cannot subtract {} and {}", self.type_name(), other.type_name()),
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
                message: format!("Cannot multiply {} and {}", self.type_name(), other.type_name()),
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
            },
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            },
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            },
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            },
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot divide {} and {}", self.type_name(), other.type_name()),
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
                        message: format!("Matrix dimensions incompatible: {}x{} and {}x{}", 
                                       rows_a, cols_a, rows_b, cols_b),
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
            },
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot matrix multiply {} and {}", self.type_name(), other.type_name()),
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
            _ => return Err(RuntimeError::TypeError {
                message: format!("Cannot compare {} and {}", self.type_name(), other.type_name()),
            }),
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
        self.bindings.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get(name))
        })
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
        // Built-in functions
        self.environment.define("print".to_string(), Value::BuiltinFunction {
            name: "print".to_string(),
            arity: 1,
            func: |args| {
                if let Some(arg) = args.first() {
                    println!("{}", format_value(arg));
                }
                Ok(Value::Unit)
            },
        });
        
        self.environment.define("len".to_string(), Value::BuiltinFunction {
            name: "len".to_string(),
            arity: 1,
            func: |args| {
                match &args[0] {
                    Value::Array(arr) => Ok(Value::Int(arr.len() as i64)),
                    Value::Matrix(mat) => Ok(Value::Int(mat.len() as i64)),
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot get length of {}", args[0].type_name()),
                    }),
                }
            },
        });
        
        self.environment.define("abs".to_string(), Value::BuiltinFunction {
            name: "abs".to_string(),
            arity: 1,
            func: |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(i.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot get absolute value of {}", args[0].type_name()),
                    }),
                }
            },
        });
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
            },
            
            Item::TypeclassDef(_) => {
                // Typeclasses are handled at compile time
                Ok(Value::Unit)
            },
            
            Item::InstanceDef(_) => {
                // Instances are handled at compile time
                Ok(Value::Unit)
            },
            
            Item::FunctionDef(func_def) => {
                let function_value = Value::Function {
                    params: func_def.params.clone(),
                    body: func_def.body.clone(),
                    closure: self.environment.clone(),
                };
                
                self.environment.define(func_def.name.clone(), function_value);
                Ok(Value::Unit)
            },
            
            Item::LetBinding(let_binding) => {
                let value = self.eval_expression(&let_binding.value)?;
                self.environment.define(let_binding.name.clone(), value.clone());
                Ok(value)
            },
            
            Item::Import(_) => {
                // TODO: Implement imports
                Ok(Value::Unit)
            },
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
                    Err(RuntimeError::UndefinedVariable {
                        name: name.clone(),
                    })
                }
            },
            
            Expression::BinaryOp { left, operator, right, .. } => {
                self.eval_binary_op(left, operator, right)
            },
            
            Expression::UnaryOp { operator, operand, .. } => {
                self.eval_unary_op(operator, operand)
            },
            
            Expression::FunctionCall { function, args, .. } => {
                self.eval_function_call(function, args)
            },
            
            Expression::FieldAccess { object, field, .. } => {
                self.eval_field_access(object, field)
            },
            
            Expression::StructCreation { name, fields, .. } => {
                self.eval_struct_creation(name, fields)
            },
            
            Expression::ArrayLiteral(elements, _) => {
                self.eval_array_literal(elements)
            },
            
            Expression::MatrixLiteral(rows, _) => {
                self.eval_matrix_literal(rows)
            },
            
            Expression::MatrixComprehension { element, generators, .. } => {
                self.eval_matrix_comprehension(element, generators)
            },
            
            Expression::IfExpression { condition, then_branch, else_branch, .. } => {
                self.eval_if_expression(condition, then_branch, else_branch)
            },
            
            Expression::Match { expression, arms, .. } => {
                self.eval_match_expression(expression, arms)
            },
            
            Expression::Let { bindings, body, .. } => {
                self.eval_let_expression(bindings, body)
            },
            
            Expression::Lambda { params, body, .. } => {
                Ok(Value::Function {
                    params: params.clone(),
                    body: (**body).clone(),
                    closure: self.environment.clone(),
                })
            },
            
            Expression::Block { statements, result, .. } => {
                self.eval_block(statements, result)
            },
            
            Expression::Parallel { expressions, .. } => {
                // For now, evaluate serially
                // TODO: Implement parallel execution
                self.eval_parallel_block(expressions)
            },
            
            Expression::Spawn { expression, .. } => {
                // For now, evaluate directly
                // TODO: Implement async spawn
                self.eval_expression(expression)
            },
            
            Expression::Wait { expression, .. } => {
                // For now, evaluate directly
                // TODO: Implement async wait
                self.eval_expression(expression)
            },
            
            Expression::GpuDirective { expression, .. } => {
                // For now, evaluate on CPU
                // TODO: Implement GPU execution
                self.eval_expression(expression)
            },
              Expression::OptionalAccess { object, field, fallback, .. } => {
                self.eval_optional_access(object, field, &Some(fallback.clone()))
            },
            
            Expression::Range { start, end, inclusive, .. } => {
                self.eval_range(start, end, *inclusive)
            },
        }
    }
  
      fn eval_binary_op(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> RuntimeResult<Value> {
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
            },
            BinaryOperator::Lt => left_val.less_than(&right_val),
            BinaryOperator::Le => {
                let lt = left_val.less_than(&right_val)?;
                let eq = left_val.equals(&right_val)?;
                match (lt, eq) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => unreachable!(),
                }
            },
            BinaryOperator::Gt => {
                let lt = left_val.less_than(&right_val)?;
                let eq = left_val.equals(&right_val)?;
                match (lt, eq) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(!a && !b)),
                    _ => unreachable!(),
                }
            },
            BinaryOperator::Ge => {
                let lt = left_val.less_than(&right_val)?;
                match lt {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => unreachable!(),
                }
            },
            BinaryOperator::And => {
                if left_val.is_truthy() {
                    Ok(right_val)
                } else {
                    Ok(left_val)
                }
            },
            BinaryOperator::Or => {
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            },
            BinaryOperator::OptionalOr => {
                // TODO: Implement proper Option handling
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    Ok(right_val)
                }
            },
            _ => Err(RuntimeError::TypeError {
                message: format!("Unimplemented binary operator: {:?}", op),
            }),
        }
    }
      fn eval_unary_op(&mut self, op: &UnaryOperator, expr: &Expression) -> RuntimeResult<Value> {
        let value = self.eval_expression(expr)?;
        
        match op {
            UnaryOperator::Not => Ok(Value::Bool(!value.is_truthy())),
            UnaryOperator::Neg => {
                match value {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot negate {}", value.type_name()),
                    }),
                }
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
                    },
                    _ => Err(RuntimeError::TypeError {
                        message: format!("Cannot transpose {}", value.type_name()),
                    }),
                }
            },
        }
    }
    
    fn eval_function_call(&mut self, func: &Expression, args: &[Expression]) -> RuntimeResult<Value> {
        let func_value = self.eval_expression(func)?;
        let arg_values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expression(arg)).collect();
        let arg_values = arg_values?;
        
        match func_value {
            Value::Function { params, body, closure } => {
                if params.len() != arg_values.len() {
                    return Err(RuntimeError::FunctionCallError {
                        message: format!("Expected {} arguments, got {}", params.len(), arg_values.len()),
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
            },
            
            Value::BuiltinFunction { func, arity, .. } => {
                if arg_values.len() != arity {
                    return Err(RuntimeError::FunctionCallError {
                        message: format!("Expected {} arguments, got {}", arity, arg_values.len()),
                    });
                }
                
                func(&arg_values)
            },
            
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
            },
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot access field of {}", value.type_name()),
            }),
        }
    }
    
    fn eval_array_access(&mut self, array: &Expression, index: &Expression) -> RuntimeResult<Value> {
        let array_value = self.eval_expression(array)?;
        let index_value = self.eval_expression(index)?;
        
        let index = match index_value {
            Value::Int(i) => i as usize,
            _ => return Err(RuntimeError::TypeError {
                message: "Array index must be integer".to_string(),
            }),
        };
        
        match array_value {
            Value::Array(arr) => {
                if index >= arr.len() {
                    Err(RuntimeError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    })
                } else {
                    Ok(arr[index].clone())
                }
            },
            Value::Matrix(mat) => {
                if index >= mat.len() {
                    Err(RuntimeError::IndexOutOfBounds {
                        index,
                        length: mat.len(),
                    })
                } else {
                    Ok(Value::Array(mat[index].clone()))
                }
            },
            _ => Err(RuntimeError::TypeError {
                message: format!("Cannot index {}", array_value.type_name()),
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
            let row_values: Result<Vec<_>, _> = row.iter().map(|e| self.eval_expression(e)).collect();
            matrix_rows.push(row_values?);
        }
        
        Ok(Value::Matrix(matrix_rows))
    }
  
    
    fn eval_if_expression(&mut self, condition: &Expression, then_expr: &Expression, else_expr: &Option<Box<Expression>>) -> RuntimeResult<Value> {
        let cond_value = self.eval_expression(condition)?;
        
        if cond_value.is_truthy() {
            self.eval_expression(then_expr)
        } else if let Some(else_expr) = else_expr {
            self.eval_expression(else_expr)
        } else {
            Ok(Value::Unit)
        }
    }
  
    
    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> RuntimeResult<bool> {
        match pattern {
            Pattern::Wildcard(_) => Ok(true),
            
            Pattern::Identifier(name, _) => {
                self.environment.define(name.clone(), value.clone());
                Ok(true)
            },
            
            Pattern::IntLiteral(expected, _) => {
                match value {
                    Value::Int(actual) => Ok(actual == expected),
                    _ => Ok(false),
                }
            },
            
            Pattern::FloatLiteral(expected, _) => {
                match value {
                    Value::Float(actual) => Ok((actual - expected).abs() < f64::EPSILON),
                    _ => Ok(false),
                }
            },
            
            Pattern::BoolLiteral(expected, _) => {
                match value {
                    Value::Bool(actual) => Ok(actual == expected),
                    _ => Ok(false),
                }
            },
            
            Pattern::StringLiteral(expected, _) => {
                match value {
                    Value::String(actual) => Ok(actual == expected),
                    _ => Ok(false),
                }
            },
            
            Pattern::Some(inner_pattern, _) => {
                // TODO: Implement proper Option handling
                self.match_pattern(inner_pattern, value)
            },
            
            Pattern::None(_) => {
                // TODO: Implement proper Option handling
                Ok(matches!(value, Value::Unit))
            },
            
            Pattern::Struct { name, fields, .. } => {
                match value {
                    Value::Struct { name: struct_name, fields: struct_fields } if struct_name == name => {
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
                    },
                    _ => Ok(false),
                }
            },
            
            Pattern::Array(patterns, _) => {
                match value {
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
                    },
                    _ => Ok(false),
                }
            },
        }
    }
    
    fn eval_let_expression(&mut self, bindings: &[LetBinding], body: &Expression) -> RuntimeResult<Value> {
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
    
    fn eval_block(&mut self, statements: &[Statement], result: &Option<Box<Expression>>) -> RuntimeResult<Value> {
        let old_env = self.environment.clone();
        
        // Execute statements
        for stmt in statements {
            match stmt {
                Statement::Expression(expr) => {
                    self.eval_expression(expr)?;
                },
                Statement::LetBinding(binding) => {
                    let value = self.eval_expression(&binding.value)?;
                    self.environment.define(binding.name.clone(), value);
                },
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

    fn eval_struct_creation(&mut self, name: &str, fields: &HashMap<String, Expression>) -> RuntimeResult<Value> {
        let mut field_values = std::collections::HashMap::new();
        
        for (field_name, field_expr) in fields {
            let field_value = self.eval_expression(field_expr)?;
            field_values.insert(field_name.clone(), field_value);
        }
        
        Ok(Value::Struct {
            name: name.to_string(),
            fields: field_values,
        })
    }    fn eval_matrix_comprehension(&mut self, element: &Expression, generators: &Vec<Generator>) -> RuntimeResult<Value> {
        // For now, implement a simple case - single generator
        if generators.is_empty() {
            return Err(RuntimeError::Generic { message: "Matrix comprehension requires at least one generator".to_string() });
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
            _ => return Err(RuntimeError::TypeError { message: "Generator must be an array".to_string() }),
        }
        
        Ok(Value::Matrix(result_rows))
    }    fn eval_match_expression(&mut self, expression: &Expression, arms: &Vec<MatchArm>) -> RuntimeResult<Value> {
        let _expr_value = self.eval_expression(expression)?;
        
        // For now, implement simple pattern matching
        // TODO: Implement proper pattern matching with guards
        for arm in arms {
            // Each arm should be a pattern => expression
            // For simplicity, just evaluate the first arm for now
            return self.eval_expression(&arm.body);
        }
        
        Err(RuntimeError::Generic { message: "No matching pattern found".to_string() })
    }

    fn eval_optional_access(&mut self, object: &Expression, field: &str, fallback: &Option<Box<Expression>>) -> RuntimeResult<Value> {
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
                    Err(RuntimeError::TypeError { message: "Optional access on non-struct type".to_string() })
                }
            }
        }
    }

    fn eval_range(&mut self, start: &Expression, end: &Expression, inclusive: bool) -> RuntimeResult<Value> {
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
            _ => Err(RuntimeError::TypeError { message: "Range bounds must be numbers".to_string() }),
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
        },
        Value::Matrix(mat) => {
            let rows: Vec<String> = mat.iter().map(|row| {
                let elements: Vec<String> = row.iter().map(format_value).collect();
                format!("[{}]", elements.join(", "))
            }).collect();
            format!("[{}]", rows.join(", "))
        },
        Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields.iter().map(|(k, v)| {
                format!("{}: {}", k, format_value(v))
            }).collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        },
        Value::Function { .. } => "<function>".to_string(),
        Value::BuiltinFunction { name, .. } => format!("<builtin: {}>", name),
    }
}
