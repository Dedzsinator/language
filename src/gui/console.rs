use super::*;

/// Console panel for logging, debugging, and command input
pub struct Console {
    log_entries: Vec<LogEntry>,
    command_history: Vec<String>,
    current_command: String,
    history_index: Option<usize>,
    auto_scroll: bool,
    show_timestamps: bool,
    filter_level: LogLevel,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: std::time::SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl LogLevel {
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Debug => egui::Color32::GRAY,
            LogLevel::Info => egui::Color32::WHITE,
            LogLevel::Warning => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warning",
            LogLevel::Error => "Error",
        }
    }
}

impl Console {
    pub fn new() -> Self {
        let mut console = Self {
            log_entries: Vec::new(),
            command_history: Vec::new(),
            current_command: String::new(),
            history_index: None,
            auto_scroll: true,
            show_timestamps: true,
            filter_level: LogLevel::Debug,
            max_entries: 1000,
        };

        // Add some initial messages
        console.log(LogLevel::Info, "Matrix Language Console", "System");
        console.log(LogLevel::Info, "Type 'help' for available commands", "System");

        console
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("console_panel")
            .default_height(200.0)
            .show(ctx, |ui| {
                ui.heading("Console");

                // Toolbar
                self.show_toolbar(ui);
                ui.separator();

                // Log area
                self.show_log_area(ui);
                ui.separator();

                // Command input
                self.show_command_input(ui);
            });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.log_entries.clear();
            }

            ui.separator();

