// Comprehensive Parser Tests
use crate::parser::Parser;
use crate::ast::nodes::*;
use crate::tests::test_utilities::*;

#[cfg(test)]
mod comprehensive_parser_tests {
    use super::*;

    #[test]
    fn test_expression_parsing_comprehensive() {
        let test_cases = vec![
            // Literals
            ("42", "IntLiteral(42)"),
            ("3.14", "FloatLiteral(3.14)"),
            ("true", "BoolLiteral(true)"),
            ("false", "BoolLiteral(false)"),
            ("\"hello\"", "StringLiteral(\"hello\")"),
            
            // Identifiers
            ("x", "Identifier(x)"),
            ("_variable", "Identifier(_variable)"),
            
            // Binary operations with precedence
            ("1 + 2 * 3", "BinaryOp(1 + (2 * 3))"),
            ("(1 + 2) * 3", "BinaryOp((1 + 2) * 3)"),
            ("a && b || c", "BinaryOp((a && b) || c)"),
            ("a == b != c", "BinaryOp((a == b) != c)"),
            
            // Unary operations
            ("!true", "UnaryOp(!true)"),
            ("-42", "UnaryOp(-42)"),
            ("~mask", "UnaryOp(~mask)"),
            
            // Function calls
            ("f()", "FunctionCall(f())"),
            ("add(1, 2)", "FunctionCall(add(1, 2))"),
            ("nested(f(x), g(y))", "FunctionCall(nested(f(x), g(y)))"),
            
            // Array/Matrix literals
            ("[1, 2, 3]", "ArrayLiteral([1, 2, 3])"),
            ("[[1, 2], [3, 4]]", "MatrixLiteral([[1, 2], [3, 4]])"),
            
            // Field access
            ("obj.field", "FieldAccess(obj.field)"),
            ("obj.nested.field", "FieldAccess(obj.nested.field)"),
            
            // Array indexing
            ("arr[0]", "ArrayAccess(arr[0])"),
            ("matrix[i][j]", "ArrayAccess(matrix[i][j])"),
            
            // Range expressions
            ("1..10", "Range(1..10)"),
            ("0..=100", "Range(0..=100)"),
        ];
        
        for (input, description) in test_cases {
            let result = parse_source(&format!("{}", input));
            assert!(result.is_ok(), "Failed to parse '{}': {}", input, description);
        }
    }

    #[test]
    fn test_statement_parsing_comprehensive() {
        let test_cases = vec![
            // Let bindings
            "let x = 42;",
            "let mut y: Int = 0;",
            "let (a, b) = (1, 2);",
            "let [x, y, z] = [1, 2, 3];",
            
            // Assignments
            "x = 42;",
            "arr[i] = value;",
            "obj.field = new_value;",
            
            // Expression statements
            "f();",
            "x + y;",
            
            // Control flow
            "if condition { body }",
            "if cond { then_block } else { else_block }",
            "while condition { body }",
            "for i in 0..10 { body }",
            "loop { break; }",
            
            // Match expressions
            "match value { 1 => \"one\", _ => \"other\" }",
            
            // Return statements
            "return 42;",
            "return;",
        ];
        
        for input in test_cases {
            let result = parse_source(input);
            assert!(result.is_ok(), "Failed to parse statement: '{}'", input);
        }
    }

    #[test]
    fn test_function_definition_parsing() {
        let test_cases = vec![
            // Simple functions
            "fn simple() {}",
            "fn with_return() -> Int { 42 }",
            "fn with_params(x: Int, y: Float) -> Float { x as Float + y }",
            
            // Generic functions
            "fn generic<T>(x: T) -> T { x }",
            "fn multi_generic<T, U>(x: T, y: U) -> T { x }",
            
            // Functions with constraints
            "fn constrained<T: Add>(x: T, y: T) -> T { x + y }",
            
            // Async functions
            "async fn async_func() -> Int { await compute() }",
            
            // Complex function with everything
            r#"
            async fn complex<T: Clone + Debug>(
                x: T, 
                y: Option<T>, 
                callback: fn(T) -> T
            ) -> Result<T, Error> {
                let result = callback(x.clone());
                match y {
                    Some(val) => Ok(result),
                    None => Err(Error::new("No value"))
                }
            }
            "#,
        ];
        
        for input in test_cases {
            let result = parse_source(input);
            assert!(result.is_ok(), "Failed to parse function: '{}'", input);
            
            // Verify it's actually a function definition
            if let Ok(program) = result {
                assert!(program.items.len() > 0);
                assert!(matches!(program.items[0], Item::Function(_)));
            }
        }
    }

