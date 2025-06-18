// JIT Optimization Passes
// Handles optimization of JIT compiled code

use super::{JitError, OptimizationLevel};

#[cfg(feature = "jit")]
use inkwell::{
    module::Module,
    passes::{PassManager, PassManagerBuilder},
    OptimizationLevel as InkwellOptLevel,
};

/// Optimizer for JIT compiled code
#[cfg(feature = "jit")]
pub struct JitOptimizer<'ctx> {
    pass_manager: PassManager<Module<'ctx>>,
    optimization_level: OptimizationLevel,
}

#[cfg(feature = "jit")]
impl<'ctx> JitOptimizer<'ctx> {
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        let pass_manager = PassManager::create(());

        JitOptimizer {
            pass_manager,
            optimization_level,
        }
    }

    /// Apply optimization passes to a module
    pub fn optimize_module(&self, module: &Module<'ctx>) -> Result<(), JitError> {
        match self.optimization_level {
            OptimizationLevel::None => Ok(()),
            OptimizationLevel::Less => self.apply_basic_optimizations(module),
            OptimizationLevel::Default => self.apply_standard_optimizations(module),
            OptimizationLevel::Aggressive => self.apply_aggressive_optimizations(module),
            OptimizationLevel::Size => self.apply_size_optimizations(module),
        }
    }

    fn apply_basic_optimizations(&self, module: &Module<'ctx>) -> Result<(), JitError> {
        // Basic optimizations: constant folding, dead code elimination
        self.pass_manager.add_constant_merge_pass();
        self.pass_manager.add_dead_store_elimination_pass();
        self.pass_manager.run_on(module);
        Ok(())
    }

    fn apply_standard_optimizations(&self, module: &Module<'ctx>) -> Result<(), JitError> {
        // Standard optimizations
        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(InkwellOptLevel::Default);
        pass_manager_builder.populate_module_pass_manager(&self.pass_manager);
        self.pass_manager.run_on(module);
        Ok(())
    }

    fn apply_aggressive_optimizations(&self, module: &Module<'ctx>) -> Result<(), JitError> {
        // Aggressive optimizations
        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(InkwellOptLevel::Aggressive);
        pass_manager_builder.populate_module_pass_manager(&self.pass_manager);
        self.pass_manager.run_on(module);
        Ok(())
    }

    fn apply_size_optimizations(&self, module: &Module<'ctx>) -> Result<(), JitError> {
        // Size optimizations
        self.pass_manager.add_constant_merge_pass();
        self.pass_manager.add_dead_store_elimination_pass();
        self.pass_manager.add_strip_dead_prototypes_pass();
        self.pass_manager.run_on(module);
        Ok(())
    }
}

#[cfg(not(feature = "jit"))]
pub struct JitOptimizer;

#[cfg(not(feature = "jit"))]
impl JitOptimizer {
    pub fn new(_optimization_level: OptimizationLevel) -> Self {
        JitOptimizer
    }

    pub fn optimize_module(&self, _module: ()) -> Result<(), JitError> {
        Err(JitError::OptimizationFailed(
            "JIT optimization not available".to_string(),
        ))
    }
}