            // Filter level
            ui.label("Filter:");
            egui::ComboBox::from_id_source("log_filter")
                .selected_text(self.filter_level.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_level, LogLevel::Debug, "Debug");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Info, "Info");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Warning, "Warning");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Error, "Error");
                });

            ui.separator();

            ui.checkbox(&mut self.auto_scroll, "Auto Scroll");
            ui.checkbox(&mut self.show_timestamps, "Timestamps");

            ui.separator();

            // Statistics
            let error_count = self.log_entries.iter().filter(|e| e.level == LogLevel::Error).count();
            let warning_count = self.log_entries.iter().filter(|e| e.level == LogLevel::Warning).count();

            if error_count > 0 {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", error_count));
            }
            if warning_count > 0 {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {}", warning_count));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Total: {}", self.log_entries.len()));
            });
        });
    }

    fn show_log_area(&mut self, ui: &mut egui::Ui) {
        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(self.auto_scroll);

        scroll_area.show(ui, |ui| {
            for entry in &self.log_entries {
                if entry.level < self.filter_level {
                    continue;
                }

                ui.horizontal(|ui| {
                    // Timestamp
                    if self.show_timestamps {
                        let timestamp = format_timestamp(entry.timestamp);
                        ui.colored_label(egui::Color32::GRAY, timestamp);
                    }

                    // Level icon and source
                    ui.label(entry.level.icon());
                    ui.colored_label(egui::Color32::GRAY, format!("[{}]", entry.source));

                    // Message
                    ui.colored_label(entry.level.color(), &entry.message);
                });
            }
        });
    }

    fn show_command_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(">");

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.current_command)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter command...")
            );

            // Handle command input
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_command();
                response.request_focus();
            }

            // Handle command history navigation
            if response.has_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.navigate_history_up();
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.navigate_history_down();
                }
            }

            if ui.button("Send").clicked() {
                self.execute_command();
            }
        });
    }

    fn execute_command(&mut self) {
        if self.current_command.trim().is_empty() {
            return;
        }

        let command = self.current_command.trim().to_string();

        // Add to history
        self.command_history.push(command.clone());
        self.history_index = None;

        // Log the command
        self.log(LogLevel::Info, &format!("> {}", command), "User");

        // Execute the command
        self.process_command(&command);

        // Clear input
        self.current_command.clear();
    }

    fn process_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0].to_lowercase().as_str() {
            "help" => {
                self.log(LogLevel::Info, "Available commands:", "System");
                self.log(LogLevel::Info, "  help - Show this help message", "System");
                self.log(LogLevel::Info, "  clear - Clear the console", "System");
                self.log(LogLevel::Info, "  echo <message> - Echo a message", "System");
                self.log(LogLevel::Info, "  run <script> - Run a Matrix Language script", "System");
                self.log(LogLevel::Info, "  scene <name> - Switch to scene", "System");
                self.log(LogLevel::Info, "  spawn <object> - Spawn an object", "System");
                self.log(LogLevel::Info, "  debug <on|off> - Toggle debug mode", "System");
                self.log(LogLevel::Info, "  fps - Show performance information", "System");
            },

            "clear" => {
                self.log_entries.clear();
            },

            "echo" => {
                if parts.len() > 1 {
                    let message = parts[1..].join(" ");
                    self.log(LogLevel::Info, &message, "Echo");
                } else {
                    self.log(LogLevel::Warning, "echo requires a message", "System");
                }
            },

            "run" => {
                if parts.len() > 1 {
                    let script_name = parts[1];
                    self.log(LogLevel::Info, &format!("Running script: {}", script_name), "System");
                    // TODO: Actually run the script
                    self.log(LogLevel::Info, "Script execution completed", "System");
                } else {
                    self.log(LogLevel::Warning, "run requires a script name", "System");
                }
            },

            "scene" => {
                if parts.len() > 1 {
                    let scene_name = parts[1];
                    self.log(LogLevel::Info, &format!("Switching to scene: {}", scene_name), "System");
                    // TODO: Actually switch scene
                } else {
                    self.log(LogLevel::Warning, "scene requires a scene name", "System");
                }
            },

            "spawn" => {
                if parts.len() > 1 {
                    let object_type = parts[1];
                    self.log(LogLevel::Info, &format!("Spawning object: {}", object_type), "System");
                    // TODO: Actually spawn object
                } else {
                    self.log(LogLevel::Warning, "spawn requires an object type", "System");
                }
            },

            "debug" => {
                if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "on" | "true" | "1" => {
                            self.log(LogLevel::Info, "Debug mode enabled", "System");
                            // TODO: Enable debug mode
                        },
                        "off" | "false" | "0" => {
                            self.log(LogLevel::Info, "Debug mode disabled", "System");
                            // TODO: Disable debug mode
                        },
                        _ => {
                            self.log(LogLevel::Warning, "debug requires 'on' or 'off'", "System");
                        }
                    }
                } else {
                    self.log(LogLevel::Warning, "debug requires 'on' or 'off'", "System");
                }
            },

            "fps" => {
                self.log(LogLevel::Info, "Performance Information:", "System");
                self.log(LogLevel::Info, "  FPS: 60", "System"); // TODO: Get actual FPS
                self.log(LogLevel::Info, "  Frame Time: 16.7ms", "System");
                self.log(LogLevel::Info, "  Memory Usage: 128MB", "System");
            },

            _ => {
                // Try to execute as Matrix Language code
                if command.contains("=") || command.contains("let") {
                    self.log(LogLevel::Info, "Executing Matrix Language code...", "Interpreter");

                    // TODO: Execute with Matrix Language interpreter
                    let lexer = Lexer::new(command);
                    match Parser::new(lexer) {
                        Ok(mut parser) => {
                            match parser.parse_expression() {
                                Ok(ast) => {
                                    self.log(LogLevel::Debug, &format!("AST: {:?}", ast), "Parser");

                                    let mut interpreter = Interpreter::new();
                                    match interpreter.eval_expression(&ast) {
                                        Ok(result) => {
                                            self.log(LogLevel::Info, &format!("Result: {:?}", result), "Interpreter");
                                        },
                                        Err(e) => {
                                            self.log(LogLevel::Error, &format!("Runtime error: {:?}", e), "Interpreter");
                                        }
                                    }
                                },
                                Err(e) => {
                                    self.log(LogLevel::Error, &format!("Parse error: {:?}", e), "Parser");
                                }
                            }
                        },
                        Err(e) => {
                            self.log(LogLevel::Error, &format!("Parser creation error: {:?}", e), "Parser");
                        }
                    }
                } else {
                    self.log(LogLevel::Warning, &format!("Unknown command: {}", parts[0]), "System");
                    self.log(LogLevel::Info, "Type 'help' for available commands", "System");
                }
            }
        }
    }

    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
            },
            Some(index) => {
                if index > 0 {
                    self.history_index = Some(index - 1);
                }
            }
        }

        if let Some(index) = self.history_index {
            self.current_command = self.command_history[index].clone();
        }
    }

    fn navigate_history_down(&mut self) {
        if let Some(index) = self.history_index {
            if index < self.command_history.len() - 1 {
                self.history_index = Some(index + 1);
                self.current_command = self.command_history[self.history_index.unwrap()].clone();
            } else {
                self.history_index = None;
                self.current_command.clear();
            }
        }
    }

    pub fn log(&mut self, level: LogLevel, message: &str, source: &str) {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now(),
            level,
            message: message.to_string(),
            source: source.to_string(),
        };

        self.log_entries.push(entry);

        // Limit the number of entries to prevent memory issues
        if self.log_entries.len() > self.max_entries {
            self.log_entries.remove(0);
        }
    }

    pub fn log_info(&mut self, message: &str, source: &str) {
        self.log(LogLevel::Info, message, source);
    }

    pub fn log_warning(&mut self, message: &str, source: &str) {
        self.log(LogLevel::Warning, message, source);
    }

    pub fn log_error(&mut self, message: &str, source: &str) {
        self.log(LogLevel::Error, message, source);
    }

    pub fn log_debug(&mut self, message: &str, source: &str) {
        self.log(LogLevel::Debug, message, source);
    }
}

fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    let duration = timestamp.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
