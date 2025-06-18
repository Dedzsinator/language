use crate::ast::nodes::*;
use std::collections::HashMap;

/// Intermediate Representation for Matrix Language
#[derive(Debug, Clone)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub types: Vec<IrType>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: IrType,
    pub basic_blocks: Vec<BasicBlock>,
    pub is_external: bool,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub name: String,
    pub param_type: IrType,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub global_type: IrType,
    pub initial_value: Option<IrConstant>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<IrInstruction>,
    pub terminator: IrTerminator,
}

/// IR Instructions
#[derive(Debug, Clone)]
pub enum IrInstruction {
    // Arithmetic operations
    Add {
        result: IrValue,
        left: IrValue,
        right: IrValue,
    },
    Sub {
        result: IrValue,
        left: IrValue,
        right: IrValue,
    },
    Mul {
        result: IrValue,
        left: IrValue,
        right: IrValue,
    },
    Div {
        result: IrValue,
        left: IrValue,
        right: IrValue,
    },
    Pow {
        result: IrValue,
        base: IrValue,
        exp: IrValue,
    },

    // Memory operations
    Load {
        result: IrValue,
        address: IrValue,
    },
    Store {
        value: IrValue,
        address: IrValue,
    },
    Alloca {
        result: IrValue,
        alloc_type: IrType,
    },

    // Vector/Matrix operations
    VectorCreate {
        result: IrValue,
        elements: Vec<IrValue>,
    },
    MatrixCreate {
        result: IrValue,
        rows: u32,
        cols: u32,
        elements: Vec<IrValue>,
    },
    VectorIndex {
        result: IrValue,
        vector: IrValue,
        index: IrValue,
    },
    MatrixIndex {
        result: IrValue,
        matrix: IrValue,
        row: IrValue,
        col: IrValue,
    },

    // Physics operations
    PhysicsStep {
        timestep: IrValue,
    },
    ApplyForce {
        object: IrValue,
        force: IrValue,
    },
    GetPosition {
        result: IrValue,
        object: IrValue,
    },
    SetPosition {
        object: IrValue,
        position: IrValue,
    },

    // Function calls
    Call {
        result: Option<IrValue>,
        function: String,
        args: Vec<IrValue>,
    },

    // Type conversions
    Cast {
        result: IrValue,
        value: IrValue,
        target_type: IrType,
    },

    // Comparisons
    ICmp {
        result: IrValue,
        predicate: IrPredicate,
        left: IrValue,
        right: IrValue,
    },
    FCmp {
        result: IrValue,
        predicate: IrPredicate,
        left: IrValue,
        right: IrValue,
    },
}

/// IR Terminators (end basic blocks)
#[derive(Debug, Clone)]
pub enum IrTerminator {
    Return(Option<IrValue>),
    Branch {
        condition: IrValue,
        true_block: String,
        false_block: String,
    },
    Jump(String),
    Unreachable,
}

/// IR Values
#[derive(Debug, Clone)]
pub enum IrValue {
    Constant(IrConstant),
    Register(String),
    Global(String),
}

/// IR Constants
#[derive(Debug, Clone)]
pub enum IrConstant {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Vector(Vec<IrConstant>),
    Matrix {
        rows: u32,
        cols: u32,
        data: Vec<IrConstant>,
    },
}

/// IR Types
#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    Void,
    Boolean,
    Integer(u32), // bit width
    Float(u32),   // bit width
    String,
    Vector(Box<IrType>, u32),      // element type, length
    Matrix(Box<IrType>, u32, u32), // element type, rows, cols
    Pointer(Box<IrType>),
    Function {
        params: Vec<IrType>,
        return_type: Box<IrType>,
    },
}

