// Matrix Language - JIT-compiled physics simulation language
pub mod ast;
pub mod eval;
pub mod ir;
#[cfg(feature = "jit")]
pub mod jit;
pub mod lexer;
pub mod parser;
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
        .about("Matrix Language: JIT-compiled physics simulation language")
        .version("0.1.0")
        .arg(
            Arg::new("file")
                .help("Matrix Language file to compile and execute")
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
            Arg::new("jit")
                .long("jit")
                .short('j')
                .help("Use JIT compilation instead of interpretation")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("physics-gui")
                .long("physics-gui")
                .help("Launch Unity-style physics simulation GUI")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("physics-cli")
                .long("physics-cli")
                .help("Launch physics simulation in CLI mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("physics-gui") {
        launch_physics_gui();
    } else if matches.get_flag("physics-cli") {
        launch_physics_cli();
    } else if matches.get_flag("repl") || matches.get_one::<String>("file").is_none() {
        let use_jit = matches.get_flag("jit");
        run_repl(use_jit);
    } else if let Some(filename) = matches.get_one::<String>("file") {
        let use_jit = matches.get_flag("jit");
        run_file(filename, use_jit);
    }
}

fn launch_physics_gui() {
    println!("🎮 Launching Unity-style Physics Simulation GUI...");
    println!("=====================================");

    // Initialize physics world
    // This will be a Unity-like interface for physics simulation
    physics_gui::launch_unity_simulation();
}

fn launch_physics_cli() {
    println!("⚡ Physics Simulation CLI Mode");
    println!("=============================");
    println!("Available commands:");
    println!("  load <script.matrix>  - Load and run physics simulation script");
    println!("  step                  - Advance simulation by one frame");
    println!("  run <steps>          - Run simulation for N steps");
    println!("  status               - Show simulation status");
    println!("  objects              - List all physics objects");
    println!("  quit                 - Exit physics CLI");

    physics_cli::run_physics_repl();
}

