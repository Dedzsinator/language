use super::*;

/// Scripting panel for writing and editing Matrix Language scripts
pub struct ScriptingPanel {
    scripts: std::collections::HashMap<String, ScriptEditor>,
    active_script: Option<String>,
    new_script_name: String,
    show_templates: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptEditor {
    pub name: String,
    pub code: String,
    pub file_path: String,
    pub is_dirty: bool,
    pub cursor_position: usize,
    pub syntax_errors: Vec<SyntaxError>,
    pub auto_save: bool,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Lexer,
    Parser,
    Runtime,
}

impl ScriptingPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            scripts: std::collections::HashMap::new(),
            active_script: None,
            new_script_name: String::new(),
            show_templates: false,
        };
        
        // Create a default script
        panel.create_new_script("Main".to_string(), get_default_script_template());
        panel.active_script = Some("Main".to_string());
        
        panel
    }
    
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("Script Editor")
            .default_width(800.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                self.show_toolbar(ui);
                ui.separator();
                self.show_script_tabs(ui);
                ui.separator();
                self.show_active_script_editor(ui);
                ui.separator();
                self.show_status_bar(ui);
            });
    }
    
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📄 New").clicked() {
                self.show_new_script_dialog(ui);
            }
            
            if ui.button("📁 Open").clicked() {
                // TODO: Open file dialog
                println!("Open file dialog");
            }
            
            if ui.button("💾 Save").clicked() {
                self.save_active_script();
            }
            
            if ui.button("💾 Save All").clicked() {
                self.save_all_scripts();
            }
            
            ui.separator();
            
            if ui.button("▶ Run").clicked() {
                self.run_active_script();
            }
            
            if ui.button("🐛 Debug").clicked() {
                self.debug_active_script();
            }
            
            ui.separator();
            
            if ui.button("📖 Templates").clicked() {
                self.show_templates = !self.show_templates;
            }
            
            ui.separator();
            
            // Search functionality
            ui.label("Search:");
            ui.text_edit_singleline(&mut String::new()); // TODO: Implement search
        });
    }
    
    fn show_script_tabs(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                let script_names: Vec<String> = self.scripts.keys().cloned().collect();
                let mut script_to_close = None;
                
                for script_name in script_names {
                    let is_active = self.active_script.as_ref() == Some(&script_name);
                    let is_dirty = self.scripts.get(&script_name).map(|s| s.is_dirty).unwrap_or(false);
                    
                    let tab_text = if is_dirty {
                        format!("● {}", script_name)
                    } else {
                        script_name.clone()
                    };
                    
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_active, &tab_text).clicked() {
                                self.active_script = Some(script_name.clone());
                            }
                            
                            if ui.small_button("✕").clicked() {
                                script_to_close = Some(script_name.clone());
                            }
                        });
                    });
                }
                
                // Close script if requested
                if let Some(script_name) = script_to_close {
                    self.close_script(script_name);
                }
                
                // Add new script button
                if ui.button("➕").clicked() {
                    self.show_new_script_dialog(ui);
                }
            });
        });
    }
    
    fn show_active_script_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(active_name) = self.active_script.clone() {
            if let Some(script) = self.scripts.get_mut(&active_name) {
                ui.columns(2, |columns| {
                    // Code editor
                    columns[0].vertical(|ui| {
                        ui.heading(&format!("Editing: {}", script.name));
                        
                        // Line numbers and code editor
                        egui::ScrollArea::both()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let lines = script.code.lines().count().max(1);
                                ui.horizontal_top(|ui| {
                                    // Line numbers
                                    ui.vertical(|ui| {
                                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                                        for i in 1..=lines {
                                            ui.label(format!("{:3}", i));
                                        }
                                    });
                                    
                                    ui.separator();
                                    
                                    // Code editor
                                    let response = ui.add(
                                        egui::TextEdit::multiline(&mut script.code)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(25)
                                    );
                                    
                                    if response.changed() {
                                        script.is_dirty = true;
                                        // TODO: Fix borrow checker issue
                                        // self.check_syntax(&active_name);
                                    }
                                });
                            });
                    });
                    
                    // Right panel - syntax highlighting, errors, etc.
                    columns[1].vertical(|ui| {
                        ui.heading("Script Info");
                        
                        // File info
                        ui.group(|ui| {
                            ui.label("File Information");
                            ui.label(format!("Path: {}", script.file_path));
                            ui.label(format!("Lines: {}", script.code.lines().count()));
                            ui.label(format!("Characters: {}", script.code.len()));
                            ui.checkbox(&mut script.auto_save, "Auto Save");
                        });
                        
                        ui.separator();
                        
                        // Syntax errors
                        ui.group(|ui| {
                            ui.label("Syntax Errors");
                            if script.syntax_errors.is_empty() {
                                ui.label("✅ No errors");
                            } else {
                                egui::ScrollArea::vertical()
                                    .max_height(150.0)
                                    .show(ui, |ui| {
                                        for error in &script.syntax_errors {
                                            ui.horizontal(|ui| {
                                                let icon = match error.error_type {
                                                    ErrorType::Lexer => "🔤",
                                                    ErrorType::Parser => "🔧",
                                                    ErrorType::Runtime => "⚠️",
                                                };
                                                ui.label(icon);
                                                ui.label(format!("Line {}: {}", error.line, error.message));
                                            });
                                        }
                                    });
                            }
                        });
                        
                        ui.separator();
                        
                        // Code templates
                        if self.show_templates {
                            ui.group(|ui| {
                                ui.label("Code Templates");
                                if ui.button("Function Template").clicked() {
                                    let template = "let myFunction = (param: Type) -> ReturnType => {\n    // Function body\n    return result\n}";
                                    script.code.push_str(template);
                                    script.is_dirty = true;
                                }
                                if ui.button("Class Template").clicked() {
                                    let template = "struct MyStruct {\n    field: Type\n}\n\nimpl MyStruct {\n    let new = (field: Type) -> MyStruct => {\n        MyStruct { field }\n    }\n}";
                                    script.code.push_str(template);
                                    script.is_dirty = true;
                                }
                                if ui.button("Physics Object").clicked() {
                                    let template = "// Create a physics object\nlet physicsObject = {\n    position: Vec3::new(0.0, 0.0, 0.0),\n    velocity: Vec3::new(0.0, 0.0, 0.0),\n    mass: 1.0\n}";
                                    script.code.push_str(template);
                                    script.is_dirty = true;
                                }
                            });
                        }
                        
                        ui.separator();
                        
                        // Script actions
                        ui.group(|ui| {
                            ui.label("Actions");
                            if ui.button("Format Code").clicked() {
                                self.format_code(&active_name);
                            }
                            if ui.button("Check Syntax").clicked() {
                                // TODO: Fix borrow checker issue
                                // self.check_syntax(&active_name);
                            }
                            if ui.button("Generate Documentation").clicked() {
                                self.generate_documentation(&active_name);
                            }
                        });
                    });
                });
            }
        } else {
            ui.label("No script selected. Create a new script or open an existing one.");
        }
    }
    
    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(active_name) = &self.active_script {
                if let Some(script) = self.scripts.get(active_name) {
                    ui.label(format!("Lines: {} | Characters: {}", 
                        script.code.lines().count(), 
                        script.code.len()));
                    
                    if script.is_dirty {
                        ui.label("● Modified");
                    } else {
                        ui.label("Saved");
                    }
                    
                    ui.separator();
                    
                    ui.label(format!("Errors: {}", script.syntax_errors.len()));
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Matrix Language v1.0");
            });
        });
    }
    
    fn show_new_script_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("New Script").show(ui.ctx(), |ui| {
            ui.label("Script Name:");
            ui.text_edit_singleline(&mut self.new_script_name);
            
            ui.horizontal(|ui| {
                if ui.button("Create").clicked() && !self.new_script_name.is_empty() {
                    self.create_new_script(self.new_script_name.clone(), get_default_script_template());
                    self.active_script = Some(self.new_script_name.clone());
                    self.new_script_name.clear();
                }
                if ui.button("Cancel").clicked() {
                    self.new_script_name.clear();
                }
            });
        });
    }
    
    fn create_new_script(&mut self, name: String, template: String) {
        let script = ScriptEditor {
            name: name.clone(),
            code: template,
            file_path: format!("{}.matrix", name),
            is_dirty: true,
            cursor_position: 0,
            syntax_errors: Vec::new(),
            auto_save: false,
        };
        
        self.scripts.insert(name, script);
    }
    
    fn close_script(&mut self, name: String) {
        if let Some(script) = self.scripts.get(&name) {
            if script.is_dirty {
                // TODO: Show save dialog
            }
        }
        
        self.scripts.remove(&name);
        
        if self.active_script.as_ref() == Some(&name) {
            self.active_script = self.scripts.keys().next().cloned();
        }
    }
    
    fn save_active_script(&mut self) {
        if let Some(active_name) = &self.active_script {
            if let Some(script) = self.scripts.get_mut(active_name) {
                // TODO: Actually save to file
                std::fs::write(&script.file_path, &script.code).unwrap_or_else(|e| {
                    println!("Failed to save script: {}", e);
                });
                script.is_dirty = false;
            }
        }
    }
    
    fn save_all_scripts(&mut self) {
        for script in self.scripts.values_mut() {
            if script.is_dirty {
                std::fs::write(&script.file_path, &script.code).unwrap_or_else(|e| {
                    println!("Failed to save script {}: {}", script.name, e);
                });
                script.is_dirty = false;
            }
        }
    }
    
    fn run_active_script(&mut self) {
        if let Some(active_name) = &self.active_script {
            if let Some(script) = self.scripts.get(active_name) {
                println!("Running script: {}", script.name);
                
                // Parse and execute the script
                let lexer = Lexer::new(&script.code);
                match Parser::new(lexer) {
                    Ok(mut parser) => {
                        match parser.parse_program() {
                            Ok(ast) => {
                                let mut interpreter = Interpreter::new();
                                match interpreter.eval_program(&ast) {
                                    Ok(result) => println!("Script result: {:?}", result),
                                    Err(e) => println!("Runtime error: {:?}", e),
                                }
                            },
                            Err(e) => println!("Parse error: {:?}", e),
                        }
                    },
                    Err(e) => println!("Parser creation error: {:?}", e),
                }
            }
        }
    }
    
    fn debug_active_script(&mut self) {
        // TODO: Implement debugging functionality
        println!("Debug mode not yet implemented");
    }
    
    fn check_syntax(&mut self, script_name: &str) {
        if let Some(script) = self.scripts.get_mut(script_name) {
            script.syntax_errors.clear();
            
            let lexer = Lexer::new(&script.code);
            match Parser::new(lexer) {
                Ok(mut parser) => {
                    if let Err(e) = parser.parse_program() {
                        script.syntax_errors.push(SyntaxError {
                            line: 1, // TODO: Extract line number from error
                            column: 1,
                            message: format!("{:?}", e),
                            error_type: ErrorType::Parser,
                        });
                    }
                },
                Err(e) => {
                    script.syntax_errors.push(SyntaxError {
                        line: 1,
                        column: 1,
                        message: format!("{:?}", e),
                        error_type: ErrorType::Parser,
                    });
                }
            }
        }
    }
    
    fn format_code(&mut self, script_name: &str) {
        // TODO: Implement code formatting
        println!("Code formatting not yet implemented");
    }
    
    fn generate_documentation(&mut self, script_name: &str) {
        // TODO: Implement documentation generation
        println!("Documentation generation not yet implemented");
    }
}

fn get_default_script_template() -> String {
    r#"// Matrix Language Script
// Welcome to the Matrix Language scripting environment!

// Basic variable declaration
let x = 5
let y = 10.5
let message = "Hello, Matrix!"

// Function definition
let add = (a: Int, b: Int) -> Int => a + b

// Physics example
let physicsObject = {
    position: Vec3::new(0.0, 0.0, 0.0),
    velocity: Vec3::new(1.0, 0.0, 0.0),
    mass: 1.0
}

// Matrix operations
let matrix = Mat3::identity()

// Array operations
let numbers = [1, 2, 3, 4, 5]

// Main execution
let result = add(x, 3)
"#.to_string()
}

impl Default for ScriptingPanel {
    fn default() -> Self {
        Self::new()
    }
}