/// Comparison predicates
#[derive(Debug, Clone)]
pub enum IrPredicate {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// IR Code generator
pub struct IrGenerator {
    current_function: Option<String>,
    current_block: Option<String>,
    register_counter: u32,
    block_counter: u32,
    symbol_table: HashMap<String, IrValue>,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
            register_counter: 0,
            block_counter: 0,
            symbol_table: HashMap::new(),
        }
    }

    /// Generate IR from AST
    pub fn generate(&mut self, program: &Program) -> IrModule {
        let mut module = IrModule {
            functions: Vec::new(),
            globals: Vec::new(),
            types: Vec::new(),
        };

        // Add standard library functions
        self.add_stdlib_functions(&mut module);

        // Generate main function
        let main_function = self.generate_main_function(&program.items);
        module.functions.push(main_function);

        module
    }

    fn generate_main_function(&mut self, items: &[Item]) -> IrFunction {
        self.current_function = Some("main".to_string());
        self.current_block = Some("entry".to_string());

        let mut basic_blocks = Vec::new();
        let mut instructions = Vec::new();

        for item in items {
            match item {
                Item::LetBinding(binding) => {
                    // Create a statement and generate IR for it
                    let stmt = Statement::LetBinding(binding.clone());
                    let mut stmt_instructions = self.generate_statement(&stmt);
                    instructions.append(&mut stmt_instructions);
                }
                // Handle other item types here as needed
                _ => {
                    // For other items, we might want to generate different IR
                    // For now, we'll skip them
                }
            }
        }

        // Create the entry block using the new method
        let entry_block = BasicBlock {
            label: "entry".to_string(),
            instructions,
            terminator: IrTerminator::Return(None),
        };
        basic_blocks.push(entry_block);

        // Example: Create additional blocks for more complex control flow
        if items.len() > 1 {
            let additional_block = self.create_new_block("additional");
            basic_blocks.push(additional_block);
        }

        IrFunction {
            name: "main".to_string(),
            params: Vec::new(),
            return_type: IrType::Void,
            basic_blocks,
            is_external: false,
        }
    }

    fn generate_let_binding(&mut self, binding: &LetBinding) -> Vec<IrInstruction> {
        let mut instructions = Vec::new();

        // Allocate space for variable
        let alloc_reg = self.next_register();
        instructions.push(IrInstruction::Alloca {
            result: IrValue::Register(alloc_reg.clone()),
            alloc_type: IrType::Float(64),
        });

        // Generate expression and store result
        let expr_value = self.generate_expression(&binding.value, &mut instructions);
        instructions.push(IrInstruction::Store {
            value: expr_value,
            address: IrValue::Register(alloc_reg.clone()),
        });

        self.symbol_table
            .insert(binding.name.clone(), IrValue::Register(alloc_reg));
        instructions
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Vec<IrInstruction> {
        let mut instructions = Vec::new();

        match stmt {
            Statement::Expression(expr) => {
                self.generate_expression(expr, &mut instructions);
            }
            Statement::LetBinding(binding) => {
                let mut binding_instructions = self.generate_let_binding(binding);
                instructions.append(&mut binding_instructions);
            }
        }

        instructions
    }

    fn generate_expression(
        &mut self,
        expr: &Expression,
        instructions: &mut Vec<IrInstruction>,
    ) -> IrValue {
        match expr {
            Expression::IntLiteral(n, _) => IrValue::Constant(IrConstant::Integer(*n)),

            Expression::FloatLiteral(n, _) => IrValue::Constant(IrConstant::Float(*n)),

            Expression::StringLiteral(s, _) => IrValue::Constant(IrConstant::String(s.clone())),

            Expression::BoolLiteral(b, _) => IrValue::Constant(IrConstant::Boolean(*b)),

            Expression::Identifier(name, _) => {
                if let Some(var_addr) = self.symbol_table.get(name).cloned() {
                    let result_reg = self.next_register();
                    instructions.push(IrInstruction::Load {
                        result: IrValue::Register(result_reg.clone()),
                        address: var_addr,
                    });
                    IrValue::Register(result_reg)
                } else {
                    // Undefined variable - should be caught in semantic analysis
                    IrValue::Constant(IrConstant::Null)
                }
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.generate_expression(left, instructions);
                let right_val = self.generate_expression(right, instructions);
                let result_reg = self.next_register();

                let instruction = match operator {
                    BinaryOperator::Add => IrInstruction::Add {
                        result: IrValue::Register(result_reg.clone()),
                        left: left_val,
                        right: right_val,
                    },
                    BinaryOperator::Sub => IrInstruction::Sub {
                        result: IrValue::Register(result_reg.clone()),
                        left: left_val,
                        right: right_val,
                    },
                    BinaryOperator::Mul => IrInstruction::Mul {
                        result: IrValue::Register(result_reg.clone()),
                        left: left_val,
                        right: right_val,
                    },
                    BinaryOperator::Div => IrInstruction::Div {
                        result: IrValue::Register(result_reg.clone()),
                        left: left_val,
                        right: right_val,
                    },
                    BinaryOperator::Pow => IrInstruction::Pow {
                        result: IrValue::Register(result_reg.clone()),
                        base: left_val,
                        exp: right_val,
                    },
                    BinaryOperator::Eq => IrInstruction::FCmp {
                        result: IrValue::Register(result_reg.clone()),
                        predicate: IrPredicate::Equal,
                        left: left_val,
                        right: right_val,
                    },
                    BinaryOperator::Lt => IrInstruction::FCmp {
                        result: IrValue::Register(result_reg.clone()),
                        predicate: IrPredicate::Less,
                        left: left_val,
                        right: right_val,
                    },
                    _ => {
                        // For other operators, create a placeholder
                        IrInstruction::Add {
                            result: IrValue::Register(result_reg.clone()),
                            left: left_val,
                            right: right_val,
                        }
                    }
                };

                instructions.push(instruction);
                IrValue::Register(result_reg)
            }

            Expression::FunctionCall { function, args, .. } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.generate_expression(arg, instructions));
                }

                // Extract function name from identifier
                let function_name = if let Expression::Identifier(name, _) = function.as_ref() {
                    name.clone()
                } else {
                    "unknown".to_string()
                };

                let result_reg = self.next_register();
                instructions.push(IrInstruction::Call {
                    result: Some(IrValue::Register(result_reg.clone())),
                    function: function_name,
                    args: arg_values,
                });
                IrValue::Register(result_reg)
            }

            Expression::ArrayLiteral(elements, _) => {
                let mut element_values = Vec::new();
                for elem in elements {
                    element_values.push(self.generate_expression(elem, instructions));
                }

                let result_reg = self.next_register();
                instructions.push(IrInstruction::VectorCreate {
                    result: IrValue::Register(result_reg.clone()),
                    elements: element_values,
                });
                IrValue::Register(result_reg)
            }

            Expression::MatrixLiteral(rows, _) => {
                let mut all_elements = Vec::new();
                let num_rows = rows.len() as u32;
                let num_cols = if !rows.is_empty() {
                    rows[0].len() as u32
                } else {
                    0
                };

                for row in rows {
                    for elem in row {
                        all_elements.push(self.generate_expression(elem, instructions));
                    }
                }

                let result_reg = self.next_register();
                instructions.push(IrInstruction::MatrixCreate {
                    result: IrValue::Register(result_reg.clone()),
                    rows: num_rows,
                    cols: num_cols,
                    elements: all_elements,
                });
                IrValue::Register(result_reg)
            }

            _ => {
                // Handle other expression types
                IrValue::Constant(IrConstant::Null)
            }
        }
    }

    fn next_register(&mut self) -> String {
        let reg = format!("%{}", self.register_counter);
        self.register_counter += 1;
        reg
    }

    fn next_block_id(&mut self) -> u32 {
        let id = self.block_counter;
        self.block_counter += 1;
        id
    }

    /// Create a new basic block with a unique ID
    fn create_new_block(&mut self, prefix: &str) -> BasicBlock {
        let block_id = self.next_block_id();
        BasicBlock {
            label: format!("{}_{}", prefix, block_id),
            instructions: Vec::new(),
            terminator: IrTerminator::Return(None),
        }
    }

    fn add_stdlib_functions(&mut self, module: &mut IrModule) {
        // Add common math functions
        let math_functions = vec![
            ("sin", IrType::Float(64), vec![IrType::Float(64)]),
            ("cos", IrType::Float(64), vec![IrType::Float(64)]),
            ("sqrt", IrType::Float(64), vec![IrType::Float(64)]),
            (
                "pow",
                IrType::Float(64),
                vec![IrType::Float(64), IrType::Float(64)],
            ),
            ("print", IrType::Void, vec![IrType::String]),
        ];

        for (name, return_type, param_types) in math_functions {
            let params = param_types
                .into_iter()
                .enumerate()
                .map(|(i, param_type)| IrParam {
                    name: format!("arg{}", i),
                    param_type,
                })
                .collect();

            module.functions.push(IrFunction {
                name: name.to_string(),
                params,
                return_type,
                basic_blocks: Vec::new(),
                is_external: true,
            });
        }
    }
}

