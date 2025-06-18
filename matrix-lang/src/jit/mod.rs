// JIT Compilation Module for Matrix Language
// Provides Just-In-Time compilation using LLVM backend

use crate::ast::nodes::*;
use std::collections::HashMap;
use thiserror::Error;

// Re-export submodules
pub mod compiler;
pub mod executor;
pub mod optimization;

pub use compiler::JitCompiler;
pub use executor::JitExecutor;
pub use optimization::{JitOptimizer, OptimizationLevel};

// Re-export LLVM types when JIT is enabled
#[cfg(feature = "jit")]
pub use inkwell::{
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    values::FunctionValue,
};

/// JIT compilation errors
#[derive(Error, Debug, Clone)]
pub enum JitError {
    #[error("JIT compilation not available")]
    NotAvailable,

    #[error("JIT context not initialized")]
    NotInitialized,

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Type inference failed: {0}")]
    TypeInferenceFailed(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("LLVM error: {0}")]
    LlvmError(String),
}

/// Statistics about JIT compilation
#[derive(Debug, Clone)]
pub struct JitStats {
    pub functions_compiled: usize,
    pub total_compilation_time_ms: u64,
    pub optimizations_applied: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl Default for JitStats {
    fn default() -> Self {
        Self {
            functions_compiled: 0,
            total_compilation_time_ms: 0,
            optimizations_applied: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

/// Compiled function representation
#[cfg(feature = "jit")]
pub struct CompiledFunction<'ctx> {
    pub name: String,
    pub function_value: FunctionValue<'ctx>,
    pub param_types: Vec<crate::types::types::Type>,
    pub return_type: crate::types::types::Type,
    pub optimization_level: OptimizationLevel,
}

#[cfg(not(feature = "jit"))]
pub struct CompiledFunction {
    pub name: String,
}

/// Main JIT context that manages compilation and execution
#[cfg(feature = "jit")]
pub struct JitContext<'ctx> {
    context: &'ctx Context,
    execution_engine: ExecutionEngine<'ctx>,
    compiler: JitCompiler<'ctx>,
    optimizer: JitOptimizer<'ctx>,
    compiled_functions: HashMap<String, CompiledFunction<'ctx>>,
    stats: JitStats,
    optimization_level: OptimizationLevel,
}

#[cfg(feature = "jit")]
impl<'ctx> JitContext<'ctx> {
    /// Create a new JIT context with default optimization level
    pub fn new() -> Result<Self, JitError> {
        Self::with_optimization(OptimizationLevel::Default)
    }

    /// Create a new JIT context with specified optimization level
    pub fn with_optimization(opt_level: OptimizationLevel) -> Result<Self, JitError> {
        let context = Box::leak(Box::new(Context::create()));
        let compiler = JitCompiler::new(context, "matrix_jit");
        let execution_engine = compiler.module
            .create_jit_execution_engine(inkwell::OptimizationLevel::Default)
            .map_err(|e| JitError::LlvmError(e.to_string()))?;
        let optimizer = JitOptimizer::new(opt_level.clone());

        Ok(JitContext {
            context,
            execution_engine,
            compiler,
            optimizer,
            compiled_functions: HashMap::new(),
            stats: JitStats::default(),
            optimization_level: opt_level,
        })
    }

    /// Compile a function to LLVM IR and native code
    pub fn compile_function(&mut self, func_def: &FunctionDef) -> Result<String, JitError> {
        let start_time = std::time::Instant::now();

        // Check if function is already compiled
        if self.compiled_functions.contains_key(&func_def.name) {
            self.stats.cache_hits += 1;
            return Ok(func_def.name.clone());
        }

        self.stats.cache_misses += 1;

        // Compile function to LLVM IR
        let function_value = self.compiler.compile_function_def(func_def)?;

        // Apply optimizations
        self.optimizer.optimize_module(&self.compiler.module)?;
        self.stats.optimizations_applied += 1;

        // Create compiled function record
        let compiled_func = CompiledFunction {
            name: func_def.name.clone(),
            function_value,
            param_types: func_def.params.iter()
                .map(|p| p.param_type.clone().unwrap_or(crate::types::types::Type::Any))
                .collect(),
            return_type: func_def.return_type.clone().unwrap_or(crate::types::types::Type::Any),
            optimization_level: self.optimization_level.clone(),
        };

        // Store compiled function
        self.compiled_functions.insert(func_def.name.clone(), compiled_func);

        // Update statistics
        self.stats.functions_compiled += 1;
        self.stats.total_compilation_time_ms += start_time.elapsed().as_millis() as u64;

        Ok(func_def.name.clone())
    }

    /// Execute a compiled function
    pub fn execute_function(&self, name: &str, args: &[crate::eval::interpreter::Value]) -> Result<crate::eval::interpreter::Value, JitError> {
        if let Some(compiled_func) = self.compiled_functions.get(name) {
            let executor = JitExecutor::new(&self.execution_engine);
            executor.execute_function(name, args)
                .map_err(|e| JitError::ExecutionFailed(e.to_string()))
        } else {
            Err(JitError::FunctionNotFound(name.to_string()))
        }
    }

    /// Check if a function is compiled
    pub fn is_function_compiled(&self, name: &str) -> bool {
        self.compiled_functions.contains_key(name)
    }

    /// Get compilation statistics
    pub fn get_stats(&self) -> &JitStats {
        &self.stats
    }

    /// Clear compiled function cache
    pub fn clear_cache(&mut self) {
        self.compiled_functions.clear();
        self.stats.cache_hits = 0;
        self.stats.cache_misses = 0;
    }

    /// Set optimization level for future compilations
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
        self.optimizer = JitOptimizer::new(level);
    }

    /// Get the LLVM module for inspection
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.compiler.module
    }

    /// Dump LLVM IR to string for debugging
    pub fn dump_ir(&self) -> String {
        self.compiler.module.print_to_string().to_string()
    }
}

// Stub implementation when JIT is not available
#[cfg(not(feature = "jit"))]
pub struct JitContext;

#[cfg(not(feature = "jit"))]
impl JitContext {
    pub fn new() -> Result<Self, JitError> {
        Err(JitError::NotAvailable)
    }

    pub fn with_optimization(_opt_level: OptimizationLevel) -> Result<Self, JitError> {
        Err(JitError::NotAvailable)
    }

    pub fn compile_function(&mut self, _func_def: &FunctionDef) -> Result<String, JitError> {
        Err(JitError::NotAvailable)
    }

    pub fn execute_function(&self, _name: &str, _args: &[crate::eval::interpreter::Value]) -> Result<crate::eval::interpreter::Value, JitError> {
        Err(JitError::NotAvailable)
    }

    pub fn is_function_compiled(&self, _name: &str) -> bool {
        false
    }

    pub fn get_stats(&self) -> &JitStats {
        &JitStats::default()
    }

    pub fn clear_cache(&mut self) {}

    pub fn set_optimization_level(&mut self, _level: OptimizationLevel) {}

    pub fn dump_ir(&self) -> String {
        "JIT not available".to_string()
    }
}