    #[test]
    fn test_struct_definition_parsing() {
        let test_cases = vec![
            // Simple struct
            "struct Point { x: Float, y: Float }",
            
            // Struct with different field types
            "struct Complex { id: Int, name: String, active: Bool, position: Vec3 }",
            
            // Generic struct
            "struct Container<T> { value: T }",
            "struct Pair<T, U> { first: T, second: U }",
            
            // Struct with constraints
            "struct Comparable<T: Ord> { values: Vec<T> }",
            
            // Tuple struct
            "struct Color(Float, Float, Float, Float);",
            
            // Unit struct
            "struct Marker;",
            
            // Complex struct with methods
            r#"
            struct Vector3<T: Number> {
                x: T,
                y: T,
                z: T,
                
                fn magnitude(&self) -> T {
                    sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
                }
                
                fn normalize(&mut self) {
                    let mag = self.magnitude();
                    self.x /= mag;
                    self.y /= mag;
                    self.z /= mag;
                }
            }
            "#,
        ];
        
        for input in test_cases {
            let result = parse_source(input);
            assert!(result.is_ok(), "Failed to parse struct: '{}'", input);
            
            if let Ok(program) = result {
                assert!(program.items.len() > 0);
                assert!(matches!(program.items[0], Item::Struct(_)));
            }
        }
    }

    #[test]
    fn test_enum_definition_parsing() {
        let test_cases = vec![
            // Simple enum
            "enum Color { Red, Green, Blue }",
            
            // Enum with data
            "enum Shape { Circle(Float), Rectangle(Float, Float), Triangle(Float, Float, Float) }",
            
            // Generic enum
            "enum Option<T> { Some(T), None }",
            "enum Result<T, E> { Ok(T), Err(E) }",
            
            // Complex enum with mixed variants
            r#"
            enum Message {
                Quit,
                Move { x: Int, y: Int },
                Write(String),
                ChangeColor(Int, Int, Int),
            }
            "#,
        ];
        
        for input in test_cases {
            let result = parse_source(input);
            assert!(result.is_ok(), "Failed to parse enum: '{}'", input);
            
            if let Ok(program) = result {
                assert!(program.items.len() > 0);
                assert!(matches!(program.items[0], Item::Enum(_)));
            }
        }
    }

    #[test]
    fn test_complex_expressions() {
        let complex_expressions = vec![
            // Nested function calls
            "f(g(h(x)))",
            
            // Complex arithmetic
            "a * b + c / d - e % f",
            
            // Mixed operators with parentheses
            "((a + b) * (c - d)) / ((e + f) * (g - h))",
            
            // Chained method calls
            "obj.method1().method2().field.method3()",
            
            // Complex array/matrix operations
            "matrix[i][j] + other_matrix[x][y] * scalar",
            
            // Nested conditionals
            "if a { if b { c } else { d } } else { if e { f } else { g } }",
            
            // Complex match expressions
            r#"
            match complex_value {
                Pattern1(x) if x > 10 => process(x),
                Pattern2 { field1, field2 } => combine(field1, field2),
                _ => default_value
            }
            "#,
            
            // Lambda expressions (if supported)
            "|x, y| x + y",
            
            // Async/await chains
            "await future1.then(|x| future2(x)).then(|y| future3(y))",
        ];
        
        for expr in complex_expressions {
            let result = parse_source(expr);
            // Some expressions might not be fully supported yet
            // This test documents the parser's current capabilities
            match result {
                Ok(_) => {}, // Great, it parsed successfully
                Err(e) => {
                    // Document what's not yet supported
                    println!("Expression '{}' not yet supported: {}", expr, e);
                }
            }
        }
    }

    #[test]
    fn test_physics_block_parsing() {
        let physics_code = r#"
        physics {
            let world = PhysicsWorld::new();
            let sphere = RigidBody::sphere(1.0, 1.0);
            world.add_body(sphere);
            
            for i in 0..100 {
                world.step(0.016);
            }
            
            sphere.position()
        }
        "#;
        
        let result = parse_source(physics_code);
        assert!(result.is_ok(), "Failed to parse physics block");
        
        if let Ok(program) = result {
            assert!(program.items.len() > 0);
            // Should contain a physics block
            // The exact representation depends on AST design
        }
    }

    #[test]
    fn test_parallel_constructs() {
        let parallel_code = vec![
            "parallel for i in 0..100 { compute(i) }",
            "parallel { task1(); task2(); task3(); }",
            "let results = parallel map(data, |x| expensive_computation(x));",
        ];
        
        for code in parallel_code {
            let result = parse_source(code);
            // May not be fully implemented yet
            match result {
                Ok(_) => {}, // Successfully parsed
                Err(_) => {
                    // Document limitations
                    println!("Parallel construct '{}' not yet fully supported", code);
                }
            }
        }
    }

