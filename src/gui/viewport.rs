use super::*;

/// Viewport panel for 2D and 3D scene rendering
pub struct Viewport {
    view_mode: ViewMode,
    camera_position: Vec3,
    camera_rotation: Vec3,
    zoom_level: f32,
    show_grid: bool,
    show_gizmos: bool,
    grid_size: f32,
    background_color: [f32; 4],
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Scene3D,
            camera_position: Vec3::new(0.0, 5.0, 10.0),
            camera_rotation: Vec3::new(-20.0, 0.0, 0.0),
            zoom_level: 1.0,
            show_grid: true,
            show_gizmos: true,
            grid_size: 1.0,
            background_color: [0.2, 0.3, 0.8, 1.0],
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, scene: &Scene, selected_object: Option<u32>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Viewport toolbar
            self.show_viewport_toolbar(ui, scene);

            ui.separator();

            // Main viewport area
            self.show_viewport_content(ui, scene, selected_object);
        });
    }

    fn show_viewport_toolbar(&mut self, ui: &mut egui::Ui, scene: &Scene) {
        ui.horizontal(|ui| {
            // View mode buttons
            ui.label("View:");
            ui.selectable_value(&mut self.view_mode, ViewMode::Scene2D, "2D");
            ui.selectable_value(&mut self.view_mode, ViewMode::Scene3D, "3D");
            ui.selectable_value(&mut self.view_mode, ViewMode::Game2D, "Game 2D");
            ui.selectable_value(&mut self.view_mode, ViewMode::Game3D, "Game 3D");

            ui.separator();

            // View options
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_gizmos, "Gizmos");

            ui.separator();

            // Camera controls
            if ui.button("Reset Camera").clicked() {
                self.reset_camera();
            }

            ui.label(format!("Zoom: {:.1}x", self.zoom_level));

            ui.separator();

            // Lighting options
            ui.menu_button("Lighting", |ui| {
                if ui.button("Realistic").clicked() {
                    ui.close_menu();
                }
                if ui.button("Flat").clicked() {
                    ui.close_menu();
                }
                if ui.button("Wireframe").clicked() {
                    ui.close_menu();
                }
            });

            ui.separator();

            // Performance info
            let object_count = scene.objects.len();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("FPS: {:.0}", 1.0 / ui.ctx().input(|i| i.stable_dt)));
                ui.separator();
                ui.label(format!("Objects: {}", object_count));
            });
        });
    }

    fn show_viewport_content(&mut self, ui: &mut egui::Ui, scene: &Scene, selected_object: Option<u32>) {
        let available_size = ui.available_size();

        // Create a custom paint widget for the 3D viewport
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        // Handle mouse input for camera controls
        if response.dragged() {
            let delta = response.drag_delta();
            match self.view_mode {
                ViewMode::Scene2D | ViewMode::Game2D => {
                    // Pan in 2D
                    self.camera_position.x -= (delta.x * 0.01) as f64;
                    self.camera_position.y += (delta.y * 0.01) as f64;
                },
                ViewMode::Scene3D | ViewMode::Game3D => {
                    // Rotate camera in 3D
                    if ui.input(|i| i.modifiers.shift) {
                        // Pan
                        let right = Vec3::new(1.0, 0.0, 0.0); // Simplified right vector
                        let up = Vec3::new(0.0, 1.0, 0.0);
                        self.camera_position = self.camera_position
                            - right * ((delta.x * 0.01) as f64)
                            + up * ((delta.y * 0.01) as f64);
                    } else {
                        // Rotate
                        self.camera_rotation.y += (delta.x * 0.5) as f64;
                        self.camera_rotation.x -= (delta.y * 0.5) as f64;
                        self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                    }
                }
            }
        }

        // Handle zoom with mouse wheel
        ui.input(|i| {
            if response.hovered() {
                let scroll_delta = i.scroll_delta.y;
                if scroll_delta != 0.0 {
                    self.zoom_level *= 1.0 + scroll_delta * 0.001;
                    self.zoom_level = self.zoom_level.clamp(0.1, 10.0);
                }
            }
        });

        // Clear background
        painter.rect_filled(
            response.rect,
            egui::Rounding::ZERO,
            egui::Color32::from_rgba_unmultiplied(
                (self.background_color[0] * 255.0) as u8,
                (self.background_color[1] * 255.0) as u8,
                (self.background_color[2] * 255.0) as u8,
                (self.background_color[3] * 255.0) as u8,
            )
        );

        // Draw grid if enabled
        if self.show_grid {
            self.draw_grid(&painter, response.rect);
        }

        // Draw scene objects
        self.draw_scene_objects(&painter, response.rect, scene, selected_object);

        // Draw gizmos if enabled
        if self.show_gizmos && selected_object.is_some() {
            self.draw_gizmos(&painter, response.rect, scene, selected_object.unwrap());
        }

        // Handle right-click context menu for creating objects
        response.context_menu(|ui| {
            ui.label("Create:");
            if ui.button("Cube").clicked() {
                ui.close_menu();
                // TODO: Create cube at mouse position
            }
            if ui.button("Sphere").clicked() {
                ui.close_menu();
                // TODO: Create sphere at mouse position
            }
            if ui.button("Light").clicked() {
                ui.close_menu();
                // TODO: Create light at mouse position
            }
        });
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let center = rect.center();
        let grid_spacing = self.grid_size * self.zoom_level * 20.0;

        // Draw grid lines
        let grid_color = egui::Color32::from_rgba_unmultiplied(128, 128, 128, 64);

        // Vertical lines
        let mut x = center.x % grid_spacing;
        while x < rect.max.x {
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                egui::Stroke::new(1.0, grid_color)
            );
            x += grid_spacing;
        }

        // Horizontal lines
        let mut y = center.y % grid_spacing;
        while y < rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                egui::Stroke::new(1.0, grid_color)
            );
            y += grid_spacing;
        }

        // Draw main axes in different color
        let axis_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 128);
        painter.line_segment(
            [egui::pos2(center.x, rect.min.y), egui::pos2(center.x, rect.max.y)],
            egui::Stroke::new(2.0, axis_color)
        );
        painter.line_segment(
            [egui::pos2(rect.min.x, center.y), egui::pos2(rect.max.x, center.y)],
            egui::Stroke::new(2.0, axis_color)
        );
    }

    fn draw_scene_objects(&self, painter: &egui::Painter, rect: egui::Rect, scene: &Scene, selected_object: Option<u32>) {
        let center = rect.center();

        for (object_id, object) in &scene.objects {
            if !object.visible {
                continue;
            }

            // Convert 3D position to 2D screen position (simple orthographic projection)
            let screen_pos = self.world_to_screen(object.transform.position, center);

            // Skip if outside viewport
            if !rect.contains(screen_pos) {
                continue;
            }

            let is_selected = selected_object == Some(*object_id);
            let object_color = if is_selected {
                egui::Color32::YELLOW
            } else {
                match &object.object_type {
                    GameObjectType::Cube => egui::Color32::LIGHT_BLUE,
                    GameObjectType::Sphere => egui::Color32::LIGHT_RED,
                    GameObjectType::Cylinder => egui::Color32::LIGHT_GREEN,
                    GameObjectType::Plane => egui::Color32::GRAY,
                    GameObjectType::Light => egui::Color32::WHITE,
                    GameObjectType::Camera => egui::Color32::BLUE,
                    GameObjectType::RigidBody(_) => egui::Color32::RED,
                    GameObjectType::SoftBody => egui::Color32::GREEN,
                    GameObjectType::FluidEmitter => egui::Color32::LIGHT_BLUE,
                    _ => egui::Color32::WHITE,
                }
            };

            // Draw object representation based on type
            match &object.object_type {
                GameObjectType::Cube | GameObjectType::RigidBody(_) => {
                    let size = (object.transform.scale.x * (self.zoom_level as f64) * 10.0) as f32;
                    painter.rect_filled(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(size)),
                        egui::Rounding::ZERO,
                        object_color
                    );
                },
                GameObjectType::Sphere => {
                    let radius = (object.transform.scale.x * (self.zoom_level as f64) * 10.0) as f32;
                    painter.circle_filled(screen_pos, radius, object_color);
                },
                GameObjectType::Light => {
                    painter.circle_filled(screen_pos, 8.0, object_color);
                    // Draw light rays
                    for i in 0..8 {
                        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
                        let end_pos = screen_pos + egui::Vec2::new(
                            angle.cos() * 15.0,
                            angle.sin() * 15.0
                        );
                        painter.line_segment(
                            [screen_pos, end_pos],
                            egui::Stroke::new(1.0, object_color)
                        );
                    }
                },
                GameObjectType::Camera => {
                    // Draw camera icon
                    let size = 12.0;
                    painter.rect_filled(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::new(size, size * 0.7)),
                        egui::Rounding::same(2.0),
                        object_color
                    );
                    painter.circle_filled(
                        screen_pos + egui::Vec2::new(size * 0.3, 0.0),
                        3.0,
                        egui::Color32::BLACK
                    );
                },
                _ => {
                    // Default representation
                    painter.circle_filled(screen_pos, 5.0, object_color);
                }
            }

            // Draw object name
            painter.text(
                screen_pos + egui::Vec2::new(0.0, 20.0),
                egui::Align2::CENTER_TOP,
                &object.name,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE
            );
        }
    }

    fn draw_gizmos(&self, painter: &egui::Painter, rect: egui::Rect, scene: &Scene, selected_object_id: u32) {
        if let Some(object) = scene.objects.get(&selected_object_id) {
            let center = rect.center();
            let screen_pos = self.world_to_screen(object.transform.position, center);

            // Draw translation gizmo
            let gizmo_size = 30.0;

            // X axis (red)
            painter.line_segment(
                [screen_pos, screen_pos + egui::Vec2::new(gizmo_size, 0.0)],
                egui::Stroke::new(3.0, egui::Color32::RED)
            );
            painter.circle_filled(
                screen_pos + egui::Vec2::new(gizmo_size, 0.0),
                4.0,
                egui::Color32::RED
            );

            // Y axis (green)
            painter.line_segment(
                [screen_pos, screen_pos + egui::Vec2::new(0.0, -gizmo_size)],
                egui::Stroke::new(3.0, egui::Color32::GREEN)
            );
            painter.circle_filled(
                screen_pos + egui::Vec2::new(0.0, -gizmo_size),
                4.0,
                egui::Color32::GREEN
            );

            // Z axis (blue) - represented as a square for 2D view
            if matches!(self.view_mode, ViewMode::Scene3D | ViewMode::Game3D) {
                painter.circle_filled(screen_pos, 6.0, egui::Color32::BLUE);
                painter.circle_stroke(screen_pos, 6.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            }
        }
    }

    fn world_to_screen(&self, world_pos: Vec3, screen_center: egui::Pos2) -> egui::Pos2 {
        match self.view_mode {
            ViewMode::Scene2D | ViewMode::Game2D => {
                egui::pos2(
                    screen_center.x + ((world_pos.x - self.camera_position.x) * (self.zoom_level as f64) * 20.0) as f32,
                    screen_center.y - ((world_pos.y - self.camera_position.y) * (self.zoom_level as f64) * 20.0) as f32
                )
            },
            ViewMode::Scene3D | ViewMode::Game3D => {
                // Simple orthographic projection for now
                egui::pos2(
                    screen_center.x + ((world_pos.x - self.camera_position.x) * (self.zoom_level as f64) * 20.0) as f32,
                    screen_center.y - ((world_pos.y - self.camera_position.y) * (self.zoom_level as f64) * 20.0) as f32
                )
            }
        }
    }

    fn reset_camera(&mut self) {
        match self.view_mode {
            ViewMode::Scene2D | ViewMode::Game2D => {
                self.camera_position = Vec3::new(0.0, 0.0, 0.0);
                self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
            },
            ViewMode::Scene3D | ViewMode::Game3D => {
                self.camera_position = Vec3::new(0.0, 5.0, 10.0);
                self.camera_rotation = Vec3::new(-20.0, 0.0, 0.0);
            }
        }
        self.zoom_level = 1.0;
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
