// Main Physics Simulation GUI Module
// Unity-style physics simulation interface

use eframe::egui;
use std::collections::HashMap;

/// 3D Vector for positions, rotations, scale
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

/// Transform component with position, rotation, and scale
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles in degrees
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: Vec3::one(),
        }
    }
}

/// Different types of game objects
#[derive(Debug, Clone, PartialEq)]
pub enum GameObjectType {
    Cube,
    Sphere,
    Cylinder,
    Plane,
    Camera,
    Light,
    Empty,
}

/// Rigid body physics component
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub mass: f32,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub is_kinematic: bool,
    pub use_gravity: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: Vec3::zero(),
            angular_velocity: Vec3::zero(),
            is_kinematic: false,
            use_gravity: true,
        }
    }
}

/// Mesh renderer component
#[derive(Debug, Clone)]
pub struct MeshRenderer {
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub material_color: [f32; 4], // RGBA
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            cast_shadows: true,
            receive_shadows: true,
            material_color: [0.8, 0.8, 0.8, 1.0], // Light gray
        }
    }
}

/// Game object with components
#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: usize,
    pub name: String,
    pub object_type: GameObjectType,
    pub transform: Transform,
    pub rigid_body: Option<RigidBody>,
    pub mesh_renderer: Option<MeshRenderer>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
}

impl GameObject {
    pub fn new(id: usize, name: String, object_type: GameObjectType) -> Self {
        let mut obj = Self {
            id,
            name,
            object_type: object_type.clone(),
            transform: Transform::default(),
            rigid_body: None,
            mesh_renderer: None,
            children: Vec::new(),
            parent: None,
        };

        // Add default components based on type
        match object_type {
            GameObjectType::Cube | GameObjectType::Sphere | GameObjectType::Cylinder => {
                obj.mesh_renderer = Some(MeshRenderer::default());
                obj.rigid_body = Some(RigidBody::default());
            }
            GameObjectType::Plane => {
                obj.mesh_renderer = Some(MeshRenderer::default());
            }
            GameObjectType::Camera => {
                obj.transform.position = Vec3::new(0.0, 1.0, -10.0);
            }
            GameObjectType::Light => {
                obj.transform.position = Vec3::new(0.0, 3.0, 0.0);
            }
            GameObjectType::Empty => {}
        }

        obj
    }
}

/// Gizmo manipulation modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Physics simulation state
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    pub gravity: Vec3,
    pub time_step: f32,
    pub is_paused: bool,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 1.0 / 60.0,
            is_paused: true,
        }
    }
}

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
    /// Selected object ID
    selected_object: Option<usize>,
    /// Scene objects
    game_objects: HashMap<usize, GameObject>,
    next_id: usize,
    /// Gizmo state
    gizmo_mode: GizmoMode,
    gizmo_dragging: bool,
    drag_start_pos: Option<egui::Pos2>,
    /// Physics world
    physics_world: PhysicsWorld,
    /// Camera controls
    camera_position: Vec3,
    camera_rotation: Vec3,
    camera_zoom: f32,
    /// Scene view interaction
    scene_hovered: bool,
    mouse_pos_in_scene: egui::Pos2,
}

impl PhysicsEditorApp {
    pub fn new() -> Self {
        let mut app = Self {
            show_about: false,
            show_preferences: false,
            is_playing: false,
            is_paused: false,
            console_messages: vec!["Physics Editor Started".to_string()],
            selected_object: None,
            game_objects: HashMap::new(),
            next_id: 1,
            gizmo_mode: GizmoMode::Translate,
            gizmo_dragging: false,
            drag_start_pos: None,
            physics_world: PhysicsWorld::default(),
            camera_position: Vec3::new(0.0, 0.0, -10.0),
            camera_rotation: Vec3::new(15.0, 0.0, 0.0),
            camera_zoom: 1.0,
            scene_hovered: false,
            mouse_pos_in_scene: egui::Pos2::ZERO,
        };

        // Create default scene objects
        app.create_default_scene();
        app
    }

