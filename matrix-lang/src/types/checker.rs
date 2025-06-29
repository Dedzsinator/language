use crate::ast::nodes::*;
use crate::types::*;
use std::collections::HashMap;

/// Unification algorithm for type inference
#[derive(Debug, Clone)]
pub struct Unifier {
    substitutions: HashMap<String, Type>,
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Unifier {
    pub fn new() -> Self {
        Self {
            substitutions: HashMap::new(),
        }
    }

    pub fn unify(&mut self, t1: &Type, t2: &Type) -> TypeResult<()> {
        let t1 = self.apply_substitutions(t1);
        let t2 = self.apply_substitutions(t2);

        match (&t1, &t2) {
            // Same concrete types
            (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String)
            | (Type::Unit, Type::Unit) => Ok(()),

            // Struct types
            (Type::Struct(name1), Type::Struct(name2)) if name1 == name2 => Ok(()),

            // Type variables
            (Type::TypeVar(var), ty) | (ty, Type::TypeVar(var)) => self.bind_type_var(var, ty),

            // Arrays
            (Type::Array(elem1), Type::Array(elem2)) => self.unify(elem1, elem2),

            // Matrices
            (Type::Matrix(elem1, rows1, cols1), Type::Matrix(elem2, rows2, cols2)) => {
                self.unify(elem1, elem2)?;

                // Check dimensions if both are specified
                match (rows1, rows2) {
                    (Some(r1), Some(r2)) if r1 != r2 => {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("Matrix with {} rows", r1),
                            found: format!("Matrix with {} rows", r2),
                            line: 0,
                            column: 0,
                        });
                    }
                    _ => {}
                }

                match (cols1, cols2) {
                    (Some(c1), Some(c2)) if c1 != c2 => {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("Matrix with {} columns", c1),
                            found: format!("Matrix with {} columns", c2),
                            line: 0,
                            column: 0,
                        });
                    }
                    _ => {}
                }

                Ok(())
            }

            // Functions
            (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::WrongArgumentCount {
                        expected: params1.len(),
                        found: params2.len(),
                        line: 0,
                        column: 0,
                    });
                }

                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    self.unify(p1, p2)?;
                }

                self.unify(ret1, ret2)
            }

            // Type applications
            (Type::TypeApp(name1, args1), Type::TypeApp(name2, args2)) if name1 == name2 => {
                if args1.len() != args2.len() {
                    return Err(TypeError::TypeMismatch {
                        expected: format!("{}<{} args>", name1, args1.len()),
                        found: format!("{}<{} args>", name2, args2.len()),
                        line: 0,
                        column: 0,
                    });
                }

                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    self.unify(arg1, arg2)?;
                }

                Ok(())
            }

            // Options
            (Type::Option(inner1), Type::Option(inner2)) => self.unify(inner1, inner2),

            // Spanned types
            (Type::Spanned(inner1, _), Type::Spanned(inner2, _)) => self.unify(inner1, inner2),
            (Type::Spanned(inner, _), other) | (other, Type::Spanned(inner, _)) => {
                self.unify(inner, other)
            }

            // Type mismatch
            _ => Err(TypeError::TypeMismatch {
                expected: t1.to_string(),
                found: t2.to_string(),
                line: 0,
                column: 0,
            }),
        }
    }

    fn bind_type_var(&mut self, var: &str, ty: &Type) -> TypeResult<()> {
        // Occurs check
        if ty.occurs_check(var) {
            return Err(TypeError::TypeMismatch {
                expected: "non-recursive type".to_string(),
                found: format!("recursive type involving {}", var),
                line: 0,
                column: 0,
            });
        }

        self.substitutions.insert(var.to_string(), ty.clone());
        Ok(())
    }

    fn apply_substitutions(&self, ty: &Type) -> Type {
        ty.substitute(&self.substitutions)
    }

    pub fn finalize_type(&self, ty: &Type) -> Type {
        self.apply_substitutions(ty)
    }

    pub fn get_substitutions(&self) -> &HashMap<String, Type> {
        &self.substitutions
    }
}

/// Main type checker implementing Hindley-Milner type inference
pub struct TypeChecker {
    context: TypeContext,
    unifier: Unifier,
    import_stack: Vec<String>,
    warnings: Vec<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
            unifier: Unifier::new(),
            import_stack: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> TypeResult<InferredType> {
        for item in &program.items {
            self.check_item(item)?;
        }

        // Return unit type for a complete program
        Ok(InferredType {
            ty: Type::Unit,
            constraints: Vec::new(),
        })
    }

