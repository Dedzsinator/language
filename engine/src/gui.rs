// Main Physics Simulation GUI Module
// Unity-style physics simulation interface

use eframe::egui;

/// Main Physics Editor Application
pub struct PhysicsEditorApp {
    /// Menu bar state
    show_about: bool,
    show_preferences: bool,
    /// Toolbar state
    is_playing: bool,
    is_paused: bool,
    /// Console messages
    console_messages: Vec<String>,
    /// Selected object
    selected_object: Option<String>,
}

impl PhysicsEditorApp {
    pub fn new() -> Self {
        Self {
            show_about: false,
            show_preferences: false,
            is_playing: false,
            is_paused: false,
            console_messages: vec!["Physics Editor Started".to_string()],
            selected_object: None,
        }
    }

    fn add_console_message(&mut self, message: String) {
        self.console_messages.push(message);
        if self.console_messages.len() > 100 {
            self.console_messages.remove(0);
        }
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        self.add_console_message("Created new scene".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Open Scene").clicked() {
                        self.add_console_message("Opening scene...".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Save Scene").clicked() {
                        self.add_console_message("Scene saved".to_string());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Preferences").clicked() {
                        self.show_preferences = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Empty").clicked() {
                        self.add_console_message("Created empty GameObject".to_string());
                        ui.close_menu();
                    }
                    if ui.button("3D Object").clicked() {
                        self.add_console_message("Created 3D Object".to_string());
                        ui.close_menu();
                    }
                });