    fn create_default_scene(&mut self) {
        // Main Camera
        let camera = GameObject::new(
            self.next_id,
            "Main Camera".to_string(),
            GameObjectType::Camera,
        );
        self.game_objects.insert(self.next_id, camera);
        self.next_id += 1;

        // Directional Light
        let mut light = GameObject::new(
            self.next_id,
            "Directional Light".to_string(),
            GameObjectType::Light,
        );
        light.transform.rotation = Vec3::new(50.0, -30.0, 0.0);
        self.game_objects.insert(self.next_id, light);
        self.next_id += 1;

        // Default Cube
        let mut cube = GameObject::new(self.next_id, "Cube".to_string(), GameObjectType::Cube);
        cube.transform.position = Vec3::new(0.0, 0.0, 0.0);
        self.game_objects.insert(self.next_id, cube);
        self.next_id += 1;

        // Ground Plane
        let mut plane = GameObject::new(self.next_id, "Plane".to_string(), GameObjectType::Plane);
        plane.transform.position = Vec3::new(0.0, -1.0, 0.0);
        plane.transform.scale = Vec3::new(10.0, 1.0, 10.0);
        if let Some(renderer) = &mut plane.mesh_renderer {
            renderer.material_color = [0.6, 0.8, 0.6, 1.0]; // Light green
        }
        self.game_objects.insert(self.next_id, plane);
        self.next_id += 1;

        self.add_console_message(
            "Default scene created with Camera, Light, Cube, and Plane".to_string(),
        );
    }

    fn create_object(&mut self, object_type: GameObjectType, base_name: String) {
        // Find a unique name by appending numbers if needed
        let mut name = base_name.clone();
        let mut counter = 1;
        while self.game_objects.values().any(|obj| obj.name == name) {
            name = format!("{} ({})", base_name, counter);
            counter += 1;
        }

        let obj = GameObject::new(self.next_id, name.clone(), object_type);
        self.game_objects.insert(self.next_id, obj);
        self.selected_object = Some(self.next_id);
        self.next_id += 1;

        self.add_console_message(format!("Created {}", name));
    }