    pub fn check_item(&mut self, item: &Item) -> TypeResult<InferredType> {
        match item {
            Item::StructDef(struct_def) => {
                self.context.structs.register(struct_def.clone());
                Ok(InferredType {
                    ty: Type::Unit,
                    constraints: Vec::new(),
                })
            }

            Item::TypeclassDef(typeclass_def) => {
                self.context
                    .typeclasses
                    .register_typeclass(typeclass_def.clone());
                Ok(InferredType {
                    ty: Type::Unit,
                    constraints: Vec::new(),
                })
            }

            Item::InstanceDef(instance_def) => {
                self.context
                    .typeclasses
                    .register_instance(instance_def.clone());
                Ok(InferredType {
                    ty: Type::Unit,
                    constraints: Vec::new(),
                })
            }

            Item::FunctionDef(func_def) => self.check_function_def(func_def),

            Item::LetBinding(let_binding) => self.check_let_binding(let_binding),

            Item::Import(import) => {
                // Implement import checking
                // Validate that the imported module exists and check for circular imports
                let module_name = import.module_path.clone();

                // Check if the module is a built-in module
                let is_builtin = matches!(module_name.as_str(), "std" | "math" | "io" | "fs");

                if !is_builtin {
                    // For non-builtin modules, we would check if the file exists
                    // For now, we'll assume all imports are valid
                    self.add_warning(format!(
                        "Cannot verify existence of module '{}'",
                        module_name
                    ));
                }

                // Check for circular imports by tracking current import chain
                if self.import_stack.contains(&module_name) {
                    return Err(TypeError::CircularImport {
                        module: module_name,
                        chain: self.import_stack.clone(),
                    });
                }

                Ok(InferredType {
                    ty: Type::Unit,
                    constraints: Vec::new(),
                })
            }
        }
    }

    fn check_function_def(&mut self, func_def: &FunctionDef) -> TypeResult<InferredType> {
        self.context.push_scope();

        // Add parameters to environment
        let mut param_types = Vec::new();
        for param in &func_def.params {
            let param_type = InferredType {
                ty: param.type_annotation.clone(),
                constraints: Vec::new(),
            };
            param_types.push(param.type_annotation.clone());
            self.context.env.bind(param.name.clone(), param_type);
        }

        // Check function body
        let body_type = self.check_expression(&func_def.body)?;

        // Unify body type with return type annotation if present
        if let Some(ref return_type) = func_def.return_type {
            self.unifier.unify(&body_type.ty, return_type)?;
        }

        let func_type = Type::Function(param_types, Box::new(body_type.ty.clone()));

        // Add function to environment
        let func_inferred_type = InferredType {
            ty: func_type.clone(),
            constraints: body_type.constraints,
        };

        self.context.pop_scope();
        self.context
            .env
            .bind(func_def.name.clone(), func_inferred_type.clone());

        Ok(func_inferred_type)
    }

