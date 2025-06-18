// JIT Compiler Implementation
// Handles compilation of Matrix Language AST to LLVM IR

use super::{JitError, OptimizationLevel};
use crate::ast::nodes::*;
use crate::types::types::Type;
use std::collections::HashMap;

#[cfg(feature = "jit")]
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
    IntPredicate,
    FloatPredicate,
};

/// Compiler for Matrix Language to LLVM IR
#[cfg(feature = "jit")]
pub struct JitCompiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
}

#[cfg(feature = "jit")]
impl<'ctx> JitCompiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        JitCompiler {
            context,
            module,
            builder,
            variables: HashMap::new(),
            current_function: None,
        }
    }

    /// Compile a function definition to LLVM IR
    pub fn compile_function_def(&mut self, func_def: &FunctionDef) -> Result<FunctionValue<'ctx>, JitError> {
        // Map Matrix types to LLVM types
        let param_types: Result<Vec<_>, _> = func_def.params.iter()
            .map(|p| self.matrix_type_to_llvm(&p.param_type.clone().unwrap_or(Type::Any)))
            .collect();
        let param_types = param_types?;

        let return_type = self.matrix_type_to_llvm(&func_def.return_type.clone().unwrap_or(Type::Any))?;

        // Create function type
        let fn_type = match return_type {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(array_type) => array_type.fn_type(&param_types, false),
            BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(vec_type) => vec_type.fn_type(&param_types, false),
        };

        // Create function
        let function = self.module.add_function(&func_def.name, fn_type, None);
        self.current_function = Some(function);

        // Create entry block
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Create variable storage for parameters
        for (i, param) in func_def.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32)
                .ok_or_else(|| JitError::CompilationFailed("Invalid parameter index".to_string()))?;

            let alloca = self.create_entry_block_alloca(&param.name, param_value.get_type());
            self.builder.build_store(alloca, param_value)
                .map_err(|e| JitError::CompilationFailed(e.to_string()))?;

            self.variables.insert(param.name.clone(), alloca);
        }

        // Compile function body
        let body_value = self.compile_expression(&func_def.body)?;

        // Build return
        if let BasicMetadataValueEnum::IntValue(int_val) = body_value {
            self.builder.build_return(Some(&int_val))
                .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
        } else if let BasicMetadataValueEnum::FloatValue(float_val) = body_value {
            self.builder.build_return(Some(&float_val))
                .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
        } else {
            return Err(JitError::UnsupportedType("Unsupported return type".to_string()));
        }

        Ok(function)
    }

    /// Compile an expression to LLVM IR
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        match expr {
            Expression::IntLiteral(i, _) => Ok(self.context.i64_type().const_int(*i as u64, false).into()),
            Expression::FloatLiteral(f, _) => Ok(self.context.f64_type().const_float(*f).into()),
            Expression::BoolLiteral(b, _) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Expression::StringLiteral(s, _) => {
                let string_val = self.context.const_string(s.as_bytes(), false);
                Ok(string_val.into())
            },
            Expression::BinaryOp { left, operator, right, .. } => self.compile_binary_op(left, operator, right),
            Expression::UnaryOp { operator, operand, .. } => self.compile_unary_op(operator, operand),
            Expression::Identifier(name, _) => self.compile_variable(name),
            Expression::FunctionCall { function, args, .. } => self.compile_function_call(function, args),
            Expression::IfExpression { condition, then_branch, else_branch, .. } => {
                self.compile_if_expression(condition, then_branch, else_branch.as_deref())
            },
            Expression::ArrayLiteral(elements, _) => self.compile_array(elements),
            Expression::Block { statements, .. } => self.compile_block(statements),
            _ => Err(JitError::UnsupportedType("Expression not implemented yet".to_string())),
        }
    }

    fn compile_literal(&self, lit: &Literal) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        match lit {
            Literal::Int(i) => Ok(self.context.i64_type().const_int(*i as u64, false).into()),
            Literal::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            Literal::Bool(b) => Ok(self.context.bool_type().const_int(*b as u64, false).into()),
            Literal::String(s) => {
                let string_val = self.context.const_string(s.as_bytes(), false);
                Ok(string_val.into())
            },
            _ => Err(JitError::UnsupportedType("Literal type not supported".to_string())),
        }
    }

    fn compile_binary_op(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;

        match op {
            BinaryOperator::Add => self.compile_arithmetic_op(left_val, right_val, "add", |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_add(l, r, "add").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_add(l, r, "fadd").map(|v| v.into())
                } else {
                    Err("Type mismatch in addition".to_string())
                }
            }),
            BinaryOperator::Sub => self.compile_arithmetic_op(left_val, right_val, "sub", |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_sub(l, r, "sub").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_sub(l, r, "fsub").map(|v| v.into())
                } else {
                    Err("Type mismatch in subtraction".to_string())
                }
            }),
            BinaryOperator::Mul => self.compile_arithmetic_op(left_val, right_val, "mul", |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_mul(l, r, "mul").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_mul(l, r, "fmul").map(|v| v.into())
                } else {
                    Err("Type mismatch in multiplication".to_string())
                }
            }),
            BinaryOperator::Div => self.compile_arithmetic_op(left_val, right_val, "div", |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_signed_div(l, r, "sdiv").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_div(l, r, "fdiv").map(|v| v.into())
                } else {
                    Err("Type mismatch in division".to_string())
                }
            }),
            BinaryOperator::Eq => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::EQ, l, r, "eq").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::OEQ, l, r, "feq").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            BinaryOperator::Ne => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::NE, l, r, "ne").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::ONE, l, r, "fne").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            BinaryOperator::Lt => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::SLT, l, r, "lt").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::OLT, l, r, "flt").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            BinaryOperator::Le => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::SLE, l, r, "le").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::OLE, l, r, "fle").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            BinaryOperator::Gt => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::SGT, l, r, "gt").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::OGT, l, r, "fgt").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            BinaryOperator::Ge => self.compile_comparison_op(left_val, right_val, |builder, l, r| {
                if let (BasicMetadataValueEnum::IntValue(l), BasicMetadataValueEnum::IntValue(r)) = (l, r) {
                    builder.build_int_compare(IntPredicate::SGE, l, r, "ge").map(|v| v.into())
                } else if let (BasicMetadataValueEnum::FloatValue(l), BasicMetadataValueEnum::FloatValue(r)) = (l, r) {
                    builder.build_float_compare(FloatPredicate::OGE, l, r, "fge").map(|v| v.into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }),
            _ => Err(JitError::UnsupportedType("Binary operator not implemented".to_string())),
        }
    }

    fn compile_unary_op(&mut self, op: &UnaryOperator, expr: &Expression) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        let val = self.compile_expression(expr)?;

        match op {
            UnaryOperator::Neg => {
                if let BasicMetadataValueEnum::IntValue(int_val) = val {
                    let zero = self.context.i64_type().const_int(0, false);
                    Ok(self.builder.build_int_sub(zero, int_val, "neg")
                        .map_err(|e| JitError::CompilationFailed(e.to_string()))?
                        .into())
                } else if let BasicMetadataValueEnum::FloatValue(float_val) = val {
                    Ok(self.builder.build_float_neg(float_val, "fneg")
                        .map_err(|e| JitError::CompilationFailed(e.to_string()))?
                        .into())
                } else {
                    Err(JitError::UnsupportedType("Invalid type for negation".to_string()))
                }
            },
            UnaryOperator::Not => {
                if let BasicMetadataValueEnum::IntValue(int_val) = val {
                    // Assuming boolean is represented as i1 or i64
                    let zero = self.context.i64_type().const_int(0, false);
                    Ok(self.builder.build_int_compare(IntPredicate::EQ, int_val, zero, "not")
                        .map_err(|e| JitError::CompilationFailed(e.to_string()))?
                        .into())
                } else {
                    Err(JitError::UnsupportedType("Invalid type for logical not".to_string()))
                }
            },
            UnaryOperator::Transpose => {
                // Matrix transpose would require more complex implementation
                Err(JitError::UnsupportedType("Matrix transpose not implemented in JIT yet".to_string()))
            }
        }
    }

    fn compile_variable(&self, name: &str) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        if let Some(ptr) = self.variables.get(name) {
            let loaded_val = self.builder.build_load(ptr.get_type().get_element_type(), *ptr, name)
                .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            Ok(loaded_val.into())
        } else {
            Err(JitError::CompilationFailed(format!("Variable '{}' not found", name)))
        }
    }

    fn compile_function_call(&mut self, func: &Expression, args: &[Expression]) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        if let Expression::Identifier(func_name, _) = func {
            let compiled_args: Result<Vec<_>, _> = args.iter()
                .map(|arg| self.compile_expression(arg))
                .collect();
            let compiled_args = compiled_args?;

            if let Some(function) = self.module.get_function(func_name) {
                let call_site = self.builder.build_call(function, &compiled_args, "call")
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;

                if let Some(return_val) = call_site.try_as_basic_value() {
                    Ok(return_val.into())
                } else {
                    // Function returns void, return unit value (0)
                    Ok(self.context.i64_type().const_int(0, false).into())
                }
            } else {
                Err(JitError::FunctionNotFound(func_name.clone()))
            }
        } else {
            Err(JitError::UnsupportedType("Function calls with complex expressions not supported yet".to_string()))
        }
    }

    fn compile_if_expression(&mut self, condition: &Expression, then_branch: &Expression, else_branch: Option<&Expression>) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        let cond_val = self.compile_expression(condition)?;

        let function = self.current_function
            .ok_or_else(|| JitError::CompilationFailed("No current function".to_string()))?;

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let cont_block = self.context.append_basic_block(function, "cont");

        // Convert condition to boolean
        let cond_bool = if let BasicMetadataValueEnum::IntValue(int_val) = cond_val {
            let zero = self.context.i64_type().const_int(0, false);
            self.builder.build_int_compare(IntPredicate::NE, int_val, zero, "tobool")
                .map_err(|e| JitError::CompilationFailed(e.to_string()))?
        } else {
            return Err(JitError::UnsupportedType("Condition must be integer type".to_string()));
        };

        // Conditional branch
        self.builder.build_conditional_branch(cond_bool, then_block, else_block)
            .map_err(|e| JitError::CompilationFailed(e.to_string()))?;

        // Then block
        self.builder.position_at_end(then_block);
        let then_val = self.compile_expression(then_branch)?;
        self.builder.build_unconditional_branch(cont_block)
            .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
        let then_end_block = self.builder.get_insert_block().unwrap();

        // Else block
        self.builder.position_at_end(else_block);
        let else_val = if let Some(else_expr) = else_branch {
            self.compile_expression(else_expr)?
        } else {
            self.context.i64_type().const_int(0, false).into() // Default to 0
        };
        self.builder.build_unconditional_branch(cont_block)
            .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
        let else_end_block = self.builder.get_insert_block().unwrap();

        // Continuation block with PHI node
        self.builder.position_at_end(cont_block);

        match (then_val, else_val) {
            (BasicMetadataValueEnum::IntValue(then_int), BasicMetadataValueEnum::IntValue(else_int)) => {
                let phi = self.builder.build_phi(self.context.i64_type(), "iftmp")
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
                phi.add_incoming(&[(&then_int, then_end_block), (&else_int, else_end_block)]);
                Ok(phi.as_basic_value().into())
            },
            (BasicMetadataValueEnum::FloatValue(then_float), BasicMetadataValueEnum::FloatValue(else_float)) => {
                let phi = self.builder.build_phi(self.context.f64_type(), "iftmp")
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
                phi.add_incoming(&[(&then_float, then_end_block), (&else_float, else_end_block)]);
                Ok(phi.as_basic_value().into())
            },
            _ => Err(JitError::UnsupportedType("Type mismatch in if expression branches".to_string())),
        }
    }

    fn compile_array(&mut self, elements: &[Expression]) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        if elements.is_empty() {
            return Err(JitError::UnsupportedType("Empty arrays not supported".to_string()));
        }

        // For now, return a simple placeholder
        // Full array implementation would require more complex type management
        Ok(self.context.i64_type().const_int(elements.len() as u64, false).into())
    }

    fn compile_block(&mut self, statements: &[Statement]) -> Result<BasicMetadataValueEnum<'ctx>, JitError> {
        let mut last_val = self.context.i64_type().const_int(0, false).into();

        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    last_val = self.compile_expression(expr)?;
                },
                Statement::LetBinding(let_binding) => {
                    self.compile_let_binding(let_binding)?;
                    // Let bindings return unit value
                    last_val = self.context.i64_type().const_int(0, false).into();
                }
            }
        }

        Ok(last_val)
    }

    fn compile_let_binding(&mut self, let_binding: &LetBinding) -> Result<(), JitError> {
        let value = self.compile_expression(&let_binding.value)?;

        // Determine the type from the value
        let value_type = match value {
            BasicMetadataValueEnum::IntValue(int_val) => int_val.get_type().into(),
            BasicMetadataValueEnum::FloatValue(float_val) => float_val.get_type().into(),
            BasicMetadataValueEnum::PointerValue(ptr_val) => ptr_val.get_type().into(),
            BasicMetadataValueEnum::ArrayValue(array_val) => array_val.get_type().into(),
            BasicMetadataValueEnum::StructValue(struct_val) => struct_val.get_type().into(),
            BasicMetadataValueEnum::VectorValue(vec_val) => vec_val.get_type().into(),
        };

        // Create alloca for the variable
        let alloca = self.create_entry_block_alloca(&let_binding.name, value_type);

        // Store the value
        match value {
            BasicMetadataValueEnum::IntValue(int_val) => {
                self.builder.build_store(alloca, int_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
            BasicMetadataValueEnum::FloatValue(float_val) => {
                self.builder.build_store(alloca, float_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
            BasicMetadataValueEnum::PointerValue(ptr_val) => {
                self.builder.build_store(alloca, ptr_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
            BasicMetadataValueEnum::ArrayValue(array_val) => {
                self.builder.build_store(alloca, array_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
            BasicMetadataValueEnum::StructValue(struct_val) => {
                self.builder.build_store(alloca, struct_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
            BasicMetadataValueEnum::VectorValue(vec_val) => {
                self.builder.build_store(alloca, vec_val)
                    .map_err(|e| JitError::CompilationFailed(e.to_string()))?;
            },
        }

        // Store the variable for later use
        self.variables.insert(let_binding.name.clone(), alloca);

        Ok(())
    }

    // Helper methods
    fn compile_arithmetic_op<F>(&self, left: BasicMetadataValueEnum<'ctx>, right: BasicMetadataValueEnum<'ctx>, name: &str, op: F) -> Result<BasicMetadataValueEnum<'ctx>, JitError>
    where
        F: Fn(&Builder<'ctx>, BasicMetadataValueEnum<'ctx>, BasicMetadataValueEnum<'ctx>) -> Result<BasicMetadataValueEnum<'ctx>, String>,
    {
        op(&self.builder, left, right)
            .map_err(|e| JitError::CompilationFailed(e))
    }

    fn compile_comparison_op<F>(&self, left: BasicMetadataValueEnum<'ctx>, right: BasicMetadataValueEnum<'ctx>, op: F) -> Result<BasicMetadataValueEnum<'ctx>, JitError>
    where
        F: Fn(&Builder<'ctx>, BasicMetadataValueEnum<'ctx>, BasicMetadataValueEnum<'ctx>) -> Result<BasicMetadataValueEnum<'ctx>, String>,
    {
        op(&self.builder, left, right)
            .map_err(|e| JitError::CompilationFailed(e))
    }

    fn matrix_type_to_llvm(&self, matrix_type: &Type) -> Result<BasicMetadataTypeEnum<'ctx>, JitError> {
        match matrix_type {
            Type::Int => Ok(self.context.i64_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::Bool => Ok(self.context.bool_type().into()),
            Type::String => Ok(self.context.i8_type().ptr_type(AddressSpace::default()).into()),
            Type::Array(element_type) => {
                let element_llvm_type = self.matrix_type_to_llvm(element_type)?;
                if let BasicMetadataTypeEnum::IntType(int_type) = element_llvm_type {
                    Ok(int_type.ptr_type(AddressSpace::default()).into())
                } else if let BasicMetadataTypeEnum::FloatType(float_type) = element_llvm_type {
                    Ok(float_type.ptr_type(AddressSpace::default()).into())
                } else {
                    Err(JitError::UnsupportedType("Unsupported array element type".to_string()))
                }
            },
            Type::Any => Ok(self.context.i64_type().into()), // Default to i64 for Any type
            _ => Err(JitError::UnsupportedType(format!("Type {:?} not supported in JIT", matrix_type))),
        }
    }

    fn create_entry_block_alloca(&self, name: &str, value_type: BasicTypeEnum<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.current_function.unwrap().get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(value_type, name).unwrap()
    }
}

#[cfg(not(feature = "jit"))]
pub struct JitCompiler;

#[cfg(not(feature = "jit"))]
impl JitCompiler {
    pub fn new(_context: (), _module_name: &str) -> Self {
        JitCompiler
    }

    pub fn compile_function_def(&mut self, _func_def: &FunctionDef) -> Result<(), JitError> {
        Err(JitError::NotAvailable)
    }

    pub fn compile_expression(&mut self, _expr: &Expression) -> Result<(), JitError> {
        Err(JitError::NotAvailable)
    }
}
