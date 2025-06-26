// Main Physics Simulation GUI Module
// Unity-style physics simulation interface with 3D rendering and dockable panels

#![allow(dead_code)] // Allow dead code for development - these will be used in future features

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use std::collections::HashMap;

/// Temporary simple scripting panel for Matrix Language integration
#[derive(Debug, Clone)]
pub struct SimpleScriptingPanel {
    script_content: String,
    show_help: bool,
}

impl SimpleScriptingPanel {
    pub fn new() -> Self {
        Self {
            script_content: "-- Matrix Language Script\n-- Use @sim { ... } for 3D physics simulation\n-- Use @plot { ... } for plotting\n\nlet x = 5\nlet y = 10\nlet result = x + y\nprintln(\"Result: \", result)".to_string(),
            show_help: false,
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Matrix Script");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("▶ Run").clicked() {
                ui.ctx().debug_painter().debug_text(
                    egui::Pos2::new(10.0, 100.0),
                    egui::Align2::LEFT_TOP,
                    egui::Color32::GREEN,
                    "Script executed! (Integration with Matrix Language)",
                );
            }

            if ui.button("📄 New").clicked() {
                self.script_content.clear();
            }

            if ui.button("💾 Save").clicked() {
                // Placeholder for save functionality
            }

            if ui.button("❓ Help").clicked() {
                self.show_help = !self.show_help;
            }
        });

        ui.separator();

        // Script editor
        egui::ScrollArea::vertical()
            .id_salt("matrix_script_editor")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.script_content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20),
                );
            });

        if self.show_help {
            ui.separator();
            ui.collapsing("Matrix Language Help", |ui| {
                ui.label("Matrix Language Features:");
                ui.label("• @sim { ... } - Launch 3D physics simulation");
                ui.label("• @plot { ... } - Launch plotting interface");
                ui.label("• Variables: let x = 5");
                ui.label("• Functions: let f = (x) -> x * 2");
                ui.label("• Arrays: [1, 2, 3, 4]");
                ui.label("• Matrices: [[1, 2], [3, 4]]");
            });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Lines: {}", self.script_content.lines().count()));
            ui.separator();
            ui.label(format!("Characters: {}", self.script_content.len()));
        });
    }
}

/// Dock tab types for the Unity-style interface
#[derive(Debug, Clone, PartialEq)]
pub enum DockTab {
    Hierarchy,
    Inspector,
    Console,
    SceneView,
    MatrixScript,
}

impl std::fmt::Display for DockTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockTab::Hierarchy => write!(f, "Hierarchy"),
            DockTab::Inspector => write!(f, "Inspector"),
            DockTab::Console => write!(f, "Console"),
            DockTab::SceneView => write!(f, "Scene"),
            DockTab::MatrixScript => write!(f, "Matrix Script"),
        }
    }
}

