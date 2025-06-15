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

    // Enhanced 3D rendering
    field_of_view: f32,
    near_clip: f32,
    far_clip: f32,

    // Animation and game view
    is_playing: bool,
    animation_time: f32,
    show_game_view: bool,
    game_camera_position: Vec3,
    game_camera_rotation: Vec3,

    // Gizmo interaction
    gizmo_mode: GizmoMode,
    gizmo_dragging: Option<GizmoAxis>,
    drag_start_pos: Option<egui::Pos2>,
    drag_start_transform: Option<Transform>,

    // Enhanced viewport features from conversation summary
    wireframe_mode: bool,
    show_bounding_boxes: bool,
    lighting_enabled: bool,
    camera_preview_size: f32,
    show_camera_frustum: bool,
    geometry_detail_level: u32,

    // Camera follow system
    follow_selected_object: bool,
    last_selected_object_position: Option<Vec3>,

    /// Callback for object creation
    pub object_creation_callback: Option<Box<dyn Fn(&str, Vec3) + Send + Sync>>,

    /// Callback for preset creation
    pub preset_creation_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,

    /// Callback for selection changes
    pub selection_callback: Option<Box<dyn Fn(Option<u32>) + Send + Sync>>,

    /// Callback for transform changes
    pub transform_changed_callback: Option<Box<dyn Fn(u32, Transform) + Send + Sync>>,

    /// Callback for simulation state changes
    pub simulation_state_callback: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

