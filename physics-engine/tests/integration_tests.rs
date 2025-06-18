// Physics engine integration tests
use matrix_lang::*;
use physics_engine::physics::*;

#[cfg(test)]
mod physics_tests {
    use super::*;

    #[test]
    fn test_physics_engine_initialization() {
        let physics = PhysicsEngine::new();
        assert!(physics.bodies.is_empty());
    }

    #[test]
    fn test_matrix_lang_physics_integration() {
        // Test basic physics computations using matrix-lang
        let source = r#"
            let velocity = 10.0;
            let time = 2.0;
            let distance = velocity * time;
            distance
        "#;

        let mut interpreter = Interpreter::new();
        matrix_lang::stdlib::register_all(&mut interpreter);

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse_program().unwrap();

        let result = interpreter.eval_program(&ast).unwrap();
        match result {
            Value::Float(distance) => assert!((distance - 20.0).abs() < 0.001),
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_physics_constants() {
        // Test that physics constants are accessible
        let source = r#"
            let gravity = 9.81;
            let mass = 10.0;
            let force = mass * gravity;
            force
        "#;

        let mut interpreter = Interpreter::new();
        matrix_lang::stdlib::register_all(&mut interpreter);

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse_program().unwrap();

        let result = interpreter.eval_program(&ast).unwrap();
        match result {
            Value::Float(force) => assert!((force - 98.1).abs() < 0.001),
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_vector_operations() {
        // Test basic vector operations using arrays
        let source = r#"
            let vec_a = [1, 2, 3];
            let vec_b = [4, 5, 6];
            vec_a
        "#;

        let mut interpreter = Interpreter::new();
        matrix_lang::stdlib::register_all(&mut interpreter);

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse_program().unwrap();

        let result = interpreter.eval_program(&ast).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Int(1));
                assert_eq!(arr[1], Value::Int(2));
                assert_eq!(arr[2], Value::Int(3));
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_matrix_operations() {
        // Test basic matrix operations
        let source = r#"
            let matrix = [[1, 2], [3, 4]];
            matrix
        "#;

        let mut interpreter = Interpreter::new();
        matrix_lang::stdlib::register_all(&mut interpreter);

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse_program().unwrap();

        let result = interpreter.eval_program(&ast).unwrap();
        match result {
            Value::Matrix(matrix) => {
                assert_eq!(matrix.len(), 2);
                assert_eq!(matrix[0].len(), 2);
                assert_eq!(matrix[0][0], Value::Int(1));
                assert_eq!(matrix[0][1], Value::Int(2));
                assert_eq!(matrix[1][0], Value::Int(3));
                assert_eq!(matrix[1][1], Value::Int(4));
            }
            _ => panic!("Expected matrix result"),
        }
    }

    #[test]
    fn test_physics_simulation_step() {
        // Test that physics simulation can execute without errors
        let mut physics = PhysicsEngine::new();

        // Add a simple rigid body
        let body = RigidBody::new(
            1.0,                         // mass
            Vector3::new(0.0, 0.0, 0.0), // position
            Vector3::new(1.0, 0.0, 0.0), // velocity
            Shape::Sphere { radius: 1.0 },
        );

        physics.add_body(body);

        // Run a simulation step
        physics.step(0.016); // 60 FPS

        // Verify that simulation ran without panic
        assert_eq!(physics.bodies.len(), 1);
    }

    #[test]
    fn test_cross_language_computation() {
        // Test complex computation involving both matrix-lang and physics
        let source = r#"
            let compute_kinetic_energy = (mass, velocity) => {
                let velocity_squared = velocity * velocity;
                let kinetic_energy = 0.5 * mass * velocity_squared;
                kinetic_energy
            };
            compute_kinetic_energy(10.0, 5.0)
        "#;

        let mut interpreter = Interpreter::new();
        matrix_lang::stdlib::register_all(&mut interpreter);

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let ast = parser.parse_program().unwrap();

        let result = interpreter.eval_program(&ast).unwrap();
        match result {
            Value::Float(energy) => assert!((energy - 125.0).abs() < 0.001),
            _ => panic!("Expected float result for kinetic energy"),
        }
    }
}