/// Tab viewer implementation for dock system
pub struct EditorTabViewer<'a> {
    pub app: &'a mut PhysicsEditorApp,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{}", tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Ensure each tab content has its own unique scope
        ui.scope(|ui| match tab {
            DockTab::Hierarchy => self.app.show_hierarchy_content(ui),
            DockTab::Inspector => self.app.show_inspector_content(ui),
            DockTab::Console => self.app.show_console_content(ui),
            DockTab::SceneView => self.app.show_scene_view_content(ui),
            DockTab::MatrixScript => self.app.show_matrix_script_content(ui),
        });
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false // Keep all core tabs open - they are essential for the Unity-style workflow
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false // Prevent closing tabs to maintain Unity-style workflow
    }

    fn add_popup(
        &mut self,
        ui: &mut egui::Ui,
        _surface: egui_dock::SurfaceIndex,
        _node: egui_dock::NodeIndex,
    ) {
        ui.label("Add Panel:");
        ui.separator();

        // Allow re-adding panels if they get closed accidentally
        if ui.button("📁 Hierarchy").clicked() {
            ui.close_menu();
        }
        if ui.button("🔍 Inspector").clicked() {
            ui.close_menu();
        }
        if ui.button("📝 Console").clicked() {
            ui.close_menu();
        }
        if ui.button("🎬 Scene View").clicked() {
            ui.close_menu();
        }
    }

    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false // Never force close essential Unity panels
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Self::Tab,
        _surface: egui_dock::SurfaceIndex,
        _node: egui_dock::NodeIndex,
    ) {
        match tab {
            DockTab::SceneView => {
                if ui.button("Reset Camera").clicked() {
                    self.app.camera.orbit_distance = 8.0;
                    self.app.camera.orbit_angle_x = 20.0;
                    self.app.camera.orbit_angle_y = 45.0;
                    ui.close_menu();
                }
                if ui.button("Frame All Objects").clicked() {
                    // Frame all objects in the scene
                    if !self.app.game_objects.is_empty() {
                        self.app.camera.orbit_distance = 15.0;
                    }
                    ui.close_menu();
                }
            }
            DockTab::Console => {
                if ui.button("Clear Console").clicked() {
                    self.app.console_messages.clear();
                    ui.close_menu();
                }
            }
            DockTab::Hierarchy => {
                if ui.button("Create Empty Object").clicked() {
                    self.app
                        .create_object(GameObjectType::Empty, "Empty Object".to_string());
                    ui.close_menu();
                }
            }
            DockTab::Inspector => {
                if ui.button("Reset Transform").clicked() {
                    if let Some(selected_id) = self.app.selected_object {
                        if let Some(obj) = self.app.game_objects.get_mut(&selected_id) {
                            obj.transform = Transform::default();
                        }
                    }
                    ui.close_menu();
                }
            }
            DockTab::MatrixScript => {
                if ui.button("New Script").clicked() {
                    // Create new Matrix script
                    ui.close_menu();
                }
                if ui.button("Run All Scripts").clicked() {
                    // Execute all Matrix scripts
                    ui.close_menu();
                }
            }
        }
    }
}

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

    pub fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            Self::zero()
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// Camera component for 3D rendering
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub fov: f32,
    pub orbit_distance: f32,
    pub orbit_angle_y: f32,
    pub orbit_angle_x: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 3.0, 5.0),
            target: Vec3::zero(),
            fov: 60.0,
            orbit_distance: 8.0,
            orbit_angle_y: 45.0,
            orbit_angle_x: 20.0,
        }
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

/// Game object with components
#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: usize,
    pub name: String,
    pub object_type: GameObjectType,
    pub transform: Transform,
    pub rigid_body: Option<RigidBody>,
}

impl GameObject {
    pub fn new(id: usize, name: String, object_type: GameObjectType) -> Self {
        let mut obj = Self {
            id,
            name,
            object_type: object_type.clone(),
            transform: Transform::default(),
            rigid_body: None,
        };

        // Add default components based on type
        match object_type {
            GameObjectType::Cube | GameObjectType::Sphere | GameObjectType::Cylinder => {
                obj.rigid_body = Some(RigidBody::default());
            }
            _ => {}
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
    /// Unique instance ID for this app to prevent ID conflicts
    instance_id: u64,
    /// Menu bar state
    show_about: bool,
    show_preferences: bool,
    /// Toolbar state
    is_playing: bool,
    /// Console messages
    console_messages: Vec<String>,
    /// Selected object ID
    selected_object: Option<usize>,
    /// Scene objects
    game_objects: HashMap<usize, GameObject>,
    next_id: usize,
    /// Gizmo state
    gizmo_mode: GizmoMode,
    /// Physics world
    physics_world: PhysicsWorld,
    /// 3D Camera system
    camera: Camera,
    /// Dock system
    dock_state: DockState<DockTab>,
    /// Matrix Script panel for Matrix Language integration
    scripting_panel: SimpleScriptingPanel,
}

impl PhysicsEditorApp {
    pub fn new() -> Self {
        // Create the default dock layout
        let mut dock_state = DockState::new(vec![DockTab::SceneView]);

        // Create hierarchy and inspector on the left
        let [left_node, _] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.25,
            vec![DockTab::Hierarchy, DockTab::Inspector],
        );

        // Create console at the bottom of the left panel
        dock_state
            .main_surface_mut()
            .split_below(left_node, 0.7, vec![DockTab::Console]);

        // Add Matrix Script tab to the right side
        dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.75,
            vec![DockTab::MatrixScript],
        );

        let mut app = Self {
            instance_id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64, // Use timestamp as unique ID
            show_about: false,
            show_preferences: false,
            is_playing: false,
            console_messages: vec!["3D Physics Editor Started".to_string()],
            selected_object: None,
            game_objects: HashMap::new(),
            next_id: 1,
            gizmo_mode: GizmoMode::Translate,
            physics_world: PhysicsWorld::default(),
            camera: Camera::default(),
            dock_state,
            scripting_panel: SimpleScriptingPanel::new(),
        };

        // Create default scene objects
        app.create_default_scene();
        app
    }

