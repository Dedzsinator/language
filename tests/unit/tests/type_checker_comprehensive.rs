use crate::types::{Type, TypeChecker, TypeEnvironment, TypeError};
use crate::ast::nodes::*;
use crate::parser::Parser;
use crate::lexer::Lexer;
use super::test_utilities::*;
use std::collections::HashMap;

#[cfg(test)]
mod type_checker_comprehensive_tests {
    use super::*;

    // Helper function to type check source code
    fn type_check_source(source: &str) -> Result<Type, TypeError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast)
    }

    // Helper function to type check expression
    fn type_check_expr(source: &str) -> Result<Type, TypeError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression().unwrap();
        
        let mut type_checker = TypeChecker::new();
        let mut env = TypeEnvironment::new();
        type_checker.check_expression(&expr, &mut env)
    }

    #[test]
    fn test_primitive_type_inference() {
        // Integer literals
        assert_eq!(type_check_expr("42").unwrap(), Type::Int);
        assert_eq!(type_check_expr("0").unwrap(), Type::Int);
        assert_eq!(type_check_expr("-123").unwrap(), Type::Int);
        
        // Float literals
        assert_eq!(type_check_expr("3.14").unwrap(), Type::Float);
        assert_eq!(type_check_expr("0.0").unwrap(), Type::Float);
        assert_eq!(type_check_expr("-2.5").unwrap(), Type::Float);
        
        // Boolean literals
        assert_eq!(type_check_expr("true").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("false").unwrap(), Type::Bool);
        
        // String literals
        assert_eq!(type_check_expr("\"hello\"").unwrap(), Type::String);
        assert_eq!(type_check_expr("\"\"").unwrap(), Type::String);
    }

    #[test]
    fn test_vector_type_inference() {
        // Vector literals
        assert_eq!(type_check_expr("[1, 2, 3]").unwrap(), Type::Vector(Box::new(Type::Int), 3));
        assert_eq!(type_check_expr("[1.0, 2.0]").unwrap(), Type::Vector(Box::new(Type::Float), 2));
        assert_eq!(type_check_expr("[true, false, true, false]").unwrap(), Type::Vector(Box::new(Type::Bool), 4));
        
        // Empty vector (should infer as generic or require annotation)
        let result = type_check_expr("[]");
        assert!(result.is_err() || matches!(result.unwrap(), Type::Vector(_, 0)));
    }

    #[test]
    fn test_matrix_type_inference() {
        // Matrix literals
        let matrix_2x2 = "[[1, 2], [3, 4]]";
        assert_eq!(type_check_expr(matrix_2x2).unwrap(), Type::Matrix(Box::new(Type::Int), 2, 2));
        
        let matrix_3x2 = "[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]";
        assert_eq!(type_check_expr(matrix_3x2).unwrap(), Type::Matrix(Box::new(Type::Float), 3, 2));
    }

    #[test]
    fn test_variable_declaration_and_usage() {
        let source = r#"
            let x = 42;
            let y = x + 10;
            y
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        let source = r#"
            let name = "Alice";
            let greeting = "Hello, " + name;
            greeting
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::String);
    }

    #[test]
    fn test_function_type_checking() {
        let source = r#"
            fn add(x: int, y: int) -> int {
                x + y
            }
            add(5, 3)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        let source = r#"
            fn multiply_vector(v: vec3, scalar: float) -> vec3 {
                v * scalar
            }
            multiply_vector([1.0, 2.0, 3.0], 2.5)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 3));
    }

    #[test]
    fn test_function_parameter_mismatch() {
        let source = r#"
            fn add(x: int, y: int) -> int {
                x + y
            }
            add(5.0, 3)
        "#;
        assert!(type_check_source(source).is_err());
        
        let source = r#"
            fn greet(name: string) -> string {
                "Hello, " + name
            }
            greet(42)
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_return_type_mismatch() {
        let source = r#"
            fn get_number() -> int {
                "not a number"
            }
        "#;
        assert!(type_check_source(source).is_err());
        
        let source = r#"
            fn get_boolean() -> bool {
                42
            }
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_arithmetic_operations() {
        // Integer arithmetic
        assert_eq!(type_check_expr("5 + 3").unwrap(), Type::Int);
        assert_eq!(type_check_expr("10 - 4").unwrap(), Type::Int);
        assert_eq!(type_check_expr("6 * 7").unwrap(), Type::Int);
        assert_eq!(type_check_expr("15 / 3").unwrap(), Type::Int);
        
        // Float arithmetic
        assert_eq!(type_check_expr("5.5 + 3.2").unwrap(), Type::Float);
        assert_eq!(type_check_expr("10.0 - 4.5").unwrap(), Type::Float);
        assert_eq!(type_check_expr("6.1 * 7.8").unwrap(), Type::Float);
        assert_eq!(type_check_expr("15.6 / 3.2").unwrap(), Type::Float);
        
        // Mixed arithmetic (should promote to float)
        assert_eq!(type_check_expr("5 + 3.0").unwrap(), Type::Float);
        assert_eq!(type_check_expr("10.5 - 4").unwrap(), Type::Float);
    }

    #[test]
    fn test_comparison_operations() {
        // Numeric comparisons
        assert_eq!(type_check_expr("5 > 3").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("10.5 <= 4.2").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("7 == 7").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("3.14 != 2.71").unwrap(), Type::Bool);
        
        // String comparisons
        assert_eq!(type_check_expr("\"apple\" == \"banana\"").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("\"hello\" != \"world\"").unwrap(), Type::Bool);
        
        // Boolean comparisons
        assert_eq!(type_check_expr("true == false").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("true != true").unwrap(), Type::Bool);
    }

    #[test]
    fn test_logical_operations() {
        assert_eq!(type_check_expr("true && false").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("true || false").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("!true").unwrap(), Type::Bool);
        
        // Complex logical expressions
        assert_eq!(type_check_expr("(5 > 3) && (2 < 4)").unwrap(), Type::Bool);
        assert_eq!(type_check_expr("!(false || true)").unwrap(), Type::Bool);
    }

    #[test]
    fn test_vector_operations() {
        // Vector arithmetic
        let source = r#"
            let v1 = [1.0, 2.0, 3.0];
            let v2 = [4.0, 5.0, 6.0];
            v1 + v2
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 3));
        
        // Vector scalar multiplication
        let source = r#"
            let v = [1.0, 2.0];
            v * 2.5
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 2));
        
        // Dot product
        let source = r#"
            let v1 = [1.0, 2.0, 3.0];
            let v2 = [4.0, 5.0, 6.0];
            dot(v1, v2)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
    }

    #[test]
    fn test_matrix_operations() {
        // Matrix addition
        let source = r#"
            let m1 = [[1.0, 2.0], [3.0, 4.0]];
            let m2 = [[5.0, 6.0], [7.0, 8.0]];
            m1 + m2
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Matrix(Box::new(Type::Float), 2, 2));
        
        // Matrix multiplication
        let source = r#"
            let m1 = [[1.0, 2.0], [3.0, 4.0]];
            let m2 = [[5.0, 6.0], [7.0, 8.0]];
            m1 * m2
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Matrix(Box::new(Type::Float), 2, 2));
        
        // Matrix vector multiplication
        let source = r#"
            let m = [[1.0, 2.0], [3.0, 4.0]];
            let v = [5.0, 6.0];
            m * v
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 2));
    }

    #[test]
    fn test_struct_type_checking() {
        let source = r#"
            struct Point {
                x: float,
                y: float
            }
            
            let p = Point { x: 1.0, y: 2.0 };
            p.x
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
        
        let source = r#"
            struct Person {
                name: string,
                age: int
            }
            
            fn create_person(n: string, a: int) -> Person {
                Person { name: n, age: a }
            }
            
            create_person("Alice", 30)
        "#;
        if let Ok(Type::Struct(name, _)) = type_check_source(source) {
            assert_eq!(name, "Person");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_enum_type_checking() {
        let source = r#"
            enum Color {
                Red,
                Green,
                Blue
            }
            
            let c = Color::Red;
            c
        "#;
        if let Ok(Type::Enum(name, _)) = type_check_source(source) {
            assert_eq!(name, "Color");
        } else {
            panic!("Expected enum type");
        }
        
        let source = r#"
            enum Option<T> {
                Some(T),
                None
            }
            
            let opt = Option::Some(42);
            opt
        "#;
        // Should be Option<int>
        assert!(matches!(type_check_source(source), Ok(Type::Enum(_, _))));
    }

    #[test]
    fn test_generic_type_inference() {
        let source = r#"
            fn identity<T>(x: T) -> T {
                x
            }
            
            identity(42)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        let source = r#"
            fn identity<T>(x: T) -> T {
                x
            }
            
            identity("hello")
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::String);
    }

    #[test]
    fn test_array_indexing() {
        let source = r#"
            let arr = [1, 2, 3, 4, 5];
            arr[2]
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        let source = r#"
            let matrix = [[1.0, 2.0], [3.0, 4.0]];
            matrix[1][0]
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
    }

    #[test]
    fn test_array_indexing_errors() {
        // Non-integer index
        let source = r#"
            let arr = [1, 2, 3];
            arr["not an index"]
        "#;
        assert!(type_check_source(source).is_err());
        
        // Indexing non-array
        let source = r#"
            let x = 42;
            x[0]
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_if_else_type_checking() {
        // Both branches return same type
        let source = r#"
            let x = 5;
            if x > 0 {
                "positive"
            } else {
                "non-positive"
            }
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::String);
        
        // Different branch types should error
        let source = r#"
            let x = 5;
            if x > 0 {
                42
            } else {
                "string"
            }
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_while_loop_type_checking() {
        let source = r#"
            let mut x = 0;
            while x < 10 {
                x = x + 1;
            }
            x
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Non-boolean condition should error
        let source = r#"
            let mut x = 0;
            while x {
                x = x + 1;
            }
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_for_loop_type_checking() {
        let source = r#"
            let arr = [1, 2, 3, 4, 5];
            let mut sum = 0;
            for x in arr {
                sum = sum + x;
            }
            sum
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Range-based for loop
        let source = r#"
            let mut sum = 0;
            for i in 0..10 {
                sum = sum + i;
            }
            sum
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
    }

    #[test]
    fn test_pattern_matching() {
        let source = r#"
            enum Result<T, E> {
                Ok(T),
                Err(E)
            }
            
            let result = Result::Ok(42);
            match result {
                Result::Ok(value) => value,
                Result::Err(_) => 0
            }
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Incomplete pattern matching should error
        let source = r#"
            enum Option<T> {
                Some(T),
                None
            }
            
            let opt = Option::Some(42);
            match opt {
                Option::Some(value) => value
                // Missing None case
            }
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_closure_type_checking() {
        let source = r#"
            let add = |x: int, y: int| -> int { x + y };
            add(5, 3)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Closure with type inference
        let source = r#"
            let numbers = [1, 2, 3, 4, 5];
            let doubled = map(numbers, |x| x * 2);
            doubled
        "#;
        // Should infer closure parameter type from context
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Int), 5));
    }

    #[test]
    fn test_method_calls() {
        let source = r#"
            struct Vector3 {
                x: float,
                y: float,
                z: float
            }
            
            impl Vector3 {
                fn magnitude(self) -> float {
                    sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
                }
            }
            
            let v = Vector3 { x: 3.0, y: 4.0, z: 0.0 };
            v.magnitude()
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
    }

    #[test]
    fn test_trait_implementation() {
        let source = r#"
            trait Display {
                fn to_string(self) -> string;
            }
            
            struct Point {
                x: int,
                y: int
            }
            
            impl Display for Point {
                fn to_string(self) -> string {
                    "(" + str(self.x) + ", " + str(self.y) + ")"
                }
            }
            
            let p = Point { x: 1, y: 2 };
            p.to_string()
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::String);
    }

    #[test]
    fn test_physics_types() {
        // RigidBody type
        let source = r#"
            let body = RigidBody::new(1.0, [0.0, 0.0, 0.0]);
            body.mass
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
        
        // Vector3 physics operations
        let source = r#"
            let force = [10.0, 0.0, -9.8];
            let acceleration = force / 2.0;  // mass = 2.0
            acceleration
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 3));
    }

    #[test]
    fn test_type_coercion() {
        // Int to Float coercion
        let source = r#"
            let x: float = 42;
            x
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
        
        // Vector element type coercion
        let source = r#"
            let v: vec3 = [1, 2.0, 3];  // Mixed int/float should coerce to float
            v
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Vector(Box::new(Type::Float), 3));
    }

    #[test]
    fn test_recursive_types() {
        let source = r#"
            struct Node {
                value: int,
                next: Option<Box<Node>>
            }
            
            let node = Node {
                value: 42,
                next: Option::Some(Box::new(Node { value: 24, next: Option::None }))
            };
            node.value
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
    }

    #[test]
    fn test_higher_order_functions() {
        let source = r#"
            fn apply_twice<T>(f: fn(T) -> T, x: T) -> T {
                f(f(x))
            }
            
            fn increment(x: int) -> int {
                x + 1
            }
            
            apply_twice(increment, 5)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
    }

    #[test]
    fn test_type_annotations() {
        // Explicit type annotations
        let source = r#"
            let x: int = 42;
            let y: float = 3.14;
            let name: string = "Alice";
            let flag: bool = true;
            x + y as int
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Type annotation mismatch should error
        let source = r#"
            let x: int = "not an integer";
        "#;
        assert!(type_check_source(source).is_err());
    }

    #[test]
    fn test_complex_expressions() {
        // Nested function calls with different types
        let source = r#"
            fn square(x: float) -> float { x * x }
            fn add(a: float, b: float) -> float { a + b }
            
            let result = add(square(3.0), square(4.0));
            result
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
        
        // Complex matrix/vector operations
        let source = r#"
            let transform = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            let point = [1.0, 2.0, 3.0];
            let transformed = transform * point;
            magnitude(transformed)
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Float);
    }

    #[test]
    fn test_error_messages() {
        // Test that type errors contain helpful information
        let result = type_check_source("let x: int = \"string\";");
        if let Err(error) = result {
            let error_msg = format!("{:?}", error);
            assert!(error_msg.contains("type") || error_msg.contains("mismatch"));
        } else {
            panic!("Expected type error");
        }
        
        // Undefined variable
        let result = type_check_source("undefined_variable + 5");
        assert!(result.is_err());
        
        // Wrong number of function arguments
        let result = type_check_source(r#"
            fn add(x: int, y: int) -> int { x + y }
            add(5)
        "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_scoping_rules() {
        // Variable shadowing
        let source = r#"
            let x = 5;
            {
                let x = "hello";
                x
            }
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::String);
        
        // Variable going out of scope
        let source = r#"
            let y = {
                let x = 42;
                x + 10
            };
            y
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
        
        // Using variable from outer scope
        let source = r#"
            let outer = 100;
            {
                let inner = 50;
                outer + inner
            }
        "#;
        assert_eq!(type_check_source(source).unwrap(), Type::Int);
    }
}