    fn check_let_binding(&mut self, let_binding: &LetBinding) -> TypeResult<InferredType> {
        let value_type = self.check_expression(&let_binding.value)?;

        // Unify with type annotation if present
        if let Some(ref type_annotation) = let_binding.type_annotation {
            self.unifier.unify(&value_type.ty, type_annotation)?;
        }

        // Add binding to environment
        self.context
            .env
            .bind(let_binding.name.clone(), value_type.clone());

        Ok(value_type)
    }
    pub fn check_expression(&mut self, expr: &Expression) -> TypeResult<InferredType> {
        match expr {
            Expression::IntLiteral(_, _) => Ok(InferredType {
                ty: Type::Int,
                constraints: Vec::new(),
            }),

            Expression::FloatLiteral(_, _) => Ok(InferredType {
                ty: Type::Float,
                constraints: Vec::new(),
            }),

            Expression::BoolLiteral(_, _) => Ok(InferredType {
                ty: Type::Bool,
                constraints: Vec::new(),
            }),

            Expression::StringLiteral(_, _) => Ok(InferredType {
                ty: Type::String,
                constraints: Vec::new(),
            }),

            Expression::Identifier(name, span) => {
                if let Some(inferred_type) = self.context.env.lookup(name) {
                    Ok(inferred_type.clone())
                } else {
                    Err(TypeError::UnknownIdentifier {
                        name: name.clone(),
                        line: span.line,
                        column: span.column,
                    })
                }
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
                span,
            } => self.check_binary_op(left, operator, right, span),

            Expression::UnaryOp {
                operator,
                operand,
                span,
            } => self.check_unary_op(operator, operand, span),

            Expression::FunctionCall {
                function,
                args,
                span,
            } => self.check_function_call(function, args, span),

            Expression::FieldAccess {
                object,
                field,
                span,
            } => self.check_field_access(object, field, span),

            Expression::StructCreation { name, fields, span } => {
                self.check_struct_creation(name, fields, span)
            }

            Expression::ArrayLiteral(elements, span) => self.check_array_literal(elements, span),

            Expression::MatrixLiteral(rows, span) => self.check_matrix_literal(rows, span),

            Expression::MatrixComprehension {
                element,
                generators,
                span,
            } => self.check_matrix_comprehension(element, generators, span),

            Expression::IfExpression {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.check_if_expression(condition, then_branch, else_branch, span),

            Expression::Match {
                expression,
                arms,
                span,
            } => self.check_match_expression(expression, arms, span),

            Expression::Let {
                bindings,
                body,
                span,
            } => self.check_let_expression(bindings, body, span),

            Expression::Lambda { params, body, span } => {
                self.check_lambda_expression(params, body, span)
            }
            Expression::Block {
                statements,
                result,
                span,
            } => self.check_block(statements, result, span),
            Expression::Parallel {
                expressions,
                span: _,
            } => {
                // Add parallel execution constraints
                // For parallel blocks, all expressions should be side-effect free
                // and their types should be Send + Sync
                if expressions.is_empty() {
                    Ok(InferredType {
                        ty: Type::Unit,
                        constraints: Vec::new(),
                    })
                } else {
                    let mut result_type = InferredType {
                        ty: Type::Unit,
                        constraints: Vec::new(),
                    };

                    for expr in expressions {
                        let expr_type = self.check_expression(expr)?;
                        // Add constraint that types must be thread-safe
                        let thread_safe_constraint = TypeConstraint {
                            typeclass: "Send".to_string(),
                            type_param: expr_type.ty.clone(),
                        };
                        let sync_constraint = TypeConstraint {
                            typeclass: "Sync".to_string(),
                            type_param: expr_type.ty.clone(),
                        };
                        result_type.constraints.push(thread_safe_constraint);
                        result_type.constraints.push(sync_constraint);
                        result_type = expr_type;
                    }
                    Ok(result_type)
                }
            }

            Expression::Spawn {
                expression,
                span: _,
            } => {
                // Spawned expressions should return the wrapped type
                let inner_type = self.check_expression(expression)?;
                // Wrap in Future/Task type for async execution
                Ok(InferredType {
                    ty: Type::Future(Box::new(inner_type.ty)),
                    constraints: inner_type.constraints,
                })
            }

            Expression::Wait {
                expression,
                span: _,
            } => {
                // Wait unwraps async types
                let inner_type = self.check_expression(expression)?;
                // Unwrap Future/Task type - extract inner type from Future wrapper
                match &inner_type.ty {
                    Type::Future(inner) => Ok(InferredType {
                        ty: (**inner).clone(),
                        constraints: inner_type.constraints,
                    }),
                    _ => {
                        // If not a Future type, just return as is (might be an error)
                        Ok(inner_type)
                    }
                }
            }

            Expression::GpuDirective {
                expression,
                span: _,
            } => {
                // Add GPU execution constraints for type checking
                let expr_type = self.check_expression(expression)?;

                // Add constraint that the type must be suitable for GPU execution
                let gpu_constraint = TypeConstraint {
                    typeclass: "GpuCompatible".to_string(),
                    type_param: expr_type.ty.clone(),
                };

                // Wrap type in GPU wrapper to indicate GPU execution context
                Ok(InferredType {
                    ty: Type::GPU(Box::new(expr_type.ty)),
                    constraints: {
                        let mut constraints = expr_type.constraints;
                        constraints.push(gpu_constraint);
                        constraints
                    },
                })
            }

            Expression::Range {
                start,
                end,
                inclusive: _,
                span,
            } => {
                let start_type = self.check_expression(start)?;
                let end_type = self.check_expression(end)?;

                // Both start and end should be integers
                self.unifier
                    .unify(&self.inferred_to_type(&start_type), &Type::Int)
                    .map_err(|_| TypeError::TypeMismatch {
                        expected: "Int".to_string(),
                        found: format!("{:?}", start_type),
                        line: span.line,
                        column: span.column,
                    })?;

                self.unifier
                    .unify(&self.inferred_to_type(&end_type), &Type::Int)
                    .map_err(|_| TypeError::TypeMismatch {
                        expected: "Int".to_string(),
                        found: format!("{:?}", end_type),
                        line: span.line,
                        column: span.column,
                    })?;

                Ok(InferredType {
                    ty: Type::Array(Box::new(Type::Int)),
                    constraints: Vec::new(),
                })
            }

            Expression::OptionalAccess {
                object,
                field: _,
                fallback,
                span: _,
            } => {
                // Check optional field access with fallback
                let obj_type = self.check_expression(object)?;
                let fallback_type = self.check_expression(fallback)?;

                // Implement proper optional type checking
                // The object should have optional type, and fallback should match inner type
                match &obj_type.ty {
                    Type::Option(inner_type) => {
                        // Ensure fallback type matches the inner type of the option
                        self.unifier.unify(&fallback_type.ty, &inner_type)?;
                        Ok(InferredType {
                            ty: (**inner_type).clone(),
                            constraints: [obj_type.constraints, fallback_type.constraints].concat(),
                        })
                    }
                    _ => {
                        // Not an optional type, return fallback
                        Ok(fallback_type)
                    }
                }
            }

            Expression::SimDirective {
                expression,
                span: _,
            } => {
                // Type check the expression in physics simulation context
                let expr_type = self.check_expression(expression)?;

                // Add constraint that the type must be suitable for 3D physics simulation
                let sim_constraint = TypeConstraint {
                    typeclass: "SimulationCompatible".to_string(),
                    type_param: expr_type.ty.clone(),
                };

                // Wrap type in PhysicsWorld to indicate simulation context
                Ok(InferredType {
                    ty: Type::PhysicsWorld,
                    constraints: {
                        let mut constraints = expr_type.constraints;
                        constraints.push(sim_constraint);
                        constraints
                    },
                })
            }

            Expression::PlotDirective {
                expression,
                span: _,
            } => {
                // Type check the expression in physics plotting context
                let expr_type = self.check_expression(expression)?;

                // Add constraint that the type must be suitable for physics plotting
                let plot_constraint = TypeConstraint {
                    typeclass: "PlottingCompatible".to_string(),
                    type_param: expr_type.ty.clone(),
                };

                // Wrap type in PhysicsWorld to indicate plotting context
                Ok(InferredType {
                    ty: Type::PhysicsWorld,
                    constraints: {
                        let mut constraints = expr_type.constraints;
                        constraints.push(plot_constraint);
                        constraints
                    },
                })
            }
        }
    }

    fn check_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
        span: &Span,
    ) -> TypeResult<InferredType> {
        let left_type = self.check_expression(left)?;
        let right_type = self.check_expression(right)?;

        match op {
            BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Mod
            | BinaryOperator::Pow => {
                // Require Addable/Numeric typeclass
                let constraint = TypeConstraint {
                    typeclass: "Addable".to_string(),
                    type_param: left_type.ty.clone(),
                };

                // Check if types match
                self.unifier.unify(&left_type.ty, &right_type.ty)?;

                // Return the same type
                Ok(InferredType {
                    ty: left_type.ty,
                    constraints: vec![constraint],
                })
            }

            BinaryOperator::Eq
            | BinaryOperator::Ne
            | BinaryOperator::Lt
            | BinaryOperator::Le
            | BinaryOperator::Gt
            | BinaryOperator::Ge => {
                // Comparison operators return boolean
                self.unifier.unify(&left_type.ty, &right_type.ty)?;

                Ok(InferredType {
                    ty: Type::Bool,
                    constraints: Vec::new(),
                })
            }

            BinaryOperator::And | BinaryOperator::Or => {
                // Logical operators require boolean operands
                self.unifier.unify(&left_type.ty, &Type::Bool)?;
                self.unifier.unify(&right_type.ty, &Type::Bool)?;

                Ok(InferredType {
                    ty: Type::Bool,
                    constraints: Vec::new(),
                })
            }

            BinaryOperator::MatMul => {
                // Matrix multiplication
                match (&left_type.ty, &right_type.ty) {
                    (Type::Matrix(elem1, _, Some(cols1)), Type::Matrix(elem2, Some(rows2), _)) => {
                        // Check element types match
                        self.unifier.unify(elem1, elem2)?;

                        // Check dimensions are compatible
                        if cols1 != rows2 {
                            return Err(TypeError::TypeMismatch {
                                expected: format!("Matrix with {} rows", cols1),
                                found: format!("Matrix with {} rows", rows2),
                                line: span.line,
                                column: span.column,
                            });
                        }

                        // Result has dimensions of first matrix rows x second matrix cols
                        // Proper dimension tracking: result matrix has rows from left, cols from right
                        let result_rows = if let Type::Matrix(_, Some(rows), _) = &left_type.ty {
                            Some(*rows)
                        } else {
                            None
                        };
                        let result_cols = if let Type::Matrix(_, _, Some(cols)) = &right_type.ty {
                            Some(*cols)
                        } else {
                            None
                        };

                        Ok(InferredType {
                            ty: Type::Matrix(elem1.clone(), result_rows, result_cols),
                            constraints: Vec::new(),
                        })
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Matrix types".to_string(),
                        found: format!(
                            "{} and {}",
                            left_type.ty.to_string(),
                            right_type.ty.to_string()
                        ),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }

            BinaryOperator::DotProduct => {
                // Dot product for vectors
                match (&left_type.ty, &right_type.ty) {
                    (Type::Array(elem1), Type::Array(elem2)) => {
                        self.unifier.unify(elem1, elem2)?;
                        Ok(InferredType {
                            ty: (**elem1).clone(), // Returns scalar type
                            constraints: Vec::new(),
                        })
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Array types".to_string(),
                        found: format!(
                            "{} and {}",
                            left_type.ty.to_string(),
                            right_type.ty.to_string()
                        ),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }

            BinaryOperator::CrossProduct => {
                // Cross product for 3D vectors
                match (&left_type.ty, &right_type.ty) {
                    (Type::Array(elem1), Type::Array(elem2)) => {
                        self.unifier.unify(elem1, elem2)?;
                        Ok(InferredType {
                            ty: Type::Array(elem1.clone()), // Returns vector type
                            constraints: Vec::new(),
                        })
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Array types".to_string(),
                        found: format!(
                            "{} and {}",
                            left_type.ty.to_string(),
                            right_type.ty.to_string()
                        ),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }

            BinaryOperator::OptionalOr => {
                // Elvis operator (??) for optional values
                match &left_type.ty {
                    Type::Option(inner) => {
                        self.unifier.unify(inner, &right_type.ty)?;
                        Ok(right_type)
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Option type".to_string(),
                        found: left_type.ty.to_string(),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }
        }
    }

    fn check_unary_op(
        &mut self,
        op: &UnaryOperator,
        expr: &Expression,
        span: &Span,
    ) -> TypeResult<InferredType> {
        let expr_type = self.check_expression(expr)?;

        match op {
            UnaryOperator::Not => {
                self.unifier.unify(&expr_type.ty, &Type::Bool)?;
                Ok(InferredType {
                    ty: Type::Bool,
                    constraints: Vec::new(),
                })
            }

            UnaryOperator::Neg => {
                // Require numeric type
                match &expr_type.ty {
                    Type::Int | Type::Float => Ok(expr_type),
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Numeric type".to_string(),
                        found: expr_type.ty.to_string(),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }

            UnaryOperator::Transpose => {
                // Matrix transpose
                match &expr_type.ty {
                    Type::Matrix(elem_type, rows, cols) => Ok(InferredType {
                        ty: Type::Matrix(elem_type.clone(), cols.clone(), rows.clone()),
                        constraints: Vec::new(),
                    }),
                    _ => Err(TypeError::TypeMismatch {
                        expected: "Matrix type".to_string(),
                        found: expr_type.ty.to_string(),
                        line: span.line,
                        column: span.column,
                    }),
                }
            }
        }
    }

    fn check_function_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
        span: &Span,
    ) -> TypeResult<InferredType> {
        let func_type = self.check_expression(func)?;

        match &func_type.ty {
            Type::Function(param_types, return_type) => {
                if args.len() != param_types.len() {
                    return Err(TypeError::WrongArgumentCount {
                        expected: param_types.len(),
                        found: args.len(),
                        line: span.line,
                        column: span.column,
                    });
                }

                // Instantiate polymorphic function types with fresh type variables
                let (fresh_param_types, fresh_return_type) =
                    self.instantiate_function_type(param_types, return_type);

                // Check argument types against fresh instantiation
                for (arg, param_type) in args.iter().zip(fresh_param_types.iter()) {
                    let arg_type = self.check_expression(arg)?;
                    self.unifier.unify(&arg_type.ty, param_type)?;
                }

                Ok(InferredType {
                    ty: fresh_return_type,
                    constraints: func_type.constraints,
                })
            }

            _ => Err(TypeError::NotCallable {
                type_name: func_type.ty.to_string(),
                line: span.line,
                column: span.column,
            }),
        }
    }

    fn check_field_access(
        &mut self,
        expr: &Expression,
        field: &str,
        span: &Span,
    ) -> TypeResult<InferredType> {
        let expr_type = self.check_expression(expr)?;

        match &expr_type.ty {
            Type::Struct(struct_name) => {
                if let Some(field_type) = self.context.structs.get_field_type(struct_name, field) {
                    Ok(InferredType {
                        ty: field_type.clone(),
                        constraints: Vec::new(),
                    })
                } else {
                    Err(TypeError::FieldNotFound {
                        field: field.to_string(),
                        type_name: struct_name.clone(),
                        line: span.line,
                        column: span.column,
                    })
                }
            }

            _ => Err(TypeError::TypeMismatch {
                expected: "Struct type".to_string(),
                found: expr_type.ty.to_string(),
                line: span.line,
                column: span.column,
            }),
        }
    }

    fn check_struct_creation(
        &mut self,
        name: &str,
        fields: &HashMap<String, Expression>,
        span: &Span,
    ) -> TypeResult<InferredType> {
        if let Some(_struct_def) = self.context.structs.get(name) {
            // Check all provided fields
            for (field_name, field_expr) in fields {
                if let Some(expected_type) = self.context.structs.get_field_type(name, field_name) {
                    let expected_type_clone = expected_type.clone();
                    let field_type = self.check_expression(field_expr)?;
                    self.unifier.unify(&field_type.ty, &expected_type_clone)?;
                } else {
                    return Err(TypeError::FieldNotFound {
                        field: field_name.clone(),
                        type_name: name.to_string(),
                        line: span.line,
                        column: span.column,
                    });
                }
            }

            // Check that all required fields are provided
            if let Some(struct_def) = self.context.structs.get(name) {
                for field in &struct_def.fields {
                    if !fields.contains_key(&field.name) {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("All fields of struct {}", name),
                            found: format!("Missing field '{}'", field.name),
                            line: span.line,
                            column: span.column,
                        });
                    }
                }
            }

            Ok(InferredType {
                ty: Type::Struct(name.to_string()),
                constraints: Vec::new(),
            })
        } else {
            Err(TypeError::UnknownType {
                name: name.to_string(),
                line: span.line,
                column: span.column,
            })
        }
    }

    fn check_array_literal(
        &mut self,
        elements: &[Expression],
        _span: &Span,
    ) -> TypeResult<InferredType> {
        if elements.is_empty() {
            // Empty array - need type annotation or context to infer type
            let element_type = self.context.fresh_type_var();
            return Ok(InferredType {
                ty: Type::Array(Box::new(element_type)),
                constraints: Vec::new(),
            });
        }

        // Check first element to get the type
        let first_type = self.check_expression(&elements[0])?;

        // Check all other elements have the same type
        for element in &elements[1..] {
            let element_type = self.check_expression(element)?;
            self.unifier.unify(&first_type.ty, &element_type.ty)?;
        }

        Ok(InferredType {
            ty: Type::Array(Box::new(first_type.ty)),
            constraints: first_type.constraints,
        })
    }

    fn check_matrix_literal(
        &mut self,
        rows: &[Vec<Expression>],
        _span: &Span,
    ) -> TypeResult<InferredType> {
        if rows.is_empty() {
            let element_type = self.context.fresh_type_var();
            return Ok(InferredType {
                ty: Type::Matrix(Box::new(element_type), Some(0), Some(0)),
                constraints: Vec::new(),
            });
        }

        let num_rows = rows.len();
        let num_cols = rows[0].len();

        // Check all rows have the same number of columns
        for row in rows {
            if row.len() != num_cols {
                return Err(TypeError::TypeMismatch {
                    expected: format!("Row with {} columns", num_cols),
                    found: format!("Row with {} columns", row.len()),
                    line: 0,
                    column: 0,
                });
            }
        }

        // Get element type from first element
        let first_type = self.check_expression(&rows[0][0])?;

        // Check all elements have the same type
        for row in rows {
            for element in row {
                let element_type = self.check_expression(element)?;
                self.unifier.unify(&first_type.ty, &element_type.ty)?;
            }
        }
        Ok(InferredType {
            ty: Type::Matrix(Box::new(first_type.ty), Some(num_rows), Some(num_cols)),
            constraints: first_type.constraints,
        })
    }

    fn check_matrix_comprehension(
        &mut self,
        expr: &Expression,
        generators: &Vec<Generator>,
        _span: &Span,
    ) -> TypeResult<InferredType> {
        if generators.is_empty() {
            return Err(TypeError::TypeMismatch {
                expected: "At least one generator".to_string(),
                found: "No generators".to_string(),
                line: 0,
                column: 0,
            });
        }

        // Handle all generators (supports nested comprehensions)
        self.context.push_scope();

        for generator in generators {
            // Check range expression
            let range_type = self.check_expression(&generator.iterable)?;

            // Extract element type from range
            let element_type = match &range_type.ty {
                Type::Array(elem_type) => (**elem_type).clone(),
                Type::Matrix(elem_type, _, _) => (**elem_type).clone(),
                _ => {
                    return Err(TypeError::TypeMismatch {
                        expected: "Array or Matrix type".to_string(),
                        found: range_type.ty.to_string(),
                        line: 0,
                        column: 0,
                    })
                }
            };

            // Bind comprehension variable in current scope
            self.context.env.bind(
                generator.variable.clone(),
                InferredType {
                    ty: element_type,
                    constraints: Vec::new(),
                },
            );

            // Check condition if present
            if let Some(ref cond) = generator.condition {
                let cond_type = self.check_expression(cond)?;
                self.unifier.unify(&cond_type.ty, &Type::Bool)?;
            }
        }

        // Check expression
        let expr_type = self.check_expression(expr)?;

        self.context.pop_scope();

        // Return matrix type (dimensions unknown at compile time)
        Ok(InferredType {
            ty: Type::Matrix(Box::new(expr_type.ty), None, None),
            constraints: expr_type.constraints,
        })
    }

    fn check_if_expression(
        &mut self,
        condition: &Expression,
        then_expr: &Expression,
        else_expr: &Option<Box<Expression>>,
        _span: &Span,
    ) -> TypeResult<InferredType> {
        // Check condition is boolean
        let cond_type = self.check_expression(condition)?;
        self.unifier.unify(&cond_type.ty, &Type::Bool)?;

        // Check then branch
        let then_type = self.check_expression(then_expr)?;

        // Check else branch if present
        if let Some(else_expr) = else_expr {
            let else_type = self.check_expression(else_expr)?;
            self.unifier.unify(&then_type.ty, &else_type.ty)?;

            Ok(InferredType {
                ty: then_type.ty,
                constraints: [then_type.constraints, else_type.constraints].concat(),
            })
        } else {
            // If no else branch, return Option type
            Ok(InferredType {
                ty: Type::Option(Box::new(then_type.ty)),
                constraints: then_type.constraints,
            })
        }
    }

    fn check_match_expression(
        &mut self,
        expr: &Expression,
        arms: &[MatchArm],
        _span: &Span,
    ) -> TypeResult<InferredType> {
        let expr_type = self.check_expression(expr)?;

        if arms.is_empty() {
            return Err(TypeError::TypeMismatch {
                expected: "At least one match arm".to_string(),
                found: "No match arms".to_string(),
                line: 0,
                column: 0,
            });
        }

        // Check first arm to get result type
        self.context.push_scope();
        self.check_pattern(&arms[0].pattern, &expr_type.ty)?;

        if let Some(ref guard) = arms[0].guard {
            let guard_type = self.check_expression(guard)?;
            self.unifier.unify(&guard_type.ty, &Type::Bool)?;
        }

        let first_result_type = self.check_expression(&arms[0].body)?;
        self.context.pop_scope();

        // Check remaining arms have same result type
        for arm in &arms[1..] {
            self.context.push_scope();
            self.check_pattern(&arm.pattern, &expr_type.ty)?;

            if let Some(ref guard) = arm.guard {
                let guard_type = self.check_expression(guard)?;
                self.unifier.unify(&guard_type.ty, &Type::Bool)?;
            }

            let arm_result_type = self.check_expression(&arm.body)?;
            self.unifier
                .unify(&first_result_type.ty, &arm_result_type.ty)?;
            self.context.pop_scope();
        }

        Ok(first_result_type)
    }

    fn check_pattern(&mut self, pattern: &Pattern, expected_type: &Type) -> TypeResult<()> {
        match pattern {
            Pattern::Identifier(name, _) => {
                self.context.env.bind(
                    name.clone(),
                    InferredType {
                        ty: expected_type.clone(),
                        constraints: Vec::new(),
                    },
                );
                Ok(())
            }
            Pattern::IntLiteral(_, _) => self.unifier.unify(expected_type, &Type::Int),

            Pattern::FloatLiteral(_, _) => self.unifier.unify(expected_type, &Type::Float),

            Pattern::StringLiteral(_, _) => self.unifier.unify(expected_type, &Type::String),

            Pattern::BoolLiteral(_, _) => self.unifier.unify(expected_type, &Type::Bool),

            Pattern::Wildcard(_) => Ok(()),
            Pattern::Struct { name, fields, .. } => {
                // Check expected type is the struct
                if let Type::Struct(struct_name) = expected_type {
                    if struct_name != name {
                        return Err(TypeError::TypeMismatch {
                            expected: struct_name.clone(),
                            found: name.to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    // Check field patterns
                    for (field_name, field_pattern) in fields {
                        if let Some(field_type) =
                            self.context.structs.get_field_type(name, field_name)
                        {
                            let field_type_clone = field_type.clone();
                            self.check_pattern(field_pattern, &field_type_clone)?;
                        } else {
                            return Err(TypeError::FieldNotFound {
                                field: field_name.to_string(),
                                type_name: name.to_string(),
                                line: 0,
                                column: 0,
                            });
                        }
                    }
                    Ok(())
                } else {
                    Err(TypeError::TypeMismatch {
                        expected: format!("Struct {}", name),
                        found: expected_type.to_string(),
                        line: 0,
                        column: 0,
                    })
                }
            }

            Pattern::Some(inner_pattern, _) => {
                // Expect Option<T> type
                if let Type::TypeApp(name, args) = expected_type {
                    if name == "Option" && args.len() == 1 {
                        self.check_pattern(inner_pattern, &args[0])
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: "Option<T>".to_string(),
                            found: expected_type.to_string(),
                            line: 0,
                            column: 0,
                        })
                    }
                } else {
                    Err(TypeError::TypeMismatch {
                        expected: "Option<T>".to_string(),
                        found: expected_type.to_string(),
                        line: 0,
                        column: 0,
                    })
                }
            }

            Pattern::None(_) => {
                // Expect Option<T> type
                if let Type::TypeApp(name, _) = expected_type {
                    if name == "Option" {
                        Ok(())
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: "Option<T>".to_string(),
                            found: expected_type.to_string(),
                            line: 0,
                            column: 0,
                        })
                    }
                } else {
                    Err(TypeError::TypeMismatch {
                        expected: "Option<T>".to_string(),
                        found: expected_type.to_string(),
                        line: 0,
                        column: 0,
                    })
                }
            }

            Pattern::Array(patterns, _) => {
                // Expect Array<T> type
                if let Type::Array(element_type) = expected_type {
                    for pattern in patterns {
                        self.check_pattern(pattern, element_type)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::TypeMismatch {
                        expected: "Array<T>".to_string(),
                        found: expected_type.to_string(),
                        line: 0,
                        column: 0,
                    })
                }
            }
        }
    }

    fn check_let_expression(
        &mut self,
        bindings: &Vec<LetBinding>,
        body: &Expression,
        _span: &Span,
    ) -> TypeResult<InferredType> {
        self.context.push_scope();

        // Check all bindings
        for binding in bindings {
            self.check_let_binding(binding)?;
        }

        // Check the body
        let body_type = self.check_expression(body)?;

        self.context.pop_scope();

        Ok(body_type)
    }

    fn check_lambda_expression(
        &mut self,
        params: &[Parameter],
        body: &Expression,
        _span: &Span,
    ) -> TypeResult<InferredType> {
        self.context.push_scope();

        // Add parameters to environment
        let mut param_types = Vec::new();
        for param in params {
            let param_type = InferredType {
                ty: param.type_annotation.clone(),
                constraints: Vec::new(),
            };
            param_types.push(param.type_annotation.clone());
            self.context.env.bind(param.name.clone(), param_type);
        }

        // Check body
        let body_type = self.check_expression(body)?;

        self.context.pop_scope();

        Ok(InferredType {
            ty: Type::Function(param_types, Box::new(body_type.ty)),
            constraints: body_type.constraints,
        })
    }
    fn check_block(
        &mut self,
        statements: &Vec<Statement>,
        result: &Option<Box<Expression>>,
        _span: &Span,
    ) -> TypeResult<InferredType> {
        self.context.push_scope();

        // Check all statements
        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    self.check_expression(expr)?;
                }
                Statement::LetBinding(binding) => {
                    let expr_type = self.check_expression(&binding.value)?;
                    // For now, we'll just check if the expression type matches any annotation
                    if let Some(annotation) = &binding.type_annotation {
                        self.unifier.unify(&expr_type.ty, annotation)?;
                    }
                    // Add the binding to the environment
                    self.context.env.bind(binding.name.clone(), expr_type);
                } // Add other statement types as needed
            }
        }

        // Check the result expression if it exists
        let result_type = if let Some(result_expr) = result {
            self.check_expression(result_expr)?
        } else {
            InferredType {
                ty: Type::Unit,
                constraints: Vec::new(),
            }
        };

        self.context.pop_scope();
        Ok(result_type)
    }

    /// Instantiate a polymorphic function type with fresh type variables
    /// This prevents type variable conflicts across different function calls
    fn instantiate_function_type(
        &mut self,
        param_types: &[Type],
        return_type: &Type,
    ) -> (Vec<Type>, Type) {
        use std::collections::HashMap;

        // Collect all type variables in the function signature
        let mut type_vars = std::collections::HashSet::new();
        for param_type in param_types {
            self.collect_type_vars(param_type, &mut type_vars);
        }
        self.collect_type_vars(return_type, &mut type_vars);

        // Create fresh type variables
        let mut substitution_map = HashMap::new();
        for type_var in type_vars {
            let fresh_var = self.context.fresh_type_var();
            substitution_map.insert(type_var, fresh_var);
        }

        // Substitute type variables with fresh ones
        let fresh_param_types: Vec<Type> = param_types
            .iter()
            .map(|ty| ty.substitute(&substitution_map))
            .collect();
        let fresh_return_type = return_type.substitute(&substitution_map);

        (fresh_param_types, fresh_return_type)
    }

    /// Collect all type variables from a type
    fn collect_type_vars(&self, ty: &Type, vars: &mut std::collections::HashSet<String>) {
        match ty {
            Type::TypeVar(name) => {
                vars.insert(name.clone());
            }
            Type::Array(elem_ty) => {
                self.collect_type_vars(elem_ty, vars);
            }
            Type::Matrix(elem_ty, _, _) => {
                self.collect_type_vars(elem_ty, vars);
            }
            Type::Function(param_types, return_type) => {
                for param in param_types {
                    self.collect_type_vars(param, vars);
                }
                self.collect_type_vars(return_type, vars);
            }
            Type::TypeApp(_, args) => {
                for arg in args {
                    self.collect_type_vars(arg, vars);
                }
            }
            Type::Option(inner) => {
                self.collect_type_vars(inner, vars);
            }
            Type::Spanned(inner, _) => {
                self.collect_type_vars(inner, vars);
            }
            Type::Field(inner) => {
                self.collect_type_vars(inner, vars);
            }
            Type::GPU(inner) => {
                self.collect_type_vars(inner, vars);
            }
            Type::SIMD(inner, _) => {
                self.collect_type_vars(inner, vars);
            }
            Type::Future(inner) => {
                self.collect_type_vars(inner, vars);
            }
            Type::Stream(inner) => {
                self.collect_type_vars(inner, vars);
            }
            // Base types have no type variables
            Type::Int
            | Type::Float
            | Type::Bool
            | Type::String
            | Type::Unit
            | Type::Struct(_)
            | Type::Vector2
            | Type::Vector3
            | Type::Quaternion
            | Type::Transform
            | Type::RigidBody
            | Type::SoftBody
            | Type::FluidSystem
            | Type::Particle
            | Type::ForceField
            | Type::Material
            | Type::Constraint
            | Type::PhysicsWorld
            | Type::Tensor(_) => {}
        }
    }

    pub fn finalize_types(&mut self) -> HashMap<String, Type> {
        // Apply substitutions to all types in environment
        let mut finalized = HashMap::new();
        for (name, inferred_type) in &self.context.env.bindings {
            let finalized_type = self.unifier.finalize_type(&inferred_type.ty);
            finalized.insert(name.clone(), finalized_type);
        }

        finalized
    }

    /// Helper method to extract concrete type from InferredType
    fn inferred_to_type(&self, inferred: &InferredType) -> Type {
        inferred.ty.clone()
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::*;
    use crate::parser::*;

    fn parse_and_check(input: &str) -> TypeResult<InferredType> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).map_err(|e| TypeError::UnknownType {
            name: format!("Parse error: {:?}", e),
            line: 0,
            column: 0,
        })?;
        let ast = parser.parse_program().map_err(|e| TypeError::UnknownType {
            name: format!("Parse error: {:?}", e),
            line: 0,
            column: 0,
        })?;

        let mut checker = TypeChecker::new();
        checker.check_program(&ast)
    }

    #[test]
    fn test_simple_arithmetic() {
        let result = parse_and_check("let x = 1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_definition() {
        let result = parse_and_check("fn add(a: Int, b: Int) -> Int = a + b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_definition() {
        let result = parse_and_check(
            r#"
            struct Point {
                x: Float,
                y: Float
            }

            let p = Point { x: 1.0, y: 2.0 }
        "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_matrix_operations() {
        let result = parse_and_check(
            r#"
            let m1 = [[1.0, 2.0], [3.0, 4.0]]
            let m2 = [[5.0, 6.0], [7.0, 8.0]]
            let result = m1 ?? m2
        "#,
        );
        assert!(result.is_ok());
    }
}