    fn create_default_scene(&mut self) {
        // Directional Light
        let mut light = GameObject::new(
            self.next_id,
            "Directional Light".to_string(),
            GameObjectType::Light,
        );
        light.transform.position = Vec3::new(2.0, 4.0, 2.0);
        self.game_objects.insert(self.next_id, light);
        self.next_id += 1;

        // Default Cube
        let mut cube = GameObject::new(self.next_id, "Cube".to_string(), GameObjectType::Cube);
        cube.transform.position = Vec3::new(0.0, 0.5, 0.0);
        self.game_objects.insert(self.next_id, cube);
        self.next_id += 1;

        // Ground Plane
        let mut plane = GameObject::new(self.next_id, "Ground".to_string(), GameObjectType::Plane);
        plane.transform.position = Vec3::new(0.0, 0.0, 0.0);
        plane.transform.scale = Vec3::new(10.0, 1.0, 10.0);
        self.game_objects.insert(self.next_id, plane);
        self.next_id += 1;

        // Add a sphere
        let mut sphere =
            GameObject::new(self.next_id, "Sphere".to_string(), GameObjectType::Sphere);
        sphere.transform.position = Vec3::new(2.0, 1.0, 0.0);
        self.game_objects.insert(self.next_id, sphere);
        self.next_id += 1;

        self.add_console_message(
            "Default scene created with Light, Cube, Sphere, and Ground".to_string(),
        );
    }

    fn create_object(&mut self, object_type: GameObjectType, base_name: String) {
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

    fn add_console_message(&mut self, message: String) {
        self.console_messages.push(message);
        if self.console_messages.len() > 100 {
            self.console_messages.remove(0);
        }
    }

    /// Show hierarchy panel content
    fn show_hierarchy_content(&mut self, ui: &mut egui::Ui) {
        ui.push_id(format!("hierarchy_panel_{}", self.instance_id), |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt(format!("hierarchy_scroll_{}", self.instance_id))
                .show(ui, |ui| {
                    let mut to_select = None;
                    let mut to_delete = None;

                    for (&id, obj) in &self.game_objects {
                        ui.horizontal(|ui| {
                            let selected = self.selected_object == Some(id);
                            if ui.selectable_label(selected, &obj.name).clicked() {
                                to_select = Some(id);
                            }

                            if ui.small_button("🗑").clicked() {
                                to_delete = Some(id);
                            }
                        });
                    }

                    if let Some(id) = to_select {
                        self.selected_object = Some(id);
                    }

                    if let Some(id) = to_delete {
                        self.game_objects.remove(&id);
                        if self.selected_object == Some(id) {
                            self.selected_object = None;
                        }
                        self.add_console_message(format!("Deleted object with ID {}", id));
                    }
                });
        });
    }

