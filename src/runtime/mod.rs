use crate::ast::nodes::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Runtime execution environment
pub struct Runtime {
    /// Memory management
    pub memory_manager: MemoryManager,
    /// Thread pool for parallel execution
    pub thread_pool: ThreadPool,
    /// Global variable storage
    pub globals: Arc<Mutex<HashMap<String, Value>>>,
    /// Execution stack
    pub stack: Vec<StackFrame>,
}

/// Memory management system
pub struct MemoryManager {
    /// Heap allocations
    heap: HashMap<u64, HeapObject>,
    /// Next allocation ID
    next_id: u64,
    /// Garbage collection threshold
    gc_threshold: usize,
    /// Current heap size
    heap_size: usize,
}

/// Heap object representation
#[derive(Debug, Clone)]
pub struct HeapObject {
    pub id: u64,
    pub data: Value,
    pub ref_count: u32,
    pub marked: bool,
}

/// Thread pool for parallel execution
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

/// Stack frame for function calls
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub locals: HashMap<String, Value>,
    pub line_number: usize,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            memory_manager: MemoryManager::new(),
            thread_pool: ThreadPool::new(4), // 4 worker threads
            globals: Arc::new(Mutex::new(HashMap::new())),
            stack: Vec::new(),
        }
    }

    /// Execute a Matrix Language program
    pub fn execute(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        // Initialize global scope
        self.push_frame("main".to_string());

        let mut result = Value::Null;

        for item in &program.items {
            result = self.execute_item(item)?;
        }

        self.pop_frame();
        Ok(result)
    }

    /// Execute a top-level item
    pub fn execute_item(&mut self, item: &Item) -> Result<Value, RuntimeError> {
        match item {
            Item::LetBinding(binding) => {
                let value = self.evaluate_expression(&binding.value)?;
                self.set_variable(binding.name.clone(), value.clone());
                Ok(value)
            }
            Item::FunctionDef(func_def) => {
                // Store function definition for later calls
                let function_value = Value::Function {
                    name: func_def.name.clone(),
                    params: func_def.params.iter().map(|p| p.name.clone()).collect(),
                    body: func_def.body.clone(),
                };
                self.set_variable(func_def.name.clone(), function_value);
                Ok(Value::Null)
            }
            _ => {
                // For now, other items are not executable at runtime
                Ok(Value::Null)
            }
        }
    }

    /// Execute a single statement
    pub fn execute_statement(&mut self, stmt: &Statement) -> Result<Value, RuntimeError> {
        match stmt {
            Statement::Expression(expr) => self.evaluate_expression(expr),
            Statement::LetBinding(binding) => {
                let value = self.evaluate_expression(&binding.value)?;
                self.set_variable(binding.name.clone(), value.clone());
                Ok(value)
            }
        }
    }

    /// Evaluate an expression
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::IntLiteral(n, _) => Ok(Value::Number(*n as f64)),
            Expression::FloatLiteral(n, _) => Ok(Value::Number(*n)),
            Expression::BoolLiteral(b, _) => Ok(Value::Boolean(*b)),
            Expression::StringLiteral(s, _) => Ok(Value::String(s.clone())),
            Expression::Identifier(name, _) => self
                .get_variable(name)
                .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone())),
            Expression::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.apply_binary_op(&left_val, operator, &right_val)
            }
            Expression::UnaryOp {
                operator, operand, ..
            } => {
                let val = self.evaluate_expression(operand)?;
                self.apply_unary_op(operator, &val)
            }
            Expression::FunctionCall { function, args, .. } => {
                if let Expression::Identifier(name, _) = function.as_ref() {
                    self.call_function(name, args)
                } else {
                    Err(RuntimeError::TypeError(
                        "Function calls must use identifiers".to_string(),
                    ))
                }
            }
            Expression::MatrixLiteral(rows, _) => {
                let mut matrix_rows = Vec::new();
                for row in rows {
                    let mut matrix_row = Vec::new();
                    for expr in row {
                        let val = self.evaluate_expression(expr)?;
                        if let Value::Number(n) = val {
                            matrix_row.push(n);
                        } else {
                            return Err(RuntimeError::TypeError(
                                "Matrix elements must be numbers".to_string(),
                            ));
                        }
                    }
                    matrix_rows.push(matrix_row);
                }
                Ok(Value::Matrix(matrix_rows))
            }
            Expression::ArrayLiteral(exprs, _) => {
                let mut vector = Vec::new();
                for expr in exprs {
                    let val = self.evaluate_expression(expr)?;
                    if let Value::Number(n) = val {
                        vector.push(n);
                    } else {
                        return Err(RuntimeError::TypeError(
                            "Array elements must be numbers".to_string(),
                        ));
                    }
                }
                Ok(Value::Vector(vector))
            }
            Expression::IfExpression {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_value = self.evaluate_expression(condition)?;
                if self.is_truthy(&cond_value) {
                    self.evaluate_expression(then_branch)
                } else if let Some(else_expr) = else_branch {
                    self.evaluate_expression(else_expr)
                } else {
                    Ok(Value::Null)
                }
            }
            Expression::Block {
                statements, result, ..
            } => {
                let mut last_value = Value::Null;
                for stmt in statements {
                    last_value = self.execute_statement(stmt)?;
                }
                if let Some(result_expr) = result {
                    self.evaluate_expression(result_expr)
                } else {
                    Ok(last_value)
                }
            }
            _ => {
                // For other expression types, return null for now
                Ok(Value::Null)
            }
        }
    }

    /// Apply binary operations
    fn apply_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => match op {
                BinaryOperator::Add => Ok(Value::Number(a + b)),
                BinaryOperator::Sub => Ok(Value::Number(a - b)),
                BinaryOperator::Mul => Ok(Value::Number(a * b)),
                BinaryOperator::Div => {
                    if *b == 0.0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Number(a / b))
                    }
                }
                BinaryOperator::Pow => Ok(Value::Number(a.powf(*b))),
                BinaryOperator::Mod => Ok(Value::Number(a % b)),
                BinaryOperator::Eq => Ok(Value::Boolean((a - b).abs() < f64::EPSILON)),
                BinaryOperator::Ne => Ok(Value::Boolean((a - b).abs() >= f64::EPSILON)),
                BinaryOperator::Lt => Ok(Value::Boolean(a < b)),
                BinaryOperator::Le => Ok(Value::Boolean(a <= b)),
                BinaryOperator::Gt => Ok(Value::Boolean(a > b)),
                BinaryOperator::Ge => Ok(Value::Boolean(a >= b)),
                _ => Err(RuntimeError::TypeError(format!(
                    "Invalid operator {:?} for numbers",
                    op
                ))),
            },
            (Value::String(a), Value::String(b)) => match op {
                BinaryOperator::Add => Ok(Value::String(format!("{}{}", a, b))),
                BinaryOperator::Eq => Ok(Value::Boolean(a == b)),
                BinaryOperator::Ne => Ok(Value::Boolean(a != b)),
                _ => Err(RuntimeError::TypeError(format!(
                    "Invalid operator {:?} for strings",
                    op
                ))),
            },
            (Value::Boolean(a), Value::Boolean(b)) => match op {
                BinaryOperator::And => Ok(Value::Boolean(*a && *b)),
                BinaryOperator::Or => Ok(Value::Boolean(*a || *b)),
                BinaryOperator::Eq => Ok(Value::Boolean(a == b)),
                BinaryOperator::Ne => Ok(Value::Boolean(a != b)),
                _ => Err(RuntimeError::TypeError(format!(
                    "Invalid operator {:?} for booleans",
                    op
                ))),
            },
            (Value::Vector(a), Value::Vector(b)) => match op {
                BinaryOperator::Add => {
                    if a.len() != b.len() {
                        return Err(RuntimeError::TypeError(
                            "Vector addition requires same length".to_string(),
                        ));
                    }
                    let result = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
                    Ok(Value::Vector(result))
                }
                BinaryOperator::Sub => {
                    if a.len() != b.len() {
                        return Err(RuntimeError::TypeError(
                            "Vector subtraction requires same length".to_string(),
                        ));
                    }
                    let result = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
                    Ok(Value::Vector(result))
                }
                _ => Err(RuntimeError::TypeError(format!(
                    "Invalid operator {:?} for vectors",
                    op
                ))),
            },
            _ => Err(RuntimeError::TypeError(
                "Type mismatch in binary operation".to_string(),
            )),
        }
    }

    /// Apply unary operations
    fn apply_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Result<Value, RuntimeError> {
        match (op, operand) {
            (UnaryOperator::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
            (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ => Err(RuntimeError::TypeError(
                "Invalid unary operation".to_string(),
            )),
        }
    }

    /// Call a function
    fn call_function(&mut self, name: &str, args: &[Expression]) -> Result<Value, RuntimeError> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        // Handle basic built-in functions
        match name {
            "print" => {
                for (i, arg) in arg_values.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    match arg {
                        Value::Number(n) => print!("{}", n),
                        Value::String(s) => print!("{}", s),
                        Value::Boolean(b) => print!("{}", b),
                        _ => print!("{:?}", arg),
                    }
                }
                println!();
                Ok(Value::Null)
            }
            "sin" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "sin expects 1 argument".to_string(),
                    ));
                }
                if let Value::Number(n) = &arg_values[0] {
                    Ok(Value::Number(n.sin()))
                } else {
                    Err(RuntimeError::TypeError("sin expects a number".to_string()))
                }
            }
            "cos" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "cos expects 1 argument".to_string(),
                    ));
                }
                if let Value::Number(n) = &arg_values[0] {
                    Ok(Value::Number(n.cos()))
                } else {
                    Err(RuntimeError::TypeError("cos expects a number".to_string()))
                }
            }
            "sqrt" => {
                if arg_values.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "sqrt expects 1 argument".to_string(),
                    ));
                }
                if let Value::Number(n) = &arg_values[0] {
                    Ok(Value::Number(n.sqrt()))
                } else {
                    Err(RuntimeError::TypeError("sqrt expects a number".to_string()))
                }
            }
            "pow" => {
                if arg_values.len() != 2 {
                    return Err(RuntimeError::TypeError(
                        "pow expects 2 arguments".to_string(),
                    ));
                }
                if let (Value::Number(base), Value::Number(exp)) = (&arg_values[0], &arg_values[1])
                {
                    Ok(Value::Number(base.powf(*exp)))
                } else {
                    Err(RuntimeError::TypeError(
                        "pow expects two numbers".to_string(),
                    ))
                }
            }
            _ => Err(RuntimeError::UndefinedVariable(format!(
                "Unknown function: {}",
                name
            ))),
        }
    }

    /// Index access for arrays/vectors/matrices
    fn index_access(&self, object: &Value, index: &Value) -> Result<Value, RuntimeError> {
        match (object, index) {
            (Value::Vector(vec), Value::Number(idx)) => {
                let i = *idx as usize;
                if i < vec.len() {
                    Ok(Value::Number(vec[i]))
                } else {
                    Err(RuntimeError::IndexOutOfBounds)
                }
            }
            (Value::Matrix(matrix), Value::Number(row_idx)) => {
                let row = *row_idx as usize;
                if row < matrix.len() {
                    Ok(Value::Vector(matrix[row].clone()))
                } else {
                    Err(RuntimeError::IndexOutOfBounds)
                }
            }
            _ => Err(RuntimeError::TypeError(
                "Invalid index operation".to_string(),
            )),
        }
    }

    /// Check if a value is truthy
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }

    /// Push a new stack frame
    fn push_frame(&mut self, function_name: String) {
        self.stack.push(StackFrame {
            function_name,
            locals: HashMap::new(),
            line_number: 0,
        });
    }

    /// Pop the top stack frame
    fn pop_frame(&mut self) {
        self.stack.pop();
    }

    /// Get a variable from current scope or global scope
    fn get_variable(&self, name: &str) -> Option<Value> {
        // Check local scope first
        if let Some(frame) = self.stack.last() {
            if let Some(value) = frame.locals.get(name) {
                return Some(value.clone());
            }
        }

        // Check global scope
        if let Ok(globals) = self.globals.lock() {
            globals.get(name).cloned()
        } else {
            None
        }
    }

    /// Set a variable in current scope
    fn set_variable(&mut self, name: String, value: Value) {
        if let Some(frame) = self.stack.last_mut() {
            frame.locals.insert(name, value);
        } else if let Ok(mut globals) = self.globals.lock() {
            globals.insert(name, value);
        }
    }

    /// Trigger garbage collection
    pub fn gc(&mut self) {
        self.memory_manager.collect_garbage();
    }
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            heap: HashMap::new(),
            next_id: 1,
            gc_threshold: 1000,
            heap_size: 0,
        }
    }

    /// Allocate memory for a value
    pub fn allocate(&mut self, value: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let heap_obj = HeapObject {
            id,
            data: value,
            ref_count: 1,
            marked: false,
        };

        self.heap.insert(id, heap_obj);
        self.heap_size += 1;

        // Trigger GC if threshold reached
        if self.heap_size > self.gc_threshold {
            self.collect_garbage();
        }

        id
    }

    /// Deallocate memory
    pub fn deallocate(&mut self, id: u64) {
        if self.heap.remove(&id).is_some() {
            self.heap_size -= 1;
        }
    }

    /// Mark and sweep garbage collection
    pub fn collect_garbage(&mut self) {
        // Mark phase - mark all reachable objects
        self.mark_reachable();

        // Sweep phase - deallocate unmarked objects
        let to_remove: Vec<u64> = self
            .heap
            .iter()
            .filter(|(_, obj)| !obj.marked)
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            self.deallocate(id);
        }

        // Reset marks for next collection
        for obj in self.heap.values_mut() {
            obj.marked = false;
        }
    }

    /// Mark reachable objects (simplified implementation)
    fn mark_reachable(&mut self) {
        // In a real implementation, this would traverse from root objects
        // For now, mark objects with positive ref_count
        for obj in self.heap.values_mut() {
            if obj.ref_count > 0 {
                obj.marked = true;
            }
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Execute a closure on the thread pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Runtime error types
#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeError(String),
    DivisionByZero,
    IndexOutOfBounds,
    StackOverflow,
    OutOfMemory,
    CustomError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            RuntimeError::StackOverflow => write!(f, "Stack overflow"),
            RuntimeError::OutOfMemory => write!(f, "Out of memory"),
            RuntimeError::CustomError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Runtime value representation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Vector(Vec<f64>),
    Matrix(Vec<Vec<f64>>),
    Function {
        name: String,
        params: Vec<String>,
        body: Expression,
    },
}