    fn step_physics(&mut self) {
        if !self.physics_world.is_paused {
            // Simple physics step - apply gravity to all rigidbodies
            for obj in self.game_objects.values_mut() {
                if let Some(ref mut rb) = obj.rigid_body {
                    if rb.use_gravity && !rb.is_kinematic {
                        // Apply gravity
                        rb.velocity.y +=
                            self.physics_world.gravity.y * self.physics_world.time_step;

                        // Update position based on velocity
                        obj.transform.position.x += rb.velocity.x * self.physics_world.time_step;
                        obj.transform.position.y += rb.velocity.y * self.physics_world.time_step;
                        obj.transform.position.z += rb.velocity.z * self.physics_world.time_step;

                        // Simple ground collision at y = -1
                        if obj.transform.position.y < -0.5 {
                            obj.transform.position.y = -0.5;
                            rb.velocity.y = -rb.velocity.y * 0.5; // Bounce with energy loss
                        }
                    }
                }
            }
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
                    self.physics_world.is_paused = !self.is_playing;
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
                    self.physics_world.is_paused = true;
                    // Reset all objects to initial positions
                    for obj in self.game_objects.values_mut() {
                        if let Some(ref mut rb) = obj.rigid_body {
                            rb.velocity = Vec3::zero();
                            rb.angular_velocity = Vec3::zero();
                        }
                    }
                    self.add_console_message("Simulation stopped and reset".to_string());
                }

                if ui.button("⏭").clicked() {
                    self.step_physics();
                    self.add_console_message("Advanced one frame".to_string());
                }

                ui.separator();

                // Transform tools
                if ui
                    .selectable_label(self.gizmo_mode == GizmoMode::Translate, "📍")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Translate;
                    self.add_console_message("Selected move tool".to_string());
                }
                if ui
                    .selectable_label(self.gizmo_mode == GizmoMode::Rotate, "🔄")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Rotate;
                    self.add_console_message("Selected rotate tool".to_string());
                }
                if ui
                    .selectable_label(self.gizmo_mode == GizmoMode::Scale, "📏")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Scale;
                    self.add_console_message("Selected scale tool".to_string());
                }

                ui.separator();

                // View options
                if ui.button("🔍").clicked() {
                    // Frame selected object
                    if let Some(selected_id) = self.selected_object {
                        if let Some(obj) = self.game_objects.get(&selected_id) {
                            self.camera_position = Vec3::new(
                                obj.transform.position.x,
                                obj.transform.position.y + 2.0,
                                obj.transform.position.z - 5.0,
                            );
                            self.add_console_message(format!("Framed {}", obj.name));
                        }
                    } else {
                        self.add_console_message("No object selected to frame".to_string());
                    }
                }

                ui.separator();

                // Physics controls
                ui.label(format!("Gravity: {:.1}", self.physics_world.gravity.y));
                ui.label(format!("Objects: {}", self.game_objects.len()));
            });
        });
    }

    fn show_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = if self.is_playing {
                    "Playing"
                } else if self.physics_world.is_paused {
                    "Paused"
                } else {
                    "Ready"
                };
                ui.label(status);
                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| 1.0 / i.stable_dt)));
                ui.separator();
                ui.label(format!("Objects: {}", self.game_objects.len()));
                ui.separator();
                if let Some(selected_id) = self.selected_object {
                    if let Some(obj) = self.game_objects.get(&selected_id) {
                        ui.label(format!("Selected: {}", obj.name));
                    }
                } else {
                    ui.label("No selection");
                }
                ui.separator();
                ui.label(format!("Gizmo: {:?}", self.gizmo_mode));
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

                // Show actual game objects from the HashMap
                let mut objects: Vec<_> = self
                    .game_objects
                    .iter()
                    .map(|(id, obj)| (*id, obj.name.clone(), obj.object_type.clone()))
                    .collect();
                objects.sort_by_key(|(id, _, _)| *id);

                for (obj_id, obj_name, obj_type) in objects {
                    let icon = match obj_type {
                        GameObjectType::Camera => "📷",
                        GameObjectType::Light => "💡",
                        GameObjectType::Cube => "📦",
                        GameObjectType::Sphere => "🔵",
                        GameObjectType::Cylinder => "⚪",
                        GameObjectType::Plane => "📄",
                        GameObjectType::Empty => "📍",
                    };

                    let is_selected = self.selected_object == Some(obj_id);

                    if ui
                        .selectable_label(is_selected, format!("{} {}", icon, obj_name))
                        .clicked()
                    {
                        self.selected_object = Some(obj_id);
                        self.add_console_message(format!("Selected {}", obj_name));
                    }
                }

                ui.separator();

                // Add object buttons
                ui.horizontal(|ui| {
                    if ui.small_button("+ Empty").clicked() {
                        self.create_object(GameObjectType::Empty, "Empty".to_string());
                    }
                    if ui.small_button("+ Cube").clicked() {
                        self.create_object(GameObjectType::Cube, "Cube".to_string());
                    }
                });
                ui.horizontal(|ui| {
                    if ui.small_button("+ Sphere").clicked() {
                        self.create_object(GameObjectType::Sphere, "Sphere".to_string());
                    }
                    if ui.small_button("+ Light").clicked() {
                        self.create_object(GameObjectType::Light, "Light".to_string());
                    }
                });
            });

        // Right panel - Inspector
        egui::SidePanel::right("inspector")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.separator();

                if let Some(selected_id) = self.selected_object {
                    if let Some(obj) = self.game_objects.get_mut(&selected_id) {
                        ui.label(format!("Name: {}", obj.name));
                        ui.separator();

                        // Transform Component
                        ui.collapsing("Transform", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                            });
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.position.x)
                                        .prefix("X: ")
                                        .speed(0.1),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.position.y)
                                        .prefix("Y: ")
                                        .speed(0.1),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.position.z)
                                        .prefix("Z: ")
                                        .speed(0.1),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Rotation:");
                            });
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.rotation.x)
                                        .prefix("X: ")
                                        .speed(1.0)
                                        .suffix("°"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.rotation.y)
                                        .prefix("Y: ")
                                        .speed(1.0)
                                        .suffix("°"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.rotation.z)
                                        .prefix("Z: ")
                                        .speed(1.0)
                                        .suffix("°"),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Scale:");
                            });
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.scale.x)
                                        .prefix("X: ")
                                        .speed(0.01)
                                        .range(0.01..=100.0),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.scale.y)
                                        .prefix("Y: ")
                                        .speed(0.01)
                                        .range(0.01..=100.0),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut obj.transform.scale.z)
                                        .prefix("Z: ")
                                        .speed(0.01)
                                        .range(0.01..=100.0),
                                );
                            });
                        });

                        ui.separator();

                        // Mesh Renderer Component
                        if let Some(ref mut renderer) = obj.mesh_renderer {
                            ui.collapsing("Mesh Renderer", |ui| {
                                ui.checkbox(&mut renderer.cast_shadows, "Cast Shadows");
                                ui.checkbox(&mut renderer.receive_shadows, "Receive Shadows");

                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    ui.color_edit_button_rgba_unmultiplied(
                                        &mut renderer.material_color,
                                    );
                                });
                            });
                            ui.separator();
                        }

                        // Rigidbody Component
                        if let Some(ref mut rigidbody) = obj.rigid_body {
                            ui.collapsing("Rigidbody", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Mass:");
                                    ui.add(
                                        egui::DragValue::new(&mut rigidbody.mass)
                                            .speed(0.1)
                                            .range(0.1..=1000.0),
                                    );
                                });

                                ui.checkbox(&mut rigidbody.is_kinematic, "Is Kinematic");
                                ui.checkbox(&mut rigidbody.use_gravity, "Use Gravity");

                                ui.horizontal(|ui| {
                                    ui.label("Velocity:");
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut rigidbody.velocity.x)
                                            .prefix("X: ")
                                            .speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut rigidbody.velocity.y)
                                            .prefix("Y: ")
                                            .speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut rigidbody.velocity.z)
                                            .prefix("Z: ")
                                            .speed(0.1),
                                    );
                                });
                            });
                            ui.separator();
                        }

                        // Add Component section
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Add Component:");
                        });

                        let mut add_mesh_renderer = false;
                        let mut add_rigidbody = false;

                        ui.horizontal(|ui| {
                            if obj.mesh_renderer.is_none() && ui.button("+ Mesh Renderer").clicked()
                            {
                                add_mesh_renderer = true;
                            }
                            if obj.rigid_body.is_none() && ui.button("+ Rigidbody").clicked() {
                                add_rigidbody = true;
                            }
                        });

                        // Apply component additions
                        if add_mesh_renderer {
                            obj.mesh_renderer = Some(MeshRenderer::default());
                            // We'll add the console message after the borrow ends
                        }
                        if add_rigidbody {
                            obj.rigid_body = Some(RigidBody::default());
                            // We'll add the console message after the borrow ends
                        }

                        ui.separator();

                        // Delete object button
                        if ui.button("🗑 Delete Object").clicked() {
                            let obj_name = obj.name.clone();
                            self.game_objects.remove(&selected_id);
                            self.selected_object = None;
                            self.add_console_message(format!("Deleted {}", obj_name));
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
            let grid_spacing = 30.0;
            let grid_lines = 20;

            for i in 0..=grid_lines {
                let x = rect.left() + (i as f32 / grid_lines as f32) * rect.width();
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(0.5, grid_color),
                );
            }
            for i in 0..=grid_lines {
                let y = rect.top() + (i as f32 / grid_lines as f32) * rect.height();
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(0.5, grid_color),
                );
            }

            // Draw world origin axes
            let center = rect.center();
            let axis_length = 40.0;

            // X axis (red)
            painter.line_segment(
                [center, egui::pos2(center.x + axis_length, center.y)],
                egui::Stroke::new(3.0, egui::Color32::RED),
            );
            painter.text(
                egui::pos2(center.x + axis_length + 5.0, center.y - 10.0),
                egui::Align2::LEFT_CENTER,
                "X",
                egui::FontId::default(),
                egui::Color32::RED,
            );

            // Y axis (green)
            painter.line_segment(
                [center, egui::pos2(center.x, center.y - axis_length)],
                egui::Stroke::new(3.0, egui::Color32::GREEN),
            );
            painter.text(
                egui::pos2(center.x + 5.0, center.y - axis_length - 5.0),
                egui::Align2::LEFT_CENTER,
                "Y",
                egui::FontId::default(),
                egui::Color32::GREEN,
            );

            // Render game objects
            for obj in self.game_objects.values() {
                self.render_game_object(&painter, &rect, obj);
            }

            // Draw gizmos for selected object
            if let Some(selected_id) = self.selected_object {
                if let Some(obj) = self.game_objects.get(&selected_id) {
                    self.draw_gizmos(&painter, &rect, obj);
                }
            }

            // Handle viewport interaction
            self.scene_hovered = response.hovered();
            self.mouse_pos_in_scene = response.hover_pos().unwrap_or(egui::Pos2::ZERO);

            if response.clicked() {
                self.add_console_message("Clicked in viewport".to_string());
            }

            if response.dragged() && self.selected_object.is_some() {
                let drag_delta = response.drag_delta();
                if let Some(selected_id) = self.selected_object {
                    if let Some(obj) = self.game_objects.get_mut(&selected_id) {
                        match self.gizmo_mode {
                            GizmoMode::Translate => {
                                obj.transform.position.x += drag_delta.x * 0.01;
                                obj.transform.position.y -= drag_delta.y * 0.01;
                            }
                            GizmoMode::Rotate => {
                                obj.transform.rotation.x += drag_delta.y * 0.5;
                                obj.transform.rotation.y += drag_delta.x * 0.5;
                            }
                            GizmoMode::Scale => {
                                let scale_factor = 1.0 + drag_delta.x * 0.01;
                                obj.transform.scale.x *= scale_factor;
                                obj.transform.scale.y *= scale_factor;
                                obj.transform.scale.z *= scale_factor;
                                obj.transform.scale.x = obj.transform.scale.x.max(0.1);
                                obj.transform.scale.y = obj.transform.scale.y.max(0.1);
                                obj.transform.scale.z = obj.transform.scale.z.max(0.1);
                            }
                        }
                    }
                }
            }
        });
    }

    fn render_game_object(&self, painter: &egui::Painter, rect: &egui::Rect, obj: &GameObject) {
        let center = rect.center();

        // Calculate screen position based on transform
        let screen_pos = egui::pos2(
            center.x + obj.transform.position.x * 50.0,
            center.y - obj.transform.position.y * 50.0,
        );

        // Get object color
        let color = if let Some(ref renderer) = obj.mesh_renderer {
            egui::Color32::from_rgba_unmultiplied(
                (renderer.material_color[0] * 255.0) as u8,
                (renderer.material_color[1] * 255.0) as u8,
                (renderer.material_color[2] * 255.0) as u8,
                (renderer.material_color[3] * 255.0) as u8,
            )
        } else {
            egui::Color32::WHITE
        };

        let size = 25.0 * obj.transform.scale.x;

        match obj.object_type {
            GameObjectType::Cube => {
                // Draw simple wireframe cube using line segments
                let half_size = size / 2.0;
                let corners = [
                    egui::pos2(screen_pos.x - half_size, screen_pos.y - half_size), // top-left
                    egui::pos2(screen_pos.x + half_size, screen_pos.y - half_size), // top-right
                    egui::pos2(screen_pos.x + half_size, screen_pos.y + half_size), // bottom-right
                    egui::pos2(screen_pos.x - half_size, screen_pos.y + half_size), // bottom-left
                ];

                let stroke = egui::Stroke::new(2.0, color);
                painter.line_segment([corners[0], corners[1]], stroke);
                painter.line_segment([corners[1], corners[2]], stroke);
                painter.line_segment([corners[2], corners[3]], stroke);
                painter.line_segment([corners[3], corners[0]], stroke);
            }
            GameObjectType::Sphere => {
                painter.circle_stroke(screen_pos, size / 2.0, egui::Stroke::new(2.0, color));
            }
            GameObjectType::Plane => {
                let plane_width = size * obj.transform.scale.x;
                let plane_height = size * obj.transform.scale.z * 0.1; // Make it flat
                let corners = [
                    egui::pos2(
                        screen_pos.x - plane_width / 2.0,
                        screen_pos.y - plane_height / 2.0,
                    ),
                    egui::pos2(
                        screen_pos.x + plane_width / 2.0,
                        screen_pos.y - plane_height / 2.0,
                    ),
                    egui::pos2(
                        screen_pos.x + plane_width / 2.0,
                        screen_pos.y + plane_height / 2.0,
                    ),
                    egui::pos2(
                        screen_pos.x - plane_width / 2.0,
                        screen_pos.y + plane_height / 2.0,
                    ),
                ];

                let stroke = egui::Stroke::new(2.0, color);
                painter.line_segment([corners[0], corners[1]], stroke);
                painter.line_segment([corners[1], corners[2]], stroke);
                painter.line_segment([corners[2], corners[3]], stroke);
                painter.line_segment([corners[3], corners[0]], stroke);
            }
            GameObjectType::Camera => {
                // Draw camera as a simple rectangle with a circle
                let cam_size = size * 0.8;
                let corners = [
                    egui::pos2(screen_pos.x - cam_size / 2.0, screen_pos.y - cam_size / 3.0),
                    egui::pos2(screen_pos.x + cam_size / 2.0, screen_pos.y - cam_size / 3.0),
                    egui::pos2(screen_pos.x + cam_size / 2.0, screen_pos.y + cam_size / 3.0),
                    egui::pos2(screen_pos.x - cam_size / 2.0, screen_pos.y + cam_size / 3.0),
                ];

                let stroke = egui::Stroke::new(2.0, egui::Color32::BLUE);
                painter.line_segment([corners[0], corners[1]], stroke);
                painter.line_segment([corners[1], corners[2]], stroke);
                painter.line_segment([corners[2], corners[3]], stroke);
                painter.line_segment([corners[3], corners[0]], stroke);
                painter.circle_stroke(screen_pos, size * 0.15, stroke);
            }
            GameObjectType::Light => {
                // Draw light icon with rays
                painter.circle_stroke(
                    screen_pos,
                    size * 0.3,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                );
                for i in 0..8 {
                    let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                    let start = screen_pos + egui::vec2(angle.cos(), angle.sin()) * size * 0.4;
                    let end = screen_pos + egui::vec2(angle.cos(), angle.sin()) * size * 0.6;
                    painter
                        .line_segment([start, end], egui::Stroke::new(2.0, egui::Color32::YELLOW));
                }
            }
            GameObjectType::Empty => {
                // Draw simple crosshair
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x - 10.0, screen_pos.y),
                        egui::pos2(screen_pos.x + 10.0, screen_pos.y),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x, screen_pos.y - 10.0),
                        egui::pos2(screen_pos.x, screen_pos.y + 10.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                );
            }
            GameObjectType::Cylinder => {
                // Draw cylinder as two circles with connecting lines
                painter.circle_stroke(screen_pos, size / 2.0, egui::Stroke::new(2.0, color));
                let top_center = egui::pos2(screen_pos.x, screen_pos.y - size / 3.0);
                painter.circle_stroke(
                    top_center,
                    size / 2.0,
                    egui::Stroke::new(1.0, color.gamma_multiply(0.7)),
                );

                // Connecting lines
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x - size / 2.0, screen_pos.y),
                        egui::pos2(top_center.x - size / 2.0, top_center.y),
                    ],
                    egui::Stroke::new(1.0, color.gamma_multiply(0.7)),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x + size / 2.0, screen_pos.y),
                        egui::pos2(top_center.x + size / 2.0, top_center.y),
                    ],
                    egui::Stroke::new(1.0, color.gamma_multiply(0.7)),
                );
            }
        }

        // Draw object name
        painter.text(
            egui::pos2(screen_pos.x, screen_pos.y + size + 10.0),
            egui::Align2::CENTER_TOP,
            &obj.name,
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
    }

    fn draw_gizmos(&self, painter: &egui::Painter, rect: &egui::Rect, obj: &GameObject) {
        let center = rect.center();
        let screen_pos = egui::pos2(
            center.x + obj.transform.position.x * 50.0,
            center.y - obj.transform.position.y * 50.0,
        );

        let gizmo_size = 40.0;

        match self.gizmo_mode {
            GizmoMode::Translate => {
                // Draw translation gizmo with colored arrows
                // X axis (red arrow)
                painter.line_segment(
                    [
                        screen_pos,
                        egui::pos2(screen_pos.x + gizmo_size, screen_pos.y),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::RED),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x + gizmo_size, screen_pos.y),
                        egui::pos2(screen_pos.x + gizmo_size - 8.0, screen_pos.y - 4.0),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::RED),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x + gizmo_size, screen_pos.y),
                        egui::pos2(screen_pos.x + gizmo_size - 8.0, screen_pos.y + 4.0),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::RED),
                );

                // Y axis (green arrow)
                painter.line_segment(
                    [
                        screen_pos,
                        egui::pos2(screen_pos.x, screen_pos.y - gizmo_size),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::GREEN),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x, screen_pos.y - gizmo_size),
                        egui::pos2(screen_pos.x - 4.0, screen_pos.y - gizmo_size + 8.0),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::GREEN),
                );
                painter.line_segment(
                    [
                        egui::pos2(screen_pos.x, screen_pos.y - gizmo_size),
                        egui::pos2(screen_pos.x + 4.0, screen_pos.y - gizmo_size + 8.0),
                    ],
                    egui::Stroke::new(3.0, egui::Color32::GREEN),
                );

                // Z axis (blue arrow) - simulated with diagonal
                let z_end = egui::pos2(
                    screen_pos.x + gizmo_size * 0.7,
                    screen_pos.y + gizmo_size * 0.7,
                );
                painter.line_segment(
                    [screen_pos, z_end],
                    egui::Stroke::new(3.0, egui::Color32::BLUE),
                );
            }
            GizmoMode::Rotate => {
                // Draw rotation gizmo with colored circles
                painter.circle_stroke(
                    screen_pos,
                    gizmo_size * 0.8,
                    egui::Stroke::new(2.0, egui::Color32::RED),
                );
                painter.circle_stroke(
                    screen_pos,
                    gizmo_size * 0.9,
                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                );
                painter.circle_stroke(
                    screen_pos,
                    gizmo_size,
                    egui::Stroke::new(2.0, egui::Color32::BLUE),
                );
            }
            GizmoMode::Scale => {
                // Draw scale gizmo with boxes on the ends
                let box_size = 6.0;

                // X axis
                painter.line_segment(
                    [
                        screen_pos,
                        egui::pos2(screen_pos.x + gizmo_size, screen_pos.y),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::RED),
                );
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(screen_pos.x + gizmo_size, screen_pos.y),
                        egui::vec2(box_size, box_size),
                    ),
                    0.0,
                    egui::Color32::RED,
                );

                // Y axis
                painter.line_segment(
                    [
                        screen_pos,
                        egui::pos2(screen_pos.x, screen_pos.y - gizmo_size),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                );
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        egui::pos2(screen_pos.x, screen_pos.y - gizmo_size),
                        egui::vec2(box_size, box_size),
                    ),
                    0.0,
                    egui::Color32::GREEN,
                );

                // Center box for uniform scaling
                painter.rect_filled(
                    egui::Rect::from_center_size(screen_pos, egui::vec2(box_size, box_size)),
                    0.0,
                    egui::Color32::WHITE,
                );
            }
        }
    }
}

