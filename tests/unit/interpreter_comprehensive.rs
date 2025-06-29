use crate::interpreter::{Interpreter, Value, RuntimeError};
use crate::parser::Parser;
use crate::lexer::Lexer;
use crate::types::TypeChecker;
use super::test_utilities::*;
use std::collections::HashMap;

#[cfg(test)]
mod interpreter_comprehensive_tests {
    use super::*;

    // Helper function to interpret source code
    fn interpret_source(source: &str) -> Result<Value, RuntimeError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        // Type check first
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast).unwrap();
        
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&ast)
    }

    // Helper function to interpret expression
    fn interpret_expr(source: &str) -> Result<Value, RuntimeError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression().unwrap();
        
        let mut interpreter = Interpreter::new();
        interpreter.eval_expression(&expr)
    }

    #[test]
    fn test_literal_evaluation() {
        // Integer literals
        assert_eq!(interpret_expr("42").unwrap(), Value::Int(42));
        assert_eq!(interpret_expr("0").unwrap(), Value::Int(0));
        assert_eq!(interpret_expr("-123").unwrap(), Value::Int(-123));
        
        // Float literals
        assert_eq!(interpret_expr("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(interpret_expr("0.0").unwrap(), Value::Float(0.0));
        assert_eq!(interpret_expr("-2.5").unwrap(), Value::Float(-2.5));
        
        // Boolean literals
        assert_eq!(interpret_expr("true").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("false").unwrap(), Value::Bool(false));
        
        // String literals
        assert_eq!(interpret_expr("\"hello\"").unwrap(), Value::String("hello".to_string()));
        assert_eq!(interpret_expr("\"\"").unwrap(), Value::String("".to_string()));
    }

    #[test]
    fn test_arithmetic_operations() {
        // Integer arithmetic
        assert_eq!(interpret_expr("5 + 3").unwrap(), Value::Int(8));
        assert_eq!(interpret_expr("10 - 4").unwrap(), Value::Int(6));
        assert_eq!(interpret_expr("6 * 7").unwrap(), Value::Int(42));
        assert_eq!(interpret_expr("15 / 3").unwrap(), Value::Int(5));
        assert_eq!(interpret_expr("17 % 5").unwrap(), Value::Int(2));
        
        // Float arithmetic
        assert_eq!(interpret_expr("5.5 + 3.2").unwrap(), Value::Float(8.7));
        assert_eq!(interpret_expr("10.0 - 4.5").unwrap(), Value::Float(5.5));
        assert_eq!(interpret_expr("6.0 * 7.0").unwrap(), Value::Float(42.0));
        assert_eq!(interpret_expr("15.0 / 3.0").unwrap(), Value::Float(5.0));
        
        // Mixed arithmetic
        assert_eq!(interpret_expr("5 + 3.0").unwrap(), Value::Float(8.0));
        assert_eq!(interpret_expr("10.5 - 4").unwrap(), Value::Float(6.5));
        
        // Complex expressions
        assert_eq!(interpret_expr("2 + 3 * 4").unwrap(), Value::Int(14));
        assert_eq!(interpret_expr("(2 + 3) * 4").unwrap(), Value::Int(20));
    }

    #[test]
    fn test_comparison_operations() {
        // Integer comparisons
        assert_eq!(interpret_expr("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("3 > 5").unwrap(), Value::Bool(false));
        assert_eq!(interpret_expr("5 >= 5").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("3 < 5").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("5 <= 5").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("5 == 5").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("5 != 3").unwrap(), Value::Bool(true));
        
        // Float comparisons
        assert_eq!(interpret_expr("5.5 > 3.2").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("3.14 == 3.14").unwrap(), Value::Bool(true));
        
        // String comparisons
        assert_eq!(interpret_expr("\"apple\" == \"apple\"").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("\"apple\" != \"banana\"").unwrap(), Value::Bool(true));
        
        // Boolean comparisons
        assert_eq!(interpret_expr("true == true").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("true != false").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_logical_operations() {
        assert_eq!(interpret_expr("true && true").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("true && false").unwrap(), Value::Bool(false));
        assert_eq!(interpret_expr("false && true").unwrap(), Value::Bool(false));
        assert_eq!(interpret_expr("false && false").unwrap(), Value::Bool(false));
        
        assert_eq!(interpret_expr("true || true").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("true || false").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("false || true").unwrap(), Value::Bool(true));
        assert_eq!(interpret_expr("false || false").unwrap(), Value::Bool(false));
        
        assert_eq!(interpret_expr("!true").unwrap(), Value::Bool(false));
        assert_eq!(interpret_expr("!false").unwrap(), Value::Bool(true));
        
        // Short-circuit evaluation
        assert_eq!(interpret_expr("false && (1/0 > 0)").unwrap(), Value::Bool(false)); // Should not evaluate 1/0
        assert_eq!(interpret_expr("true || (1/0 > 0)").unwrap(), Value::Bool(true)); // Should not evaluate 1/0
    }

    #[test]
    fn test_variable_declarations() {
        let source = r#"
            let x = 42;
            x
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(42));
        
        let source = r#"
            let name = "Alice";
            let age = 30;
            name
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("Alice".to_string()));
        
        // Mutable variables
        let source = r#"
            let mut x = 10;
            x = x + 5;
            x
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(15));
    }

    #[test]
    fn test_vector_operations() {
        // Vector creation
        let result = interpret_expr("[1, 2, 3]").unwrap();
        if let Value::Vector(vec) = result {
            assert_eq!(vec, vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        } else {
            panic!("Expected vector value");
        }
        
        // Vector arithmetic
        let source = r#"
            let v1 = [1.0, 2.0, 3.0];
            let v2 = [4.0, 5.0, 6.0];
            v1 + v2
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Vector(vec) = result {
            assert_eq!(vec, vec![Value::Float(5.0), Value::Float(7.0), Value::Float(9.0)]);
        } else {
            panic!("Expected vector value");
        }
        
        // Vector scalar multiplication
        let source = r#"
            let v = [2.0, 3.0];
            v * 2.5
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Vector(vec) = result {
            assert_eq!(vec, vec![Value::Float(5.0), Value::Float(7.5)]);
        } else {
            panic!("Expected vector value");
        }
        
        // Vector indexing
        let source = r#"
            let v = [10, 20, 30, 40];
            v[2]
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(30));
    }

    #[test]
    fn test_matrix_operations() {
        // Matrix creation
        let result = interpret_expr("[[1, 2], [3, 4]]").unwrap();
        if let Value::Matrix(matrix) = result {
            assert_eq!(matrix.len(), 2);
            assert_eq!(matrix[0], vec![Value::Int(1), Value::Int(2)]);
            assert_eq!(matrix[1], vec![Value::Int(3), Value::Int(4)]);
        } else {
            panic!("Expected matrix value");
        }
        
        // Matrix addition
        let source = r#"
            let m1 = [[1.0, 2.0], [3.0, 4.0]];
            let m2 = [[5.0, 6.0], [7.0, 8.0]];
            m1 + m2
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Matrix(matrix) = result {
            assert_eq!(matrix[0], vec![Value::Float(6.0), Value::Float(8.0)]);
            assert_eq!(matrix[1], vec![Value::Float(10.0), Value::Float(12.0)]);
        } else {
            panic!("Expected matrix value");
        }
        
        // Matrix indexing
        let source = r#"
            let m = [[1, 2, 3], [4, 5, 6]];
            m[1][2]
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(6));
    }

    #[test]
    fn test_function_definitions_and_calls() {
        let source = r#"
            fn add(x: int, y: int) -> int {
                x + y
            }
            add(5, 3)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(8));
        
        let source = r#"
            fn factorial(n: int) -> int {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
            factorial(5)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(120));
        
        // Function with vector parameters
        let source = r#"
            fn dot_product(v1: vec3, v2: vec3) -> float {
                v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
            }
            dot_product([1.0, 2.0, 3.0], [4.0, 5.0, 6.0])
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Float(32.0));
    }

    #[test]
    fn test_if_else_statements() {
        let source = r#"
            let x = 10;
            if x > 5 {
                "greater"
            } else {
                "less or equal"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("greater".to_string()));
        
        let source = r#"
            let x = 3;
            if x > 5 {
                "greater"
            } else {
                "less or equal"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("less or equal".to_string()));
        
        // Nested if-else
        let source = r#"
            let x = 0;
            if x > 0 {
                "positive"
            } else if x < 0 {
                "negative"
            } else {
                "zero"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("zero".to_string()));
    }

    #[test]
    fn test_while_loops() {
        let source = r#"
            let mut sum = 0;
            let mut i = 1;
            while i <= 5 {
                sum = sum + i;
                i = i + 1;
            }
            sum
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(15)); // 1+2+3+4+5
        
        // Factorial using while loop
        let source = r#"
            let mut result = 1;
            let mut n = 5;
            while n > 0 {
                result = result * n;
                n = n - 1;
            }
            result
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(120));
    }

    #[test]
    fn test_for_loops() {
        // For loop with array
        let source = r#"
            let numbers = [1, 2, 3, 4, 5];
            let mut sum = 0;
            for x in numbers {
                sum = sum + x;
            }
            sum
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(15));
        
        // For loop with range
        let source = r#"
            let mut product = 1;
            for i in 1..=5 {
                product = product * i;
            }
            product
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(120));
    }

    #[test]
    fn test_struct_creation_and_access() {
        let source = r#"
            struct Point {
                x: float,
                y: float
            }
            
            let p = Point { x: 3.0, y: 4.0 };
            p.x
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Float(3.0));
        
        let source = r#"
            struct Person {
                name: string,
                age: int
            }
            
            let person = Person { name: "Alice", age: 30 };
            person.name
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("Alice".to_string()));
        
        // Nested struct access
        let source = r#"
            struct Address {
                street: string,
                city: string
            }
            
            struct Person {
                name: string,
                address: Address
            }
            
            let person = Person {
                name: "Bob",
                address: Address { street: "Main St", city: "Anytown" }
            };
            person.address.city
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("Anytown".to_string()));
    }

    #[test]
    fn test_enum_creation_and_matching() {
        let source = r#"
            enum Color {
                Red,
                Green,
                Blue
            }
            
            let c = Color::Red;
            match c {
                Color::Red => "red",
                Color::Green => "green",
                Color::Blue => "blue"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("red".to_string()));
        
        // Enum with data
        let source = r#"
            enum Option<T> {
                Some(T),
                None
            }
            
            let opt = Option::Some(42);
            match opt {
                Option::Some(value) => value,
                Option::None => 0
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_pattern_matching() {
        // Simple pattern matching
        let source = r#"
            let x = 5;
            match x {
                1 => "one",
                2 => "two",
                5 => "five",
                _ => "other"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("five".to_string()));
        
        // Pattern matching with guards
        let source = r#"
            let x = 15;
            match x {
                n if n < 10 => "small",
                n if n < 20 => "medium",
                _ => "large"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("medium".to_string()));
        
        // Destructuring patterns
        let source = r#"
            let point = (3, 4);
            match point {
                (0, 0) => "origin",
                (x, 0) => "on x-axis",
                (0, y) => "on y-axis",
                (x, y) => "general point"
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::String("general point".to_string()));
    }

    #[test]
    fn test_closures() {
        // Simple closure
        let source = r#"
            let add_one = |x: int| -> int { x + 1 };
            add_one(5)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(6));
        
        // Closure capturing environment
        let source = r#"
            let base = 10;
            let add_base = |x: int| -> int { x + base };
            add_base(5)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(15));
        
        // Higher-order function with closure
        let source = r#"
            fn apply_twice(f: fn(int) -> int, x: int) -> int {
                f(f(x))
            }
            
            let double = |x: int| -> int { x * 2 };
            apply_twice(double, 3)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(12)); // ((3*2)*2)
    }

    #[test]
    fn test_array_methods() {
        // Map function
        let source = r#"
            let numbers = [1, 2, 3, 4];
            let doubled = map(numbers, |x| x * 2);
            doubled
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Vector(vec) = result {
            assert_eq!(vec, vec![Value::Int(2), Value::Int(4), Value::Int(6), Value::Int(8)]);
        } else {
            panic!("Expected vector value");
        }
        
        // Filter function
        let source = r#"
            let numbers = [1, 2, 3, 4, 5, 6];
            let evens = filter(numbers, |x| x % 2 == 0);
            evens
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Vector(vec) = result {
            assert_eq!(vec, vec![Value::Int(2), Value::Int(4), Value::Int(6)]);
        } else {
            panic!("Expected vector value");
        }
        
        // Reduce function
        let source = r#"
            let numbers = [1, 2, 3, 4, 5];
            let sum = reduce(numbers, 0, |acc, x| acc + x);
            sum
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(15));
    }

    #[test]
    fn test_string_operations() {
        // String concatenation
        assert_eq!(interpret_expr("\"hello\" + \" world\"").unwrap(), 
                   Value::String("hello world".to_string()));
        
        // String interpolation
        let source = r#"
            let name = "Alice";
            let age = 30;
            "Name: " + name + ", Age: " + str(age)
        "#;
        assert_eq!(interpret_source(source).unwrap(), 
                   Value::String("Name: Alice, Age: 30".to_string()));
        
        // String methods
        let source = r#"
            let text = "Hello, World!";
            len(text)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(13));
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
                
                fn normalize(self) -> Vector3 {
                    let mag = self.magnitude();
                    Vector3 { x: self.x / mag, y: self.y / mag, z: self.z / mag }
                }
            }
            
            let v = Vector3 { x: 3.0, y: 4.0, z: 0.0 };
            v.magnitude()
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Float(5.0));
    }

    #[test]
    fn test_physics_simulation() {
        // Basic rigid body physics
        let source = r#"
            let mut body = RigidBody::new(1.0, [0.0, 0.0, 0.0]);
            body.apply_force([0.0, -9.8, 0.0]); // Gravity
            body.update(1.0); // 1 second time step
            body.position[1] // Y position
        "#;
        // After 1 second with gravity, should fall 4.9 meters (1/2 * g * t^2)
        let result = interpret_source(source).unwrap();
        if let Value::Float(y) = result {
            assert!((y + 4.9).abs() < 0.1); // Allow some floating point error
        } else {
            panic!("Expected float value");
        }
        
        // Collision detection
        let source = r#"
            let body1 = RigidBody::new(1.0, [0.0, 0.0, 0.0]);
            let body2 = RigidBody::new(1.0, [1.0, 0.0, 0.0]);
            let collision = detect_collision(body1, body2);
            collision.is_some()
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Bool(false)); // No collision at distance 1.0
    }

    #[test]
    fn test_error_handling() {
        // Division by zero
        let result = interpret_expr("10 / 0");
        assert!(result.is_err());
        
        // Array index out of bounds
        let result = interpret_source(r#"
            let arr = [1, 2, 3];
            arr[5]
        "#);
        assert!(result.is_err());
        
        // Undefined variable
        let result = interpret_source("undefined_variable");
        assert!(result.is_err());
        
        // Type mismatch in operation
        let result = interpret_expr("\"string\" + 42");
        // This should be caught at type checking, but if it reaches runtime...
        // The behavior depends on implementation
    }

    #[test]
    fn test_scoping() {
        // Block scoping
        let source = r#"
            let x = 10;
            {
                let x = 20;
                let y = x + 5;
                y
            }
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(25));
        
        // Variable shadowing
        let source = r#"
            let x = 5;
            let x = x * 2;
            x
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(10));
        
        // Function parameter shadowing
        let source = r#"
            let x = 100;
            fn test(x: int) -> int {
                x * 2
            }
            test(5)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(10));
    }

    #[test]
    fn test_recursion() {
        // Fibonacci sequence
        let source = r#"
            fn fib(n: int) -> int {
                if n <= 1 {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }
            fib(8)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(21));
        
        // Mutual recursion
        let source = r#"
            fn is_even(n: int) -> bool {
                if n == 0 {
                    true
                } else {
                    is_odd(n - 1)
                }
            }
            
            fn is_odd(n: int) -> bool {
                if n == 0 {
                    false
                } else {
                    is_even(n - 1)
                }
            }
            
            is_even(8)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_complex_data_structures() {
        // Binary tree
        let source = r#"
            struct TreeNode {
                value: int,
                left: Option<Box<TreeNode>>,
                right: Option<Box<TreeNode>>
            }
            
            fn tree_sum(node: Option<Box<TreeNode>>) -> int {
                match node {
                    Option::Some(n) => n.value + tree_sum(n.left) + tree_sum(n.right),
                    Option::None => 0
                }
            }
            
            let tree = TreeNode {
                value: 10,
                left: Option::Some(Box::new(TreeNode { value: 5, left: Option::None, right: Option::None })),
                right: Option::Some(Box::new(TreeNode { value: 15, left: Option::None, right: Option::None }))
            };
            
            tree_sum(Option::Some(Box::new(tree)))
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(30)); // 10 + 5 + 15
    }

    #[test]
    fn test_memory_management() {
        // Large data structures should be handled properly
        let source = r#"
            let mut large_array = [];
            for i in 0..1000 {
                large_array = push(large_array, i);
            }
            len(large_array)
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Int(1000));
        
        // Nested function calls with temporary values
        let source = r#"
            fn create_vector(size: int) -> vec3 {
                [size as float, (size * 2) as float, (size * 3) as float]
            }
            
            fn process_vectors() -> float {
                let v1 = create_vector(2);
                let v2 = create_vector(3);
                magnitude(v1 + v2)
            }
            
            process_vectors()
        "#;
        // v1 = [2.0, 4.0, 6.0], v2 = [3.0, 6.0, 9.0], sum = [5.0, 10.0, 15.0]
        // magnitude = sqrt(25 + 100 + 225) = sqrt(350) ≈ 18.71
        let result = interpret_source(source).unwrap();
        if let Value::Float(magnitude) = result {
            assert!((magnitude - 18.708).abs() < 0.01);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_performance_characteristics() {
        // Tail recursion (should not cause stack overflow for reasonable sizes)
        let source = r#"
            fn factorial_tail(n: int, acc: int) -> int {
                if n <= 1 {
                    acc
                } else {
                    factorial_tail(n - 1, acc * n)
                }
            }
            
            factorial_tail(20, 1)
        "#;
        let result = interpret_source(source).unwrap();
        if let Value::Int(fact) = result {
            assert_eq!(fact, 2432902008176640000); // 20!
        } else {
            panic!("Expected int value");
        }
        
        // Large matrix operations
        let source = r#"
            fn create_identity_matrix(size: int) -> mat3 {
                let mut matrix = [];
                for i in 0..size {
                    let mut row = [];
                    for j in 0..size {
                        if i == j {
                            row = push(row, 1.0);
                        } else {
                            row = push(row, 0.0);
                        }
                    }
                    matrix = push(matrix, row);
                }
                matrix
            }
            
            let identity = create_identity_matrix(3);
            identity[1][1]
        "#;
        assert_eq!(interpret_source(source).unwrap(), Value::Float(1.0));
    }
}