/// IR Optimizer
pub struct IrOptimizer;

impl IrOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Optimize IR module
    pub fn optimize(&self, mut module: IrModule) -> IrModule {
        // Apply various optimization passes
        module = self.constant_folding(module);
        module = self.dead_code_elimination(module);
        module = self.common_subexpression_elimination(module);
        module
    }

    /// Constant folding optimization
    fn constant_folding(&self, mut module: IrModule) -> IrModule {
        for function in &mut module.functions {
            for block in &mut function.basic_blocks {
                let mut optimized_instructions = Vec::new();

                for instruction in &block.instructions {
                    match instruction {
                        IrInstruction::Add {
                            result,
                            left,
                            right,
                        } => {
                            if let (
                                IrValue::Constant(IrConstant::Float(a)),
                                IrValue::Constant(IrConstant::Float(b)),
                            ) = (left, right)
                            {
                                // Fold constant addition
                                optimized_instructions.push(IrInstruction::Load {
                                    result: result.clone(),
                                    address: IrValue::Constant(IrConstant::Float(a + b)),
                                });
                            } else {
                                optimized_instructions.push(instruction.clone());
                            }
                        }
                        _ => optimized_instructions.push(instruction.clone()),
                    }
                }

                block.instructions = optimized_instructions;
            }
        }
        module
    }

    /// Dead code elimination
    fn dead_code_elimination(&self, mut module: IrModule) -> IrModule {
        // Remove unused instructions and basic blocks
        // This is a simplified implementation
        for function in &mut module.functions {
            function
                .basic_blocks
                .retain(|block| !block.instructions.is_empty());
        }
        module
    }

    /// Common subexpression elimination
    fn common_subexpression_elimination(&self, module: IrModule) -> IrModule {
        // Identify and eliminate redundant computations
        // This would require more sophisticated analysis
        module
    }
}

