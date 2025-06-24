// Main Physics Simulation GUI Module
// Unity-style physics simulation interface with 3D rendering

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

    pub fn forward() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    pub fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn right() -> Self {
        Self::new(1.0, 0.0, 0.0)
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

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
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

/// 4x4 Matrix for 3D transformations
#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(pos: Vec3) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, pos.x],
                [0.0, 1.0, 0.0, pos.y],
                [0.0, 0.0, 1.0, pos.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(angle_deg: f32) -> Self {
        let angle = angle_deg.to_radians();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            m: [
                [cos_a, 0.0, sin_a, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin_a, 0.0, cos_a, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_x(angle_deg: f32) -> Self {
        let angle = angle_deg.to_radians();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos_a, -sin_a, 0.0],
                [0.0, sin_a, cos_a, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_z(angle_deg: f32) -> Self {
        let angle = angle_deg.to_radians();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            m: [
                [cos_a, -sin_a, 0.0, 0.0],
                [sin_a, cos_a, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(scale: Vec3) -> Self {
        Self {
            m: [
                [scale.x, 0.0, 0.0, 0.0],
                [0.0, scale.y, 0.0, 0.0],
                [0.0, 0.0, scale.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn perspective(fov_deg: f32, aspect: f32, near: f32, far: f32) -> Self {
        let fov_rad = fov_deg.to_radians();
        let f = 1.0 / (fov_rad / 2.0).tan();
        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [
                    0.0,
                    0.0,
                    (far + near) / (near - far),
                    (2.0 * far * near) / (near - far),
                ],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }

    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let f = (center - eye).normalize();
        let s = f.cross(&up).normalize();
        let u = s.cross(&f);

        Self {
            m: [
                [s.x, u.x, -f.x, 0.0],
                [s.y, u.y, -f.y, 0.0],
                [s.z, u.z, -f.z, 0.0],
                [-s.dot(&eye), -u.dot(&eye), f.dot(&eye), 1.0],
            ],
        }
    }

    pub fn multiply(&self, other: &Mat4) -> Mat4 {
        let mut result = Mat4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = 0.0;
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        result
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let x =
            self.m[0][0] * point.x + self.m[0][1] * point.y + self.m[0][2] * point.z + self.m[0][3];
        let y =
            self.m[1][0] * point.x + self.m[1][1] * point.y + self.m[1][2] * point.z + self.m[1][3];
        let z =
            self.m[2][0] * point.x + self.m[2][1] * point.y + self.m[2][2] * point.z + self.m[2][3];
        let w =
            self.m[3][0] * point.x + self.m[3][1] * point.y + self.m[3][2] * point.z + self.m[3][3];

        if w.abs() > 0.0001 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }
}

/// Camera component for 3D rendering
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_distance: f32,
    pub orbit_angle_y: f32,
    pub orbit_angle_x: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 3.0, 5.0),
            target: Vec3::zero(),
            up: Vec3::up(),
            fov: 60.0,
            near: 0.1,
            far: 100.0,
            orbit_distance: 8.0,
            orbit_angle_y: 45.0,
            orbit_angle_x: 20.0,
        }
    }
}

impl Camera {
    pub fn update_orbit_position(&mut self) {
        let y_rad = self.orbit_angle_y.to_radians();
        let x_rad = self.orbit_angle_x.to_radians();

        self.position = Vec3::new(
            self.target.x + self.orbit_distance * y_rad.cos() * x_rad.cos(),
            self.target.y + self.orbit_distance * x_rad.sin(),
            self.target.z + self.orbit_distance * y_rad.sin() * x_rad.cos(),
        );
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    pub fn get_projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective(self.fov, aspect, self.near, self.far)
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
    /// 3D Camera system
    camera: Camera,
    /// Scene view interaction
    scene_hovered: bool,
    mouse_pos_in_scene: egui::Pos2,
    /// Mouse interaction for camera
    last_mouse_pos: Option<egui::Pos2>,
    mouse_dragging: bool,
}

impl PhysicsEditorApp {
    pub fn new() -> Self {
        let mut app = Self {
            show_about: false,
            show_preferences: false,
            is_playing: false,
            is_paused: false,
            console_messages: vec!["3D Physics Editor Started".to_string()],
            selected_object: None,
            game_objects: HashMap::new(),
            next_id: 1,
            gizmo_mode: GizmoMode::Translate,
            gizmo_dragging: false,
            drag_start_pos: None,
            physics_world: PhysicsWorld::default(),
            camera: Camera::default(),
            scene_hovered: false,
            mouse_pos_in_scene: egui::Pos2::ZERO,
            last_mouse_pos: None,
            mouse_dragging: false,
        };

        // Create default scene objects
        app.create_default_scene();
        app
    }

    fn create_default_scene(&mut self) {
        // Note: We don't create a "Main Camera" GameObject because
        // the 3D camera is managed by self.camera

        // Directional Light
        let mut light = GameObject::new(
            self.next_id,
            "Directional Light".to_string(),
            GameObjectType::Light,
        );
        light.transform.position = Vec3::new(2.0, 4.0, 2.0);
        light.transform.rotation = Vec3::new(50.0, -30.0, 0.0);
        self.game_objects.insert(self.next_id, light);
        self.next_id += 1;

        // Default Cube
        let mut cube = GameObject::new(self.next_id, "Cube".to_string(), GameObjectType::Cube);
        cube.transform.position = Vec3::new(0.0, 0.5, 0.0);
        cube.rigid_body = Some(RigidBody {
            mass: 1.0,
            velocity: Vec3::zero(),
            angular_velocity: Vec3::zero(),
            use_gravity: true,
            is_kinematic: false,
        });
        self.game_objects.insert(self.next_id, cube);
        self.next_id += 1;

        // Ground Plane
        let mut plane = GameObject::new(self.next_id, "Ground".to_string(), GameObjectType::Plane);
        plane.transform.position = Vec3::new(0.0, 0.0, 0.0);
        plane.transform.scale = Vec3::new(10.0, 1.0, 10.0);
        if let Some(renderer) = &mut plane.mesh_renderer {
            renderer.material_color = [0.4, 0.6, 0.4, 1.0]; // Green ground
        }
        self.game_objects.insert(self.next_id, plane);
        self.next_id += 1;

        // Add a sphere for more variety
        let mut sphere =
            GameObject::new(self.next_id, "Sphere".to_string(), GameObjectType::Sphere);
        sphere.transform.position = Vec3::new(2.0, 1.0, 0.0);
        sphere.rigid_body = Some(RigidBody {
            mass: 0.5,
            velocity: Vec3::zero(),
            angular_velocity: Vec3::zero(),
            use_gravity: true,
            is_kinematic: false,
        });
        if let Some(renderer) = &mut sphere.mesh_renderer {
            renderer.material_color = [0.8, 0.2, 0.2, 1.0]; // Red sphere
        }
        self.game_objects.insert(self.next_id, sphere);
        self.next_id += 1;

        self.add_console_message(
            "Default scene created with Light, Cube, Sphere, and Ground".to_string(),
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

    /// Load and execute a Matrix-Lang example script
    fn load_matrix_example(&mut self, example_path: &str) {
        // Check if file exists
        let full_path = std::path::Path::new("../").join(example_path);

        match std::fs::read_to_string(&full_path) {
            Ok(script_content) => {
                self.add_console_message(format!(
                    "📄 Loading Matrix-Lang example: {}",
                    example_path
                ));
                self.add_console_message(
                    "🔧 Executing Matrix-Lang physics simulation...".to_string(),
                );

                // Execute the Matrix-Lang script
                match self.execute_matrix_script(&script_content) {
                    Ok(message) => {
                        self.add_console_message(format!("✅ {}", message));
                        self.add_console_message(
                            "🎮 Simulation ready! Press Play to run physics.".to_string(),
                        );
                    }
                    Err(error) => {
                        self.add_console_message(format!("❌ Script execution failed: {}", error));
                    }
                }
            }
            Err(e) => {
                self.add_console_message(format!(
                    "❌ Failed to load example {}: {}",
                    example_path, e
                ));
                self.add_console_message(
                    "💡 Tip: Make sure you're running from the correct directory".to_string(),
                );
            }
        }
    }

    /// Execute Matrix-Lang script and apply physics setup to the engine
    fn execute_matrix_script(&mut self, script_content: &str) -> Result<String, String> {
        // For now, we'll parse the script content and extract physics setup
        // This is a simplified implementation that recognizes common Matrix-Lang patterns

        let mut objects_created = 0;
        let mut gravity_set = false;

        // Simple parser to recognize Matrix-Lang physics commands
        for line in script_content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.starts_with('#') || line.starts_with("//") || line.is_empty() {
                continue;
            }

            // Detect physics world creation
            if line.contains("create_physics_world") {
                self.add_console_message("🌍 Physics world created".to_string());
            }

            // Detect gravity setting
            if line.contains("set_gravity") && line.contains("-9.81") {
                self.physics_world.gravity.y = -9.81;
                gravity_set = true;
                self.add_console_message("🔽 Gravity set to Earth gravity (-9.81)".to_string());
            }

            // Detect rigid body creation
            if line.contains("add_rigid_body") {
                objects_created += 1;

                // Parse object type and properties from the script
                if line.contains("\"cube\"") || line.contains("cube") {
                    self.create_object(
                        GameObjectType::Cube,
                        format!("ScriptCube_{}", objects_created),
                    );
                } else if line.contains("\"sphere\"") || line.contains("sphere") {
                    self.create_object(
                        GameObjectType::Sphere,
                        format!("ScriptSphere_{}", objects_created),
                    );
                } else if line.contains("\"box\"") || line.contains("box") {
                    // Ground plane typically
                    let obj_name = if line.contains("mass: 0.0") || line.contains("mass = 0.0") {
                        format!("Ground_{}", objects_created)
                    } else {
                        format!("Box_{}", objects_created)
                    };
                    self.create_object(GameObjectType::Cube, obj_name);
                }

                self.add_console_message(format!("📦 Physics object {} created", objects_created));
            }
        }

        // Generate summary message
        let mut summary = format!(
            "Matrix-Lang script executed: {} objects created",
            objects_created
        );
        if gravity_set {
            summary.push_str(", gravity configured");
        }

        Ok(summary)
    }

    /// Open the examples folder in the system file manager
    fn open_examples_folder(&mut self) {
        let examples_path = std::path::Path::new("../examples");

        #[cfg(target_os = "windows")]
        {
            if let Err(e) = std::process::Command::new("explorer")
                .arg(examples_path)
                .spawn()
            {
                self.add_console_message(format!("Failed to open examples folder: {}", e));
            } else {
                self.add_console_message("📁 Opened examples folder in Explorer".to_string());
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = std::process::Command::new("open")
                .arg(examples_path)
                .spawn()
            {
                self.add_console_message(format!("Failed to open examples folder: {}", e));
            } else {
                self.add_console_message("📁 Opened examples folder in Finder".to_string());
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(examples_path)
                .spawn()
            {
                self.add_console_message(format!("Failed to open examples folder: {}", e));
            } else {
                self.add_console_message("📁 Opened examples folder in file manager".to_string());
            }
        }

        // Fallback message for unsupported systems
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            self.add_console_message("📁 Examples folder: ../examples/".to_string());
            self.add_console_message("Available examples: physics_basic.matrix, physics_multi_object.matrix, physics_tower.matrix, physics_pendulum.matrix".to_string());
        }
    }

    /// Show the main menu bar with File, Edit, Window, Help menus
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

                    ui.separator();

                    ui.menu_button("Matrix-Lang Examples", |ui| {
                        if ui.button("Basic Physics").clicked() {
                            self.load_matrix_example("physics_basic.matrix");
                            ui.close_menu();
                        }
                        if ui.button("Multi Object").clicked() {
                            self.load_matrix_example("physics_multi_object.matrix");
                            ui.close_menu();
                        }
                        if ui.button("Tower Destruction").clicked() {
                            self.load_matrix_example("physics_tower.matrix");
                            ui.close_menu();
                        }
                        if ui.button("Pendulum Simulation").clicked() {
                            self.load_matrix_example("physics_pendulum.matrix");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Open Examples Folder").clicked() {
                            self.open_examples_folder();
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Preferences").clicked() {
                        self.show_preferences = true;
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

                ui.menu_button("Window", |ui| {
                    ui.label("3D View always visible");
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

    /// Show the toolbar with play/pause controls and gizmo tools
    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Play/Pause controls
                if ui.button(if self.is_playing { "⏸" } else { "▶" }).clicked() {
                    self.is_playing = !self.is_playing;
                    if self.is_playing {
                        self.add_console_message("Physics simulation started".to_string());
                    } else {
                        self.add_console_message("Physics simulation paused".to_string());
                    }
                }

                if ui.button("⏹").clicked() {
                    self.is_playing = false;
                    self.physics_world = PhysicsWorld::default();
                    // Reset all objects to their original positions
                    for (_, obj) in self.game_objects.iter_mut() {
                        if let Some(rigid_body) = &mut obj.rigid_body {
                            rigid_body.velocity = Vec3::zero();
                            rigid_body.angular_velocity = Vec3::zero();
                        }
                    }
                    self.add_console_message("Physics simulation stopped and reset".to_string());
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

                ui.separator();

                // Physics settings
                ui.label("Physics:");
                ui.checkbox(&mut self.physics_world.is_paused, "Pause");

                ui.label("Gravity:");
                ui.add(
                    egui::DragValue::new(&mut self.physics_world.gravity.y)
                        .speed(0.1)
                        .range(-50.0..=50.0),
                );
            });
        });
    }

    /// Show the status bar at the bottom with simulation info
    fn show_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Objects: {}", self.game_objects.len()));
                ui.separator();
                ui.label(format!(
                    "Selected: {}",
                    self.selected_object.map_or("None".to_string(), |id| self
                        .game_objects
                        .get(&id)
                        .map_or("Unknown".to_string(), |obj| obj.name.clone()))
                ));
                ui.separator();
                ui.label(format!(
                    "Status: {}",
                    if self.is_playing {
                        "Playing"
                    } else {
                        "Stopped"
                    }
                ));
                ui.separator();
                ui.label(format!(
                    "FPS: {:.1}",
                    ctx.input(|i| 1.0 / i.unstable_dt.max(0.001))
                ));

                // Right-aligned status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Unity-Style Physics Engine v0.1.0");
                });
            });
        });
    }

    /// Show the main Unity-style layout with hierarchy, scene view, inspector, and console
    fn show_main_layout(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Split into left (hierarchy + inspector) and right (scene + console)
            ui.horizontal(|ui| {
                // Left panel - Hierarchy and Inspector
                ui.vertical(|ui| {
                    ui.set_width(300.0);

                    // Hierarchy Panel
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Hierarchy");
                            ui.separator();

                            egui::ScrollArea::vertical()
                                .max_height(200.0)
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
                                        self.add_console_message(format!(
                                            "Deleted object with ID {}",
                                            id
                                        ));
                                    }
                                });
                        });
                    });

                    ui.add_space(10.0);

                    // Inspector Panel
                    ui.group(|ui| {
                        ui.vertical(|ui| {
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
                    });
                });

                ui.separator();

                // Right panel - Scene View and Console
                ui.vertical(|ui| {
                    // Scene View
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading("Scene View");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(format!(
                                            "Camera: Orbit Distance {:.1}",
                                            self.camera.orbit_distance
                                        ));
                                    },
                                );
                            });
                            ui.separator();

                            // 3D Scene rendering area
                            let scene_response = ui.allocate_response(
                                egui::Vec2::new(ui.available_width(), 400.0),
                                egui::Sense::click_and_drag(),
                            );

                            self.scene_hovered = scene_response.hovered();

                            // Handle camera controls
                            if scene_response.dragged_by(egui::PointerButton::Primary) {
                                let delta = scene_response.drag_delta();
                                self.camera.orbit_angle_x += delta.y * 0.01;
                                self.camera.orbit_angle_y += delta.x * 0.01;

                                // Clamp vertical rotation
                                self.camera.orbit_angle_x =
                                    self.camera.orbit_angle_x.clamp(-1.5, 1.5);
                            }

                            // Mouse wheel zoom
                            if scene_response.hovered() {
                                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                                if scroll != 0.0 {
                                    self.camera.orbit_distance -= scroll * 0.01;
                                    self.camera.orbit_distance =
                                        self.camera.orbit_distance.clamp(2.0, 50.0);
                                }
                            }

                            // Draw 3D scene
                            let painter = ui.painter_at(scene_response.rect);
                            self.draw_3d_scene(&painter, scene_response.rect);
                        });
                    });

                    ui.add_space(10.0);

                    // Console Panel
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Console");
                            ui.separator();

                            egui::ScrollArea::vertical()
                                .max_height(150.0)
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
                    });
                });
            });
        });
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

                // Convert 3D world position to 2D screen position
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
                        GameObjectType::Cylinder => egui::Color32::from_rgb(255, 165, 0), // Orange
                        GameObjectType::Plane => egui::Color32::LIGHT_GREEN,
                        GameObjectType::Camera => egui::Color32::GRAY,
                        GameObjectType::Light => egui::Color32::WHITE,
                        GameObjectType::Empty => egui::Color32::from_gray(150),
                    }
                };

                match obj.object_type {
                    GameObjectType::Cube => {
                        self.draw_3d_cube(
                            painter,
                            rect,
                            obj.transform.position,
                            obj.transform.scale,
                            color,
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
                        // Draw cylinder as an ellipse for now
                        painter.circle_filled(pos, 25.0 * obj.transform.scale.x, color);
                        painter.circle_stroke(
                            pos,
                            25.0 * obj.transform.scale.x,
                            egui::Stroke::new(2.0, egui::Color32::BLACK),
                        );
                    }
                    GameObjectType::Plane => {
                        self.draw_3d_plane(
                            painter,
                            rect,
                            obj.transform.position,
                            obj.transform.scale,
                            color,
                        );
                    }
                    GameObjectType::Camera => {
                        // Draw camera as a pyramid/triangle
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
                        // Draw empty as a small square
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

    /// Draw a 3D cube with proper perspective
    fn draw_3d_cube(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        position: Vec3,
        scale: Vec3,
        color: egui::Color32,
    ) {
        let half_size = Vec3::new(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5);

        // Define cube vertices
        let vertices = [
            position + Vec3::new(-half_size.x, -half_size.y, -half_size.z), // 0
            position + Vec3::new(half_size.x, -half_size.y, -half_size.z),  // 1
            position + Vec3::new(half_size.x, half_size.y, -half_size.z),   // 2
            position + Vec3::new(-half_size.x, half_size.y, -half_size.z),  // 3
            position + Vec3::new(-half_size.x, -half_size.y, half_size.z),  // 4
            position + Vec3::new(half_size.x, -half_size.y, half_size.z),   // 5
            position + Vec3::new(half_size.x, half_size.y, half_size.z),    // 6
            position + Vec3::new(-half_size.x, half_size.y, half_size.z),   // 7
        ];

        // Convert to screen coordinates
        let screen_vertices: Vec<Option<egui::Pos2>> = vertices
            .iter()
            .map(|&v| self.world_to_screen(v, rect))
            .collect();

        // Draw visible faces
        let faces = [
            [0, 1, 2, 3], // front
            [5, 4, 7, 6], // back
            [4, 0, 3, 7], // left
            [1, 5, 6, 2], // right
            [3, 2, 6, 7], // top
            [4, 5, 1, 0], // bottom
        ];

        for face in &faces {
            let face_vertices: Vec<egui::Pos2> =
                face.iter().filter_map(|&i| screen_vertices[i]).collect();

            if face_vertices.len() == 4 {
                painter.add(egui::Shape::convex_polygon(
                    face_vertices,
                    color.gamma_multiply(0.3),
                    egui::Stroke::new(1.0, egui::Color32::BLACK),
                ));
            }
        }

        // Draw edges
        let edges = [
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 0], // front face
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 4], // back face
            [0, 4],
            [1, 5],
            [2, 6],
            [3, 7], // connecting edges
        ];

        for edge in &edges {
            if let (Some(start), Some(end)) = (screen_vertices[edge[0]], screen_vertices[edge[1]]) {
                painter.line_segment([start, end], egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
    }

    /// Draw a 3D plane (ground)
    fn draw_3d_plane(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        position: Vec3,
        scale: Vec3,
        color: egui::Color32,
    ) {
        let half_size_x = scale.x * 5.0;
        let half_size_z = scale.z * 5.0;

        // Define plane corners
        let corners = [
            position + Vec3::new(-half_size_x, 0.0, -half_size_z),
            position + Vec3::new(half_size_x, 0.0, -half_size_z),
            position + Vec3::new(half_size_x, 0.0, half_size_z),
            position + Vec3::new(-half_size_x, 0.0, half_size_z),
        ];

        // Convert to screen coordinates
        let screen_corners: Vec<egui::Pos2> = corners
            .iter()
            .filter_map(|&corner| self.world_to_screen(corner, rect))
            .collect();

        if screen_corners.len() == 4 {
            // Draw filled plane
            painter.add(egui::Shape::convex_polygon(
                screen_corners.clone(),
                color.gamma_multiply(0.3),
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            ));

            // Draw grid lines on the plane
            let grid_count = 10;
            for i in 0..=grid_count {
                let t = i as f32 / grid_count as f32;

                // Horizontal lines
                let start =
                    position + Vec3::new(-half_size_x + 2.0 * half_size_x * t, 0.0, -half_size_z);
                let end =
                    position + Vec3::new(-half_size_x + 2.0 * half_size_x * t, 0.0, half_size_z);

                if let (Some(start_screen), Some(end_screen)) = (
                    self.world_to_screen(start, rect),
                    self.world_to_screen(end, rect),
                ) {
                    painter.line_segment(
                        [start_screen, end_screen],
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                    );
                }

                // Vertical lines
                let start =
                    position + Vec3::new(-half_size_x, 0.0, -half_size_z + 2.0 * half_size_z * t);
                let end =
                    position + Vec3::new(half_size_x, 0.0, -half_size_z + 2.0 * half_size_z * t);

                if let (Some(start_screen), Some(end_screen)) = (
                    self.world_to_screen(start, rect),
                    self.world_to_screen(end, rect),
                ) {
                    painter.line_segment(
                        [start_screen, end_screen],
                        egui::Stroke::new(1.0, egui::Color32::GRAY),
                    );
                }
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

        // Simple projection (not a proper camera matrix, but good enough for our needs)
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

impl Default for PhysicsEditorApp {
    fn default() -> Self {
        Self::new()
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
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Physics Settings");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Gravity:");
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

                    ui.separator();

                    if ui.button("Reset to Defaults").clicked() {
                        self.physics_world = PhysicsWorld::default();
                        self.camera.orbit_distance = 10.0;
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
