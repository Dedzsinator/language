use super::*;
use crate::eval::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

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
    pub scene_callback: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    pub spawn_callback: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    debug_mode: bool,
    frame_times: std::collections::VecDeque<std::time::Instant>,
    last_frame_time: std::time::Instant,
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
            scene_callback: None,
            spawn_callback: None,
            debug_mode: false,
            frame_times: std::collections::VecDeque::with_capacity(60),
            last_frame_time: std::time::Instant::now(),
        };

        // Add some initial messages
        console.log(LogLevel::Info, "Matrix Language Console", "System");
        console.log(
            LogLevel::Info,
            "Type 'help' for available commands",
            "System",
        );

        console
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("console_panel")
            .default_height(200.0)
            .show(ctx, |ui| {
                self.show_ui_content(ui);
            });
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.show_ui_content(ui);
    }

    fn show_ui_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Console");

        // Toolbar
        self.show_toolbar(ui);
        ui.separator();

        // Log area
        self.show_log_area(ui);
        ui.separator();

        // Command input
        self.show_command_input(ui);
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.log_entries.clear();
            }

            ui.separator();

            // Filter level
            ui.label("Filter:");
            egui::ComboBox::from_id_salt("log_filter")
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
            let error_count = self
                .log_entries
                .iter()
                .filter(|e| e.level == LogLevel::Error)
                .count();
            let warning_count = self
                .log_entries
                .iter()
                .filter(|e| e.level == LogLevel::Warning)
                .count();

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
                    .hint_text("Enter command..."),
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
                self.log(
                    LogLevel::Info,
                    "  echo <message> - Echo a message",
                    "System",
                );
                self.log(
                    LogLevel::Info,
                    "  run <script> - Run a Matrix Language script",
                    "System",
                );
                self.log(LogLevel::Info, "  scene <name> - Switch to scene", "System");
                self.log(
                    LogLevel::Info,
                    "  spawn <object> - Spawn an object",
                    "System",
                );
                self.log(
                    LogLevel::Info,
                    "  debug <on|off> - Toggle debug mode",
                    "System",
                );
                self.log(
                    LogLevel::Info,
                    "  fps - Show performance information",
                    "System",
                );
            }

            "clear" => {
                self.log_entries.clear();
            }

            "echo" => {
                if parts.len() > 1 {
                    let message = parts[1..].join(" ");
                    self.log(LogLevel::Info, &message, "Echo");
                } else {
                    self.log(LogLevel::Warning, "echo requires a message", "System");
                }
            }

            "run" => {
                if parts.len() > 1 {
                    let script_name = parts[1];
                    self.log(
                        LogLevel::Info,
                        &format!("Running script: {}", script_name),
                        "System",
                    );
                    // Execute the script file
                    match self.execute_script(script_name) {
                        Ok(result) => {
                            self.log(
                                LogLevel::Info,
                                &format!("Script result: {}", result),
                                "System",
                            );
                        }
                        Err(error) => {
                            self.log(
                                LogLevel::Error,
                                &format!("Script error: {}", error),
                                "System",
                            );
                        }
                    }
                } else {
                    self.log(LogLevel::Warning, "run requires a script name", "System");
                }
            }

            "scene" => {
                if parts.len() > 1 {
                    let scene_name = parts[1];
                    if let Some(ref callback) = self.scene_callback {
                        if callback(scene_name) {
                            self.log(
                                LogLevel::Info,
                                &format!("Switched to scene: {}", scene_name),
                                "System",
                            );
                        } else {
                            self.log(
                                LogLevel::Warning,
                                &format!("Failed to switch to scene: {}", scene_name),
                                "System",
                            );
                        }
                    } else {
                        self.log(LogLevel::Warning, "Scene switching not available", "System");
                    }
                } else {
                    self.log(LogLevel::Warning, "scene requires a scene name", "System");
                }
            }

            "spawn" => {
                if parts.len() > 1 {
                    let object_type = parts[1];
                    if let Some(ref callback) = self.spawn_callback {
                        if callback(object_type) {
                            self.log(
                                LogLevel::Info,
                                &format!("Spawned object: {}", object_type),
                                "System",
                            );
                        } else {
                            self.log(
                                LogLevel::Warning,
                                &format!("Failed to spawn object: {}", object_type),
                                "System",
                            );
                        }
                    } else {
                        self.log(LogLevel::Warning, "Object spawning not available", "System");
                    }
                } else {
                    self.log(LogLevel::Warning, "spawn requires an object type", "System");
                }
            }

            "debug" => {
                if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "on" | "true" | "1" => {
                            self.set_debug_mode(true);
                        }
                        "off" | "false" | "0" => {
                            self.set_debug_mode(false);
                        }
                        _ => {
                            self.log(LogLevel::Warning, "debug requires 'on' or 'off'", "System");
                        }
                    }
                } else {
                    self.log(LogLevel::Warning, "debug requires 'on' or 'off'", "System");
                }
            }

            "fps" => {
                let current_fps = self.calculate_fps();
                let frame_time_ms = 1000.0 / current_fps;

                self.log(LogLevel::Info, "Performance Information:", "System");
                self.log(
                    LogLevel::Info,
                    &format!("  FPS: {:.1}", current_fps),
                    "System",
                );
                self.log(
                    LogLevel::Info,
                    &format!("  Frame Time: {:.1}ms", frame_time_ms),
                    "System",
                );

                // Get memory usage (approximate)
                let memory_usage = self.log_entries.len() * std::mem::size_of::<LogEntry>()
                    + self.command_history.iter().map(|s| s.len()).sum::<usize>()
                    + self.frame_times.len() * std::mem::size_of::<std::time::Instant>();
                self.log(
                    LogLevel::Info,
                    &format!("  Console Memory: {:.1}KB", memory_usage as f32 / 1024.0),
                    "System",
                );
            }

            _ => {
                // Try to execute as Matrix Language code
                if command.contains("=") || command.contains("let") {
                    self.log(
                        LogLevel::Info,
                        "Executing Matrix Language code...",
                        "Interpreter",
                    );

                    // Execute with Matrix Language interpreter
                    let lexer = Lexer::new(command);
                    match Parser::new(lexer) {
                        Ok(mut parser) => match parser.parse_expression() {
                            Ok(ast) => {
                                if self.debug_mode {
                                    self.log(LogLevel::Debug, &format!("AST: {:?}", ast), "Parser");
                                }

                                let mut interpreter = Interpreter::new();
                                match interpreter.eval_expression(&ast) {
                                    Ok(result) => {
                                        self.log(
                                            LogLevel::Info,
                                            &format!("Result: {:?}", result),
                                            "Interpreter",
                                        );
                                    }
                                    Err(e) => {
                                        self.log(
                                            LogLevel::Error,
                                            &format!("Runtime error: {:?}", e),
                                            "Interpreter",
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                self.log(
                                    LogLevel::Error,
                                    &format!("Parse error: {:?}", e),
                                    "Parser",
                                );
                            }
                        },
                        Err(e) => {
                            self.log(
                                LogLevel::Error,
                                &format!("Parser creation error: {:?}", e),
                                "Parser",
                            );
                        }
                    }
                } else {
                    self.log(
                        LogLevel::Warning,
                        &format!("Unknown command: {}", parts[0]),
                        "System",
                    );
                    self.log(
                        LogLevel::Info,
                        "Type 'help' for available commands",
                        "System",
                    );
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
            }
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
        // Skip debug messages when debug mode is disabled
        if level == LogLevel::Debug && !self.debug_mode {
            return;
        }

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

    /// Execute a Matrix Language script
    fn execute_script(&mut self, script_name: &str) -> Result<String, String> {
        let script_path = if script_name.ends_with(".matrix") {
            script_name.to_string()
        } else {
            format!("{}.matrix", script_name)
        };

        // Try to read the script file
        let script_content = match std::fs::read_to_string(&script_path) {
            Ok(content) => content,
            Err(_) => {
                // Try in current directory
                match std::fs::read_to_string(&format!("./{}", script_path)) {
                    Ok(content) => content,
                    Err(e) => {
                        return Err(format!(
                            "Could not read script file '{}': {}",
                            script_path, e
                        ))
                    }
                }
            }
        };

        // Parse and execute the script
        let lexer = Lexer::new(&script_content);

        let mut parser = Parser::new(lexer).map_err(|e| format!("Parser init error: {:?}", e))?;
        let ast = parser
            .parse_program()
            .map_err(|e| format!("Parser error: {:?}", e))?;

        let mut interpreter = Interpreter::new();
        let result = interpreter
            .eval_program(&ast)
            .map_err(|e| format!("Runtime error: {:?}", e))?;

        // Convert result to string for display
        Ok(format!("{:?}", result))
    }

    /// Set scene switch callback
    pub fn set_scene_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.scene_callback = Some(Box::new(callback));
    }

    /// Set object spawn callback
    pub fn set_spawn_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.spawn_callback = Some(Box::new(callback));
    }

    /// Update FPS tracking - should be called every frame
    pub fn update_fps(&mut self) {
        let now = std::time::Instant::now();
        self.frame_times.push_back(now);

        // Keep only last 60 frames for FPS calculation
        while self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }

        self.last_frame_time = now;
    }

    /// Calculate current FPS based on recent frame times
    fn calculate_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 60.0; // Default fallback
        }

        let duration = self
            .frame_times
            .back()
            .unwrap()
            .duration_since(*self.frame_times.front().unwrap());

        if duration.as_secs_f32() > 0.0 {
            (self.frame_times.len() - 1) as f32 / duration.as_secs_f32()
        } else {
            60.0
        }
    }

    /// Get current debug mode status
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// Set debug mode and log the change
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
        if enabled {
            self.log(LogLevel::Info, "Debug logging enabled", "System");
            self.log(LogLevel::Debug, "This is a debug message", "System");
        } else {
            self.log(LogLevel::Info, "Debug logging disabled", "System");
        }
    }
}

fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    let duration = timestamp
        .duration_since(std::time::UNIX_EPOCH)
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