/// Gizmo axis for interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    Center, // For uniform scaling or multi-axis movement
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

            // Enhanced 3D rendering
            field_of_view: 75.0,
            near_clip: 0.1,
            far_clip: 1000.0,

            // Animation and game view
            is_playing: false,
            animation_time: 0.0,
            show_game_view: false,
            game_camera_position: Vec3::new(0.0, 2.0, 5.0),
            game_camera_rotation: Vec3::new(-10.0, 0.0, 0.0),

            // Gizmo interaction
            gizmo_mode: GizmoMode::Translate,
            gizmo_dragging: None,
            drag_start_pos: None,
            drag_start_transform: None,

            // Enhanced viewport features
            wireframe_mode: false,
            show_bounding_boxes: false,
            lighting_enabled: true,
            camera_preview_size: 150.0,
            show_camera_frustum: false,
            geometry_detail_level: 2,

            // Camera follow system
            follow_selected_object: false,
            last_selected_object_position: None,

            object_creation_callback: None,
            preset_creation_callback: None,
            selection_callback: None,
            transform_changed_callback: None,
            simulation_state_callback: None,
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

    /// Set the simulation state callback
    pub fn set_simulation_state_callback(&mut self, callback: Box<dyn Fn(bool) + Send + Sync>) {
        self.simulation_state_callback = Some(callback);
    }

    /// Set the playing state (can be called from external simulation manager)
    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
    }

    /// Get the playing state
    pub fn is_playing(&self) -> bool {
        self.is_playing
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
            // Simple 2D/3D toggle (Unity-like)
            ui.label("View:");
            if ui
                .selectable_value(&mut self.view_mode, ViewMode::Scene2D, "2D")
                .clicked()
            {
                // When switching to 2D, adjust camera for better 2D view
                self.camera_position = Vec3::new(0.0, 0.0, 10.0);
                self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
                self.zoom_level = 1.0;
            }
            if ui
                .selectable_value(&mut self.view_mode, ViewMode::Scene3D, "3D")
                .clicked()
            {
                // When switching to 3D, adjust camera for better 3D view
                self.camera_position = Vec3::new(5.0, 5.0, 10.0);
                self.camera_rotation = Vec3::new(-20.0, -25.0, 0.0);
                self.zoom_level = 1.0;
            }

            ui.separator();

            // Game view toggle (simpler than before)
            if ui.toggle_value(&mut self.show_game_view, "Game").clicked() {
                // Auto-switch to 3D when enabling game view for better experience
                if self.show_game_view {
                    self.view_mode = ViewMode::Scene3D;
                }
            }

            ui.separator();

            // Animation controls with reset functionality
            ui.label("Simulation:");
            if ui
                .button(if self.is_playing {
                    "⏸ Pause"
                } else {
                    "▶ Play"
                })
                .clicked()
            {
                self.is_playing = !self.is_playing;
                // Notify about simulation state change
                if let Some(ref callback) = self.simulation_state_callback {
                    callback(self.is_playing);
                }
            }
            if ui.button("⏹ Stop").clicked() {
                self.is_playing = false;
                self.animation_time = 0.0;
                // Notify about simulation state change
                if let Some(ref callback) = self.simulation_state_callback {
                    callback(false);
                }
                // Clear any animation state to prevent rubber-banding
                ui.ctx().request_repaint();
            }
            if ui.button("⏮ Reset Scene").clicked() {
                self.is_playing = false;
                self.animation_time = 0.0;
                // Notify about simulation state change
                if let Some(ref callback) = self.simulation_state_callback {
                    callback(false);
                }
                // Reset camera to default position
                if matches!(self.view_mode, ViewMode::Scene3D) {
                    self.camera_position = Vec3::new(5.0, 5.0, 10.0);
                    self.camera_rotation = Vec3::new(-20.0, -25.0, 0.0);
                } else {
                    self.camera_position = Vec3::new(0.0, 0.0, 10.0);
                    self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
                }
                self.zoom_level = 1.0;
                ui.ctx().request_repaint();
            }

            ui.label(format!("Time: {:.1}s", self.animation_time));

            ui.separator();

            // Gizmo mode selection
            ui.label("Gizmo:");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Translate, "🔄 Move");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Rotate, "🔄 Rotate");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Scale, "📏 Scale");

            ui.separator();

            // View options
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_gizmos, "Gizmos");

            ui.separator();

            // Camera controls
            if ui.button("Reset Camera").clicked() {
                self.reset_camera();
            }

            ui.checkbox(&mut self.follow_selected_object, "Follow");

            ui.label(format!("Zoom: {:.1}x", self.zoom_level));

            ui.separator();

            // 3D rendering settings
            if matches!(self.view_mode, ViewMode::Scene3D) {
                ui.menu_button("3D Settings", |ui| {
                    ui.add(egui::Slider::new(&mut self.field_of_view, 30.0..=120.0).text("FOV"));
                    ui.add(egui::Slider::new(&mut self.near_clip, 0.01..=1.0).text("Near Clip"));
                    ui.add(egui::Slider::new(&mut self.far_clip, 100.0..=10000.0).text("Far Clip"));
                });
                ui.separator();
            }

            // Lighting options
            ui.menu_button("Lighting", |ui| {
                if ui
                    .checkbox(&mut self.lighting_enabled, "Enable Lighting")
                    .changed()
                {
                    // Lighting toggle
                }
                if ui.button("Realistic").clicked() {
                    self.lighting_enabled = true;
                    ui.close_menu();
                }
                if ui.button("Flat").clicked() {
                    self.lighting_enabled = false;
                    ui.close_menu();
                }
            });

            // Render options
            ui.menu_button("Render", |ui| {
                ui.checkbox(&mut self.wireframe_mode, "Wireframe Mode");
                ui.checkbox(&mut self.show_bounding_boxes, "Show Bounding Boxes");
                ui.checkbox(&mut self.show_camera_frustum, "Show Camera Frustum");

                ui.separator();
                ui.label("Geometry Detail:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.geometry_detail_level, 1, "Low");
                    ui.selectable_value(&mut self.geometry_detail_level, 2, "Medium");
                    ui.selectable_value(&mut self.geometry_detail_level, 3, "High");
                });

                ui.separator();
                ui.add(
                    egui::Slider::new(&mut self.camera_preview_size, 50.0..=300.0)
                        .text("Preview Size"),
                );
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

        // Update animation time if playing
        if self.is_playing {
            self.animation_time += ui.ctx().input(|i| i.stable_dt);
            ui.ctx().request_repaint(); // Keep animating
        }

        // Update camera follow system
        self.update_camera_follow(scene, selected_object);

        // Create a custom paint widget for the 3D viewport
        let (response, painter) =
            ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        let mut new_selection = None;

        // Determine which camera to use
        let (camera_pos, camera_rot) = if self.show_game_view {
            (self.game_camera_position, self.game_camera_rotation)
        } else {
            (self.camera_position, self.camera_rotation)
        };

        // Handle keyboard input for WASD camera movement
        let mut camera_moved = false;
        ui.input(|i| {
            let camera_speed = if i.modifiers.shift { 1.0 } else { 0.3 };

            // WASD movement
            if i.key_down(egui::Key::W) {
                match self.view_mode {
                    ViewMode::Scene3D => {
                        // Move forward in 3D
                        let forward = self.get_forward_vector_for_camera(camera_rot);
                        if self.show_game_view {
                            self.game_camera_position += forward * camera_speed;
                        } else {
                            self.camera_position += forward * camera_speed;
                        }
                    }
                    ViewMode::Scene2D => {
                        // Move up in 2D
                        if self.show_game_view {
                            self.game_camera_position.y += camera_speed;
                        } else {
                            self.camera_position.y += camera_speed;
                        }
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::S) {
                match self.view_mode {
                    ViewMode::Scene3D => {
                        // Move backward in 3D
                        let forward = self.get_forward_vector_for_camera(camera_rot);
                        if self.show_game_view {
                            self.game_camera_position -= forward * camera_speed;
                        } else {
                            self.camera_position -= forward * camera_speed;
                        }
                    }
                    ViewMode::Scene2D => {
                        // Move down in 2D
                        if self.show_game_view {
                            self.game_camera_position.y -= camera_speed;
                        } else {
                            self.camera_position.y -= camera_speed;
                        }
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::A) {
                match self.view_mode {
                    ViewMode::Scene3D => {
                        // Strafe left in 3D
                        let right = self.get_right_vector_for_camera(camera_rot);
                        if self.show_game_view {
                            self.game_camera_position -= right * camera_speed;
                        } else {
                            self.camera_position -= right * camera_speed;
                        }
                    }
                    ViewMode::Scene2D => {
                        // Move left in 2D
                        if self.show_game_view {
                            self.game_camera_position.x -= camera_speed;
                        } else {
                            self.camera_position.x -= camera_speed;
                        }
                    }
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::D) {
                match self.view_mode {
                    ViewMode::Scene3D => {
                        // Strafe right in 3D
                        let right = self.get_right_vector_for_camera(camera_rot);
                        if self.show_game_view {
                            self.game_camera_position += right * camera_speed;
                        } else {
                            self.camera_position += right * camera_speed;
                        }
                    }
                    ViewMode::Scene2D => {
                        // Move right in 2D
                        if self.show_game_view {
                            self.game_camera_position.x += camera_speed;
                        } else {
                            self.camera_position.x += camera_speed;
                        }
                    }
                }
                camera_moved = true;
            }

            // Q/E for up/down movement in 3D
            if i.key_down(egui::Key::Q) && matches!(self.view_mode, ViewMode::Scene3D) {
                if self.show_game_view {
                    self.game_camera_position.y -= camera_speed;
                } else {
                    self.camera_position.y -= camera_speed;
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::E) && matches!(self.view_mode, ViewMode::Scene3D) {
                if self.show_game_view {
                    self.game_camera_position.y += camera_speed;
                } else {
                    self.camera_position.y += camera_speed;
                }
                camera_moved = true;
            }

            // Arrow keys for camera rotation
            let rotation_speed = 3.0;
            if i.key_down(egui::Key::ArrowUp) {
                if self.show_game_view {
                    self.game_camera_rotation.x += rotation_speed;
                    self.game_camera_rotation.x = self.game_camera_rotation.x.clamp(-89.0, 89.0);
                } else {
                    self.camera_rotation.x += rotation_speed;
                    self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowDown) {
                if self.show_game_view {
                    self.game_camera_rotation.x -= rotation_speed;
                    self.game_camera_rotation.x = self.game_camera_rotation.x.clamp(-89.0, 89.0);
                } else {
                    self.camera_rotation.x -= rotation_speed;
                    self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowLeft) {
                if self.show_game_view {
                    self.game_camera_rotation.y -= rotation_speed;
                } else {
                    self.camera_rotation.y -= rotation_speed;
                }
                camera_moved = true;
            }
            if i.key_down(egui::Key::ArrowRight) {
                if self.show_game_view {
                    self.game_camera_rotation.y += rotation_speed;
                } else {
                    self.camera_rotation.y += rotation_speed;
                }
                camera_moved = true;
            }
        });

        // Request repaint if camera moved
        if camera_moved {
            ui.ctx().request_repaint();
        }

        // Handle mouse input for camera controls and gizmo interaction
        if response.dragged() {
            let delta = response.drag_delta();

            // Check if we're dragging a gizmo
            if let Some(gizmo_axis) = self.gizmo_dragging {
                if let Some(selected_id) = selected_object {
                    self.handle_gizmo_drag(delta, gizmo_axis, selected_id, response.rect);
                }
            } else {
                // Normal camera controls
                match self.view_mode {
                    ViewMode::Scene2D => {
                        // Pan in 2D
                        if self.show_game_view {
                            self.game_camera_position.x -= (delta.x * 0.02) as f64;
                            self.game_camera_position.y += (delta.y * 0.02) as f64;
                        } else {
                            self.camera_position.x -= (delta.x * 0.02) as f64;
                            self.camera_position.y += (delta.y * 0.02) as f64;
                        }
                    }
                    ViewMode::Scene3D => {
                        // Rotate camera in 3D
                        if ui.input(|i| i.modifiers.shift) {
                            // Pan
                            let right = self.get_right_vector_for_camera(camera_rot);
                            let up = self.get_up_vector_for_camera(camera_rot);
                            let pan_delta =
                                right * (-(delta.x * 0.02) as f64) + up * ((delta.y * 0.02) as f64);
                            if self.show_game_view {
                                self.game_camera_position += pan_delta;
                            } else {
                                self.camera_position += pan_delta;
                            }
                        } else {
                            // Rotate
                            if self.show_game_view {
                                self.game_camera_rotation.y += (delta.x * 0.8) as f64;
                                self.game_camera_rotation.x -= (delta.y * 0.8) as f64;
                                self.game_camera_rotation.x =
                                    self.game_camera_rotation.x.clamp(-89.0, 89.0);
                            } else {
                                self.camera_rotation.y += (delta.x * 0.8) as f64;
                                self.camera_rotation.x -= (delta.y * 0.8) as f64;
                                self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                            }
                            camera_moved = true;
                        }
                    }
                }
            }
        }

        // Handle gizmo selection on click
        if response.clicked() && selected_object.is_some() && self.show_gizmos {
            let click_pos = response
                .interact_pointer_pos()
                .unwrap_or(response.rect.center());
            if let Some(gizmo_axis) =
                self.check_gizmo_click(click_pos, selected_object.unwrap(), scene, response.rect)
            {
                self.gizmo_dragging = Some(gizmo_axis);
                self.drag_start_pos = Some(click_pos);
                if let Some(object) = scene.objects.get(&selected_object.unwrap()) {
                    self.drag_start_transform = Some(object.transform);
                }
            } else {
                // Handle object selection by clicking
                let world_click = self.screen_to_world_with_camera(
                    click_pos,
                    response.rect.center(),
                    camera_pos,
                    camera_rot,
                );

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
        }

        // Stop gizmo dragging on release
        if !response.dragged() {
            self.gizmo_dragging = None;
            self.drag_start_pos = None;
            self.drag_start_transform = None;
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
            ViewMode::Scene2D => [0.3, 0.3, 0.3, 1.0], // Darker for 2D
            ViewMode::Scene3D => {
                if self.show_game_view {
                    [0.05, 0.05, 0.1, 1.0] // Darker blue for game view
                } else {
                    self.background_color // Normal 3D background
                }
            }
        };

        // Clear background
        painter.rect_filled(
            response.rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                (bg_color[0] * 255.0) as u8,
                (bg_color[1] * 255.0) as u8,
                (bg_color[2] * 255.0) as u8,
                (bg_color[3] * 255.0) as u8,
            ),
        );

        // Draw grid if enabled
        if self.show_grid {
            self.draw_grid(&painter, response.rect, camera_pos, camera_rot);
        }

        // Draw scene objects
        self.draw_scene_objects(
            &painter,
            response.rect,
            scene,
            selected_object,
            camera_pos,
            camera_rot,
        );

        // Draw gizmos if enabled
        if self.show_gizmos {
            if let Some(selected_id) = selected_object {
                self.draw_enhanced_gizmos(&painter, response.rect, scene, selected_id);
            }
        }

        // Show camera info overlay
        self.draw_camera_info(ui, response.rect, camera_pos, camera_rot);

        // Draw Unity-like orientation gizmo in top-right corner
        self.draw_orientation_gizmo(&painter, response.rect);

        // Draw view mode indicator
        self.draw_view_mode_indicator(&painter, response.rect);

        // Draw game camera preview if enabled
        if self.show_game_view {
            self.draw_game_camera_preview(&painter, response.rect, scene);
        }

        // Handle right-click context menu for creating objects
        response.context_menu(|ui| {
            ui.label("Create:");
            if ui.button("Cube").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Cube",
                        self.screen_to_world_with_camera(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                            camera_pos,
                            camera_rot,
                        ),
                    );
                }
            }
            if ui.button("Sphere").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Sphere",
                        self.screen_to_world_with_camera(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                            camera_pos,
                            camera_rot,
                        ),
                    );
                }
            }
            if ui.button("Light").clicked() {
                ui.close_menu();
                if let Some(ref callback) = self.object_creation_callback {
                    callback(
                        "Light",
                        self.screen_to_world_with_camera(
                            response.interact_pointer_pos().unwrap_or_default(),
                            response.rect.center(),
                            camera_pos,
                            camera_rot,
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

        new_selection
    }

    /// Update camera to follow selected object if enabled
    pub fn update_camera_follow(&mut self, scene: &Scene, selected_object: Option<u32>) {
        if !self.follow_selected_object {
            // If follow is disabled, clear tracking
            self.last_selected_object_position = None;
            return;
        }

        if let Some(object_id) = selected_object {
            if let Some(object) = scene.objects.get(&object_id) {
                let current_pos = object.transform.position;

                // Check if object has moved since last frame
                if let Some(last_pos) = self.last_selected_object_position {
                    let movement = current_pos - last_pos;
                    let movement_magnitude = movement.magnitude();

                    // If object moved significantly, update camera to follow
                    if movement_magnitude > 0.1 {
                        // Smoothly follow the object movement
                        let follow_factor = 0.5; // How much the camera should follow (0 = none, 1 = full)

                        if self.show_game_view {
                            self.game_camera_position += movement * follow_factor;
                        } else {
                            self.camera_position += movement * follow_factor;
                        }
                    }
                }

                // Store current position for next frame
                self.last_selected_object_position = Some(current_pos);
            }
        } else {
            // No object selected, clear tracking
            self.last_selected_object_position = None;
        }
    }

    fn draw_grid(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        _camera_pos: Vec3,
        _camera_rot: Vec3,
    ) {
        let center = rect.center();

        match self.view_mode {
            ViewMode::Scene2D => {
                // 2D Grid - simple orthogonal grid
                let grid_spacing = self.grid_size * self.zoom_level * 20.0;
                let grid_color = egui::Color32::from_rgba_unmultiplied(100, 100, 100, 80);
                let axis_color = egui::Color32::from_rgba_unmultiplied(150, 150, 150, 150);

                // Draw fine grid lines
                let mut x = center.x % grid_spacing;
                while x < rect.max.x {
                    painter.line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(0.5, grid_color),
                    );
                    x += grid_spacing;
                }

                let mut y = center.y % grid_spacing;
                while y < rect.max.y {
                    painter.line_segment(
                        [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                        egui::Stroke::new(0.5, grid_color),
                    );
                    y += grid_spacing;
                }

                // Draw main X and Y axes with different colors
                painter.line_segment(
                    [
                        egui::pos2(center.x, rect.min.y),
                        egui::pos2(center.x, rect.max.y),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::GREEN), // Y-axis green
                );
                painter.line_segment(
                    [
                        egui::pos2(rect.min.x, center.y),
                        egui::pos2(rect.max.x, center.y),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::RED), // X-axis red
                );
            }
            ViewMode::Scene3D => {
                // 3D Grid - improved perspective grid with better depth cues
                let base_spacing = self.grid_size * 25.0;
                let grid_color = egui::Color32::from_rgba_unmultiplied(100, 100, 100, 40);
                let major_grid_color = egui::Color32::from_rgba_unmultiplied(140, 140, 140, 80);

                // Create perspective grid that looks more natural
                let grid_lines = 15i32; // Reduced for better performance
                let line_spacing = base_spacing * self.zoom_level;

                // Ground plane grid - horizontal lines (going into depth)
                for i in -(grid_lines / 2)..=(grid_lines / 2) {
                    let i_f = i as f32;
                    let depth_factor = 1.0 - (i.abs() as f32 / (grid_lines as f32 / 2.0)) * 0.4;

                    // Converging lines towards vanishing point
                    let y_offset = i_f * line_spacing * 0.2 * depth_factor;
                    let line_y = center.y + y_offset + 50.0; // Offset below center

                    let start_x =
                        center.x - (grid_lines as f32 / 2.0) * line_spacing * depth_factor;
                    let end_x = center.x + (grid_lines as f32 / 2.0) * line_spacing * depth_factor;

                    let color = if i % 5 == 0 {
                        major_grid_color
                    } else {
                        grid_color
                    };
                    let width = if i % 5 == 0 { 1.2 } else { 0.6 };

                    painter.line_segment(
                        [egui::pos2(start_x, line_y), egui::pos2(end_x, line_y)],
                        egui::Stroke::new(width, color),
                    );
                }

                // Vertical lines (perspective)
                for i in -(grid_lines / 2)..=(grid_lines / 2) {
                    let i_f = i as f32;
                    let x_offset = i_f * line_spacing;

                    // Lines converge toward a vanishing point
                    let convergence_factor = 0.7;
                    let start_y = center.y + 50.0 - (grid_lines as f32 / 3.0) * line_spacing * 0.15;
                    let end_y = center.y + 50.0 + (grid_lines as f32 / 2.0) * line_spacing * 0.4;

                    let start_x = center.x + x_offset;
                    let end_x = center.x + x_offset * convergence_factor;

                    let color = if i % 5 == 0 {
                        major_grid_color
                    } else {
                        grid_color
                    };
                    let width = if i % 5 == 0 { 1.2 } else { 0.6 };

                    painter.line_segment(
                        [egui::pos2(start_x, start_y), egui::pos2(end_x, end_y)],
                        egui::Stroke::new(width, color),
                    );
                }

                // Draw 3D coordinate axes at origin
                let axis_length = 80.0;
                let axis_start = center + egui::Vec2::new(-180.0, 120.0); // Better positioning

                // X-axis (red, pointing right)
                painter.line_segment(
                    [axis_start, axis_start + egui::Vec2::new(axis_length, 0.0)],
                    egui::Stroke::new(4.0, egui::Color32::RED),
                );
                painter.text(
                    axis_start + egui::Vec2::new(axis_length + 8.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "X",
                    egui::FontId::proportional(16.0),
                    egui::Color32::RED,
                );

                // Y-axis (green, pointing up)
                painter.line_segment(
                    [axis_start, axis_start + egui::Vec2::new(0.0, -axis_length)],
                    egui::Stroke::new(4.0, egui::Color32::GREEN),
                );
                painter.text(
                    axis_start + egui::Vec2::new(0.0, -axis_length - 8.0),
                    egui::Align2::CENTER_BOTTOM,
                    "Y",
                    egui::FontId::proportional(16.0),
                    egui::Color32::GREEN,
                );

                // Z-axis (blue, pointing into perspective)
                let z_end = axis_start + egui::Vec2::new(-axis_length * 0.7, axis_length * 0.5);
                painter.line_segment(
                    [axis_start, z_end],
                    egui::Stroke::new(4.0, egui::Color32::BLUE),
                );
                painter.text(
                    z_end + egui::Vec2::new(-15.0, 8.0),
                    egui::Align2::RIGHT_CENTER,
                    "Z",
                    egui::FontId::proportional(16.0),
                    egui::Color32::BLUE,
                );

                // Add origin point
                painter.circle_filled(axis_start, 5.0, egui::Color32::WHITE);
            }
        }
    }

    fn draw_scene_objects(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &Scene,
        selected_object: Option<u32>,
        camera_pos: Vec3,
        camera_rot: Vec3,
    ) {
        let center = rect.center();

        // Collect objects with their screen positions and depths for proper sorting
        let mut objects_to_draw: Vec<_> = scene
            .objects
            .iter()
            .filter(|(_, object)| object.visible)
            .map(|(object_id, object)| {
                let (screen_pos, depth) = match self.view_mode {
                    ViewMode::Scene2D => {
                        let pos = self.world_to_screen_with_camera(
                            object.transform.position,
                            center,
                            camera_pos,
                            camera_rot,
                        );
                        (pos, 0.0)
                    }
                    ViewMode::Scene3D => {
                        let pos = self.world_to_screen_with_camera(
                            object.transform.position,
                            center,
                            camera_pos,
                            camera_rot,
                        );
                        let depth = (object.transform.position - camera_pos).magnitude();
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

            // Calculate size based on distance and scale for better 3D perspective
            let base_size = object.transform.scale.x;
            let size_factor = match self.view_mode {
                ViewMode::Scene2D => self.zoom_level as f64 * 10.0,
                ViewMode::Scene3D => {
                    // Apply perspective scaling with improved formula
                    let distance = (object.transform.position - camera_pos).magnitude();
                    let perspective_scale = 100.0 / (distance.max(1.0)); // Improved perspective
                    let fov_scale = (self.field_of_view / 75.0) as f64; // Scale with FOV
                    self.zoom_level as f64 * perspective_scale * fov_scale
                }
            };

            // Apply animation transformations (only if playing, don't override manual edits)
            let animated_position = if self.is_playing {
                // Simple rotation animation for demonstration
                // Only apply to objects that haven't been manually moved recently
                let rotation_speed = 1.0;
                let angle = self.animation_time as f64 * rotation_speed;
                Vec3::new(
                    object.transform.position.x * angle.cos()
                        - object.transform.position.z * angle.sin(),
                    object.transform.position.y,
                    object.transform.position.x * angle.sin()
                        + object.transform.position.z * angle.cos(),
                )
            } else {
                // When not playing, always use the object's actual transform position
                // This prevents rubber-banding back to original positions
                object.transform.position
            };

            // Recalculate screen position with animation
            let animated_screen_pos = if self.is_playing {
                self.world_to_screen_with_camera(animated_position, center, camera_pos, camera_rot)
            } else {
                screen_pos
            };

            // Draw object representation based on type
            match &object.object_type {
                GameObjectType::Cube | GameObjectType::RigidBody(_) => {
                    let size = (base_size * size_factor) as f32;
                    painter.rect_filled(
                        egui::Rect::from_center_size(animated_screen_pos, egui::Vec2::splat(size)),
                        0.0,
                        object_color,
                    );
                }
                GameObjectType::Sphere => {
                    let radius = (base_size * size_factor) as f32;
                    painter.circle_filled(animated_screen_pos, radius, object_color);
                }
                GameObjectType::Light => {
                    painter.circle_filled(animated_screen_pos, 8.0, object_color);
                    // Draw light rays
                    for i in 0..8 {
                        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
                        let end_pos = animated_screen_pos
                            + egui::Vec2::new(angle.cos() * 15.0, angle.sin() * 15.0);
                        painter.line_segment(
                            [animated_screen_pos, end_pos],
                            egui::Stroke::new(1.0, object_color),
                        );
                    }
                }
                GameObjectType::Camera => {
                    // Draw camera icon
                    let size = 12.0;
                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            animated_screen_pos,
                            egui::Vec2::new(size, size * 0.7),
                        ),
                        2.0,
                        object_color,
                    );
                    painter.circle_filled(
                        animated_screen_pos + egui::Vec2::new(size * 0.3, 0.0),
                        3.0,
                        egui::Color32::BLACK,
                    );
                }
                _ => {
                    // Default representation
                    painter.circle_filled(animated_screen_pos, 5.0, object_color);
                }
            }

            // Draw object name
            painter.text(
                animated_screen_pos + egui::Vec2::new(0.0, 20.0),
                egui::Align2::CENTER_TOP,
                &object.name,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }
    }

    // Enhanced gizmos drawing with proper camera support
    fn draw_enhanced_gizmos(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &Scene,
        selected_object_id: u32,
    ) {
        if let Some(object) = scene.objects.get(&selected_object_id) {
            let center = rect.center();
            let camera_pos = if self.show_game_view {
                self.game_camera_position
            } else {
                self.camera_position
            };
            let camera_rot = if self.show_game_view {
                self.game_camera_rotation
            } else {
                self.camera_rotation
            };
            let screen_pos = self.world_to_screen_with_camera(
                object.transform.position,
                center,
                camera_pos,
                camera_rot,
            );

            // Draw based on gizmo mode
            match self.gizmo_mode {
                GizmoMode::Translate => self.draw_translate_gizmo(painter, screen_pos),
                GizmoMode::Rotate => self.draw_rotate_gizmo(painter, screen_pos),
                GizmoMode::Scale => self.draw_scale_gizmo(painter, screen_pos),
            }
        }
    }

    fn draw_translate_gizmo(&self, painter: &egui::Painter, center: egui::Pos2) {
        let gizmo_size = 40.0;
        let thickness = 3.0;

        // X axis (red arrow)
        let x_end = center + egui::Vec2::new(gizmo_size, 0.0);
        painter.line_segment(
            [center, x_end],
            egui::Stroke::new(thickness, egui::Color32::RED),
        );
        painter.circle_filled(x_end, 5.0, egui::Color32::RED);

        // Y axis (green arrow)
        let y_end = center + egui::Vec2::new(0.0, -gizmo_size);
        painter.line_segment(
            [center, y_end],
            egui::Stroke::new(thickness, egui::Color32::GREEN),
        );
        painter.circle_filled(y_end, 5.0, egui::Color32::GREEN);

        // Z axis (blue arrow) - only in 3D
        if matches!(self.view_mode, ViewMode::Scene3D) {
            painter.circle_filled(center, 7.0, egui::Color32::BLUE);
            painter.circle_stroke(center, 7.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
        }
    }

    fn draw_rotate_gizmo(&self, painter: &egui::Painter, center: egui::Pos2) {
        let radius = 35.0;
        let thickness = 2.0;

        // X rotation (red circle)
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(thickness, egui::Color32::RED),
        );

        // Y rotation (green circle) - slightly smaller
        painter.circle_stroke(
            center,
            radius * 0.8,
            egui::Stroke::new(thickness, egui::Color32::GREEN),
        );

        // Z rotation (blue circle) - only in 3D
        if matches!(self.view_mode, ViewMode::Scene3D) {
            painter.circle_stroke(
                center,
                radius * 0.6,
                egui::Stroke::new(thickness, egui::Color32::BLUE),
            );
        }
    }

    fn draw_scale_gizmo(&self, painter: &egui::Painter, center: egui::Pos2) {
        let gizmo_size = 35.0;
        let thickness = 3.0;
        let cube_size = 6.0;

        // X axis (red)
        let x_end = center + egui::Vec2::new(gizmo_size, 0.0);
        painter.line_segment(
            [center, x_end],
            egui::Stroke::new(thickness, egui::Color32::RED),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(x_end, egui::Vec2::splat(cube_size)),
            0.0,
            egui::Color32::RED,
        );

        // Y axis (green)
        let y_end = center + egui::Vec2::new(0.0, -gizmo_size);
        painter.line_segment(
            [center, y_end],
            egui::Stroke::new(thickness, egui::Color32::GREEN),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(y_end, egui::Vec2::splat(cube_size)),
            0.0,
            egui::Color32::GREEN,
        );

        // Z axis (blue) - only in 3D
        if matches!(self.view_mode, ViewMode::Scene3D) {
            painter.rect_filled(
                egui::Rect::from_center_size(center, egui::Vec2::splat(cube_size * 1.2)),
                0.0,
                egui::Color32::BLUE,
            );
        }
    }

    // Gizmo interaction methods
    fn check_gizmo_click(
        &self,
        click_pos: egui::Pos2,
        object_id: u32,
        scene: &Scene,
        rect: egui::Rect,
    ) -> Option<GizmoAxis> {
        if let Some(object) = scene.objects.get(&object_id) {
            let center = rect.center();
            let camera_pos = if self.show_game_view {
                self.game_camera_position
            } else {
                self.camera_position
            };
            let camera_rot = if self.show_game_view {
                self.game_camera_rotation
            } else {
                self.camera_rotation
            };
            let gizmo_center = self.world_to_screen_with_camera(
                object.transform.position,
                center,
                camera_pos,
                camera_rot,
            );

            let distance_to_center = (click_pos - gizmo_center).length();
            let gizmo_size = 40.0;

            // Check center first
            if distance_to_center < 10.0 {
                return Some(GizmoAxis::Center);
            }

            // Check X axis
            let x_end = gizmo_center + egui::Vec2::new(gizmo_size, 0.0);
            if (click_pos - x_end).length() < 8.0 {
                return Some(GizmoAxis::X);
            }

            // Check Y axis
            let y_end = gizmo_center + egui::Vec2::new(0.0, -gizmo_size);
            if (click_pos - y_end).length() < 8.0 {
                return Some(GizmoAxis::Y);
            }

            // Check Z axis (only in 3D)
            if matches!(self.view_mode, ViewMode::Scene3D) && distance_to_center < 12.0 {
                return Some(GizmoAxis::Z);
            }
        }
        None
    }

    fn handle_gizmo_drag(
        &mut self,
        delta: egui::Vec2,
        axis: GizmoAxis,
        object_id: u32,
        _rect: egui::Rect,
    ) {
        let sensitivity = 0.01;

        match (self.gizmo_mode, axis) {
            (GizmoMode::Translate, GizmoAxis::X) => {
                let world_delta = crate::gui::Transform {
                    position: Vec3::new((delta.x * sensitivity) as f64, 0.0, 0.0),
                    rotation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, world_delta);
                }
            }
            (GizmoMode::Translate, GizmoAxis::Y) => {
                let world_delta = crate::gui::Transform {
                    position: Vec3::new(0.0, (-(delta.y * sensitivity)) as f64, 0.0),
                    rotation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, world_delta);
                }
            }
            (GizmoMode::Translate, GizmoAxis::Z) => {
                let world_delta = crate::gui::Transform {
                    position: Vec3::new(0.0, 0.0, (delta.y * sensitivity) as f64),
                    rotation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, world_delta);
                }
            }
            (GizmoMode::Translate, GizmoAxis::Center) => {
                // For translation center, move in screen plane
                let world_delta = crate::gui::Transform {
                    position: Vec3::new(
                        (delta.x * sensitivity) as f64,
                        (-(delta.y * sensitivity)) as f64,
                        0.0,
                    ),
                    rotation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, world_delta);
                }
            }
            (GizmoMode::Rotate, _) => {
                let rotation_delta = match axis {
                    GizmoAxis::X => Vec3::new((delta.y * sensitivity) as f64, 0.0, 0.0),
                    GizmoAxis::Y => Vec3::new(0.0, (delta.x * sensitivity) as f64, 0.0),
                    GizmoAxis::Z => Vec3::new(0.0, 0.0, (delta.x * sensitivity) as f64),
                    GizmoAxis::Center => Vec3::new(0.0, (delta.x * sensitivity) as f64, 0.0),
                };
                let transform_delta = crate::gui::Transform {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    rotation: rotation_delta,
                    scale: Vec3::new(1.0, 1.0, 1.0),
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, transform_delta);
                }
            }
            (GizmoMode::Scale, _) => {
                let scale_factor = 1.0 + (delta.length() * sensitivity * 0.1) as f64;
                let scale_delta = match axis {
                    GizmoAxis::X => Vec3::new(scale_factor, 1.0, 1.0),
                    GizmoAxis::Y => Vec3::new(1.0, scale_factor, 1.0),
                    GizmoAxis::Z => Vec3::new(1.0, 1.0, scale_factor),
                    GizmoAxis::Center => Vec3::new(scale_factor, scale_factor, scale_factor),
                };
                let transform_delta = crate::gui::Transform {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Vec3::new(0.0, 0.0, 0.0),
                    scale: scale_delta,
                };
                if let Some(callback) = &self.transform_changed_callback {
                    callback(object_id, transform_delta);
                }
            }
        }
    }

    /// Draw Unity-like orientation gizmo in the top-right corner
    fn draw_orientation_gizmo(&self, painter: &egui::Painter, rect: egui::Rect) {
        if !matches!(self.view_mode, ViewMode::Scene3D) {
            return; // Only show in 3D mode
        }

        let gizmo_size = 80.0;
        let gizmo_rect = egui::Rect::from_center_size(
            rect.right_top() + egui::Vec2::new(-gizmo_size / 2.0 - 15.0, gizmo_size / 2.0 + 15.0),
            egui::Vec2::splat(gizmo_size),
        );

        // Background circle
        painter.circle_filled(
            gizmo_rect.center(),
            gizmo_size * 0.45,
            egui::Color32::from_black_alpha(120),
        );
        painter.circle_stroke(
            gizmo_rect.center(),
            gizmo_size * 0.45,
            egui::Stroke::new(1.5, egui::Color32::from_white_alpha(100)),
        );

        let center = gizmo_rect.center();
        let axis_length = 25.0;

        // Get current camera rotation for proper axis visualization
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };

        // Calculate axis directions based on camera rotation
        let yaw = camera_rot.y.to_radians() as f32;
        let pitch = camera_rot.x.to_radians() as f32;

        // X-axis (Red) - points right, affected by yaw
        let x_dir = egui::Vec2::new(yaw.cos(), yaw.sin() * pitch.cos());
        let x_end = center + x_dir * axis_length;
        painter.line_segment([center, x_end], egui::Stroke::new(3.0, egui::Color32::RED));
        painter.circle_filled(x_end, 4.0, egui::Color32::RED);
        painter.text(
            x_end + egui::Vec2::new(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "X",
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );

        // Y-axis (Green) - points up
        let y_dir = egui::Vec2::new(0.0, -1.0);
        let y_end = center + y_dir * axis_length;
        painter.line_segment(
            [center, y_end],
            egui::Stroke::new(3.0, egui::Color32::GREEN),
        );
        painter.circle_filled(y_end, 4.0, egui::Color32::GREEN);
        painter.text(
            y_end + egui::Vec2::new(0.0, -12.0),
            egui::Align2::CENTER_BOTTOM,
            "Y",
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );

        // Z-axis (Blue) - points into screen, affected by camera rotation
        let z_dir = egui::Vec2::new(-yaw.sin() * 0.7, pitch.sin() * 0.7);
        let z_end = center + z_dir * axis_length;
        painter.line_segment([center, z_end], egui::Stroke::new(3.0, egui::Color32::BLUE));
        painter.circle_filled(z_end, 4.0, egui::Color32::BLUE);
        painter.text(
            z_end + egui::Vec2::new(-8.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "Z",
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );

        // Draw center dot
        painter.circle_filled(center, 3.0, egui::Color32::WHITE);
    }

    /// Draw view mode indicator in bottom-left corner
    fn draw_view_mode_indicator(&self, painter: &egui::Painter, rect: egui::Rect) {
        let indicator_pos = rect.left_bottom() + egui::Vec2::new(10.0, -30.0);

        // Background
        let bg_rect = egui::Rect::from_min_size(indicator_pos, egui::Vec2::new(80.0, 25.0));
        painter.rect_filled(bg_rect, 4.0, egui::Color32::from_black_alpha(180));
        // Border around the indicator
        painter.line_segment(
            [bg_rect.left_top(), bg_rect.right_top()],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(100)),
        );
        painter.line_segment(
            [bg_rect.right_top(), bg_rect.right_bottom()],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(100)),
        );
        painter.line_segment(
            [bg_rect.right_bottom(), bg_rect.left_bottom()],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(100)),
        );
        painter.line_segment(
            [bg_rect.left_bottom(), bg_rect.left_top()],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(100)),
        );

        // View mode text
        let mode_text = match self.view_mode {
            ViewMode::Scene2D => "2D",
            ViewMode::Scene3D => {
                if self.show_game_view {
                    "3D Game"
                } else {
                    "3D Scene"
                }
            }
        };

        let text_color = match self.view_mode {
            ViewMode::Scene2D => egui::Color32::LIGHT_GREEN,
            ViewMode::Scene3D => {
                if self.show_game_view {
                    egui::Color32::LIGHT_BLUE
                } else {
                    egui::Color32::WHITE
                }
            }
        };

        painter.text(
            bg_rect.center(),
            egui::Align2::CENTER_CENTER,
            mode_text,
            egui::FontId::proportional(14.0),
            text_color,
        );
    }

    // Camera information display
    fn draw_camera_info(
        &self,
        ui: &mut egui::Ui,
        _rect: egui::Rect,
        camera_pos: Vec3,
        camera_rot: Vec3,
    ) {
        // Display camera information in the corner
        let info_text = format!(
            "Camera: ({:.1}, {:.1}, {:.1})\nRotation: ({:.1}°, {:.1}°, {:.1}°)\nFOV: {:.1}°\nZoom: {:.2}x",
            camera_pos.x, camera_pos.y, camera_pos.z,
            camera_rot.x, camera_rot.y, camera_rot.z,
            self.field_of_view,
            self.zoom_level
        );

        let info_rect = egui::Rect::from_min_size(
            _rect.min + egui::Vec2::new(10.0, 10.0),
            egui::Vec2::new(200.0, 100.0),
        );
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(info_rect));
        child_ui.label(
            egui::RichText::new(info_text)
                .small()
                .color(egui::Color32::WHITE),
        );
    }

    /// Draw game camera preview window (picture-in-picture)
    fn draw_game_camera_preview(&self, painter: &egui::Painter, rect: egui::Rect, scene: &Scene) {
        if !self.show_game_view {
            return;
        }

        // Calculate preview window position and size
        let preview_size = self.camera_preview_size;
        let preview_rect = egui::Rect::from_min_size(
            rect.max - egui::Vec2::new(preview_size + 20.0, preview_size + 20.0),
            egui::Vec2::splat(preview_size),
        );

        // Draw preview window border
        let stroke = egui::Stroke::new(2.0, egui::Color32::YELLOW);
        painter.line_segment([preview_rect.left_top(), preview_rect.right_top()], stroke);
        painter.line_segment(
            [preview_rect.right_top(), preview_rect.right_bottom()],
            stroke,
        );
        painter.line_segment(
            [preview_rect.right_bottom(), preview_rect.left_bottom()],
            stroke,
        );
        painter.line_segment(
            [preview_rect.left_bottom(), preview_rect.left_top()],
            stroke,
        );

        // Clear preview background
        painter.rect_filled(
            preview_rect,
            5.0,
            egui::Color32::from_rgba_unmultiplied(20, 20, 30, 200),
        );

        // Render scene from game camera perspective in preview window
        let preview_center = preview_rect.center();
        for (_, object) in &scene.objects {
            if !object.visible {
                continue;
            }

            let screen_pos = self.world_to_screen_with_camera(
                object.transform.position,
                preview_center,
                self.game_camera_position,
                self.game_camera_rotation,
            );

            // Check if object is within preview bounds
            if preview_rect.contains(screen_pos) {
                let object_color = match &object.object_type {
                    GameObjectType::Cube => egui::Color32::LIGHT_BLUE,
                    GameObjectType::Sphere => egui::Color32::LIGHT_RED,
                    GameObjectType::Cylinder => egui::Color32::LIGHT_GREEN,
                    GameObjectType::Plane => egui::Color32::GRAY,
                    GameObjectType::Light => egui::Color32::YELLOW,
                    GameObjectType::Camera => egui::Color32::BLUE,
                    _ => egui::Color32::WHITE,
                };

                // Scale objects smaller for preview
                let preview_scale = 0.3;
                match &object.object_type {
                    GameObjectType::Sphere => {
                        painter.circle_filled(screen_pos, 3.0 * preview_scale, object_color);
                    }
                    GameObjectType::Cube => {
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                screen_pos,
                                egui::Vec2::splat(6.0 * preview_scale),
                            ),
                            0.0,
                            object_color,
                        );
                    }
                    _ => {
                        painter.circle_filled(screen_pos, 2.0 * preview_scale, object_color);
                    }
                }
            }
        }

        // Draw "Game Camera" label
        painter.text(
            preview_rect.min + egui::Vec2::new(5.0, 5.0),
            egui::Align2::LEFT_TOP,
            "Game Camera",
            egui::FontId::proportional(10.0),
            egui::Color32::YELLOW,
        );
    }

    // Camera helper methods
    fn get_forward_vector_for_camera(&self, camera_rot: Vec3) -> Vec3 {
        let pitch = camera_rot.x.to_radians();
        let yaw = camera_rot.y.to_radians();

        Vec3::new(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        )
    }

    fn get_right_vector_for_camera(&self, camera_rot: Vec3) -> Vec3 {
        let yaw = camera_rot.y.to_radians();
        Vec3::new(yaw.cos(), 0.0, -yaw.sin())
    }

    fn get_up_vector_for_camera(&self, camera_rot: Vec3) -> Vec3 {
        let forward = self.get_forward_vector_for_camera(camera_rot);
        let right = self.get_right_vector_for_camera(camera_rot);
        right.cross(forward).normalized()
    }

    // Enhanced coordinate transformation methods
    fn world_to_screen_with_camera(
        &self,
        world_pos: Vec3,
        screen_center: egui::Pos2,
        camera_pos: Vec3,
        camera_rot: Vec3,
    ) -> egui::Pos2 {
        let camera_space =
            self.transform_world_to_camera_with_params(world_pos, camera_pos, camera_rot);
        let projected = self.apply_perspective_projection(camera_space);

        screen_center + egui::Vec2::new(projected.x as f32 * 100.0, projected.y as f32 * 100.0)
    }

    fn screen_to_world_with_camera(
        &self,
        screen_pos: egui::Pos2,
        screen_center: egui::Pos2,
        camera_pos: Vec3,
        camera_rot: Vec3,
    ) -> Vec3 {
        let screen_delta = screen_pos - screen_center;
        let depth = 5.0; // Default depth for screen-to-world conversion

        let forward = self.get_forward_vector_for_camera(camera_rot);
        let right = self.get_right_vector_for_camera(camera_rot);
        let up = self.get_up_vector_for_camera(camera_rot);

        camera_pos
            + forward * depth
            + right * (screen_delta.x as f64 * 0.01)
            + up * (-screen_delta.y as f64 * 0.01)
    }

    fn transform_world_to_camera_with_params(
        &self,
        world_pos: Vec3,
        camera_pos: Vec3,
        camera_rot: Vec3,
    ) -> Vec3 {
        // Transform world position relative to camera
        let relative_pos = world_pos - camera_pos;

        // Apply camera rotation (inverse transform)
        let forward = self.get_forward_vector_for_camera(camera_rot);
        let right = self.get_right_vector_for_camera(camera_rot);
        let up = self.get_up_vector_for_camera(camera_rot);

        Vec3::new(
            relative_pos.dot(right),
            relative_pos.dot(up),
            relative_pos.dot(forward),
        )
    }

    // Legacy compatibility functions for existing codebase

    /// Legacy world to screen conversion (uses current camera)
    pub fn world_to_screen(&self, world_pos: Vec3, screen_center: egui::Pos2) -> egui::Pos2 {
        let camera_pos = if self.show_game_view {
            self.game_camera_position
        } else {
            self.camera_position
        };
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.world_to_screen_with_camera(world_pos, screen_center, camera_pos, camera_rot)
    }

    /// Legacy screen to world conversion (uses current camera)
    pub fn screen_to_world(&self, screen_pos: egui::Pos2, screen_center: egui::Pos2) -> Vec3 {
        let camera_pos = if self.show_game_view {
            self.game_camera_position
        } else {
            self.camera_position
        };
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.screen_to_world_with_camera(screen_pos, screen_center, camera_pos, camera_rot)
    }

    /// Legacy camera forward vector
    pub fn get_forward_vector(&self) -> Vec3 {
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.get_forward_vector_for_camera(camera_rot)
    }

    /// Legacy camera right vector
    pub fn get_right_vector(&self) -> Vec3 {
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.get_right_vector_for_camera(camera_rot)
    }

    /// Legacy camera up vector
    pub fn get_up_vector(&self) -> Vec3 {
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.get_up_vector_for_camera(camera_rot)
    }

    /// Legacy transform world to camera
    pub fn transform_world_to_camera(&self, world_pos: Vec3) -> Vec3 {
        let camera_pos = if self.show_game_view {
            self.game_camera_position
        } else {
            self.camera_position
        };
        let camera_rot = if self.show_game_view {
            self.game_camera_rotation
        } else {
            self.camera_rotation
        };
        self.transform_world_to_camera_with_params(world_pos, camera_pos, camera_rot)
    }

    /// Legacy apply perspective projection
    fn apply_perspective_projection(&self, camera_pos: Vec3) -> Vec3 {
        let fov = self.field_of_view.to_radians() as f64;
        let _near = self.near_clip as f64;
        let _far = self.far_clip as f64;

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

    /// Legacy draw gizmos function
    pub fn draw_gizmos(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &Scene,
        selected_object_id: u32,
    ) {
        self.draw_enhanced_gizmos(painter, rect, scene, selected_object_id);
    }

    /// Reset camera function
    fn reset_camera(&mut self) {
        match self.view_mode {
            ViewMode::Scene2D => {
                if self.show_game_view {
                    self.game_camera_position = Vec3::new(0.0, 0.0, 0.0);
                    self.game_camera_rotation = Vec3::new(0.0, 0.0, 0.0);
                } else {
                    self.camera_position = Vec3::new(0.0, 0.0, 0.0);
                    self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
                }
            }
            ViewMode::Scene3D => {
                if self.show_game_view {
                    self.game_camera_position = Vec3::new(0.0, 2.0, 5.0);
                    self.game_camera_rotation = Vec3::new(-10.0, 0.0, 0.0);
                } else {
                    self.camera_position = Vec3::new(0.0, 5.0, 10.0);
                    self.camera_rotation = Vec3::new(-20.0, 0.0, 0.0);
                }
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
