use crate::ast::nodes::{Expression, Item};
use crate::eval::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_sim_directive_parsing() {
    let source = r#"
        let test = @sim {
            let x = 42;
            x
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);

    if let Item::LetBinding(binding) = &program.items[0] {
        if let Expression::SimDirective { expression: _, .. } = &binding.value {
            // Successfully parsed a @sim directive
            assert!(true);
        } else {
            panic!("Expected SimDirective, got: {:?}", binding.value);
        }
    } else {
        panic!("Expected LetBinding, got: {:?}", program.items[0]);
    }
}

#[test]
fn test_plot_directive_parsing() {
    let source = r#"
        let test_plot = @plot {
            let y = 42;
            y
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.items.len(), 1);

    if let Item::LetBinding(binding) = &program.items[0] {
        if let Expression::PlotDirective { expression: _, .. } = &binding.value {
            // Successfully parsed a @plot directive
            assert!(true);
        } else {
            panic!("Expected PlotDirective, got: {:?}", binding.value);
        }
    } else {
        panic!("Expected LetBinding, got: {:?}", program.items[0]);
    }
}

#[test]
fn test_sim_directive_evaluation() {
    let source = r#"
        let test = @sim {
            let x = 42;
            x
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).unwrap();
    let program = parser.parse_program().unwrap();

    let mut interpreter = Interpreter::new();
    crate::stdlib::register_all(&mut interpreter);

    let result = interpreter.eval_program(&program).unwrap();

    // The directives currently return their PhysicsWorld as the program result
    // This happens because the let binding evaluates to the directive result
    match result {
        Value::PhysicsWorld(_) => {
            // Check that the variable was also bound correctly in the environment
            let test_value = interpreter.environment.get("test").unwrap();
            match test_value {
                Value::PhysicsWorld(_) => assert!(true),
                _ => panic!(
                    "Expected PhysicsWorld in environment, got: {:?}",
                    test_value
                ),
            }
        }
        _ => panic!("Expected PhysicsWorld as program result, got: {:?}", result),
    }
}

#[test]
fn test_plot_directive_evaluation() {
    let source = r#"
        let test_plot = @plot {
            let x = 10;
            x
        }
    "#;

    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).unwrap();
    let program = parser.parse_program().unwrap();

    let mut interpreter = Interpreter::new();
    crate::stdlib::register_all(&mut interpreter);

    let result = interpreter.eval_program(&program).unwrap();

    // The directives currently return their PhysicsWorld as the program result
    match result {
        Value::PhysicsWorld(_) => {
            // Check that the variable was also bound correctly in the environment
            let plot_value = interpreter.environment.get("test_plot").unwrap();
            match plot_value {
                Value::PhysicsWorld(_) => assert!(true),
                _ => panic!(
                    "Expected PhysicsWorld in environment, got: {:?}",
                    plot_value
                ),
            }
        }
        _ => panic!("Expected PhysicsWorld as program result, got: {:?}", result),
    }
}
