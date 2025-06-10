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
        egui::SidePanel::right("scripting_panel")
            .default_width(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Script Editor");
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
                    let is_dirty = self
                        .scripts
                        .get(&script_name)
                        .map(|s| s.is_dirty)
                        .unwrap_or(false);

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
            // Get script information first to avoid borrowing conflicts
            let script_info = if let Some(script) = self.scripts.get(&active_name) {
                Some((
                    script.name.clone(),
                    script.code.clone(),
                    script.file_path.clone(),
                    script.syntax_errors.clone(),
                    script.auto_save,
                    script.is_dirty,
                    script.cursor_position,
                ))
            } else {
                None
            };

            if let Some((
                name,
                mut code,
                file_path,
                syntax_errors,
                mut auto_save,
                mut is_dirty,
                mut cursor_position,
            )) = script_info
            {
                let mut code_changed = false;
                let mut format_requested = false;
                let mut check_syntax_requested = false;
                let mut generate_docs_requested = false;

                ui.columns(2, |columns| {
                    // Code editor
                    columns[0].vertical(|ui| {
                        ui.heading(&format!("Editing: {}", name));

                        // Line numbers and code editor
                        egui::ScrollArea::both()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let lines = code.lines().count().max(1);
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
                                        egui::TextEdit::multiline(&mut code)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(25)
                                    );

                                    if response.changed() {
                                        is_dirty = true;
                                        code_changed = true;
                                    }

                                    // Track cursor position if the text editor has focus
                                    if response.has_focus() {
                                        if let Some(cursor_range) = response.ctx.input(|i| i.events.iter().find_map(|event| {
                                            if let egui::Event::Text(text) = event {
                                                Some(text.len())
                                            } else {
                                                None
                                            }
                                        })) {
                                            // Simple cursor position tracking
                                            cursor_position = code.len().min(cursor_position + cursor_range);
                                        }
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
                            ui.label(format!("Path: {}", file_path));
                            ui.label(format!("Lines: {}", code.lines().count()));
                            ui.label(format!("Characters: {}", code.len()));
                            ui.label(format!("Cursor Position: {}", cursor_position));
                            ui.checkbox(&mut auto_save, "Auto Save");
                        });

                        ui.separator();

                        // Syntax errors
                        ui.group(|ui| {
                            ui.label("Syntax Errors");
                            if syntax_errors.is_empty() {
                                ui.label("✅ No errors");
                            } else {
                                egui::ScrollArea::vertical()
                                    .max_height(150.0)
                                    .show(ui, |ui| {
                                        for error in &syntax_errors {
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
                                    code.push_str(template);
                                    is_dirty = true;
                                    code_changed = true;
                                }
                                if ui.button("Class Template").clicked() {
                                    let template = "struct MyStruct {\n    field: Type\n}\n\nimpl MyStruct {\n    let new = (field: Type) -> MyStruct => {\n        MyStruct { field }\n    }\n}";
                                    code.push_str(template);
                                    is_dirty = true;
                                    code_changed = true;
                                }
                                if ui.button("Physics Object").clicked() {
                                    let template = "// Create a physics object\nlet physicsObject = {\n    position: Vec3::new(0.0, 0.0, 0.0),\n    velocity: Vec3::new(0.0, 0.0, 0.0),\n    mass: 1.0\n}";
                                    code.push_str(template);
                                    is_dirty = true;
                                    code_changed = true;
                                }
                            });
                        }

                        ui.separator();

                        // Script actions
                        ui.group(|ui| {
                            ui.label("Actions");
                            if ui.button("Format Code").clicked() {
                                format_requested = true;
                            }
                            if ui.button("Check Syntax").clicked() {
                                check_syntax_requested = true;
                            }
                            if ui.button("Generate Documentation").clicked() {
                                generate_docs_requested = true;
                            }
                        });
                    });
                });

                // Apply changes back to the script
                if let Some(script) = self.scripts.get_mut(&active_name) {
                    script.code = code;
                    script.auto_save = auto_save;
                    script.is_dirty = is_dirty;
                    script.cursor_position = cursor_position;
                }

                // Handle deferred actions
                if format_requested {
                    self.format_code(&active_name);
                }
                if check_syntax_requested {
                    self.check_syntax(&active_name);
                }
                if generate_docs_requested {
                    self.generate_documentation(&active_name);
                }
            }
        } else {
            ui.label("No script selected. Create a new script or open an existing one.");
        }
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(active_name) = &self.active_script {
                if let Some(script) = self.scripts.get(active_name) {
                    ui.label(format!(
                        "Lines: {} | Characters: {}",
                        script.code.lines().count(),
                        script.code.len()
                    ));

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
                    self.create_new_script(
                        self.new_script_name.clone(),
                        get_default_script_template(),
                    );
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
                    Ok(mut parser) => match parser.parse_program() {
                        Ok(ast) => {
                            let mut interpreter = Interpreter::new();
                            match interpreter.eval_program(&ast) {
                                Ok(result) => println!("Script result: {:?}", result),
                                Err(e) => println!("Runtime error: {:?}", e),
                            }
                        }
                        Err(e) => println!("Parse error: {:?}", e),
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
                }
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
        if let Some(script) = self.scripts.get_mut(script_name) {
            // Basic code formatting - normalize whitespace and indentation
            let formatted = script
                .code
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .map(|line| {
                    // Add basic indentation for blocks
                    if line.starts_with('}') {
                        format!("{}", line)
                    } else if line.ends_with('{') {
                        format!("{}", line)
                    } else {
                        format!("    {}", line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            script.code = formatted;
            script.is_dirty = true;
        }
    }

    fn generate_documentation(&mut self, script_name: &str) {
        if let Some(script) = self.scripts.get(script_name) {
            // Generate basic documentation from comments and function signatures
            let mut docs = format!("# Documentation for {}\n\n", script_name);

            for (i, line) in script.code.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    docs.push_str(&format!("Line {}: {}\n", i + 1, trimmed));
                } else if trimmed.starts_with("let") && trimmed.contains("=>") {
                    docs.push_str(&format!("Function found at line {}: {}\n", i + 1, trimmed));
                } else if trimmed.starts_with("struct") {
                    docs.push_str(&format!("Struct found at line {}: {}\n", i + 1, trimmed));
                }
            }

            // Create documentation as a new script
            let doc_name = format!("{}_docs", script_name);
            let doc_script = ScriptEditor {
                name: doc_name.clone(),
                code: docs,
                file_path: format!("{}.md", doc_name),
                is_dirty: true,
                auto_save: false,
                syntax_errors: vec![],
                cursor_position: 0,
            };

            self.scripts.insert(doc_name.clone(), doc_script);
            self.active_script = Some(doc_name);
        }
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
"#
    .to_string()
}

impl Default for ScriptingPanel {
    fn default() -> Self {
        Self::new()
    }
}
