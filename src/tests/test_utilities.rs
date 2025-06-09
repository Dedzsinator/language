// Test utilities and helpers for comprehensive testing
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use crate::types::checker::TypeChecker;
use crate::eval::interpreter::Interpreter;
use crate::ast::nodes::*;

/// Helper to create a complete pipeline from source to interpreted result
pub fn full_pipeline_test(source: &str) -> Result<String, String> {
    // Lexing & Parsing (combined in new API)
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|e| format!("Parser creation error: {:?}", e))?;
    let ast = parser.parse_program().map_err(|e| format!("Parse error: {:?}", e))?;
    
    // Type checking
    let mut type_checker = TypeChecker::new();
    let _typed_ast = type_checker.check_program(&ast)
        .map_err(|e| format!("Type check error: {:?}", e))?;
    
    // Interpretation
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast)
        .map_err(|e| format!("Runtime error: {:?}", e))?;
    
    Ok(format!("{:?}", result))
}

/// Create test tokens from source
pub fn tokenize_source(source: &str) -> Result<Vec<Token>, String> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Tokenize error: {:?}", e))?;
    Ok(tokens.into_iter().map(|t| t.token).collect())
}

/// Create test AST from source
pub fn parse_source(source: &str) -> Result<Program, String> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|e| format!("{:?}", e))?;
    parser.parse_program().map_err(|e| format!("{:?}", e))
}

/// Helper to create test expressions
pub fn make_int_literal(value: i64) -> Expression {
    Expression::IntLiteral(value, Span::new(0, 0, 0, 0))
}

pub fn make_float_literal(value: f64) -> Expression {
    Expression::FloatLiteral(value, Span::new(0, 0, 0, 0))
}

pub fn make_string_literal(value: &str) -> Expression {
    Expression::StringLiteral(value.to_string(), Span::new(0, 0, 0, 0))
}

pub fn make_bool_literal(value: bool) -> Expression {
    Expression::BoolLiteral(value, Span::new(0, 0, 0, 0))
}

pub fn make_identifier(name: &str) -> Expression {
    Expression::Identifier(name.to_string(), Span::new(0, 0, 0, 0))
}

pub fn make_binary_op(left: Expression, op: BinaryOperator, right: Expression) -> Expression {
    Expression::BinaryOp {
        left: Box::new(left),
        operator: op,
        right: Box::new(right),
        span: Span::new(0, 0, 0, 0),
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn complex_program() -> &'static str {
        r#"
        struct Vec3 {
            x: Float,
            y: Float,
            z: Float
        }
        
        fn magnitude(v: Vec3) -> Float {
            sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
        }
        
        fn normalize(v: Vec3) -> Vec3 {
            let mag = magnitude(v);
            Vec3 { x: v.x / mag, y: v.y / mag, z: v.z / mag }
        }
        
        let vector = Vec3 { x: 3.0, y: 4.0, z: 0.0 };
        let normalized = normalize(vector);
        normalized.x
        "#
    }
    
    pub fn matrix_operations() -> &'static str {
        r#"
        let a = [[1.0, 2.0], [3.0, 4.0]];
        let b = [[5.0, 6.0], [7.0, 8.0]];
        let c = a * b;
        c[0][0]
        "#
    }
    
    pub fn physics_simulation() -> &'static str {
        r#"
        physics {
            let world = PhysicsWorld::new();
            let sphere = RigidBody::sphere(1.0, 1.0);
            world.add_body(sphere);
            world.step();
            sphere.position()
        }
        "#
    }
    
    pub fn control_flow() -> &'static str {
        r#"
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        factorial(5)
        "#
    }
    
    pub fn pattern_matching() -> &'static str {
        r#"
        enum Shape {
            Circle(Float),
            Rectangle(Float, Float),
            Triangle(Float, Float, Float)
        }
        
        fn area(shape: Shape) -> Float {
            match shape {
                Circle(r) => 3.14159 * r * r,
                Rectangle(w, h) => w * h,
                Triangle(a, b, c) => {
                    let s = (a + b + c) / 2.0;
                    sqrt(s * (s - a) * (s - b) * (s - c))
                }
            }
        }
        
        area(Circle(5.0))
        "#
    }
    
    pub fn async_operations() -> &'static str {
        r#"
        async fn compute_heavy(n: Int) -> Int {
            await delay(100);
            n * n
        }
        
        async fn main() -> Int {
            let future1 = compute_heavy(10);
            let future2 = compute_heavy(20);
            let result1 = await future1;
            let result2 = await future2;
            result1 + result2
        }
        
        await main()
        "#
    }
    
    pub fn parallel_computation() -> &'static str {
        r#"
        fn process_array(arr: [Int]) -> [Int] {
            parallel for x in arr {
                x * x
            }
        }
        
        let data = [1, 2, 3, 4, 5];
        let processed = process_array(data);
        processed[2]
        "#
    }
}

/// Performance test utilities
pub struct PerformanceTestHelper;

impl PerformanceTestHelper {
    pub fn large_matrix(size: usize) -> String {
        let mut result = String::from("[[");
        for i in 0..size {
            if i > 0 { result.push_str("], ["); }
            for j in 0..size {
                if j > 0 { result.push_str(", "); }
                result.push_str(&format!("{}.0", i * size + j + 1));
            }
        }
        result.push_str("]]");
        result
    }
    
    pub fn deep_recursion(depth: usize) -> String {
        let mut result = String::from("fn deep(n: Int) -> Int { if n <= 0 { 1 } else { deep(n - 1) } }\ndeep(");
        result.push_str(&depth.to_string());
        result.push(')');
        result
    }
    
    pub fn complex_expression(depth: usize) -> String {
        if depth == 0 {
            "1".to_string()
        } else {
            format!("({} + {})", Self::complex_expression(depth - 1), Self::complex_expression(depth - 1))
        }
    }
}
