use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::TypeChecker;
use crate::interpreter::Interpreter;
use crate::physics::PhysicsWorld;
use super::test_utilities::*;

/// Integration tests that verify the complete pipeline from source code to execution
#[cfg(test)]
mod integration_tests {
    use super::*;

    // Helper function to run complete pipeline
    fn run_complete_pipeline(source: &str) -> Result<crate::interpreter::Value, Box<dyn std::error::Error>> {
        // Lexing
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        
        // Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Type checking
        let mut type_checker = TypeChecker::new();
        let _type_result = type_checker.check(&ast)?;
        
        // Interpretation
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(&ast)?;
        
        Ok(result)
    }

    #[test]
    fn test_simple_arithmetic_pipeline() {
        let source = r#"
            let x = 5;
            let y = 3;
            x + y * 2
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(11));
    }

    #[test]
    fn test_function_definition_and_call_pipeline() {
        let source = r#"
            fn multiply(a: int, b: int) -> int {
                a * b
            }
            
            fn add(x: int, y: int) -> int {
                x + y
            }
            
            let result = add(multiply(3, 4), 5);
            result
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(17)); // (3*4) + 5
    }

    #[test]
    fn test_vector_mathematics_pipeline() {
        let source = r#"
            let v1 = [1.0, 2.0, 3.0];
            let v2 = [4.0, 5.0, 6.0];
            
            fn dot_product(a: vec3, b: vec3) -> float {
                a.x * b.x + a.y * b.y + a.z * b.z
            }
            
            fn vector_add(a: vec3, b: vec3) -> vec3 {
                [a.x + b.x, a.y + b.y, a.z + b.z]
            }
            
            let sum = vector_add(v1, v2);
            let dot = dot_product(v1, v2);
            
            dot
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(32.0)); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_matrix_operations_pipeline() {
        let source = r#"
            let identity = [[1.0, 0.0], [0.0, 1.0]];
            let matrix = [[2.0, 3.0], [4.0, 5.0]];
            
            fn matrix_multiply(a: mat2, b: mat2) -> mat2 {
                [
                    [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
                    [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]]
                ]
            }
            
            let result = matrix_multiply(matrix, identity);
            result[0][0]
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(2.0));
    }

    #[test]
    fn test_struct_and_method_pipeline() {
        let source = r#"
            struct Point {
                x: float,
                y: float
            }
            
            impl Point {
                fn new(x: float, y: float) -> Point {
                    Point { x: x, y: y }
                }
                
                fn distance_from_origin(self) -> float {
                    sqrt(self.x * self.x + self.y * self.y)
                }
                
                fn distance_to(self, other: Point) -> float {
                    let dx = self.x - other.x;
                    let dy = self.y - other.y;
                    sqrt(dx * dx + dy * dy)
                }
            }
            
            let p1 = Point::new(3.0, 4.0);
            let p2 = Point::new(0.0, 0.0);
            
            p1.distance_to(p2)
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(5.0)); // 3-4-5 triangle
    }

    #[test]
    fn test_enum_and_pattern_matching_pipeline() {
        let source = r#"
            enum Shape {
                Circle(float),
                Rectangle(float, float),
                Triangle(float, float, float)
            }
            
            fn calculate_area(shape: Shape) -> float {
                match shape {
                    Shape::Circle(radius) => 3.14159 * radius * radius,
                    Shape::Rectangle(width, height) => width * height,
                    Shape::Triangle(a, b, c) => {
                        let s = (a + b + c) / 2.0;
                        sqrt(s * (s - a) * (s - b) * (s - c))
                    }
                }
            }
            
            let circle = Shape::Circle(2.0);
            let rectangle = Shape::Rectangle(3.0, 4.0);
            
            calculate_area(rectangle)
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(12.0));
    }

    #[test]
    fn test_control_flow_pipeline() {
        let source = r#"
            fn fibonacci(n: int) -> int {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
            
            fn factorial(n: int) -> int {
                let mut result = 1;
                let mut i = 1;
                while i <= n {
                    result = result * i;
                    i = i + 1;
                }
                result
            }
            
            let fib_result = fibonacci(7);
            let fact_result = factorial(5);
            
            fib_result + fact_result
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(133)); // fib(7)=13, fact(5)=120, 13+120=133
    }

    #[test]
    fn test_closure_and_higher_order_functions_pipeline() {
        let source = r#"
            fn map(arr: [int], f: fn(int) -> int) -> [int] {
                let mut result = [];
                for item in arr {
                    result = push(result, f(item));
                }
                result
            }
            
            fn filter(arr: [int], predicate: fn(int) -> bool) -> [int] {
                let mut result = [];
                for item in arr {
                    if predicate(item) {
                        result = push(result, item);
                    }
                }
                result
            }
            
            let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let doubled = map(numbers, |x| x * 2);
            let evens = filter(doubled, |x| x % 2 == 0);
            
            len(evens)
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(10)); // All doubled numbers are even
    }

    #[test]
    fn test_physics_simulation_pipeline() {
        let source = r#"
            struct PhysicsObject {
                position: vec3,
                velocity: vec3,
                mass: float
            }
            
            impl PhysicsObject {
                fn new(pos: vec3, mass: float) -> PhysicsObject {
                    PhysicsObject { position: pos, velocity: [0.0, 0.0, 0.0], mass: mass }
                }
                
                fn apply_force(mut self, force: vec3, dt: float) -> PhysicsObject {
                    let acceleration = force / self.mass;
                    self.velocity = self.velocity + acceleration * dt;
                    self.position = self.position + self.velocity * dt;
                    self
                }
            }
            
            let mut ball = PhysicsObject::new([0.0, 10.0, 0.0], 1.0);
            let gravity = [0.0, -9.8, 0.0];
            
            // Simulate for 1 second with 10 steps
            for _ in 0..10 {
                ball = ball.apply_force(gravity, 0.1);
            }
            
            ball.position.y
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        if let crate::interpreter::Value::Float(y) = result {
            // After 1 second of free fall: y = 10 - 0.5 * 9.8 * 1^2 = 5.1
            assert!((y - 5.1).abs() < 0.5); // Allow for numerical integration error
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_complex_data_structures_pipeline() {
        let source = r#"
            struct TreeNode {
                value: int,
                left: Option<Box<TreeNode>>,
                right: Option<Box<TreeNode>>
            }
            
            fn create_leaf(value: int) -> TreeNode {
                TreeNode { value: value, left: Option::None, right: Option::None }
            }
            
            fn create_node(value: int, left: TreeNode, right: TreeNode) -> TreeNode {
                TreeNode {
                    value: value,
                    left: Option::Some(Box::new(left)),
                    right: Option::Some(Box::new(right))
                }
            }
            
            fn tree_sum(node: TreeNode) -> int {
                let mut sum = node.value;
                
                match node.left {
                    Option::Some(left_node) => sum = sum + tree_sum(*left_node),
                    Option::None => {}
                }
                
                match node.right {
                    Option::Some(right_node) => sum = sum + tree_sum(*right_node),
                    Option::None => {}
                }
                
                sum
            }
            
            let leaf1 = create_leaf(1);
            let leaf2 = create_leaf(2);
            let leaf3 = create_leaf(3);
            let leaf4 = create_leaf(4);
            
            let subtree1 = create_node(5, leaf1, leaf2);  // 5 + 1 + 2 = 8
            let subtree2 = create_node(6, leaf3, leaf4);  // 6 + 3 + 4 = 13
            let root = create_node(10, subtree1, subtree2); // 10 + 8 + 13 = 31
            
            tree_sum(root)
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(31));
    }

    #[test]
    fn test_generic_functions_pipeline() {
        let source = r#"
            fn identity<T>(x: T) -> T {
                x
            }
            
            fn pair<T, U>(first: T, second: U) -> (T, U) {
                (first, second)
            }
            
            fn swap<T, U>(p: (T, U)) -> (U, T) {
                (p.1, p.0)
            }
            
            let int_val = identity(42);
            let str_val = identity("hello");
            let int_str_pair = pair(int_val, str_val);
            let swapped = swap(int_str_pair);
            
            swapped.1
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(42));
    }

    #[test]
    fn test_trait_system_pipeline() {
        let source = r#"
            trait Drawable {
                fn draw(self) -> string;
            }
            
            struct Circle {
                radius: float
            }
            
            struct Square {
                side: float
            }
            
            impl Drawable for Circle {
                fn draw(self) -> string {
                    "Circle with radius " + str(self.radius)
                }
            }
            
            impl Drawable for Square {
                fn draw(self) -> string {
                    "Square with side " + str(self.side)
                }
            }
            
            fn draw_shape<T: Drawable>(shape: T) -> string {
                shape.draw()
            }
            
            let circle = Circle { radius: 5.0 };
            let square = Square { side: 3.0 };
            
            draw_shape(circle)
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::String("Circle with radius 5".to_string()));
    }

    #[test]
    fn test_error_handling_pipeline() {
        let source = r#"
            enum Result<T, E> {
                Ok(T),
                Err(E)
            }
            
            fn divide(a: float, b: float) -> Result<float, string> {
                if b == 0.0 {
                    Result::Err("Division by zero")
                } else {
                    Result::Ok(a / b)
                }
            }
            
            fn safe_divide_chain(a: float, b: float, c: float) -> Result<float, string> {
                match divide(a, b) {
                    Result::Ok(first_result) => divide(first_result, c),
                    Result::Err(error) => Result::Err(error)
                }
            }
            
            let result = safe_divide_chain(20.0, 4.0, 2.0);
            
            match result {
                Result::Ok(value) => value,
                Result::Err(_) => -1.0
            }
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(2.5)); // 20/4/2 = 2.5
    }

    #[test]
    fn test_concurrent_simulation_pipeline() {
        let source = r#"
            struct Particle {
                id: int,
                position: vec3,
                velocity: vec3
            }
            
            fn update_particle(p: Particle, dt: float) -> Particle {
                Particle {
                    id: p.id,
                    position: p.position + p.velocity * dt,
                    velocity: p.velocity
                }
            }
            
            fn simulate_particles(particles: [Particle], steps: int, dt: float) -> [Particle] {
                let mut current_particles = particles;
                
                for _ in 0..steps {
                    let mut updated_particles = [];
                    for particle in current_particles {
                        let updated = update_particle(particle, dt);
                        updated_particles = push(updated_particles, updated);
                    }
                    current_particles = updated_particles;
                }
                
                current_particles
            }
            
            let initial_particles = [
                Particle { id: 1, position: [0.0, 0.0, 0.0], velocity: [1.0, 0.0, 0.0] },
                Particle { id: 2, position: [1.0, 1.0, 0.0], velocity: [0.0, 1.0, 0.0] },
                Particle { id: 3, position: [2.0, 0.0, 1.0], velocity: [-1.0, 0.0, 1.0] }
            ];
            
            let final_particles = simulate_particles(initial_particles, 10, 0.1);
            
            final_particles[0].position.x
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(1.0)); // 0.0 + 1.0 * 0.1 * 10 = 1.0
    }

    #[test]
    fn test_mathematical_operations_pipeline() {
        let source = r#"
            fn square(x: float) -> float { x * x }
            fn cube(x: float) -> float { x * x * x }
            
            fn polynomial(x: float, coeffs: [float]) -> float {
                let mut result = 0.0;
                let mut power = 1.0;
                
                for coeff in coeffs {
                    result = result + coeff * power;
                    power = power * x;
                }
                
                result
            }
            
            fn derivative_at_point(f: fn(float) -> float, x: float, h: float) -> float {
                (f(x + h) - f(x - h)) / (2.0 * h)
            }
            
            // Test polynomial: 3x^2 + 2x + 1
            let coeffs = [1.0, 2.0, 3.0];
            let poly = |x| polynomial(x, coeffs);
            
            // Derivative at x=2 should be 6*2 + 2 = 14
            let derivative = derivative_at_point(poly, 2.0, 0.001);
            
            // Round to nearest integer for comparison
            round(derivative) as int
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(14));
    }

    #[test]
    fn test_full_language_features_pipeline() {
        let source = r#"
            // Test combining multiple language features
            
            trait Processable<T> {
                fn process(self, input: T) -> T;
            }
            
            struct Multiplier {
                factor: float
            }
            
            struct Adder {
                value: float
            }
            
            impl Processable<float> for Multiplier {
                fn process(self, input: float) -> float {
                    input * self.factor
                }
            }
            
            impl Processable<float> for Adder {
                fn process(self, input: float) -> float {
                    input + self.value
                }
            }
            
            fn pipeline<T, P: Processable<T>>(input: T, processors: [P]) -> T {
                let mut result = input;
                for processor in processors {
                    result = processor.process(result);
                }
                result
            }
            
            let multiplier = Multiplier { factor: 2.0 };
            let adder = Adder { value: 10.0 };
            
            let processors = [multiplier, adder];
            let result = pipeline(5.0, processors);
            
            result
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Float(20.0)); // (5 * 2) + 10 = 20
    }

    #[test]
    fn test_type_inference_pipeline() {
        let source = r#"
            // Test type inference across complex expressions
            
            fn infer_from_usage() {
                let x = 5;  // Should infer int
                let y = 3.14;  // Should infer float
                let z = x as float + y;  // Should infer float
                
                let array = [1, 2, 3];  // Should infer [int]
                let first = array[0];  // Should infer int
                
                let mixed_calc = first as float * y;  // Should infer float
                
                mixed_calc
            }
            
            fn generic_inference<T>(x: T, y: T) -> T {
                x  // Return first argument
            }
            
            let result1 = infer_from_usage();
            let result2 = generic_inference(42, 24);  // T should be inferred as int
            let result3 = generic_inference("hello", "world");  // T should be inferred as string
            
            result1 + result2 as float
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        if let crate::interpreter::Value::Float(val) = result {
            // result1 should be approximately 1 * 3.14 = 3.14, result2 is 42
            assert!((val - 45.14).abs() < 0.01);
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_memory_safety_pipeline() {
        let source = r#"
            // Test that references and borrowing work correctly
            
            fn safe_array_access(arr: [int], index: int) -> Option<int> {
                if index >= 0 && index < len(arr) {
                    Option::Some(arr[index])
                } else {
                    Option::None
                }
            }
            
            fn safe_sum(arr: [int]) -> int {
                let mut sum = 0;
                for i in 0..len(arr) {
                    match safe_array_access(arr, i) {
                        Option::Some(value) => sum = sum + value,
                        Option::None => {}
                    }
                }
                sum
            }
            
            let numbers = [1, 2, 3, 4, 5];
            let total = safe_sum(numbers);
            
            // Test out of bounds access
            let out_of_bounds = safe_array_access(numbers, 10);
            
            match out_of_bounds {
                Option::Some(_) => total + 1000,  // This shouldn't happen
                Option::None => total  // This should happen
            }
        "#;
        
        let result = run_complete_pipeline(source).unwrap();
        assert_eq!(result, crate::interpreter::Value::Int(15)); // 1+2+3+4+5 = 15
    }
}