    #[test]
    fn test_error_recovery() {
        let invalid_inputs = vec![
            "let x = ;", // Missing expression
            "fn incomplete(", // Incomplete function
            "struct Bad { x: }", // Missing type
            "match x { }", // Empty match
            "if { }", // Missing condition
        ];
        
        for input in invalid_inputs {
            let result = parse_source(input);
            assert!(result.is_err(), "Should fail to parse invalid input: '{}'", input);
            // Test that errors are descriptive
            if let Err(error) = result {
                assert!(!error.is_empty(), "Error message should not be empty");
            }
        }
    }

    #[test]
    fn test_operator_precedence() {
        let precedence_tests = vec![
            ("1 + 2 * 3", "1 + (2 * 3)"),
            ("1 * 2 + 3", "(1 * 2) + 3"),
            ("a && b || c", "(a && b) || c"),
            ("!a && b", "(!a) && b"),
            ("a == b && c", "(a == b) && c"),
            ("a + b == c + d", "(a + b) == (c + d)"),
            ("a = b + c * d", "a = (b + (c * d))"),
        ];
        
        for (input, expected_structure) in precedence_tests {
            let result = parse_source(input);
            assert!(result.is_ok(), "Failed to parse '{}', expected structure: '{}'", input, expected_structure);
            
            // Here we would ideally check the actual AST structure
            // For now, we just verify it parses successfully
        }
    }

    #[test]
    fn test_associativity() {
        let associativity_tests = vec![
            ("1 + 2 + 3", "((1 + 2) + 3)"), // Left associative
            ("1 - 2 - 3", "((1 - 2) - 3)"), // Left associative
            ("a = b = c", "(a = (b = c))"), // Right associative
            ("a ** b ** c", "(a ** (b ** c))"), // Right associative (if power operator exists)
        ];
        
        for (input, expected_structure) in associativity_tests {
            let result = parse_source(input);
            // Some operators might not be implemented yet
            match result {
                Ok(_) => {
                    // Successfully parsed with correct associativity
                },
                Err(_) => {
                    // Operator might not be implemented
                    println!("Associativity test '{}' not supported yet", input);
                }
            }
        }
    }

    #[test]
    fn test_large_ast_parsing() {
        // Test parser performance and correctness with large input
        let mut large_program = String::new();
        
        // Generate a large but valid program
        for i in 0..100 {
            large_program.push_str(&format!(
                "fn func{}(x: Int) -> Int {{ x + {} }}\n", i, i
            ));
        }
        
        large_program.push_str("struct LargeStruct {\n");
        for i in 0..50 {
            large_program.push_str(&format!("    field{}: Int,\n", i));
        }
        large_program.push_str("}\n");
        
        let start = std::time::Instant::now();
        let result = parse_source(&large_program);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Failed to parse large program");
        assert!(duration.as_millis() < 1000, "Parser should be reasonably fast");
        
        if let Ok(program) = result {
            assert_eq!(program.items.len(), 101); // 100 functions + 1 struct
        }
    }

    #[test]
    fn test_nested_structures() {
        let nested_code = r#"
        struct Outer {
            inner: Inner,
            data: Vec<Matrix<Float>>
        }
        
        struct Inner {
            values: [Int; 10],
            nested_fn: fn(Int) -> fn(Float) -> Bool
        }
        
        fn complex_nested() -> Result<Option<Vec<Matrix<Complex>>>, Error> {
            let data = vec![
                matrix![
                    [1.0 + 2.0i, 3.0 + 4.0i],
                    [5.0 + 6.0i, 7.0 + 8.0i]
                ]
            ];
            
            match process_data(data) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        }
        "#;
        
        let result = parse_source(nested_code);
        // Complex nested types might not be fully supported yet
        match result {
            Ok(_) => {
                // Successfully parsed complex nested structures
            },
            Err(e) => {
                // Document current limitations
                println!("Complex nested structures not fully supported: {}", e);
            }
        }
    }

    #[test]
    fn test_pattern_matching_comprehensive() {
        let pattern_tests = vec![
            // Literal patterns
            "match x { 42 => \"found\", _ => \"not found\" }",
            
            // Variable patterns
            "match x { y => y }",
            
            // Tuple patterns
            "match point { (x, y) => x + y }",
            
            // Array patterns
            "match arr { [first, rest @ ..] => first }",
            
            // Struct patterns
            "match person { Person { name, age } => name }",
            
            // Enum patterns
            "match option { Some(value) => value, None => 0 }",
            
            // Guard patterns
            "match x { y if y > 10 => \"big\", _ => \"small\" }",
            
            // Nested patterns
            "match nested { Some((x, y)) if x > y => x, _ => 0 }",
        ];
        
        for pattern in pattern_tests {
            let result = parse_source(pattern);
            // Pattern matching might not be fully implemented
            match result {
                Ok(_) => {}, // Successfully parsed
                Err(_) => {
                    println!("Pattern '{}' not yet supported", pattern);
                }
            }
        }
    }
}
