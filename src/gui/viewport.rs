use super::*;
use crate::physics::math::Vec3;

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

    /// Callback for object creation
    pub object_creation_callback: Option<Box<dyn Fn(&str, Vec3) + Send + Sync>>,

    /// Callback for preset creation
    pub preset_creation_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,

    /// Callback for selection changes
    pub selection_callback: Option<Box<dyn Fn(Option<u32>) + Send + Sync>>,
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

            object_creation_callback: None,
            preset_creation_callback: None,
            selection_callback: None,
        }
    }

    /// Get the current view mode
    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set the view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
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

    pub fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        selected_object: Option<u32>,
    ) -> Option<u32> {
        // Viewport toolbar
        self.show_viewport_toolbar(ui, scene);

        ui.separator();

        // Main viewport area
        self.show_viewport_content(ui, scene, selected_object)
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

    fn show_viewport_content(
        &mut self,
        ui: &mut egui::Ui,
        scene: &Scene,
        selected_object: Option<u32>,
    ) -> Option<u32> {
        let available_size = ui.available_size();

        // Create a custom paint widget for the 3D viewport
        let (response, painter) =
            ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        let mut new_selection = None;

        // Handle keyboard input for WASD camera movement
        let mut camera_moved = false;
        ui.input(|i| {
            let camera_speed = if i.modifiers.shift { 1.0 } else { 0.3 };

            // WASD movement
            if i.key_down(egui::Key::W) {
                match self.view_mode {
                    ViewMode::Scene3D | ViewMode::Game3D => {
                        // Move forward in 3D
                        let forward = self.get_forward_vector();
                        self.camera_position += forward * camera_speed;
                    }
                    ViewMode::Scene2D | ViewMode::Game2D => {
                        // Move up in 2D
                        self.camera_position.y += camera_speed;
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::S) {
                match self.view_mode {
                    ViewMode::Scene3D | ViewMode::Game3D => {
                        // Move backward in 3D
                        let forward = self.get_forward_vector();
                        self.camera_position -= forward * camera_speed;
                    }
                    ViewMode::Scene2D | ViewMode::Game2D => {
                        // Move down in 2D
                        self.camera_position.y -= camera_speed;
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::A) {
                match self.view_mode {
                    ViewMode::Scene3D | ViewMode::Game3D => {
                        // Strafe left in 3D
                        let right = self.get_right_vector();
                        self.camera_position -= right * camera_speed;
                    }
                    ViewMode::Scene2D | ViewMode::Game2D => {
                        // Move left in 2D
                        self.camera_position.x -= camera_speed;
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::D) {
                match self.view_mode {
                    ViewMode::Scene3D | ViewMode::Game3D => {
                        // Strafe right in 3D
                        let right = self.get_right_vector();
                        self.camera_position += right * camera_speed;
                    }
                    ViewMode::Scene2D | ViewMode::Game2D => {
                        // Move right in 2D
                        self.camera_position.x += camera_speed;
                    }
                }
                camera_moved = true;
            }

            // Q/E for up/down movement in 3D
            if i.key_down(egui::Key::Q)
                && matches!(self.view_mode, ViewMode::Scene3D | ViewMode::Game3D)
            {
                self.camera_position.y -= camera_speed;
                camera_moved = true;
            }
            if i.key_down(egui::Key::E)
                && matches!(self.view_mode, ViewMode::Scene3D | ViewMode::Game3D)
            {
                self.camera_position.y += camera_speed;
                camera_moved = true;
            }

            // Arrow keys for camera rotation
            let rotation_speed = 3.0;
            if i.key_down(egui::Key::ArrowUp) {
                self.camera_rotation.x += rotation_speed;
                self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowDown) {
                self.camera_rotation.x -= rotation_speed;
                self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowLeft) {
                self.camera_rotation.y -= rotation_speed;
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowRight) {
                self.camera_rotation.y += rotation_speed;
                camera_moved = true;
            }
        });

        // Request repaint if camera moved
        if camera_moved {
            ui.ctx().request_repaint();
        }

        // Handle mouse input for camera controls
        if response.dragged() {
            let delta = response.drag_delta();
            match self.view_mode {
                ViewMode::Scene2D | ViewMode::Game2D => {
                    // Pan in 2D
                    self.camera_position.x -= (delta.x * 0.02) as f64;
                    self.camera_position.y += (delta.y * 0.02) as f64;
                }
                ViewMode::Scene3D | ViewMode::Game3D => {
                    // Rotate camera in 3D
                    if ui.input(|i| i.modifiers.shift) {
                        // Pan
                        let right = self.get_right_vector();
                        let up = self.get_up_vector();
                        self.camera_position = self.camera_position
                            - right * ((delta.x * 0.02) as f64)
                            + up * ((delta.y * 0.02) as f64);
                    } else {
                        // Rotate
                        self.camera_rotation.y += (delta.x * 0.8) as f64;
                        self.camera_rotation.x -= (delta.y * 0.8) as f64;
                        self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                        camera_moved = true;
                    }
                }
            }
        }

        // Handle zoom with mouse wheel
        ui.input(|i| {
            if response.hovered() {
                let scroll_delta = i.raw_scroll_delta.y;
                if scroll_delta != 0.0 {
                    self.zoom_level *= 1.0 + scroll_delta * 0.002;
                    self.zoom_level = self.zoom_level.clamp(0.1, 20.0);
                    camera_moved = true;
                }
            }
        });

        // Set background color based on view mode
        let bg_color = match self.view_mode {
            ViewMode::Scene2D => [0.3, 0.3, 0.3, 1.0],
            ViewMode::Scene3D => self.background_color,
            ViewMode::Game2D => [0.1, 0.1, 0.1, 1.0],
            ViewMode::Game3D => [0.05, 0.05, 0.1, 1.0],
        };

        // Clear background
        painter.rect_filled(
            response.rect,
            egui::CornerRadius::ZERO,
            egui::Color32::from_rgba_unmultiplied(
                (bg_color[0] * 255.0) as u8,
                (bg_color[1] * 255.0) as u8,
                (bg_color[2] * 255.0) as u8,
                (bg_color[3] * 255.0) as u8,
            ),
        );

        // Draw grid if enabled
        if self.show_grid {
            self.draw_grid(&painter, response.rect);
        }

        // Draw scene objects
        self.draw_scene_objects(&painter, response.rect, scene, selected_object);

        // Draw gizmos if enabled
        if self.show_gizmos {
            if let Some(selected_id) = selected_object {
                self.draw_gizmos(&painter, response.rect, scene, selected_id);
            }
        }

        // Show camera info overlay
        self.draw_camera_info(ui, response.rect);

        // Handle right-click context menu for creating objects
        response.context_menu(|ui| {
            ui.label("Create:");
            if ui.button("Cube").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Cube",
                        self.screen_to_world(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                        ),
                    );
                }
            }
            if ui.button("Sphere").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Sphere",
                        self.screen_to_world(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                        ),
                    );
                }
            }
            if ui.button("Light").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Light",
                        self.screen_to_world(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                        ),
                    );
                }
            }
            ui.separator();
            ui.label("Presets:");
            if ui.button("Aquarium 3D").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.preset_creation_callback {
                    callback("Aquarium3D");
                }
            }
            if ui.button("Aquarium 2D").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.preset_creation_callback {
                    callback("Aquarium2D");
                }
            }
            if ui.button("Physics Playground").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.preset_creation_callback {
                    callback("PhysicsPlayground");
                }
            }
        });

        // Handle object selection by clicking
        if response.clicked() {
            // Find which object was clicked (simplified ray casting)
            let click_pos = response
                .interact_pointer_pos()
                .unwrap_or(response.rect.center());
            let world_click = self.screen_to_world(click_pos, response.rect.center());

            // Find the closest object to the click position
            let mut closest_object = None;
            let mut closest_distance = f64::INFINITY;

            for (object_id, object) in &scene.objects {
                if !object.visible {
                    continue;
                }

                let distance = (object.transform.position - world_click).magnitude();
                let selection_threshold = match &object.object_type {
                    GameObjectType::Sphere => {
                        (object.transform.scale.x * self.zoom_level as f64 * 10.0) as f64
                    }
                    GameObjectType::Cube => {
                        (object.transform.scale.x * self.zoom_level as f64 * 10.0) as f64
                    }
                    _ => 2.0, // Default threshold
                };

                if distance < closest_distance && distance < selection_threshold {
                    closest_distance = distance;
                    closest_object = Some(*object_id);
                }
            }

            new_selection = closest_object;
        }

        // Draw camera information overlay
        self.draw_camera_info(ui, response.rect);

        new_selection
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
                egui::Stroke::new(1.0, grid_color),
            );
            x += grid_spacing;
        }

        // Horizontal lines
        let mut y = center.y % grid_spacing;
        while y < rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                egui::Stroke::new(1.0, grid_color),
            );
            y += grid_spacing;
        }

        // Draw main axes in different color
        let axis_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 128);
        painter.line_segment(
            [
                egui::pos2(center.x, rect.min.y),
                egui::pos2(center.x, rect.max.y),
            ],
            egui::Stroke::new(2.0, axis_color),
        );
        painter.line_segment(
            [
                egui::pos2(rect.min.x, center.y),
                egui::pos2(rect.max.x, center.y),
            ],
            egui::Stroke::new(2.0, axis_color),
        );
    }

    fn draw_scene_objects(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &Scene,
        selected_object: Option<u32>,
    ) {
        let center = rect.center();

        // Collect objects with their screen positions and depths for proper sorting
        let mut objects_to_draw: Vec<_> = scene
            .objects
            .iter()
            .filter(|(_, object)| object.visible)
            .map(|(object_id, object)| {
                let (screen_pos, depth) = match self.view_mode {
                    ViewMode::Scene2D | ViewMode::Game2D => {
                        let pos = self.world_to_screen(object.transform.position, center);
                        (pos, 0.0)
                    }
                    ViewMode::Scene3D | ViewMode::Game3D => {
                        let pos = self.world_to_screen(object.transform.position, center);
                        let depth = (object.transform.position - self.camera_position).magnitude();
                        (pos, depth)
                    }
                };

                (object_id, object, screen_pos, depth)
            })
            .collect();

        // Sort by depth (back to front for proper rendering)
        objects_to_draw.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

        for (object_id, object, screen_pos, _depth) in objects_to_draw {
            // Skip if outside viewport (with some margin)
            let margin = 50.0;
            if screen_pos.x < rect.min.x - margin
                || screen_pos.x > rect.max.x + margin
                || screen_pos.y < rect.min.y - margin
                || screen_pos.y > rect.max.y + margin
            {
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

            // Calculate size based on distance and scale for 3D perspective
            let base_size = object.transform.scale.x;
            let size_factor = match self.view_mode {
                ViewMode::Scene2D | ViewMode::Game2D => self.zoom_level as f64 * 10.0,
                ViewMode::Scene3D | ViewMode::Game3D => {
                    // Apply perspective scaling
                    let distance = (object.transform.position - self.camera_position).magnitude();
                    let perspective_scale = 50.0 / (distance.max(1.0)); // Prevent division by zero
                    self.zoom_level as f64 * perspective_scale
                }
            };

            // Draw object representation based on type
            match &object.object_type {
                GameObjectType::Cube | GameObjectType::RigidBody(_) => {
                    let size = (base_size * size_factor) as f32;
                    painter.rect_filled(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(size)),
                        egui::CornerRadius::ZERO,
                        object_color,
                    );
                }
                GameObjectType::Sphere => {
                    let radius = (base_size * size_factor) as f32;
                    painter.circle_filled(screen_pos, radius, object_color);
                }
                GameObjectType::Light => {
                    painter.circle_filled(screen_pos, 8.0, object_color);
                    // Draw light rays
                    for i in 0..8 {
                        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
                        let end_pos =
                            screen_pos + egui::Vec2::new(angle.cos() * 15.0, angle.sin() * 15.0);
                        painter.line_segment(
                            [screen_pos, end_pos],
                            egui::Stroke::new(1.0, object_color),
                        );
                    }
                }
                GameObjectType::Camera => {
                    // Draw camera icon
                    let size = 12.0;
                    painter.rect_filled(
                        egui::Rect::from_center_size(screen_pos, egui::Vec2::new(size, size * 0.7)),
                        egui::CornerRadius::same(2),
                        object_color,
                    );
                    painter.circle_filled(
                        screen_pos + egui::Vec2::new(size * 0.3, 0.0),
                        3.0,
                        egui::Color32::BLACK,
                    );
                }
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
                egui::Color32::WHITE,
            );
        }
    }

    fn draw_gizmos(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &Scene,
        selected_object_id: u32,
    ) {
        if let Some(object) = scene.objects.get(&selected_object_id) {
            let center = rect.center();
            let screen_pos = self.world_to_screen(object.transform.position, center);

            // Draw translation gizmo
            let gizmo_size = 30.0;

            // X axis (red)
            painter.line_segment(
                [screen_pos, screen_pos + egui::Vec2::new(gizmo_size, 0.0)],
                egui::Stroke::new(3.0, egui::Color32::RED),
            );
            painter.circle_filled(
                screen_pos + egui::Vec2::new(gizmo_size, 0.0),
                4.0,
                egui::Color32::RED,
            );

            // Y axis (green)
            painter.line_segment(
                [screen_pos, screen_pos + egui::Vec2::new(0.0, -gizmo_size)],
                egui::Stroke::new(3.0, egui::Color32::GREEN),
            );
            painter.circle_filled(
                screen_pos + egui::Vec2::new(0.0, -gizmo_size),
                4.0,
                egui::Color32::GREEN,
            );

            // Z axis (blue) - represented as a square for 2D view
            if matches!(self.view_mode, ViewMode::Scene3D | ViewMode::Game3D) {
                painter.circle_filled(screen_pos, 6.0, egui::Color32::BLUE);
                painter.circle_stroke(
                    screen_pos,
                    6.0,
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            }
        }
    }

    fn world_to_screen(&self, world_pos: Vec3, screen_center: egui::Pos2) -> egui::Pos2 {
        match self.view_mode {
            ViewMode::Scene2D | ViewMode::Game2D => egui::pos2(
                screen_center.x
                    + ((world_pos.x - self.camera_position.x) * (self.zoom_level as f64) * 20.0)
                        as f32,
                screen_center.y
                    - ((world_pos.y - self.camera_position.y) * (self.zoom_level as f64) * 20.0)
                        as f32,
            ),
            ViewMode::Scene3D | ViewMode::Game3D => {
                // Transform to camera space
                let camera_pos = self.transform_world_to_camera(world_pos);

                // Check if behind camera
                if camera_pos.z <= 0.1 {
                    return egui::pos2(-10000.0, -10000.0); // Off-screen
                }

                // Apply perspective projection with proper field of view
                let fov = 60.0_f64.to_radians();
                let tan_half_fov = (fov / 2.0).tan();
                let aspect_ratio = 1.0; // Square viewport

                // Project to normalized device coordinates
                let ndc_x = camera_pos.x / (camera_pos.z * tan_half_fov * aspect_ratio);
                let ndc_y = camera_pos.y / (camera_pos.z * tan_half_fov);

                // Convert to screen coordinates with proper scaling
                let scale_factor = self.zoom_level as f64 * 150.0;
                egui::pos2(
                    screen_center.x + (ndc_x * scale_factor) as f32,
                    screen_center.y - (ndc_y * scale_factor) as f32, // Flip Y for screen coordinates
                )
            }
        }
    }

    fn screen_to_world(&self, screen_pos: egui::Pos2, screen_center: egui::Pos2) -> Vec3 {
        match self.view_mode {
            ViewMode::Scene2D | ViewMode::Game2D => Vec3::new(
                self.camera_position.x
                    + ((screen_pos.x - screen_center.x) / (self.zoom_level * 20.0)) as f64,
                self.camera_position.y
                    - ((screen_pos.y - screen_center.y) / (self.zoom_level * 20.0)) as f64,
                0.0,
            ),
            ViewMode::Scene3D | ViewMode::Game3D => {
                // Convert screen coordinates to normalized device coordinates
                let ndc_x = (screen_pos.x - screen_center.x) / (self.zoom_level * 200.0);
                let ndc_y = -(screen_pos.y - screen_center.y) / (self.zoom_level * 200.0);

                // Unproject to world space (assuming depth = 5.0 units from camera)
                let depth = 5.0;
                let fov = 60.0_f64.to_radians();
                let tan_half_fov = (fov / 2.0).tan();

                // Convert NDC to camera space
                let camera_x = ndc_x as f64 * depth * tan_half_fov;
                let camera_y = ndc_y as f64 * depth * tan_half_fov;
                let camera_z = depth;

                // Transform from camera space to world space
                let forward = self.get_forward_vector();
                let right = self.get_right_vector();
                let up = self.get_up_vector();

                let world_offset = right * camera_x + up * camera_y + forward * camera_z;
                self.camera_position + world_offset
            }
        }
    }

    fn reset_camera(&mut self) {
        match self.view_mode {
            ViewMode::Scene2D | ViewMode::Game2D => {
                self.camera_position = Vec3::new(0.0, 0.0, 0.0);
                self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
            }
            ViewMode::Scene3D | ViewMode::Game3D => {
                self.camera_position = Vec3::new(0.0, 5.0, 10.0);
                self.camera_rotation = Vec3::new(-20.0, 0.0, 0.0);
            }
        }
        self.zoom_level = 1.0;
    }

    /// Get camera forward vector based on rotation
    fn get_forward_vector(&self) -> Vec3 {
        let pitch = self.camera_rotation.x.to_radians();
        let yaw = self.camera_rotation.y.to_radians();

        Vec3::new(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        )
        .normalized()
    }

    /// Get camera right vector based on rotation
    fn get_right_vector(&self) -> Vec3 {
        let yaw = self.camera_rotation.y.to_radians();
        Vec3::new(yaw.cos(), 0.0, -yaw.sin()).normalized()
    }

    /// Get camera up vector based on rotation
    fn get_up_vector(&self) -> Vec3 {
        self.get_right_vector()
            .cross(self.get_forward_vector())
            .normalized()
    }

    /// Transform world position to camera space
    fn transform_world_to_camera(&self, world_pos: Vec3) -> Vec3 {
        // Translate relative to camera
        let relative_pos = world_pos - self.camera_position;

        // Create camera basis vectors
        let forward = self.get_forward_vector();
        let right = self.get_right_vector();
        let up = self.get_up_vector();

        // Transform to camera space
        Vec3::new(
            relative_pos.dot(right),    // X in camera space
            relative_pos.dot(up),       // Y in camera space
            -relative_pos.dot(forward), // Z in camera space (negative because we look down -Z)
        )
    }

    /// Apply perspective projection to camera space position
    fn apply_perspective_projection(&self, camera_pos: Vec3) -> Vec3 {
        let fov = 60.0_f64.to_radians(); // Field of view
        let near = 0.1;
        let far = 1000.0;

        // Avoid division by zero for points at camera
        if camera_pos.z.abs() < 0.001 {
            return Vec3::new(0.0, 0.0, camera_pos.z);
        }

        // Perspective division
        let aspect = 1.0; // Assume square viewport for simplicity
        let tan_half_fov = (fov / 2.0).tan();

        Vec3::new(
            camera_pos.x / (camera_pos.z * tan_half_fov * aspect),
            camera_pos.y / (camera_pos.z * tan_half_fov),
            camera_pos.z,
        )
    }

    /// Draw camera information overlay
    fn draw_camera_info(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let info_text = format!(
            "Camera: ({:.1}, {:.1}, {:.1})\nRotation: ({:.1}°, {:.1}°, {:.1}°)\nZoom: {:.1}x\nView: {:?}\n[WASD: Move, Arrows: Rotate, Q/E: Up/Down]",
            self.camera_position.x,
            self.camera_position.y,
            self.camera_position.z,
            self.camera_rotation.x,
            self.camera_rotation.y,
            self.camera_rotation.z,
            self.zoom_level,
            self.view_mode
        );

        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                rect.min + egui::Vec2::new(10.0, 10.0),
                egui::Vec2::new(300.0, 100.0),
            )),
            |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                ui.label(info_text);
            },
        );
    }

    /// Set object creation callback
    pub fn set_object_creation_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str, Vec3) + Send + Sync + 'static,
    {
        self.object_creation_callback = Some(Box::new(callback));
    }

    /// Set preset creation callback
    pub fn set_preset_creation_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.preset_creation_callback = Some(Box::new(callback));
    }

    /// Set selection callback
    pub fn set_selection_callback<F>(&mut self, callback: F)
    where
        F: Fn(Option<u32>) + Send + Sync + 'static,
    {
        self.selection_callback = Some(Box::new(callback));
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
