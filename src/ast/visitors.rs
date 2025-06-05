use crate::ast::nodes::*;

/// Visitor trait for AST traversal
pub trait AstVisitor {
    type Result;

    fn visit_program(&mut self, program: &Program) -> Self::Result;
    fn visit_item(&mut self, item: &Item) -> Self::Result;
    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Self::Result;
    fn visit_typeclass_def(&mut self, typeclass_def: &TypeclassDef) -> Self::Result;
    fn visit_instance_def(&mut self, instance_def: &InstanceDef) -> Self::Result;
    fn visit_function_def(&mut self, function_def: &FunctionDef) -> Self::Result;
    fn visit_let_binding(&mut self, let_binding: &LetBinding) -> Self::Result;
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result;
    fn visit_type(&mut self, type_: &Type) -> Self::Result;
    fn visit_pattern(&mut self, pattern: &Pattern) -> Self::Result;
}

/// Mutable visitor for AST transformation
pub trait AstVisitorMut {
    fn visit_program_mut(&mut self, program: &mut Program);
    fn visit_item_mut(&mut self, item: &mut Item);
    fn visit_struct_def_mut(&mut self, struct_def: &mut StructDef);
    fn visit_typeclass_def_mut(&mut self, typeclass_def: &mut TypeclassDef);
    fn visit_instance_def_mut(&mut self, instance_def: &mut InstanceDef);
    fn visit_function_def_mut(&mut self, function_def: &mut FunctionDef);
    fn visit_let_binding_mut(&mut self, let_binding: &mut LetBinding);
    fn visit_expression_mut(&mut self, expression: &mut Expression);
    fn visit_type_mut(&mut self, type_: &mut Type);
    fn visit_pattern_mut(&mut self, pattern: &mut Pattern);
}

/// Default implementation for walking the AST
pub fn walk_program<V: AstVisitor>(visitor: &mut V, program: &Program) -> V::Result {
    visitor.visit_program(program)
}

pub fn walk_expression<V: AstVisitor>(visitor: &mut V, expr: &Expression) -> V::Result {
    match expr {
        Expression::StructCreation { fields, .. } => {
            for field_expr in fields.values() {
                visitor.visit_expression(field_expr);
            }
        }
        Expression::ArrayLiteral(elements, _) => {
            for element in elements {
                visitor.visit_expression(element);
            }
        }
        Expression::MatrixLiteral(rows, _) => {
            for row in rows {
                for element in row {
                    visitor.visit_expression(element);
                }
            }
        }
        Expression::MatrixComprehension {
            element,
            generators,
            ..
        } => {
            visitor.visit_expression(element);
            for generator in generators {
                visitor.visit_expression(&generator.iterable);
                if let Some(condition) = &generator.condition {
                    visitor.visit_expression(condition);
                }
            }
        }
        Expression::FunctionCall { function, args, .. } => {
            visitor.visit_expression(function);
            for arg in args {
                visitor.visit_expression(arg);
            }
        }
        Expression::Lambda { body, .. } => {
            visitor.visit_expression(body);
        }
        Expression::BinaryOp { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_expression(right);
        }
        Expression::UnaryOp { operand, .. } => {
            visitor.visit_expression(operand);
        }
        Expression::FieldAccess { object, .. } => {
            visitor.visit_expression(object);
        }
        Expression::OptionalAccess {
            object, fallback, ..
        } => {
            visitor.visit_expression(object);
            visitor.visit_expression(fallback);
        }
        Expression::IfExpression {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visitor.visit_expression(condition);
            visitor.visit_expression(then_branch);
            if let Some(else_expr) = else_branch {
                visitor.visit_expression(else_expr);
            }
        }
        Expression::Match {
            expression, arms, ..
        } => {
            visitor.visit_expression(expression);
            for arm in arms {
                visitor.visit_pattern(&arm.pattern);
                if let Some(guard) = &arm.guard {
                    visitor.visit_expression(guard);
                }
                visitor.visit_expression(&arm.body);
            }
        }
        Expression::Let { bindings, body, .. } => {
            for binding in bindings {
                visitor.visit_let_binding(binding);
            }
            visitor.visit_expression(body);
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    Statement::Expression(expr) => visitor.visit_expression(expr),
                    Statement::LetBinding(binding) => visitor.visit_let_binding(binding),
                };
            }
            if let Some(result_expr) = result {
                visitor.visit_expression(result_expr);
            }
        }
        Expression::Parallel { expressions, .. } => {
            for expr in expressions {
                visitor.visit_expression(expr);
            }
        }
        Expression::Spawn { expression, .. } => {
            visitor.visit_expression(expression);
        }
        Expression::Wait { expression, .. } => {
            visitor.visit_expression(expression);
        }
        Expression::GpuDirective { expression, .. } => {
            visitor.visit_expression(expression);
        }
        Expression::Range { start, end, .. } => {
            visitor.visit_expression(start);
            visitor.visit_expression(end);
        }
        // Literals and identifiers don't have children to visit
        _ => {}
    }

    visitor.visit_expression(expr)
}