                ui.menu_button("Component", |ui| {
                    if ui.button("Physics").clicked() {
                        self.add_console_message("Added Physics component".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Renderer").clicked() {
                        self.add_console_message("Added Renderer component".to_string());
                        ui.close_menu();
                    }
                });

                ui.menu_button("Window", |ui| {
                    if ui.button("Physics Debugger").clicked() {
                        self.add_console_message("Opened Physics Debugger".to_string());
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // Play controls
                let play_text = if self.is_playing { "⏸" } else { "▶" };
                if ui.button(play_text).clicked() {
                    self.is_playing = !self.is_playing;
                    let status = if self.is_playing {
                        "started"
                    } else {
                        "stopped"
                    };
                    self.add_console_message(format!("Simulation {}", status));
                }

                if ui.button("⏹").clicked() {
                    self.is_playing = false;
                    self.is_paused = false;
                    self.add_console_message("Simulation stopped".to_string());
                }

                if ui.button("⏭").clicked() {
                    self.add_console_message("Advanced one frame".to_string());
                }

                ui.separator();

                // Transform tools
                if ui.selectable_label(false, "📍").clicked() {
                    self.add_console_message("Selected move tool".to_string());
                }
                if ui.selectable_label(false, "🔄").clicked() {
                    self.add_console_message("Selected rotate tool".to_string());
                }
                if ui.selectable_label(false, "📏").clicked() {
                    self.add_console_message("Selected scale tool".to_string());
                }

                ui.separator();

                // View options
                if ui.button("🔍").clicked() {
                    self.add_console_message("Frame selected".to_string());
                }
            });
        });
    }

    fn show_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| 1.0 / i.stable_dt)));
                ui.separator();
                ui.label("Objects: 0");
            });
        });
    }

    fn show_main_layout(&mut self, ctx: &egui::Context) {
        // Left panel - Hierarchy
        egui::SidePanel::left("hierarchy")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");
                ui.separator();

                if ui
                    .selectable_label(
                        self.selected_object.as_deref() == Some("Main Camera"),
                        "📷 Main Camera",
                    )
                    .clicked()
                {
                    self.selected_object = Some("Main Camera".to_string());
                }
                if ui
                    .selectable_label(
                        self.selected_object.as_deref() == Some("Directional Light"),
                        "💡 Directional Light",
                    )
                    .clicked()
                {
                    self.selected_object = Some("Directional Light".to_string());
                }
                if ui
                    .selectable_label(self.selected_object.as_deref() == Some("Cube"), "📦 Cube")
                    .clicked()
                {
                    self.selected_object = Some("Cube".to_string());
                }
            });

        // Right panel - Inspector
        egui::SidePanel::right("inspector")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                if let Some(ref selected) = self.selected_object {
                    ui.label(format!("Selected: {}", selected));
                    ui.separator();

                    match selected.as_str() {
                        "Main Camera" => {
                            ui.label("Transform");
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("X: "));
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Y: "));
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Z: "));
                            });
                            ui.separator();
                            ui.label("Camera");
                            ui.horizontal(|ui| {
                                ui.label("Field of View:");
                                ui.add(egui::Slider::new(&mut 60.0f32, 1.0..=179.0));
                            });
                        }
                        "Cube" => {
                            ui.label("Transform");
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("X: "));
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Y: "));
                                ui.add(egui::DragValue::new(&mut 0.0f32).prefix("Z: "));
                            });
                            ui.separator();
                            ui.label("Mesh Renderer");
                            ui.checkbox(&mut true, "Cast Shadows");
                            ui.checkbox(&mut true, "Receive Shadows");
                        }
                        _ => {
                            ui.label("Select an object to view its properties");
                        }
                    }
                } else {
                    ui.label("Select an object to view its properties");
                }
            });

        // Bottom panel - Console
        egui::TopBottomPanel::bottom("console")
            .default_height(150.0)
            .show(ctx, |ui| {
                ui.heading("Console");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for message in &self.console_messages {
                        ui.label(message);
                    }
                });
            });

        // Central panel - Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Scene View");
            ui.separator();

            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::drag());

            // Draw a simple 3D-ish scene background
            let rect = response.rect;
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(64, 64, 64));

            // Draw grid
            let grid_color = egui::Color32::from_rgb(128, 128, 128);
            for i in 0..20 {
                let x = rect.left() + (i as f32 / 19.0) * rect.width();
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(1.0, grid_color),
                );
            }
            for i in 0..15 {
                let y = rect.top() + (i as f32 / 14.0) * rect.height();
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0, grid_color),
                );
            }

            // Draw a simple wireframe cube in the center
            let center = rect.center();
            let cube_size = 50.0;

            // Front face
            let front_corners = [
                egui::pos2(center.x - cube_size / 2.0, center.y - cube_size / 2.0), // top-left
                egui::pos2(center.x + cube_size / 2.0, center.y - cube_size / 2.0), // top-right
                egui::pos2(center.x + cube_size / 2.0, center.y + cube_size / 2.0), // bottom-right
                egui::pos2(center.x - cube_size / 2.0, center.y + cube_size / 2.0), // bottom-left
            ];

            // Draw front face
            let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
            painter.line_segment([front_corners[0], front_corners[1]], stroke);
            painter.line_segment([front_corners[1], front_corners[2]], stroke);
            painter.line_segment([front_corners[2], front_corners[3]], stroke);
            painter.line_segment([front_corners[3], front_corners[0]], stroke);

            // Back face (with offset for 3D effect)
            let offset = egui::vec2(20.0, -20.0);
            let back_corners = [
                front_corners[0] + offset,
                front_corners[1] + offset,
                front_corners[2] + offset,
                front_corners[3] + offset,
            ];

            // Draw back face
            let back_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
            painter.line_segment([back_corners[0], back_corners[1]], back_stroke);
            painter.line_segment([back_corners[1], back_corners[2]], back_stroke);
            painter.line_segment([back_corners[2], back_corners[3]], back_stroke);
            painter.line_segment([back_corners[3], back_corners[0]], back_stroke);

            // Connect front and back faces
            for i in 0..4 {
                painter.line_segment([front_corners[i], back_corners[i]], back_stroke);
            }

            if response.clicked() {
                self.add_console_message("Clicked in viewport".to_string());
            }
        });
    }
}

impl eframe::App for PhysicsEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        self.show_menu_bar(ctx);

        // Toolbar
        self.show_toolbar(ctx);

        // Status bar
        self.show_status_bar(ctx);

        // Main Unity-style layout
        self.show_main_layout(ctx);

        // Modal dialogs
        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Unity-Style Physics Engine v0.1.0");
                    ui.label("Built with egui and eframe");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }

        if self.show_preferences {
            egui::Window::new("Preferences")
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.label("Application Preferences");
                    ui.separator();

                    ui.checkbox(&mut false, "Show grid by default");
                    ui.checkbox(&mut false, "Auto-save scenes");

                    if ui.button("Close").clicked() {
                        self.show_preferences = false;
                    }
                });
        }
    }
}

/// Launch the Physics Editor application
pub fn launch_physics_editor() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Unity-Style Physics Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Physics Editor",
        options,
        Box::new(|_cc| Ok(Box::new(PhysicsEditorApp::new()))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