impl eframe::App for PhysicsEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Run physics simulation if playing
        if self.is_playing && !self.physics_world.is_paused {
            self.step_physics();
        }

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
                    ui.label("Features:");
                    ui.label("• Interactive Unity-style gizmos");
                    ui.label("• Real-time physics simulation");
                    ui.label("• Component-based architecture");
                    ui.label("• GameObject hierarchy");
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
                    ui.label("Physics Settings");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Gravity Y:");
                        ui.add(
                            egui::DragValue::new(&mut self.physics_world.gravity.y)
                                .speed(0.1)
                                .range(-50.0..=50.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Time Step:");
                        ui.add(
                            egui::DragValue::new(&mut self.physics_world.time_step)
                                .speed(0.001)
                                .range(0.001..=0.1),
                        );
                    });

                    ui.separator();
                    ui.label("Camera Settings");

                    ui.horizontal(|ui| {
                        ui.label("Zoom:");
                        ui.add(
                            egui::DragValue::new(&mut self.camera_zoom)
                                .speed(0.1)
                                .range(0.1..=5.0),
                        );
                    });

                    ui.separator();

                    if ui.button("Reset to Defaults").clicked() {
                        self.physics_world = PhysicsWorld::default();
                        self.camera_zoom = 1.0;
                        self.add_console_message("Reset preferences to defaults".to_string());
                    }

                    if ui.button("Close").clicked() {
                        self.show_preferences = false;
                    }
                });
        }

        // Request repaint for smooth animation when playing
        if self.is_playing {
            ctx.request_repaint();
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
