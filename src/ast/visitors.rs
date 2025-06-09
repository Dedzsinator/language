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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_span() -> Span {
        Span::new(0, 10, 1, 1)
    }

    // Test visitor that counts expression types
    struct ExpressionCounter {
        int_literals: usize,
        float_literals: usize,
        bool_literals: usize,
        string_literals: usize,
        identifiers: usize,
        binary_ops: usize,
        function_calls: usize,
        total_expressions: usize,
    }

    impl ExpressionCounter {
        fn new() -> Self {
            Self {
                int_literals: 0,
                float_literals: 0,
                bool_literals: 0,
                string_literals: 0,
                identifiers: 0,
                binary_ops: 0,
                function_calls: 0,
                total_expressions: 0,
            }
        }
    }

    impl AstVisitor for ExpressionCounter {
        type Result = ();

        fn visit_program(&mut self, program: &Program) -> Self::Result {
            for item in &program.items {
                self.visit_item(item);
            }
        }

        fn visit_item(&mut self, item: &Item) -> Self::Result {
            match item {
                Item::FunctionDef(func_def) => self.visit_function_def(func_def),
                Item::LetBinding(let_binding) => self.visit_let_binding(let_binding),
                _ => (), // Other items don't contain expressions directly
            }
        }

        fn visit_struct_def(&mut self, _struct_def: &StructDef) -> Self::Result {}

        fn visit_typeclass_def(&mut self, _typeclass_def: &TypeclassDef) -> Self::Result {}

        fn visit_instance_def(&mut self, instance_def: &InstanceDef) -> Self::Result {
            for impl_ in &instance_def.implementations {
                self.visit_expression(&impl_.body);
            }
        }

        fn visit_function_def(&mut self, function_def: &FunctionDef) -> Self::Result {
            self.visit_expression(&function_def.body);
        }

        fn visit_let_binding(&mut self, let_binding: &LetBinding) -> Self::Result {
            self.visit_expression(&let_binding.value);
        }

        fn visit_expression(&mut self, expression: &Expression) -> Self::Result {
            self.total_expressions += 1;

            match expression {
                Expression::IntLiteral(_, _) => self.int_literals += 1,
                Expression::FloatLiteral(_, _) => self.float_literals += 1,
                Expression::BoolLiteral(_, _) => self.bool_literals += 1,
                Expression::StringLiteral(_, _) => self.string_literals += 1,
                Expression::Identifier(_, _) => self.identifiers += 1,
                Expression::BinaryOp { left, right, .. } => {
                    self.binary_ops += 1;
                    self.visit_expression(left);
                    self.visit_expression(right);
                }
                Expression::FunctionCall { function, args, .. } => {
                    self.function_calls += 1;
                    self.visit_expression(function);
                    for arg in args {
                        self.visit_expression(arg);
                    }
                }
                Expression::ArrayLiteral(elements, _) => {
                    for element in elements {
                        self.visit_expression(element);
                    }
                }
                Expression::IfExpression {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.visit_expression(condition);
                    self.visit_expression(then_branch);
                    if let Some(else_expr) = else_branch {
                        self.visit_expression(else_expr);
                    }
                }
                _ => {
                    // Use the walk function for other expression types
                    walk_expression(self, expression);
                }
            }
        }

        fn visit_type(&mut self, _type_: &Type) -> Self::Result {}

        fn visit_pattern(&mut self, _pattern: &Pattern) -> Self::Result {}
    }

    // Test mutable visitor that adds span information
    struct SpanAdder {
        new_span: Span,
    }

    impl SpanAdder {
        fn new(span: Span) -> Self {
            Self { new_span: span }
        }
    }

    impl AstVisitorMut for SpanAdder {
        fn visit_program_mut(&mut self, program: &mut Program) {
            program.span = self.new_span.clone();
            for item in &mut program.items {
                self.visit_item_mut(item);
            }
        }

        fn visit_item_mut(&mut self, item: &mut Item) {
            match item {
                Item::StructDef(struct_def) => self.visit_struct_def_mut(struct_def),
                Item::FunctionDef(func_def) => self.visit_function_def_mut(func_def),
                Item::LetBinding(let_binding) => self.visit_let_binding_mut(let_binding),
                _ => {}
            }
        }

        fn visit_struct_def_mut(&mut self, struct_def: &mut StructDef) {
            struct_def.span = self.new_span.clone();
        }

        fn visit_typeclass_def_mut(&mut self, typeclass_def: &mut TypeclassDef) {
            typeclass_def.span = self.new_span.clone();
        }

        fn visit_instance_def_mut(&mut self, instance_def: &mut InstanceDef) {
            instance_def.span = self.new_span.clone();
        }

        fn visit_function_def_mut(&mut self, function_def: &mut FunctionDef) {
            function_def.span = self.new_span.clone();
            self.visit_expression_mut(&mut function_def.body);
        }

        fn visit_let_binding_mut(&mut self, let_binding: &mut LetBinding) {
            let_binding.span = self.new_span.clone();
            self.visit_expression_mut(&mut let_binding.value);
        }

        fn visit_expression_mut(&mut self, expression: &mut Expression) {
            // Update the span of the expression
            match expression {
                Expression::IntLiteral(_, span) => *span = self.new_span.clone(),
                Expression::Identifier(_, span) => *span = self.new_span.clone(),
                Expression::BinaryOp {
                    left, right, span, ..
                } => {
                    *span = self.new_span.clone();
                    self.visit_expression_mut(left);
                    self.visit_expression_mut(right);
                }
                _ => {} // Add more cases as needed
            }
        }

        fn visit_type_mut(&mut self, _type_: &mut Type) {}

        fn visit_pattern_mut(&mut self, _pattern: &mut Pattern) {}
    }

    #[test]
    fn test_expression_counter_visitor() {
        let span = create_test_span();

        // Create a program with various expressions
        let left = Box::new(Expression::IntLiteral(1, span.clone()));
        let right = Box::new(Expression::IntLiteral(2, span.clone()));
        let binary_expr = Expression::BinaryOp {
            left,
            operator: BinaryOperator::Add,
            right,
            span: span.clone(),
        };

        let func_def = FunctionDef {
            name: "test".to_string(),
            params: vec![],
            return_type: Some(Type::Int),
            body: binary_expr,
            attributes: vec![],
            span: span.clone(),
        };

        let program = Program {
            items: vec![Item::FunctionDef(func_def)],
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        counter.visit_program(&program);

        assert_eq!(counter.int_literals, 2);
        assert_eq!(counter.binary_ops, 1);
        assert_eq!(counter.total_expressions, 3); // 2 int literals + 1 binary op
    }

    #[test]
    fn test_function_call_visitor() {
        let span = create_test_span();

        let function = Box::new(Expression::Identifier("add".to_string(), span.clone()));
        let args = vec![
            Expression::IntLiteral(1, span.clone()),
            Expression::IntLiteral(2, span.clone()),
        ];

        let call_expr = Expression::FunctionCall {
            function,
            args,
            span: span.clone(),
        };

        let func_def = FunctionDef {
            name: "test".to_string(),
            params: vec![],
            return_type: Some(Type::Int),
            body: call_expr,
            attributes: vec![],
            span: span.clone(),
        };

        let program = Program {
            items: vec![Item::FunctionDef(func_def)],
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        counter.visit_program(&program);

        assert_eq!(counter.function_calls, 1);
        assert_eq!(counter.identifiers, 1);
        assert_eq!(counter.int_literals, 2);
        assert_eq!(counter.total_expressions, 4); // 1 call + 1 identifier + 2 int literals
    }

    #[test]
    fn test_if_expression_visitor() {
        let span = create_test_span();

        let condition = Box::new(Expression::BoolLiteral(true, span.clone()));
        let then_branch = Box::new(Expression::IntLiteral(1, span.clone()));
        let else_branch = Some(Box::new(Expression::IntLiteral(2, span.clone())));

        let if_expr = Expression::IfExpression {
            condition,
            then_branch,
            else_branch,
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        counter.visit_expression(&if_expr);

        assert_eq!(counter.bool_literals, 1);
        assert_eq!(counter.int_literals, 2);
        assert_eq!(counter.total_expressions, 4); // 1 if + 1 bool + 2 int
    }

    #[test]
    fn test_array_literal_visitor() {
        let span = create_test_span();

        let elements = vec![
            Expression::IntLiteral(1, span.clone()),
            Expression::IntLiteral(2, span.clone()),
            Expression::IntLiteral(3, span.clone()),
        ];

        let array_expr = Expression::ArrayLiteral(elements, span.clone());

        let mut counter = ExpressionCounter::new();
        counter.visit_expression(&array_expr);

        assert_eq!(counter.int_literals, 3);
        assert_eq!(counter.total_expressions, 4); // 1 array + 3 int literals
    }

    #[test]
    fn test_mutable_visitor() {
        let old_span = create_test_span();
        let new_span = Span::new(10, 20, 2, 5);

        let mut program = Program {
            items: vec![],
            span: old_span.clone(),
        };

        let mut span_adder = SpanAdder::new(new_span.clone());
        span_adder.visit_program_mut(&mut program);

        assert_eq!(program.span, new_span);
    }

    #[test]
    fn test_walk_expression_function() {
        let span = create_test_span();

        let left = Box::new(Expression::IntLiteral(1, span.clone()));
        let right = Box::new(Expression::IntLiteral(2, span.clone()));
        let binary_expr = Expression::BinaryOp {
            left,
            operator: BinaryOperator::Add,
            right,
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        walk_expression(&mut counter, &binary_expr);

        // The walk function should visit the binary expression and its children
        assert_eq!(counter.binary_ops, 1);
        assert_eq!(counter.int_literals, 2);
    }

    #[test]
    fn test_nested_expressions_visitor() {
        let span = create_test_span();

        // Create a nested expression: (1 + 2) * 3
        let inner_left = Box::new(Expression::IntLiteral(1, span.clone()));
        let inner_right = Box::new(Expression::IntLiteral(2, span.clone()));
        let inner_binary = Expression::BinaryOp {
            left: inner_left,
            operator: BinaryOperator::Add,
            right: inner_right,
            span: span.clone(),
        };

        let outer_left = Box::new(inner_binary);
        let outer_right = Box::new(Expression::IntLiteral(3, span.clone()));
        let outer_binary = Expression::BinaryOp {
            left: outer_left,
            operator: BinaryOperator::Mul,
            right: outer_right,
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        counter.visit_expression(&outer_binary);

        assert_eq!(counter.binary_ops, 2);
        assert_eq!(counter.int_literals, 3);
        assert_eq!(counter.total_expressions, 5);
    }

    #[test]
    fn test_visitor_with_let_binding() {
        let span = create_test_span();

        let value = Expression::IntLiteral(42, span.clone());
        let let_binding = LetBinding {
            name: "x".to_string(),
            type_annotation: Some(Type::Int),
            value,
            span: span.clone(),
        };

        let program = Program {
            items: vec![Item::LetBinding(let_binding)],
            span: span.clone(),
        };

        let mut counter = ExpressionCounter::new();
        counter.visit_program(&program);

        assert_eq!(counter.int_literals, 1);
        assert_eq!(counter.total_expressions, 1);
    }
}