    /// Show inspector panel content
    fn show_inspector_content(&mut self, ui: &mut egui::Ui) {
        ui.push_id(format!("inspector_panel_{}", self.instance_id), |ui| {
            ui.heading("Inspector");
            ui.separator();

            if let Some(selected_id) = self.selected_object {
                if let Some(obj) = self.game_objects.get_mut(&selected_id) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut obj.name);
                    });

                    ui.separator();
                    ui.label("Transform");

                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.position.x)
                                .speed(0.1)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.position.y)
                                .speed(0.1)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.position.z)
                                .speed(0.1)
                                .prefix("Z: "),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.rotation.x)
                                .speed(1.0)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.rotation.y)
                                .speed(1.0)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.rotation.z)
                                .speed(1.0)
                                .prefix("Z: "),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.scale.x)
                                .speed(0.01)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.scale.y)
                                .speed(0.01)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut obj.transform.scale.z)
                                .speed(0.01)
                                .prefix("Z: "),
                        );
                    });

                    if let Some(rigid_body) = &mut obj.rigid_body {
                        ui.separator();
                        ui.label("RigidBody");

                        ui.horizontal(|ui| {
                            ui.label("Mass:");
                            ui.add(
                                egui::DragValue::new(&mut rigid_body.mass)
                                    .speed(0.1)
                                    .range(0.1..=100.0),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Velocity:");
                            ui.add(
                                egui::DragValue::new(&mut rigid_body.velocity.x)
                                    .speed(0.1)
                                    .prefix("X: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rigid_body.velocity.y)
                                    .speed(0.1)
                                    .prefix("Y: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut rigid_body.velocity.z)
                                    .speed(0.1)
                                    .prefix("Z: "),
                            );
                        });

                        ui.checkbox(&mut rigid_body.use_gravity, "Use Gravity");
                        ui.checkbox(&mut rigid_body.is_kinematic, "Is Kinematic");
                    }
                } else {
                    ui.label("Selected object not found");
                }
            } else {
                ui.label("No object selected");
            }
        });
    }

    /// Show console panel content
    fn show_console_content(&mut self, ui: &mut egui::Ui) {
        ui.push_id(format!("console_panel_{}", self.instance_id), |ui| {
            ui.heading("Console");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt(format!("console_scroll_{}", self.instance_id))
                .show(ui, |ui| {
                    for message in &self.console_messages {
                        ui.label(message);
                    }
                });

            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.console_messages.clear();
                }

                ui.separator();
                ui.label(format!("Messages: {}", self.console_messages.len()));
            });
        });
    }

    /// Show scene view panel content
    fn show_scene_view_content(&mut self, ui: &mut egui::Ui) {
        ui.push_id(format!("scene_view_panel_{}", self.instance_id), |ui| {
            ui.horizontal(|ui| {
                ui.heading("Scene View");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "Camera: Orbit Distance {:.1}",
                        self.camera.orbit_distance
                    ));
                });
            });
            ui.separator();

            // 3D Scene rendering area
            let scene_response = ui.allocate_response(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Sense::click_and_drag(),
            );

            // Handle object selection by clicking
            if scene_response.clicked() {
                if let Some(click_pos) = scene_response.interact_pointer_pos() {
                    if let Some(clicked_object_id) =
                        self.pick_object_at_screen_pos(click_pos, scene_response.rect)
                    {
                        self.selected_object = Some(clicked_object_id);
                        if let Some(obj) = self.game_objects.get(&clicked_object_id) {
                            self.add_console_message(format!("Selected object: {}", obj.name));
                        }
                    } else {
                        self.selected_object = None;
                        self.add_console_message("Deselected object".to_string());
                    }
                }
            }

            // Handle camera controls (only when not clicking objects)
            if scene_response.dragged_by(egui::PointerButton::Primary) && !scene_response.clicked()
            {
                let delta = scene_response.drag_delta();
                self.camera.orbit_angle_x += delta.y * 0.01;
                self.camera.orbit_angle_y += delta.x * 0.01;

                // Clamp vertical rotation
                self.camera.orbit_angle_x = self.camera.orbit_angle_x.clamp(-1.5, 1.5);
            }

            // Mouse wheel zoom
            if scene_response.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    self.camera.orbit_distance -= scroll * 0.01;
                    self.camera.orbit_distance = self.camera.orbit_distance.clamp(2.0, 50.0);
                }
            }

            // Draw 3D scene
            let painter = ui.painter_at(scene_response.rect);
            self.draw_3d_scene(&painter, scene_response.rect);
        });
    }

    /// Show Matrix Script panel content
    fn show_matrix_script_content(&mut self, ui: &mut egui::Ui) {
        ui.push_id(format!("matrix_script_panel_{}", self.instance_id), |ui| {
            // Delegate to the scripting panel
            self.scripting_panel.show_ui(ui);
        });
    }

    /// Pick object at screen position for selection
    fn pick_object_at_screen_pos(
        &self,
        screen_pos: egui::Pos2,
        scene_rect: egui::Rect,
    ) -> Option<usize> {
        let mut closest_object = None;
        let mut closest_distance = f32::INFINITY;

        for (&id, obj) in &self.game_objects {
            if let Some(obj_screen_pos) = self.world_to_screen(obj.transform.position, scene_rect) {
                let distance = (obj_screen_pos - screen_pos).length();

                // Check if click is within object bounds (approximate)
                let object_size = match obj.object_type {
                    GameObjectType::Cube => {
                        40.0 * obj
                            .transform
                            .scale
                            .x
                            .max(obj.transform.scale.y)
                            .max(obj.transform.scale.z)
                    }
                    GameObjectType::Sphere => 20.0 * obj.transform.scale.x,
                    GameObjectType::Cylinder => 25.0 * obj.transform.scale.x,
                    GameObjectType::Plane => 30.0,
                    GameObjectType::Camera => 15.0,
                    GameObjectType::Light => 15.0,
                    GameObjectType::Empty => 8.0,
                };

                if distance < object_size && distance < closest_distance {
                    closest_distance = distance;
                    closest_object = Some(id);
                }
            }
        }

        closest_object
    }

    /// Draw the 3D scene with all GameObjects and gizmos
    fn draw_3d_scene(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        // Clear background
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));

        // Draw grid
        self.draw_3d_grid(painter, rect);

        // Draw all GameObjects
        for (&id, obj) in &self.game_objects {
            self.draw_3d_object(painter, rect, obj, id == self.selected_object.unwrap_or(0));
        }

        // Draw gizmos for selected object
        if let Some(selected_id) = self.selected_object {
            if let Some(obj) = self.game_objects.get(&selected_id) {
                self.draw_3d_gizmo(painter, rect, &obj.transform.position);
            }
        }
    }

    /// Draw a 3D grid on the ground plane
    fn draw_3d_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_size = 20;
        let grid_spacing = 1.0;

        for i in -grid_size..=grid_size {
            for j in -grid_size..=grid_size {
                let x = i as f32 * grid_spacing;
                let z = j as f32 * grid_spacing;

                let screen_pos = self.world_to_screen(Vec3::new(x, 0.0, z), rect);

                if let Some(pos) = screen_pos {
                    if rect.contains(pos) {
                        painter.circle_filled(pos, 1.0, egui::Color32::from_gray(100));
                    }
                }
            }
        }

        // Draw main axes
        if let (Some(origin), Some(x_axis), Some(z_axis)) = (
            self.world_to_screen(Vec3::zero(), rect),
            self.world_to_screen(Vec3::new(5.0, 0.0, 0.0), rect),
            self.world_to_screen(Vec3::new(0.0, 0.0, 5.0), rect),
        ) {
            painter.line_segment([origin, x_axis], egui::Stroke::new(2.0, egui::Color32::RED));
            painter.line_segment(
                [origin, z_axis],
                egui::Stroke::new(2.0, egui::Color32::BLUE),
            );
        }
    }

    /// Draw a 3D GameObject
    fn draw_3d_object(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        obj: &GameObject,
        is_selected: bool,
    ) {
        let screen_pos = self.world_to_screen(obj.transform.position, rect);

        if let Some(pos) = screen_pos {
            if rect.contains(pos) {
                let color = if is_selected {
                    egui::Color32::YELLOW
                } else {
                    match obj.object_type {
                        GameObjectType::Cube => egui::Color32::LIGHT_BLUE,
                        GameObjectType::Sphere => egui::Color32::LIGHT_RED,
                        GameObjectType::Cylinder => egui::Color32::from_rgb(255, 165, 0),
                        GameObjectType::Plane => egui::Color32::LIGHT_GREEN,
                        GameObjectType::Camera => egui::Color32::GRAY,
                        GameObjectType::Light => egui::Color32::WHITE,
                        GameObjectType::Empty => egui::Color32::from_gray(150),
                    }
                };

                match obj.object_type {
                    GameObjectType::Cube => {
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                pos,
                                egui::Vec2::splat(40.0 * obj.transform.scale.x),
                            ),
                            0.0,
                            color,
                        );
                        painter.rect_stroke(
                            egui::Rect::from_center_size(
                                pos,
                                egui::Vec2::splat(40.0 * obj.transform.scale.x),
                            ),
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                            egui::StrokeKind::Outside,
                        );
                    }
                    GameObjectType::Sphere => {
                        painter.circle_filled(pos, 20.0 * obj.transform.scale.x, color);
                        painter.circle_stroke(
                            pos,
                            20.0 * obj.transform.scale.x,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );
                    }
                    GameObjectType::Cylinder => {
                        painter.circle_filled(pos, 25.0 * obj.transform.scale.x, color);
                        painter.circle_stroke(
                            pos,
                            25.0 * obj.transform.scale.x,
                            egui::Stroke::new(2.0, egui::Color32::BLACK),
                        );
                    }
                    GameObjectType::Plane => {
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                pos,
                                egui::Vec2::new(
                                    100.0 * obj.transform.scale.x,
                                    100.0 * obj.transform.scale.z,
                                ),
                            ),
                            0.0,
                            color.gamma_multiply(0.3),
                        );
                        painter.rect_stroke(
                            egui::Rect::from_center_size(
                                pos,
                                egui::Vec2::new(
                                    100.0 * obj.transform.scale.x,
                                    100.0 * obj.transform.scale.z,
                                ),
                            ),
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                            egui::StrokeKind::Outside,
                        );
                    }
                    GameObjectType::Camera => {
                        let size = 15.0;
                        let points = vec![
                            pos + egui::Vec2::new(0.0, -size),
                            pos + egui::Vec2::new(-size, size),
                            pos + egui::Vec2::new(size, size),
                        ];
                        painter.add(egui::Shape::convex_polygon(
                            points,
                            color,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        ));
                    }
                    GameObjectType::Light => {
                        painter.circle_filled(pos, 10.0, color);
                        // Draw light rays
                        for i in 0..8 {
                            let angle = i as f32 * std::f32::consts::PI / 4.0;
                            let end_pos = pos + egui::Vec2::new(angle.cos(), angle.sin()) * 15.0;
                            painter.line_segment([pos, end_pos], egui::Stroke::new(1.0, color));
                        }
                    }
                    GameObjectType::Empty => {
                        painter.rect_filled(
                            egui::Rect::from_center_size(pos, egui::Vec2::splat(8.0)),
                            0.0,
                            color,
                        );
                    }
                }

                // Draw object name
                painter.text(
                    pos + egui::Vec2::new(0.0, -30.0),
                    egui::Align2::CENTER_CENTER,
                    &obj.name,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }
        }
    }

    /// Draw 3D gizmo for the selected object
    fn draw_3d_gizmo(&self, painter: &egui::Painter, rect: egui::Rect, position: &Vec3) {
        let center = self.world_to_screen(*position, rect);

        if let Some(center_pos) = center {
            if rect.contains(center_pos) {
                let size = 30.0;

                match self.gizmo_mode {
                    GizmoMode::Translate => {
                        // Draw translate gizmo (arrows)
                        let x_end = center_pos + egui::Vec2::new(size, 0.0);
                        let y_end = center_pos + egui::Vec2::new(0.0, -size);

                        painter.line_segment(
                            [center_pos, x_end],
                            egui::Stroke::new(3.0, egui::Color32::RED),
                        );
                        painter.line_segment(
                            [center_pos, y_end],
                            egui::Stroke::new(3.0, egui::Color32::GREEN),
                        );

                        // Arrow heads
                        painter.circle_filled(x_end, 5.0, egui::Color32::RED);
                        painter.circle_filled(y_end, 5.0, egui::Color32::GREEN);
                    }
                    GizmoMode::Rotate => {
                        // Draw rotate gizmo (circles)
                        painter.circle_stroke(
                            center_pos,
                            size,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        );
                        painter.circle_stroke(
                            center_pos,
                            size * 0.7,
                            egui::Stroke::new(2.0, egui::Color32::CYAN),
                        );
                    }
                    GizmoMode::Scale => {
                        // Draw scale gizmo (squares)
                        let square_size = 8.0;
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                center_pos + egui::Vec2::new(size, 0.0),
                                egui::Vec2::splat(square_size),
                            ),
                            0.0,
                            egui::Color32::RED,
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                center_pos + egui::Vec2::new(0.0, -size),
                                egui::Vec2::splat(square_size),
                            ),
                            0.0,
                            egui::Color32::GREEN,
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                center_pos,
                                egui::Vec2::splat(square_size),
                            ),
                            0.0,
                            egui::Color32::BLUE,
                        );
                    }
                }
            }
        }
    }

    /// Convert 3D world position to 2D screen position
    fn world_to_screen(&self, world_pos: Vec3, rect: egui::Rect) -> Option<egui::Pos2> {
        // Simple perspective projection
        let camera_pos = Vec3::new(
            self.camera.orbit_angle_y.sin() * self.camera.orbit_distance,
            self.camera.orbit_angle_x.sin() * self.camera.orbit_distance,
            self.camera.orbit_angle_y.cos() * self.camera.orbit_distance,
        );

        let relative_pos = world_pos - camera_pos;

        // Simple projection
        let z_distance = relative_pos.z.max(0.1);
        let fov_scale = 1.0 / (self.camera.fov * 0.01);

        let screen_x = (relative_pos.x / z_distance) * fov_scale * 100.0;
        let screen_y = (relative_pos.y / z_distance) * fov_scale * 100.0;

        let center = rect.center();
        let screen_pos = egui::Pos2::new(
            center.x + screen_x,
            center.y - screen_y, // Flip Y for screen coordinates
        );

        Some(screen_pos)
    }

    /// Show the main menu bar
    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        self.game_objects.clear();
                        self.selected_object = None;
                        self.create_default_scene();
                        self.add_console_message("New scene created".to_string());
                        ui.close_menu();
                    }
                });

                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Cube").clicked() {
                        self.create_object(GameObjectType::Cube, "Cube".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Sphere").clicked() {
                        self.create_object(GameObjectType::Sphere, "Sphere".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Cylinder").clicked() {
                        self.create_object(GameObjectType::Cylinder, "Cylinder".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Plane").clicked() {
                        self.create_object(GameObjectType::Plane, "Plane".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Light").clicked() {
                        self.create_object(GameObjectType::Light, "Light".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Camera").clicked() {
                        self.create_object(GameObjectType::Camera, "Camera".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Create Empty").clicked() {
                        self.create_object(GameObjectType::Empty, "Empty".to_string());
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

    /// Show the toolbar
    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Play/Pause controls
                if ui.button(if self.is_playing { "⏸" } else { "▶" }).clicked() {
                    self.is_playing = !self.is_playing;
                    self.add_console_message(if self.is_playing {
                        "Physics simulation started".to_string()
                    } else {
                        "Physics simulation paused".to_string()
                    });
                }

                ui.separator();

                // Gizmo mode selection
                ui.label("Gizmos:");

                if ui
                    .selectable_label(matches!(self.gizmo_mode, GizmoMode::Translate), "📐 Move")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Translate;
                }

                if ui
                    .selectable_label(matches!(self.gizmo_mode, GizmoMode::Rotate), "🔄 Rotate")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Rotate;
                }

                if ui
                    .selectable_label(matches!(self.gizmo_mode, GizmoMode::Scale), "📏 Scale")
                    .clicked()
                {
                    self.gizmo_mode = GizmoMode::Scale;
                }
            });
        });
    }

    /// Show the main layout with dockable panels
    fn show_main_layout(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a clone of the dock state to avoid borrowing issues
            let mut dock_state_clone = self.dock_state.clone();

            let mut tab_viewer = EditorTabViewer { app: self };
            DockArea::new(&mut dock_state_clone)
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut tab_viewer);

            // Update our dock state with the modified clone
            self.dock_state = dock_state_clone;
        });
    }
}

impl Default for PhysicsEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for PhysicsEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        self.show_menu_bar(ctx);

        // Toolbar
        self.show_toolbar(ctx);

        // Main dockable layout
        self.show_main_layout(ctx);

        // Modal dialogs
        if self.show_about {
            egui::Window::new(format!("About##{}", self.instance_id))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Unity-Style Physics Engine v0.1.0");
                    ui.label("Built with egui and eframe");
                    ui.separator();
                    ui.label("Features:");
                    ui.label("• Dockable interface panels");
                    ui.label("• 3D object selection by clicking");
                    ui.label("• Interactive Unity-style gizmos");
                    ui.label("• Real-time physics simulation");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }

        if self.show_preferences {
            egui::Window::new(format!("Preferences##{}", self.instance_id))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Camera Settings");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Orbit Distance:");
                        ui.add(
                            egui::DragValue::new(&mut self.camera.orbit_distance)
                                .speed(0.1)
                                .range(1.0..=50.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Field of View:");
                        ui.add(
                            egui::DragValue::new(&mut self.camera.fov)
                                .speed(1.0)
                                .range(10.0..=120.0),
                        );
                    });

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
