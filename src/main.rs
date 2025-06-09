pub mod ast;
pub mod ecs;
pub mod eval;
pub mod gpu;
pub mod gui;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod physics;
pub mod runtime;
pub mod stdlib;
pub mod types;

use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::path::Path;

use crate::eval::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::TypeChecker;

fn main() {
    let matches = Command::new("matrix-lang")
        .version("0.1.0")
        .about("A functional matrix-oriented scripting language for physics simulation and GPU acceleration")
        .arg(
            Arg::new("file")
                .help("The matrix language file to execute")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("repl")
                .long("repl")
                .short('r')
                .help("Start interactive REPL mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("parse-only")
                .long("parse-only")
                .help("Only parse the file, don't execute")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("gui")
                .long("gui")
                .short('g')
                .help("Launch physics visualization GUI")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("gui") {
        println!("Launching physics visualization GUI...");
        if let Err(e) = crate::gui::launch_physics_gui() {
            eprintln!("Failed to launch GUI: {}", e);
            std::process::exit(1);
        }
        return;
    } else if matches.get_flag("repl") || matches.get_one::<String>("file").is_none() {
        run_repl();
    } else if let Some(filename) = matches.get_one::<String>("file") {
        let parse_only = matches.get_flag("parse-only");
        run_file(filename, parse_only);
    }
}

fn run_file(filename: &str, parse_only: bool) {
    let path = Path::new(filename);

    if !path.exists() {
        eprintln!("Error: File '{}' not found", filename);
        std::process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            std::process::exit(1);
        }
    };

    match execute_source(&source, parse_only) {
        Ok(_) => {
            if parse_only {
                println!("✓ Parsing completed successfully");
            } else {
                println!("✓ Execution completed successfully");
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    println!("Matrix Language REPL v0.1.0");
    println!("Type 'exit' to quit, 'help' for commands");

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()
        .expect("Failed to create readline editor");

    // Load history if it exists
    let _ = rl.load_history("matrix_lang_history.txt");

    // Create persistent interpreter and type checker for REPL session
    let mut interpreter = Interpreter::new();
    let mut type_checker = TypeChecker::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                match trimmed {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "help" => {
                        print_help();
                        continue;
                    }
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    _ => {}
                }

                match execute_repl_line(&line, &mut interpreter, &mut type_checker) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history("matrix_lang_history.txt");
}

fn execute_repl_line(
    source: &str,
    interpreter: &mut Interpreter,
    type_checker: &mut TypeChecker,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to parse as a standalone expression first
    let lexer = Lexer::new(source);
    let mut parser =
        Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;

    // Try parsing as expression first, fallback to program if it fails
    let expr_result = parser.parse_expression();

    let result = if let Ok(expr) = expr_result {
        // Standalone expression - evaluate directly
        match interpreter.eval_expression(&expr) {
            Ok(value) => {
                // Only print result if it's not Unit (empty)
                match value {
                    crate::eval::Value::Unit => Ok(()),
                    _ => {
                        println!("{}", format_result(&value));
                        Ok(())
                    }
                }
            }
            Err(e) => Err(format!("Runtime error: {}", e)),
        }
    } else {
        // Try parsing as a full program (for let bindings, function definitions, etc.)
        let lexer = Lexer::new(source);
        let mut parser =
            Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;

        let ast = parser
            .parse_program()
            .map_err(|e| format!("Parse error: {}", e))?;

        // Type checking
        type_checker
            .check_program(&ast)
            .map_err(|e| format!("Type error: {}", e))?;

        // Evaluation/Interpretation
        let result = interpreter
            .eval_program(&ast)
            .map_err(|e| format!("Runtime error: {}", e))?;

        // Only print result if it's not Unit (empty)
        match result {
            crate::eval::Value::Unit => Ok(()),
            _ => {
                println!("{}", format_result(&result));
                Ok(())
            }
        }
    };

    result.map_err(|e| e.into())
}

fn execute_source(source: &str, parse_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Tokenize
    let lexer = Lexer::new(source);
    let _tokens = lexer
        .tokenize()
        .map_err(|e| format!("Lexical error: {}", e))?;

    // Parse
    let lexer = Lexer::new(source);
    let mut parser =
        Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;
    let ast = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    if parse_only {
        println!("AST: {:#?}", ast);
        return Ok(());
    }

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    println!("✓ Type checking passed");

    // Evaluation/Interpretation
    let mut interpreter = Interpreter::new();
    let result = interpreter
        .eval_program(&ast)
        .map_err(|e| format!("Runtime error: {}", e))?;

    // Only print result if it's not Unit (empty)
    match result {
        crate::eval::Value::Unit => {}
        _ => println!("Result: {}", format_result(&result)),
    }

    Ok(())
}

fn print_help() {
    println!("Matrix Language REPL Commands:");
    println!("  help           - Show this help message");
    println!("  exit, quit     - Exit the REPL");
    println!("  clear          - Clear the screen");
    println!();
    println!("Language Features:");
    println!("  struct Vector2 {{ x: Float, y: Float }}");
    println!("  let add = (a: Float, b: Float) => a + b");
    println!("  let matrix = [[1, 2], [3, 4]]");
    println!("  let comp = [i * j | i in 1..3, j in 1..3]");
    println!("  match value {{ Some(x) => x, None => 0 }}");
    println!("  parallel {{ expr1; expr2 }}");
    println!("  @gpu let compute = (data: Matrix) => ...");
}

fn format_result(value: &crate::eval::Value) -> String {
    match value {
        crate::eval::Value::Int(i) => i.to_string(),
        crate::eval::Value::Float(f) => f.to_string(),
        crate::eval::Value::Bool(b) => b.to_string(),
        crate::eval::Value::String(s) => format!("\"{}\"", s),
        crate::eval::Value::Unit => "()".to_string(),
        crate::eval::Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_result).collect();
            format!("[{}]", elements.join(", "))
        }
        crate::eval::Value::Matrix(mat) => {
            let rows: Vec<String> = mat
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row.iter().map(format_result).collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            format!("[{}]", rows.join(", "))
        }
        crate::eval::Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_result(v)))
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        crate::eval::Value::Function { .. } => "<function>".to_string(),
        crate::eval::Value::BuiltinFunction { name, .. } => format!("<builtin: {}>", name),
        crate::eval::Value::PhysicsWorldHandle(_) => "<physics_world>".to_string(),
    }
}