/// IR Printer for debugging
pub struct IrPrinter;

impl IrPrinter {
    pub fn print_module(&self, module: &IrModule) -> String {
        let mut output = String::new();

        output.push_str("; Matrix Language IR Module\n\n");

        // Print globals
        for global in &module.globals {
            output.push_str(&format!(
                "@{} = global {} {}\n",
                global.name,
                self.type_to_string(&global.global_type),
                if let Some(init) = &global.initial_value {
                    self.constant_to_string(init)
                } else {
                    "undef".to_string()
                }
            ));
        }

        if !module.globals.is_empty() {
            output.push('\n');
        }

        // Print functions
        for function in &module.functions {
            output.push_str(&self.print_function(function));
            output.push('\n');
        }

        output
    }

    fn print_function(&self, function: &IrFunction) -> String {
        let mut output = String::new();

        if function.is_external {
            output.push_str("declare ");
        } else {
            output.push_str("define ");
        }

        output.push_str(&self.type_to_string(&function.return_type));
        output.push_str(&format!(" @{}(", function.name));

        for (i, param) in function.params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!(
                "{} %{}",
                self.type_to_string(&param.param_type),
                param.name
            ));
        }

        output.push(')');

        if function.is_external {
            output.push('\n');
            return output;
        }

        output.push_str(" {\n");

        for block in &function.basic_blocks {
            output.push_str(&format!("{}:\n", block.label));

            for instruction in &block.instructions {
                output.push_str(&format!("  {}\n", self.instruction_to_string(instruction)));
            }

            output.push_str(&format!(
                "  {}\n",
                self.terminator_to_string(&block.terminator)
            ));
        }

        output.push_str("}\n");
        output
    }

    fn instruction_to_string(&self, instruction: &IrInstruction) -> String {
        match instruction {
            IrInstruction::Add {
                result,
                left,
                right,
            } => {
                format!(
                    "{} = fadd {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
            IrInstruction::Load { result, address } => {
                format!(
                    "{} = load {}",
                    self.value_to_string(result),
                    self.value_to_string(address)
                )
            }
            IrInstruction::Store { value, address } => {
                format!(
                    "store {}, {}",
                    self.value_to_string(value),
                    self.value_to_string(address)
                )
            }
            IrInstruction::Call {
                result,
                function,
                args,
            } => {
                let result_str = if let Some(res) = result {
                    format!("{} = ", self.value_to_string(res))
                } else {
                    String::new()
                };
                let args_str = args
                    .iter()
                    .map(|arg| self.value_to_string(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}call @{}({})", result_str, function, args_str)
            }
            IrInstruction::Sub {
                result,
                left,
                right,
            } => {
                format!(
                    "{} = fsub {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
            IrInstruction::Mul {
                result,
                left,
                right,
            } => {
                format!(
                    "{} = fmul {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
            IrInstruction::Div {
                result,
                left,
                right,
            } => {
                format!(
                    "{} = fdiv {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
            IrInstruction::Pow { result, base, exp } => {
                format!(
                    "{} = fpow {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(base),
                    self.value_to_string(exp)
                )
            }
            IrInstruction::Alloca { result, alloc_type } => {
                format!(
                    "{} = alloca {}",
                    self.value_to_string(result),
                    self.type_to_string(alloc_type)
                )
            }
            IrInstruction::VectorCreate { result, elements } => {
                let elements_str = elements
                    .iter()
                    .map(|elem| self.value_to_string(elem))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{} = vector_create [{}]",
                    self.value_to_string(result),
                    elements_str
                )
            }
            IrInstruction::MatrixCreate {
                result,
                rows,
                cols,
                elements,
            } => {
                let elements_str = elements
                    .iter()
                    .map(|elem| self.value_to_string(elem))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{} = matrix_create {}x{} [{}]",
                    self.value_to_string(result),
                    rows,
                    cols,
                    elements_str
                )
            }
            IrInstruction::VectorIndex {
                result,
                vector,
                index,
            } => {
                format!(
                    "{} = vector_index {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(vector),
                    self.value_to_string(index)
                )
            }
            IrInstruction::MatrixIndex {
                result,
                matrix,
                row,
                col,
            } => {
                format!(
                    "{} = matrix_index {}, {}, {}",
                    self.value_to_string(result),
                    self.value_to_string(matrix),
                    self.value_to_string(row),
                    self.value_to_string(col)
                )
            }
            IrInstruction::PhysicsStep { timestep } => {
                format!("physics_step {}", self.value_to_string(timestep))
            }
            IrInstruction::ApplyForce { object, force } => {
                format!(
                    "apply_force {}, {}",
                    self.value_to_string(object),
                    self.value_to_string(force)
                )
            }
            IrInstruction::GetPosition { result, object } => {
                format!(
                    "{} = get_position {}",
                    self.value_to_string(result),
                    self.value_to_string(object)
                )
            }
            IrInstruction::SetPosition { object, position } => {
                format!(
                    "set_position {}, {}",
                    self.value_to_string(object),
                    self.value_to_string(position)
                )
            }
            IrInstruction::Cast {
                result,
                value,
                target_type,
            } => {
                format!(
                    "{} = cast {} to {}",
                    self.value_to_string(result),
                    self.value_to_string(value),
                    self.type_to_string(target_type)
                )
            }
            IrInstruction::ICmp {
                result,
                predicate,
                left,
                right,
            } => {
                let pred_str = match predicate {
                    IrPredicate::Equal => "eq",
                    IrPredicate::NotEqual => "ne",
                    IrPredicate::Less => "slt",
                    IrPredicate::LessEqual => "sle",
                    IrPredicate::Greater => "sgt",
                    IrPredicate::GreaterEqual => "sge",
                };
                format!(
                    "{} = icmp {} {}, {}",
                    self.value_to_string(result),
                    pred_str,
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
            IrInstruction::FCmp {
                result,
                predicate,
                left,
                right,
            } => {
                let pred_str = match predicate {
                    IrPredicate::Equal => "oeq",
                    IrPredicate::NotEqual => "one",
                    IrPredicate::Less => "olt",
                    IrPredicate::LessEqual => "ole",
                    IrPredicate::Greater => "ogt",
                    IrPredicate::GreaterEqual => "oge",
                };
                format!(
                    "{} = fcmp {} {}, {}",
                    self.value_to_string(result),
                    pred_str,
                    self.value_to_string(left),
                    self.value_to_string(right)
                )
            }
        }
    }

    fn terminator_to_string(&self, terminator: &IrTerminator) -> String {
        match terminator {
            IrTerminator::Return(Some(value)) => {
                format!("ret {}", self.value_to_string(value))
            }
            IrTerminator::Return(None) => "ret void".to_string(),
            IrTerminator::Branch {
                condition,
                true_block,
                false_block,
            } => {
                format!(
                    "br {}, label %{}, label %{}",
                    self.value_to_string(condition),
                    true_block,
                    false_block
                )
            }
            IrTerminator::Jump(target) => format!("br label %{}", target),
            IrTerminator::Unreachable => "unreachable".to_string(),
        }
    }

    fn value_to_string(&self, value: &IrValue) -> String {
        match value {
            IrValue::Constant(constant) => self.constant_to_string(constant),
            IrValue::Register(name) => name.clone(),
            IrValue::Global(name) => format!("@{}", name),
        }
    }

    fn constant_to_string(&self, constant: &IrConstant) -> String {
        match constant {
            IrConstant::Null => "null".to_string(),
            IrConstant::Boolean(b) => b.to_string(),
            IrConstant::Integer(i) => i.to_string(),
            IrConstant::Float(f) => f.to_string(),
            IrConstant::String(s) => format!("\"{}\"", s),
            IrConstant::Vector(elements) => {
                let elements_str = elements
                    .iter()
                    .map(|e| self.constant_to_string(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements_str)
            }
            IrConstant::Matrix { rows, cols, data } => {
                format!(
                    "matrix<{}x{}> [{}]",
                    rows,
                    cols,
                    data.iter()
                        .map(|e| self.constant_to_string(e))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }

    fn type_to_string(&self, ir_type: &IrType) -> String {
        match ir_type {
            IrType::Void => "void".to_string(),
            IrType::Boolean => "i1".to_string(),
            IrType::Integer(bits) => format!("i{}", bits),
            IrType::Float(bits) => format!("f{}", bits),
            IrType::String => "i8*".to_string(),
            IrType::Vector(elem_type, length) => {
                format!("{}[{}]", self.type_to_string(elem_type), length)
            }
            IrType::Matrix(elem_type, rows, cols) => {
                format!("{}[{}][{}]", self.type_to_string(elem_type), rows, cols)
            }
            IrType::Pointer(pointee) => {
                format!("{}*", self.type_to_string(pointee))
            }
            IrType::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| self.type_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, self.type_to_string(return_type))
            }
        }
    }
}