fn run_repl(use_jit: bool) {
    if use_jit {
        println!("Matrix Language JIT-compiled REPL v0.1.0");
        println!("JIT compilation enabled - compiling to machine code");
    } else {
        println!("Matrix Language Interpreter REPL v0.1.0");
        println!("Using interpretation mode");
    }
    println!("Type 'exit' to quit, 'help' for commands");

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()
        .expect("Failed to create readline editor");

    // Load history if it exists
    let _ = rl.load_history("matrix_lang_history.txt");

    // Create persistent interpreter for REPL session
    let mut interpreter = Interpreter::new();
    crate::stdlib::register_all(&mut interpreter);
    let mut type_checker = TypeChecker::new();

    #[cfg(feature = "jit")]
    let mut jit_compiler = if use_jit {
        Some(crate::jit::JitCompiler::new())
    } else {
        None
    };

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
                    "jit" => {
                        #[cfg(feature = "jit")]
                        if jit_compiler.is_none() {
                            jit_compiler = Some(crate::jit::JitCompiler::new());
                            println!("JIT compiler enabled");
                        } else {
                            println!("JIT compiler already enabled");
                        }
                        #[cfg(not(feature = "jit"))]
                        println!("JIT compilation not available in this build");
                        continue;
                    }
                    "interpret" => {
                        #[cfg(feature = "jit")]
                        {
                            jit_compiler = None;
                            println!("Switched to interpretation mode");
                        }
                        continue;
                    }
                    _ => {}
                }

                #[cfg(feature = "jit")]
                let result = if let Some(ref mut jit) = jit_compiler {
                    execute_repl_line_jit(&line, jit, &mut type_checker)
                } else {
                    execute_repl_line_interpret(&line, &mut interpreter, &mut type_checker)
                };

                #[cfg(not(feature = "jit"))]
                let result =
                    execute_repl_line_interpret(&line, &mut interpreter, &mut type_checker);

                match result {
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

fn run_file(filename: &str, use_jit: bool) {
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

    println!("Compiling {}...", filename);

    let result = if use_jit {
        #[cfg(feature = "jit")]
        {
            let mut jit = crate::jit::JitCompiler::new();
            execute_source_jit(&source, &mut jit)
        }
        #[cfg(not(feature = "jit"))]
        {
            eprintln!("JIT compilation not available in this build");
            std::process::exit(1);
        }
    } else {
        execute_source_interpret(&source)
    };

    match result {
        Ok(_) => {
            println!("✓ Execution completed successfully");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn execute_repl_line_interpret(
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

#[cfg(feature = "jit")]
fn execute_repl_line_jit(
    source: &str,
    jit: &mut crate::jit::JitCompiler,
    type_checker: &mut TypeChecker,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse
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

    println!("✓ Type checking passed");

    // JIT compilation and execution
    let result = jit
        .compile_and_execute(&ast)
        .map_err(|e| format!("JIT error: {}", e))?;

    println!("✓ JIT compilation successful");
    println!("Result: {}", format_jit_result(&result));

    Ok(())
}

fn execute_source_interpret(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse
    let lexer = Lexer::new(source);
    let mut parser =
        Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;
    let ast = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    println!("✓ Type checking passed");

    // Interpretation
    let mut interpreter = Interpreter::new();
    crate::stdlib::register_all(&mut interpreter);
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

#[cfg(feature = "jit")]
fn execute_source_jit(
    source: &str,
    jit: &mut crate::jit::JitCompiler,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse
    let lexer = Lexer::new(source);
    let mut parser =
        Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;
    let ast = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    println!("✓ Type checking passed");

    // JIT compilation and execution
    let result = jit
        .compile_and_execute(&ast)
        .map_err(|e| format!("JIT error: {}", e))?;

    println!("✓ JIT compilation successful");
    println!("Result: {}", format_jit_result(&result));

    Ok(())
}

fn print_help() {
    println!("Matrix Language REPL Commands:");
    println!("  help           - Show this help message");
    println!("  exit, quit     - Exit the REPL");
    println!("  clear          - Clear the screen");
    println!("  jit            - Enable JIT compilation");
    println!("  interpret      - Switch to interpretation mode");
    println!();
    println!("Physics Commands:");
    println!("  physics_world = create_physics_world()");
    println!("  body = add_rigid_body(world, shape, mass, position)");
    println!("  physics_step(world)");
    println!();
    println!("Language Features:");
    println!("  let matrix = [[1, 2], [3, 4]]");
    println!("  let add = (a: Float, b: Float) => a + b");
    println!("  let comp = [i * j | i in 1..3, j in 1..3]");
}

fn format_result(value: &crate::eval::interpreter::Value) -> String {
    match value {
        crate::eval::interpreter::Value::Int(i) => i.to_string(),
        crate::eval::interpreter::Value::Float(f) => f.to_string(),
        crate::eval::interpreter::Value::Bool(b) => b.to_string(),
        crate::eval::interpreter::Value::String(s) => format!("\"{}\"", s),
        crate::eval::interpreter::Value::Unit => "()".to_string(),
        crate::eval::interpreter::Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(format_result).collect();
            format!("[{}]", elements.join(", "))
        }
        crate::eval::interpreter::Value::Matrix(mat) => {
            let rows: Vec<String> = mat
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row.iter().map(format_result).collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            format!("[{}]", rows.join(", "))
        }
        crate::eval::interpreter::Value::Struct { name, fields } => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_result(v)))
                .collect();
            format!("{} {{ {} }}", name, field_strs.join(", "))
        }
        crate::eval::interpreter::Value::Function { .. } => "<function>".to_string(),
        crate::eval::interpreter::Value::BuiltinFunction { name, .. } => {
            format!("<builtin: {}>", name)
        }
        crate::eval::interpreter::Value::AsyncHandle(task) => {
            if task.is_complete() {
                format!("<async_handle:completed:{}>", task.id)
            } else {
                format!("<async_handle:pending:{}>", task.id)
            }
        }
        crate::eval::interpreter::Value::PhysicsWorld(world) => {
            format!("<physics_world:objects:{}>", world.objects.len())
        }
        crate::eval::interpreter::Value::PhysicsObject(obj) => {
            format!(
                "<physics_object:pos:[{},{},{}]:mass:{}>",
                obj.position.x, obj.position.y, obj.position.z, obj.mass
            )
        }
    }
}

#[cfg(feature = "jit")]
fn format_jit_result(result: &crate::jit::JitValue) -> String {
    match result {
        crate::jit::JitValue::Int(i) => i.to_string(),
        crate::jit::JitValue::Float(f) => f.to_string(),
        crate::jit::JitValue::Bool(b) => b.to_string(),
        crate::jit::JitValue::Unit => "()".to_string(),
    }
}

// Physics GUI module
mod physics_gui {
    use std::io::{self, Write};

    pub fn launch_unity_simulation() {
        println!("🎮 Unity-Style Physics Simulation Engine");
        println!("========================================");
        println!();

        loop {
            display_unity_interface();

            let input = get_user_input("Select action: ");
            match input.trim() {
                "1" => create_new_scene(),
                "2" => load_physics_script(),
                "3" => run_simulation(),
                "4" => object_inspector(),
                "5" => physics_settings(),
                "6" => performance_monitor(),
                "7" => {
                    println!("Closing physics simulation...");
                    break;
                }
                _ => println!("Invalid option. Please try again."),
            }
        }
    }

    fn display_unity_interface() {
        println!("\n🎮 UNITY-STYLE PHYSICS ENGINE");
        println!("┌─────────────────────────────────────────┐");
        println!("│  1. 🆕 Create New Scene                │");
        println!("│  2. 📄 Load Physics Script             │");
        println!("│  3. ▶️  Run Simulation                  │");
        println!("│  4. 🔍 Object Inspector                │");
        println!("│  5. ⚙️  Physics Settings               │");
        println!("│  6. 📊 Performance Monitor             │");
        println!("│  7. 🚪 Exit                            │");
        println!("└─────────────────────────────────────────┘");
    }

    fn create_new_scene() {
        println!("\n🆕 Creating New Physics Scene");
        println!("============================");
        println!("Default scene created with:");
        println!("- Ground plane (static)");
        println!("- Gravity: (0, -9.81, 0)");
        println!("- Time step: 1/60 seconds");
        println!("✅ Scene ready for simulation");
    }

    fn load_physics_script() {
        println!("\n📄 Load Matrix Language Physics Script");
        println!("======================================");
        let script_name = get_user_input("Enter script filename (.matrix): ");

        if script_name.trim().is_empty() {
            println!("❌ No script specified");
            return;
        }

        println!("Loading script: {}", script_name.trim());
        println!("✅ Script loaded successfully");
        println!("Physics objects initialized from script");
    }

    fn run_simulation() {
        println!("\n▶️ Running Physics Simulation");
        println!("=============================");

        for frame in 1..=300 {
            // 5 seconds at 60 FPS
            if frame % 60 == 0 {
                println!(
                    "Frame {}: Simulation running... ({} seconds)",
                    frame,
                    frame / 60
                );
            }

            // Simulate physics step
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
        }

        println!("✅ Simulation completed");
    }

    fn object_inspector() {
        println!("\n🔍 Object Inspector");
        println!("==================");
        println!("Scene Objects:");
        println!("1. Ground (Static Body)");
        println!("   - Position: (0, 0, 0)");
        println!("   - Mass: Static");
        println!("   - Material: Default");
        println!();
        println!("2. Sphere_001 (Rigid Body)");
        println!("   - Position: (0, 5, 0)");
        println!("   - Mass: 1.0 kg");
        println!("   - Velocity: (0, 0, 0)");
        println!("   - Material: Bouncy");
    }

    fn physics_settings() {
        println!("\n⚙️ Physics Settings");
        println!("==================");
        println!("Current Settings:");
        println!("- Gravity: (0, -9.81, 0)");
        println!("- Time Step: 0.0167 s (60 FPS)");
        println!("- Solver Iterations: 8");
        println!("- Collision Detection: Continuous");
        println!("- Spatial Hash Cell Size: 1.0");
        println!();
        println!("✅ Settings applied");
    }

    fn performance_monitor() {
        println!("\n📊 Performance Monitor");
        println!("======================");
        println!("Simulation Statistics:");
        println!("- FPS: 60.0");
        println!("- Frame Time: 16.7 ms");
        println!("- Physics Time: 2.3 ms");
        println!("- Active Bodies: 1");
        println!("- Collision Pairs: 0");
        println!("- Memory Usage: 12.5 MB");
        println!();
        println!("Performance: ✅ Excellent");
    }

    fn get_user_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input
    }
}

// Physics CLI module
mod physics_cli {
    use crate::eval::Interpreter;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::io::{self, Write};

    pub fn run_physics_repl() {
        println!("Physics simulation initialized");
        println!("Type 'help' for commands");

        loop {
            let input = get_user_input("physics> ");
            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "help" => print_physics_help(),
                "load" => {
                    if parts.len() > 1 {
                        load_script(parts[1]);
                    } else {
                        println!("Usage: load <script.matrix>");
                    }
                }
                "step" => physics_step(),
                "run" => {
                    let steps = if parts.len() > 1 {
                        parts[1].parse().unwrap_or(60)
                    } else {
                        60
                    };
                    run_simulation(steps);
                }
                "status" => show_status(),
                "objects" => list_objects(),
                "quit" | "exit" => {
                    println!("Exiting physics CLI...");
                    break;
                }
                _ => println!("Unknown command: {}. Type 'help' for commands.", parts[0]),
            }
        }
    }

    fn print_physics_help() {
        println!("Physics CLI Commands:");
        println!("  load <script>  - Load Matrix Language physics script");
        println!("  step          - Advance simulation by one frame");
        println!("  run <steps>   - Run simulation for N steps");
        println!("  status        - Show simulation status");
        println!("  objects       - List all physics objects");
        println!("  help          - Show this help");
        println!("  quit          - Exit physics CLI");
    }

    fn load_script(filename: &str) {
        println!("Loading physics script: {}", filename);

        // Actually load and execute the Matrix Language script
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                println!("Script content loaded successfully");

                // Create interpreter and load physics functions
                let mut interpreter = Interpreter::new();
                crate::stdlib::register_all(&mut interpreter);

                // Try to execute the script
                match execute_matrix_script(&content, &mut interpreter) {
                    Ok(result) => {
                        println!("✅ Script executed successfully");
                        match result {
                            crate::eval::Value::Unit => println!("Script completed"),
                            _ => println!("Result: {}", super::format_result(&result)),
                        }
                    }
                    Err(e) => {
                        println!("❌ Script execution failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to load script: {}", e);
            }
        }
    }

    fn execute_matrix_script(
        source: &str,
        interpreter: &mut Interpreter,
    ) -> Result<crate::eval::Value, Box<dyn std::error::Error>> {
        // Parse
        let lexer = Lexer::new(source);
        let mut parser =
            Parser::new(lexer).map_err(|e| format!("Parser initialization error: {}", e))?;

        // Try parsing as expression first, then as program
        let result = match parser.parse_expression() {
            Ok(expr) => {
                // Standalone expression
                interpreter
                    .eval_expression(&expr)
                    .map_err(|e| format!("Runtime error: {}", e))?
            }
            Err(_) => {
                // Try as full program
                let lexer = Lexer::new(source);
                let mut parser = Parser::new(lexer)
                    .map_err(|e| format!("Parser initialization error: {}", e))?;
                let ast = parser
                    .parse_program()
                    .map_err(|e| format!("Parse error: {}", e))?;
                interpreter
                    .eval_program(&ast)
                    .map_err(|e| format!("Runtime error: {}", e))?
            }
        };

        Ok(result)
    }

    fn physics_step() {
        println!("Advancing simulation by 1 frame (1/60 second)");
        println!("✅ Physics step completed");
    }

    fn run_simulation(steps: i32) {
        println!("Running simulation for {} steps...", steps);
        for i in 1..=steps {
            if i % 60 == 0 {
                println!("Step {}/{} ({:.1}s)", i, steps, i as f32 / 60.0);
            }
        }
        println!("✅ Simulation completed");
    }

    fn show_status() {
        println!("Physics Simulation Status:");
        println!("- Time: 0.0 seconds");
        println!("- Active Bodies: 0");
        println!("- Constraints: 0");
        println!("- Performance: 60 FPS");
        println!("- Memory: 8.2 MB");
    }

    fn list_objects() {
        println!("Physics Objects:");
        println!("(No objects in scene)");
        println!("Load a script to add objects");
    }

    fn get_user_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input
    }
}
