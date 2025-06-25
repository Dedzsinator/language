// Physics Engine GUI - Unity-style physics simulation interface
// This application provides a GUI for physics simulation using the matrix-lang as a dependency

use clap::{Arg, Command};
use matrix_lang::eval::Interpreter;
use matrix_lang::lexer::Lexer;
use matrix_lang::parser::Parser;
use matrix_lang::types::TypeChecker;
use std::fs;
use std::path::Path;

// GUI modules for physics simulation (keep only GUI-related modules)
pub mod animation_view;
pub mod game_view;
pub mod gui;
pub mod inspector;
pub mod object_hierarchy;
pub mod physics_comprehensive;
pub mod physics_debugger;
pub mod project_browser;
pub mod scene_manager;
pub mod scene_view;
pub mod scripting_panel;
pub mod unity_layout;
pub mod viewport;

fn main() {
    let matches = Command::new("physics-engine-gui")
        .version("0.1.0")
        .about("Unity-style Physics Simulation Engine with GUI")
        .arg(
            Arg::new("script")
                .help("Matrix language script to load for physics simulation")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("gui")
                .long("gui")
                .short('g')
                .help("Launch Unity-style GUI interface")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Enable physics debugging mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("viewport-only")
                .long("viewport-only")
                .help("Launch only the viewport interface")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .help("Specify the launch mode (3d_sim, plot_anim)")
                .value_name("MODE")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    if matches.get_flag("gui") {
        launch_unity_gui();
    } else if matches.get_flag("viewport-only") {
        launch_viewport_only();
    } else if matches.get_flag("debug") {
        launch_physics_debugger();
    } else if let Some(script_file) = matches.get_one::<String>("script") {
        load_and_run_script(script_file);
    } else if let Some(mode) = matches.get_one::<String>("mode") {
        launch_with_mode(mode);
    } else {
        // Default to GUI mode
        launch_unity_gui();
    }
}

/// Launch the Unity-style GUI interface
fn launch_unity_gui() {
    println!("🎮 Launching Unity-style Physics Engine GUI...");
    if let Err(e) = gui::launch_physics_editor() {
        eprintln!("Failed to launch GUI: {}", e);
        std::process::exit(1);
    }
}

/// Launch viewport-only mode
fn launch_viewport_only() {
    println!("🔍 Launching Viewport-only mode...");
    // For now, just launch the standard GUI
    if let Err(e) = gui::launch_physics_editor() {
        eprintln!("Failed to launch viewport: {}", e);
        std::process::exit(1);
    }
}

/// Launch physics debugger mode
fn launch_physics_debugger() {
    println!("🔧 Launching Physics Debugger mode...");
    // For now, just launch the standard GUI which includes debugging
    if let Err(e) = gui::launch_physics_editor() {
        eprintln!("Failed to launch debugger: {}", e);
        std::process::exit(1);
    }
}

/// Launch with specific mode for Matrix Language directives
fn launch_with_mode(mode: &str) {
    match mode {
        "3d_sim" => {
            println!("🎬 Launching 3D Physics Simulation mode...");
            // Launch GUI in simulation mode
            if let Err(e) = gui::launch_physics_editor() {
                eprintln!("Failed to launch 3D simulation: {}", e);
                std::process::exit(1);
            }
        }
        "plot_anim" => {
            println!("📊 Launching Plot Animation mode...");
            // Launch GUI in plotting mode
            if let Err(e) = gui::launch_physics_editor() {
                eprintln!("Failed to launch plot animation: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown mode: {}. Available modes: 3d_sim, plot_anim", mode);
            std::process::exit(1);
        }
    }
}

/// Load and run a Matrix Language script
fn load_and_run_script(script_file: &str) {
    let path = Path::new(script_file);

    if !path.exists() {
        eprintln!("Error: File '{}' not found", script_file);
        std::process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", script_file, err);
            std::process::exit(1);
        }
    };

    match execute_source(&source, false) {
        Ok(_) => {
            println!("✓ Execution completed successfully");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
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
        matrix_lang::eval::Value::Unit => {}
        _ => println!("Result: {}", format_result(&result)),
    }

    Ok(())
}

fn format_result(value: &matrix_lang::eval::interpreter::Value) -> String {
    match value {
        matrix_lang::eval::interpreter::Value::Int(i) => i.to_string(),
        matrix_lang::eval::interpreter::Value::Float(f) => f.to_string(),
        matrix_lang::eval::interpreter::Value::Bool(b) => b.to_string(),
        matrix_lang::eval::interpreter::Value::String(s) => format!("\"{}\"", s),
        matrix_lang::eval::interpreter::Value::Unit => "()".to_string(),
        matrix_lang::eval::interpreter::Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_result).collect();
            format!("[{}]", elements.join(", "))
        }
        matrix_lang::eval::interpreter::Value::Matrix(mat) => {
            let rows: Vec<String> = mat
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row.iter().map(format_result).collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            format!("[{}]", rows.join(", "))
        }
        matrix_lang::eval::interpreter::Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_result(v)))
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        matrix_lang::eval::interpreter::Value::Function { .. } => "<function>".to_string(),
        matrix_lang::eval::interpreter::Value::BuiltinFunction { name, .. } => {
            format!("<builtin: {}>", name)
        }
        matrix_lang::eval::interpreter::Value::PhysicsWorld(_) => "<physics_world>".to_string(),
        matrix_lang::eval::interpreter::Value::PhysicsObject(_) => "<physics_object>".to_string(),
    }
}
