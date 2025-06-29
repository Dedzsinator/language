use eframe::egui;
// use std::collections::HashMap; // Unused for now

// Import Matrix Language components
use matrix_lang::ast::Program;
use matrix_lang::eval::Interpreter;
use matrix_lang::lexer::Lexer;
use matrix_lang::parser::Parser;

/// Scripting panel for writing and editing Matrix Language scripts
pub struct ScriptingPanel {
    scripts: std::collections::HashMap<String, ScriptEditor>,
    active_script: Option<String>,
    new_script_name: String,
    show_templates: bool,
    show_file_dialog: bool,
    // Search functionality
    search_text: String,
    search_results: Vec<SearchResult>,
    show_search_results: bool,
    replace_text: String,
    show_replace_dialog: bool,
    case_sensitive: bool,
    regex_search: bool,
    // Callback for script execution results
    pub script_execution_callback: Option<Box<dyn Fn(&str, &Program) + Send + Sync>>,
    // Track last executed script for integration with viewport
    last_executed_script: Option<Program>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub script_name: String,
    pub line: usize,
    pub column: usize,
    pub match_text: String,
    pub context_before: String,
    pub context_after: String,
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
            show_file_dialog: false,
            search_text: String::new(),
            search_results: Vec::new(),
            show_search_results: false,
            replace_text: String::new(),
            show_replace_dialog: false,
            case_sensitive: false,
            regex_search: false,
            script_execution_callback: None,
            last_executed_script: None,
        };

        // Create a default script with proper 3D object creation
        panel.create_new_script("Main".to_string(), get_default_script_template());
        panel.active_script = Some("Main".to_string());

        panel
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("scripting_panel")
            .default_width(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.show_ui_content(ui);
            });
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.show_ui_content(ui);
    }

    /// Set callback for script execution results
    pub fn set_script_execution_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, &Program) + Send + Sync + 'static,
    {
        self.script_execution_callback = Some(Box::new(callback));
    }

    /// Get the AST of the last executed script for integration with viewport
    pub fn get_last_executed_script(&mut self) -> Option<Program> {
        self.last_executed_script.take()
    }

    fn show_ui_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Script Editor");
        self.show_toolbar(ui);
        ui.separator();
        self.show_script_tabs(ui);
        ui.separator();
        self.show_active_script_editor(ui);
        ui.separator();
        self.show_status_bar(ui);

        // Show file dialog if requested
        if self.show_file_dialog {
            self.show_open_file_dialog(ui);
        }

        // Show replace dialog if requested
        if self.show_replace_dialog {
            self.show_replace_dialog_ui(ui);
        }
    }

    /// Show open file dialog
    fn show_open_file_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Open Script")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.label("Select a Matrix Language script file to open:");
                ui.separator();

                // Simple file browser (in a real implementation, you'd use native file dialogs)
                if let Ok(entries) = std::fs::read_dir(".") {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for entry in entries.flatten() {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.ends_with(".matrix") || name.ends_with(".ml") {
                                        if ui.selectable_label(false, name).clicked() {
                                            self.open_script_file(entry.path());
                                            self.show_file_dialog = false;
                                        }
                                    }
                                }
                            }
                        });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_file_dialog = false;
                    }
                });
            });
    }

    /// Open a script file from disk
    fn open_script_file(&mut self, path: std::path::PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();

                let script = ScriptEditor {
                    name: name.clone(),
                    code: content,
                    file_path: path.to_string_lossy().to_string(),
                    is_dirty: false,
                    cursor_position: 0,
                    syntax_errors: Vec::new(),
                    auto_save: false,
                };

                self.scripts.insert(name.clone(), script);
                self.active_script = Some(name);
            }
            Err(e) => {
                eprintln!("Failed to open file {:?}: {}", path, e);
            }
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📄 New").clicked() {
                self.show_new_script_dialog(ui);
            }

            if ui.button("📁 Open").clicked() {
                self.show_file_dialog = true;
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
            let search_response = ui.text_edit_singleline(&mut self.search_text);
            if search_response.changed() && !self.search_text.is_empty() {
                self.perform_search();
            }

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.case_sensitive, "Case sensitive");
                ui.checkbox(&mut self.regex_search, "Regex");

                if ui.button("Find All").clicked() {
                    self.perform_search();
                }

                if ui.button("Replace").clicked() {
                    self.show_replace_dialog = true;
                }
            });

            // Show search results if any
            if !self.search_results.is_empty() {
                ui.separator();
                ui.label(format!("Found {} matches:", self.search_results.len()));

                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        let search_results = self.search_results.clone(); // Clone to avoid borrowing issues
                        for (i, result) in search_results.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    false,
                                    format!(
                                        "{}:{} - {}",
                                        result.script_name, result.line, result.match_text
                                    ),
                                )
                                .clicked()
                            {
                                self.goto_search_result(i);
                            }
                        }
                    });
            }
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
                                    let template = "let myFunction = (param: Type) -> ReturnType => {\n    -- Function body\n    return result\n}";
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
                                    let template = "-- Create a physics object\nlet physicsObject = {\n    position: Vec3::new(0.0, 0.0, 0.0),\n    velocity: Vec3::new(0.0, 0.0, 0.0),\n    mass: 1.0\n}";
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
                // Show save dialog - for now, auto-save unsaved changes
                self.save_script(&name);
            }
        }

        self.scripts.remove(&name);

        if self.active_script.as_ref() == Some(&name) {
            self.active_script = self.scripts.keys().next().cloned();
        }
    }

    fn save_active_script(&mut self) {
        if let Some(active_name) = self.active_script.clone() {
            self.save_script(&active_name);
        }
    }

    fn save_script(&mut self, script_name: &str) {
        if let Some(script) = self.scripts.get_mut(script_name) {
            // Actually save to file
            match std::fs::write(&script.file_path, &script.code) {
                Ok(_) => {
                    script.is_dirty = false;
                    println!("Script saved: {}", script.file_path);
                }
                Err(e) => {
                    eprintln!("Failed to save script '{}': {}", script.file_path, e);
                }
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
                            // Store AST for integration with viewport
                            self.last_executed_script = Some(ast.clone());

                            // Call callback with AST for 3D object creation
                            if let Some(ref callback) = self.script_execution_callback {
                                callback(&script.name, &ast);
                            }

                            // Execute script in interpreter
                            let mut interpreter = Interpreter::new();
                            match interpreter.eval_program(&ast) {
                                Ok(result) => {
                                    println!("✅ Script executed successfully");
                                    println!("Result: {:?}", result);
                                }
                                Err(e) => {
                                    println!("❌ Runtime error: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Parse error: {:?}", e);
                        }
                    },
                    Err(e) => {
                        println!("❌ Parser creation error: {:?}", e);
                    }
                }
            }
        }
    }

    fn debug_active_script(&mut self) {
        if let Some(active_name) = &self.active_script {
            if let Some(script) = self.scripts.get(active_name) {
                println!("🐛 Debugging script: {}", script.name);

                // Parse and analyze the script for debugging
                let lexer = Lexer::new(&script.code);
                match Parser::new(lexer) {
                    Ok(mut parser) => match parser.parse_program() {
                        Ok(ast) => {
                            println!("📋 AST Analysis:");
                            println!("{:#?}", ast);

                            // Step-by-step execution with debug info
                            let mut interpreter = Interpreter::new();
                            println!("🔍 Starting step-by-step execution...");

                            match interpreter.eval_program(&ast) {
                                Ok(result) => {
                                    println!("✅ Debug execution completed successfully");
                                    println!("Result: {:?}", result);
                                }
                                Err(e) => {
                                    println!("❌ Debug execution failed:");
                                    println!("Runtime error: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Parse error during debugging:");
                            println!("{:?}", e);
                        }
                    },
                    Err(e) => {
                        println!("❌ Parser creation error during debugging:");
                        println!("{:?}", e);
                    }
                }
            }
        } else {
            println!("No active script to debug");
        }
    }

    fn check_syntax(&mut self, script_name: &str) {
        if let Some(script) = self.scripts.get_mut(script_name) {
            script.syntax_errors.clear();

            let lexer = Lexer::new(&script.code);
            match Parser::new(lexer) {
                Ok(mut parser) => {
                    if let Err(e) = parser.parse_program() {
                        let error_msg = format!("{:?}", e);
                        // Extract line number before borrowing script mutably again
                        let line_num =
                            ScriptingPanel::extract_line_number_from_error_static(&error_msg);
                        script.syntax_errors.push(SyntaxError {
                            line: line_num,
                            column: 1,
                            message: error_msg,
                            error_type: ErrorType::Parser,
                        });
                    }
                }
                Err(e) => {
                    let error_message = format!("{:?}", e);
                    let line_number = Self::extract_line_number_from_error_static(&error_message);
                    script.syntax_errors.push(SyntaxError {
                        line: line_number,
                        column: 1,
                        message: error_message,
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
                if trimmed.starts_with("--") {
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

    /// Perform search across all open scripts
    fn perform_search(&mut self) {
        self.search_results.clear();

        if self.search_text.is_empty() {
            return;
        }

        for (script_name, script) in &self.scripts {
            let lines: Vec<&str> = script.code.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                let matches = if self.regex_search {
                    self.find_regex_matches(line, line_num + 1, script_name)
                } else {
                    self.find_text_matches(line, line_num + 1, script_name)
                };

                self.search_results.extend(matches);
            }
        }

        self.show_search_results = !self.search_results.is_empty();
    }

    /// Find text matches in a line
    fn find_text_matches(
        &self,
        line: &str,
        line_num: usize,
        script_name: &str,
    ) -> Vec<SearchResult> {
        let mut matches = Vec::new();
        let search_text = if self.case_sensitive {
            self.search_text.clone()
        } else {
            self.search_text.to_lowercase()
        };

        let line_text = if self.case_sensitive {
            line.to_string()
        } else {
            line.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = line_text[start..].find(&search_text) {
            let actual_pos = start + pos;

            // Get context around the match
            let context_start = if actual_pos >= 10 { actual_pos - 10 } else { 0 };
            let context_end = std::cmp::min(actual_pos + search_text.len() + 10, line.len());

            matches.push(SearchResult {
                script_name: script_name.to_string(),
                line: line_num,
                column: actual_pos + 1,
                match_text: line[actual_pos..actual_pos + self.search_text.len()].to_string(),
                context_before: line[context_start..actual_pos].to_string(),
                context_after: line[actual_pos + self.search_text.len()..context_end].to_string(),
            });

            start = actual_pos + 1;
        }

        matches
    }

    /// Find regex matches in a line
    fn find_regex_matches(
        &self,
        line: &str,
        line_num: usize,
        script_name: &str,
    ) -> Vec<SearchResult> {
        // Simple regex implementation - in a real implementation, use regex crate
        // For now, fall back to text search
        self.find_text_matches(line, line_num, script_name)
    }

    /// Navigate to a specific search result
    fn goto_search_result(&mut self, result_index: usize) {
        if let Some(result) = self.search_results.get(result_index) {
            // Switch to the script containing the result
            self.active_script = Some(result.script_name.clone());

            // In a real implementation, we would set the cursor position to the result location
            // For now, just switch to the script
        }
    }

    /// Show replace dialog UI
    fn show_replace_dialog_ui(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Find and Replace")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.label("Find:");
                ui.text_edit_singleline(&mut self.search_text);

                ui.label("Replace with:");
                ui.text_edit_singleline(&mut self.replace_text);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.case_sensitive, "Case sensitive");
                    ui.checkbox(&mut self.regex_search, "Regex");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Find All").clicked() {
                        self.perform_search();
                    }

                    if ui.button("Replace All").clicked() && !self.search_text.is_empty() {
                        self.perform_replace_all();
                        self.show_replace_dialog = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_replace_dialog = false;
                    }
                });
            });
    }

    /// Perform replace all operation
    fn perform_replace_all(&mut self) {
        if let Some(active_name) = &self.active_script.clone() {
            if let Some(script) = self.scripts.get_mut(active_name) {
                if self.regex_search {
                    // Simple regex replacement (basic implementation)
                    script.code = script.code.replace(&self.search_text, &self.replace_text);
                } else {
                    // Simple text replacement
                    if self.case_sensitive {
                        script.code = script.code.replace(&self.search_text, &self.replace_text);
                    } else {
                        // Case-insensitive replacement (simple approach)
                        let lower_search = self.search_text.to_lowercase();
                        let mut result = String::new();
                        let mut remaining = &script.code[..];

                        while let Some(pos) = remaining.to_lowercase().find(&lower_search) {
                            result.push_str(&remaining[..pos]);
                            result.push_str(&self.replace_text);
                            remaining = &remaining[pos + self.search_text.len()..];
                        }
                        result.push_str(remaining);
                        script.code = result;
                    }
                }
                script.is_dirty = true;
            }
        }
    }

    /// Extract line number from error message (static version)
    fn extract_line_number_from_error_static(error_message: &str) -> usize {
        // Simple line number extraction without regex

        // Look for "line " followed by a number
        if let Some(line_pos) = error_message.to_lowercase().find("line ") {
            let after_line = &error_message[line_pos + 5..];
            if let Some(space_pos) = after_line.find(' ') {
                if let Ok(line_num) = after_line[..space_pos].parse::<usize>() {
                    return line_num;
                }
            } else if let Ok(line_num) = after_line.parse::<usize>() {
                return line_num;
            }
        }

        // Look for ":line_number:" pattern
        if let Some(colon_pos) = error_message.find(':') {
            let after_colon = &error_message[colon_pos + 1..];
            if let Some(end_colon) = after_colon.find(':') {
                if let Ok(line_num) = after_colon[..end_colon].parse::<usize>() {
                    return line_num;
                }
            }
        }

        1 // Default to line 1 if no line number found
    }
}

fn get_default_script_template() -> String {
    r#"-- Matrix Language Script
-- Welcome to the Matrix Language scripting environment!

-- Basic variables
let x = 5
let y = 10.5
let message = "Hello, Matrix!"

-- Function definition
let add = (a: Int, b: Int) => a + b

-- Define a 3D game object structure
struct GameObject {
    name: String,
    x: Float,
    y: Float,
    z: Float,
    r: Float,
    g: Float,
    b: Float,
    a: Float
}

-- Create a 3D cube object in the viewport
let cube = GameObject {
    name: "Cube",
    x: 0.0,
    y: 0.0,
    z: 0.0,
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0
}

-- Create a plane object
let plane = GameObject {
    name: "Plane",
    x: 0.0,
    y: -1.0,
    z: 0.0,
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0
}

-- Define physics object structure
struct PhysicsObject {
    name: String,
    x: Float,
    y: Float,
    z: Float,
    vx: Float,
    vy: Float,
    vz: Float,
    mass: Float
}

-- Physics object with mass and velocity
let physicsObject = PhysicsObject {
    name: "PhysicsObject",
    x: 3.0,
    y: 5.0,
    z: 0.0,
    vx: 0.0,
    vy: -1.0,
    vz: 0.0,
    mass: 1.0
}

-- Calculate result
let result = add(x, 3)
"#
    .to_string()
}

impl Default for ScriptingPanel {
    fn default() -> Self {
        Self::new()
    }
}
