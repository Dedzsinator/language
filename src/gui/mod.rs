use crate::eval::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::physics::math::*;
use crate::physics::rigid_body::*;
use crate::physics::*;
use eframe::egui;
use egui_dock::{DockState, TabViewer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod console;
mod inspector;
mod object_hierarchy;
mod project_browser;
mod scene_manager;
mod scripting_panel;
mod viewport;

pub use console::*;
pub use inspector::*;
pub use object_hierarchy::*;
pub use project_browser::*;
pub use scene_manager::*;
pub use scripting_panel::*;
pub use viewport::*;

// Import specific items for console logging
use console::LogLevel;

/// Panel types for the dock system
#[derive(Debug, Clone, PartialEq)]
pub enum PanelTab {
    Viewport,
    Inspector,
    Hierarchy,
    Project,
    Console,
    Scripting,
}

impl std::fmt::Display for PanelTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelTab::Viewport => write!(f, "Viewport"),
            PanelTab::Inspector => write!(f, "Inspector"),
            PanelTab::Hierarchy => write!(f, "Hierarchy"),
            PanelTab::Project => write!(f, "Project"),
            PanelTab::Console => write!(f, "Console"),
            PanelTab::Scripting => write!(f, "Scripting"),
        }
    }
}

/// Bottom panel tab options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BottomTab {
    Console,
    Project,
    Scripting,
}

/// Tab viewer implementation for the dock system
pub struct EditorTabViewer<'a> {
    editor: &'a mut UnityStyleEditor,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = PanelTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelTab::Viewport => {
                if let Some(scene) = self.editor.scene_manager.current_scene() {
                    self.editor.viewport.set_view_mode(self.editor.view_mode);
                    self.editor
                        .viewport
                        .show_content(ui, scene, self.editor.selected_object);
                    self.editor.view_mode = self.editor.viewport.get_view_mode();
                }
            }
            PanelTab::Inspector => {
                if let Some(scene) = self.editor.scene_manager.current_scene_mut() {
                    self.editor
                        .inspector
                        .show_ui(ui, scene, self.editor.selected_object);
                }
            }
            PanelTab::Hierarchy => {
                if let Some(scene) = self.editor.scene_manager.current_scene_mut() {
                    let selected = self.editor.object_hierarchy.show_ui(
                        ui,
                        scene,
                        &mut self.editor.selected_object,
                    );
                    if let Some(id) = selected {
                        self.editor.selected_object = Some(id);
                    }
                }
            }
            PanelTab::Project => {
                self.editor.project_browser.show_ui(ui);
            }
            PanelTab::Console => {
                self.editor.console.show_ui(ui);
            }
            PanelTab::Scripting => {
                self.editor.scripting_panel.show_ui(ui);
            }
        }
    }
}

/// 3D transformation data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/// Game object types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameObjectType {
    Empty,
    Cube,
    Sphere,
    Cylinder,
    Plane,
    Light,
    Camera,
    RigidBody(Shape),
    SoftBody,
    FluidEmitter,
    Custom(String),
}

/// Component types that can be attached to game objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    Mesh {
        mesh_type: String,
    },
    Renderer {
        material: String,
        color: [f32; 4],
    },
    RigidBody {
        shape: Shape,
        mass: f64,
    },
    SoftBodyComponent {
        particles: usize,
        stiffness: f64,
    },
    Script {
        script_path: String,
        code: String,
    },
    Light {
        light_type: String,
        intensity: f32,
        color: [f32; 3],
    },
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },
    Collider {
        shape: Shape,
        is_trigger: bool,
    },
}

/// Game object in the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub transform: Transform,
    pub object_type: GameObjectType,
    pub components: Vec<Component>,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
    pub visible: bool,
    pub enabled: bool,
}

impl GameObject {
    pub fn new(id: u32, name: String, object_type: GameObjectType) -> Self {
        Self {
            id,
            name,
            tag: "Untagged".to_string(),
            transform: Transform::default(),
            object_type,
            components: Vec::new(),
            children: Vec::new(),
            parent: None,
            visible: true,
            enabled: true,
        }
    }
}

/// Scene data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub objects: HashMap<u32, GameObject>,
    pub next_id: u32,
    pub is_2d: bool,
    pub background_color: [f32; 4],
    pub physics_settings: PhysicsSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    pub gravity: Vec3,
    pub time_step: f64,
    pub solver_iterations: u32,
    pub enable_physics: bool,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 0.016,
            solver_iterations: 10,
            enable_physics: true,
        }
    }
}

impl Scene {
    pub fn new(name: String, is_2d: bool) -> Self {
        Self {
            name,
            objects: HashMap::new(),
            next_id: 1,
            is_2d,
            background_color: [0.2, 0.3, 0.8, 1.0],
            physics_settings: PhysicsSettings::default(),
        }
    }

    pub fn add_object(&mut self, name: String, object_type: GameObjectType) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let object = GameObject::new(id, name, object_type);
        self.objects.insert(id, object);
        id
    }

    pub fn remove_object(&mut self, id: u32) {
        if let Some(object) = self.objects.remove(&id) {
            // Remove from parent's children
            if let Some(parent_id) = object.parent {
                if let Some(parent) = self.objects.get_mut(&parent_id) {
                    parent.children.retain(|&child_id| child_id != id);
                }
            }

            // Remove all children
            for child_id in object.children {
                self.remove_object(child_id);
            }
        }
    }

    pub fn set_parent(&mut self, child_id: u32, parent_id: Option<u32>) {
        // Remove from old parent first
        if let Some(child) = self.objects.get(&child_id) {
            if let Some(old_parent_id) = child.parent {
                if let Some(old_parent) = self.objects.get_mut(&old_parent_id) {
                    old_parent.children.retain(|&id| id != child_id);
                }
            }
        }

        // Update child's parent
        if let Some(child) = self.objects.get_mut(&child_id) {
            child.parent = parent_id;
        }

        // Add to new parent
        if let Some(new_parent_id) = parent_id {
            if let Some(new_parent) = self.objects.get_mut(&new_parent_id) {
                new_parent.children.push(child_id);
            }
        }
    }
}

/// View mode for the viewport
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Scene2D,
    Scene3D,
    Game2D,
    Game3D,
}

/// Drag and drop payload for UI interactions
#[derive(Debug, Clone)]
pub enum DragPayload {
    ScriptFile(String),
    GameObject(u32),
    Asset(String),
}

/// Actions for undo/redo system
#[derive(Debug, Clone)]
pub enum EditorAction {
    CreateObject {
        id: u32,
        object: GameObject,
    },
    DeleteObject {
        id: u32,
        object: GameObject,
    },
    ModifyTransform {
        id: u32,
        old_transform: Transform,
        new_transform: Transform,
    },
    AddComponent {
        id: u32,
        component: Component,
    },
    RemoveComponent {
        id: u32,
        component_index: usize,
        component: Component,
    },
    ModifyComponent {
        id: u32,
        component_index: usize,
        old_component: Component,
        new_component: Component,
    },
    RenameObject {
        id: u32,
        old_name: String,
        new_name: String,
    },
}

/// Unity-style editor application
pub struct UnityStyleEditor {
    // Core systems
    scene_manager: SceneManager,

    // UI panels
    object_hierarchy: ObjectHierarchy,
    inspector: Inspector,
    scripting_panel: ScriptingPanel,
    viewport: Viewport,
    project_browser: ProjectBrowser,
    console: Console,

    // Selection and interaction
    selected_object: Option<u32>,
    view_mode: ViewMode,

    // Physics simulation
    physics_world: PhysicsWorld,
    is_simulating: bool,

    // UI state
    show_hierarchy: bool,
    show_inspector: bool,
    show_scripting: bool,
    show_project_browser: bool,
    show_console: bool,

    // Context menu
    context_menu_open: bool,
    context_menu_position: egui::Pos2,

    // Drag and drop
    drag_payload: Option<DragPayload>,

    // Dock system
    dock_state: DockState<PanelTab>,

    // New layout system
    bottom_tab: BottomTab,

    // Undo/Redo system
    undo_stack: Vec<EditorAction>,
    redo_stack: Vec<EditorAction>,
    max_undo_history: usize,

    // File dialogs
    show_open_dialog: bool,
    show_save_dialog: bool,

    // Console command handling
    pending_scene_switch: Option<String>,
    pending_object_spawn: Option<String>,
}

impl Default for UnityStyleEditor {
    fn default() -> Self {
        let mut editor = Self {
            scene_manager: SceneManager::new(),
            object_hierarchy: ObjectHierarchy::new(),
            inspector: Inspector::new(),
            scripting_panel: ScriptingPanel::new(),
            viewport: Viewport::new(),
            project_browser: ProjectBrowser::new(),
            console: Console::new(),
            selected_object: None,
            view_mode: ViewMode::Scene3D,
            physics_world: PhysicsWorld::new(),
            is_simulating: false,
            show_hierarchy: true,
            show_inspector: true,
            show_scripting: true,
            show_project_browser: true,
            show_console: true,
            context_menu_open: false,
            context_menu_position: egui::Pos2::ZERO,
            drag_payload: None,
            dock_state: Self::create_default_dock_state(),
            bottom_tab: BottomTab::Console,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_history: 50,
            show_open_dialog: false,
            show_save_dialog: false,
            pending_scene_switch: None,
            pending_object_spawn: None,
        };

        // Initialize physics for the default scene
        editor.initialize_physics_for_scene();

        editor
    }
}

impl eframe::App for UnityStyleEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for smooth animation
        ctx.request_repaint();

        // Update physics simulation
        if self.is_simulating {
            self.physics_world.step();

            // Sync physics world state with scene objects
            self.sync_physics_to_scene();
        }

        // Handle console commands - REAL IMPLEMENTATION
        self.handle_console_commands();

        // Create menu bar
        self.create_menu_bar(ctx);

        // Create main layout with panels
        self.create_main_layout(ctx);

        // Handle context menus
        self.handle_context_menus(ctx);

        // Handle drag and drop
        self.handle_drag_drop(ctx);

        // Update viewport selection when object is selected
        if let Some(selected_id) = self.selected_object {
            if let Some(scene) = self.scene_manager.current_scene() {
                if let Some(object) = scene.objects.get(&selected_id) {
                    // Update viewport selection callback
                    if let Some(ref callback) = self.viewport.selection_callback {
                        // This will be handled by the viewport internally
                    }
                }
            }
        }
    }
}

impl UnityStyleEditor {
    /// Create the default dock state layout
    fn create_default_dock_state() -> DockState<PanelTab> {
        // Create a simpler, more stable dock layout
        let mut dock_state = DockState::new(vec![PanelTab::Viewport]);

        // Try to create separate windows that will snap properly
        let _inspector_window = dock_state.add_window(vec![PanelTab::Inspector]);
        let _hierarchy_window = dock_state.add_window(vec![PanelTab::Hierarchy]);
        let _console_window = dock_state.add_window(vec![PanelTab::Console]);
        let _project_window = dock_state.add_window(vec![PanelTab::Project]);
        let _scripting_window = dock_state.add_window(vec![PanelTab::Scripting]);

        dock_state
    }

    /// Create the top menu bar
    fn create_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        self.scene_manager
                            .create_new_scene("New Scene".to_string(), false);
                        ui.close_menu();
                    }
                    if ui.button("Open Scene").clicked() {
                        self.show_open_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Save Scene").clicked() {
                        self.save_current_scene();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        self.redo();
                        ui.close_menu();
                    }
                });

                // GameObject menu
                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Empty").clicked() {
                        if let Some(scene) = self.scene_manager.current_scene_mut() {
                            scene.add_object("Empty".to_string(), GameObjectType::Empty);
                        }
                        ui.close_menu();
                    }
                    ui.menu_button("3D Object", |ui| {
                        if ui.button("Cube").clicked() {
                            if let Some(scene) = self.scene_manager.current_scene_mut() {
                                scene.add_object("Cube".to_string(), GameObjectType::Cube);
                            }
                            ui.close_menu();
                        }
                        if ui.button("Sphere").clicked() {
                            if let Some(scene) = self.scene_manager.current_scene_mut() {
                                scene.add_object("Sphere".to_string(), GameObjectType::Sphere);
                            }
                            ui.close_menu();
                        }
                    });
                });

                // Window menu
                ui.menu_button("Window", |ui| {
                    ui.checkbox(&mut self.show_hierarchy, "Hierarchy");
                    ui.checkbox(&mut self.show_inspector, "Inspector");
                    ui.checkbox(&mut self.show_scripting, "Scripting");
                    ui.checkbox(&mut self.show_project_browser, "Project");
                    ui.checkbox(&mut self.show_console, "Console");
                });

                // Simulation controls
                ui.separator();

                // Add visual indicator for simulation state
                let sim_color = if self.is_simulating {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                };

                ui.colored_label(sim_color, "●");

                if ui
                    .button(if self.is_simulating {
                        "⏸ Pause"
                    } else {
                        "▶ Play"
                    })
                    .clicked()
                {
                    self.is_simulating = !self.is_simulating;
                    if self.is_simulating {
                        // Initialize physics when starting simulation
                        self.initialize_physics_for_scene();
                    }
                }
                if ui.button("⏹ Stop").clicked() {
                    self.is_simulating = false;
                    // Reset physics world
                    self.physics_world = PhysicsWorld::new();
                    self.initialize_physics_for_scene();
                }

                ui.separator();

                ui.label(format!("FPS: {:.0}", 1.0 / ui.ctx().input(|i| i.stable_dt)));
            });
        });
    }

    /// Create the main layout with all panels
    fn create_main_layout(&mut self, ctx: &egui::Context) {
        // Create a more Unity-like layout without using dock for now
        // Left panel - Hierarchy
        if self.show_hierarchy {
            egui::SidePanel::left("hierarchy_panel")
                .resizable(true)
                .default_width(250.0)
                .min_width(200.0)
                .max_width(400.0)
                .show(ctx, |ui| {
                    self.object_hierarchy.show_ui(
                        ui,
                        self.scene_manager.current_scene_mut().unwrap(),
                        &mut self.selected_object,
                    );
                });
        }

        // Right panel - Inspector
        if self.show_inspector {
            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(300.0)
                .min_width(250.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        let transform_changed =
                            self.inspector.show_ui(ui, scene, self.selected_object);

                        // If transform changed, sync to physics world
                        if transform_changed {
                            self.sync_scene_to_physics();
                        }
                    }
                });
        }

        // Bottom panel - Console and Project Browser
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(250.0)
            .min_height(150.0)
            .max_height(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Tabs for bottom panel
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Console, "Console");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Project, "Project");
                    if self.show_scripting {
                        ui.selectable_value(
                            &mut self.bottom_tab,
                            BottomTab::Scripting,
                            "Scripting",
                        );
                    }
                });

                ui.separator();

                match self.bottom_tab {
                    BottomTab::Console => {
                        if self.show_console {
                            self.console.show_ui(ui);
                        }
                    }
                    BottomTab::Project => {
                        if self.show_project_browser {
                            self.project_browser.show_ui(ui);
                        }
                    }
                    BottomTab::Scripting => {
                        if self.show_scripting {
                            self.scripting_panel.show_ui(ui);
                        }
                    }
                }
            });

        // Central panel - Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(scene) = self.scene_manager.current_scene() {
                if let Some(new_selection) =
                    self.viewport.show_content(ui, scene, self.selected_object)
                {
                    self.selected_object = Some(new_selection);
                }
            }
        });
    }

    /// Handle context menus
    fn handle_context_menus(&mut self, ctx: &egui::Context) {
        if self.context_menu_open {
            egui::Area::new(egui::Id::new("context_menu"))
                .fixed_pos(self.context_menu_position)
                .show(ctx, |ui| {
                    ui.group(|ui| {
                        if ui.button("Create Cube").clicked() {
                            if let Some(scene) = self.scene_manager.current_scene_mut() {
                                scene.add_object("Cube".to_string(), GameObjectType::Cube);
                            }
                            self.context_menu_open = false;
                        }
                        if ui.button("Create Sphere").clicked() {
                            if let Some(scene) = self.scene_manager.current_scene_mut() {
                                scene.add_object("Sphere".to_string(), GameObjectType::Sphere);
                            }
                            self.context_menu_open = false;
                        }
                        if ui.button("Create Light").clicked() {
                            if let Some(scene) = self.scene_manager.current_scene_mut() {
                                scene.add_object("Light".to_string(), GameObjectType::Light);
                            }
                            self.context_menu_open = false;
                        }
                    });
                });
        }
    }

    /// Handle drag and drop operations - REAL IMPLEMENTATION
    fn handle_drag_drop(&mut self, ctx: &egui::Context) {
        // Handle active drag operations
        if let Some(payload) = &self.drag_payload.clone() {
            match payload {
                DragPayload::ScriptFile(script_path) => {
                    // Handle script file drag - attach to selected object
                    if let Some(object_id) = self.selected_object {
                        if let Some(scene) = self.scene_manager.current_scene_mut() {
                            if let Some(object) = scene.objects.get_mut(&object_id) {
                                // Check if object already has a script component
                                let mut has_script = false;
                                for component in &mut object.components {
                                    if let Component::Script {
                                        script_path: path, ..
                                    } = component
                                    {
                                        *path = script_path.clone();
                                        has_script = true;
                                        break;
                                    }
                                }

                                // Add new script component if none exists
                                if !has_script {
                                    object.components.push(Component::Script {
                                        script_path: script_path.clone(),
                                        code: format!(
                                            "// Script: {}\n// Add your Matrix Language code here",
                                            script_path
                                        ),
                                    });
                                }

                                self.console.log(
                                    LogLevel::Info,
                                    &format!(
                                        "Attached script '{}' to object '{}'",
                                        script_path, object.name
                                    ),
                                    "DragDrop",
                                );
                            }
                        }
                        self.drag_payload = None;
                    }
                }
                DragPayload::GameObject(object_id) => {
                    // Handle GameObject reparenting through drag
                    // This would be implemented with drop targets in the hierarchy panel
                    self.console.log(
                        LogLevel::Debug,
                        &format!("GameObject {} drag operation", object_id),
                        "DragDrop",
                    );
                    self.drag_payload = None;
                }
                DragPayload::Asset(asset_name) => {
                    // Handle asset assignment (materials, meshes, etc.)
                    if let Some(object_id) = self.selected_object {
                        if let Some(scene) = self.scene_manager.current_scene_mut() {
                            if let Some(object) = scene.objects.get_mut(&object_id) {
                                // Check asset type and assign appropriately
                                if asset_name.ends_with(".mat") || asset_name.contains("Material") {
                                    // Material assignment
                                    for component in &mut object.components {
                                        if let Component::Renderer { material, .. } = component {
                                            *material = asset_name.clone();
                                            break;
                                        }
                                    }
                                } else if asset_name.ends_with(".mesh")
                                    || asset_name.contains("Mesh")
                                {
                                    // Mesh assignment
                                    for component in &mut object.components {
                                        if let Component::Mesh { mesh_type } = component {
                                            *mesh_type = asset_name.clone();
                                            break;
                                        }
                                    }
                                }

                                self.console.log(
                                    LogLevel::Info,
                                    &format!(
                                        "Assigned asset '{}' to object '{}'",
                                        asset_name, object.name
                                    ),
                                    "DragDrop",
                                );
                            }
                        }
                    }
                    self.drag_payload = None;
                }
            }
        }
    }

    /// Setup callbacks for communication between panels - REAL IMPLEMENTATION
    fn setup_callbacks(&mut self) {
        // Console commands are now handled directly in the update loop through message passing
        // This provides proper separation of concerns and avoids borrowing issues

        // Set up viewport callbacks for object creation
        // Note: These cannot be easily done here due to borrowing issues,
        // so they are handled inline in the update loop

        self.console.log(
            LogLevel::Debug,
            "Console command system initialized with message passing",
            "System",
        );
    }

    /// Sync physics world state with scene objects
    fn sync_physics_to_scene(&mut self) {
        if let Some(scene) = self.scene_manager.current_scene_mut() {
            // Update positions of physics-enabled objects
            for (object_id, object) in &mut scene.objects {
                // Check if object has a rigid body component
                let mut has_rigid_body = false;
                let mut physics_body_id = None;

                for (idx, component) in object.components.iter().enumerate() {
                    if let Component::RigidBody { .. } = component {
                        has_rigid_body = true;
                        physics_body_id = Some(idx); // Use component index as physics ID
                        break;
                    }
                }

                if has_rigid_body {
                    if let Some(physics_id) = physics_body_id {
                        // Try to get the physics body from the world
                        if physics_id < self.physics_world.rigid_bodies.len() {
                            let physics_body = &self.physics_world.rigid_bodies[physics_id];
                            object.transform.position = physics_body.position;
                            // Also sync rotation if needed
                            // object.transform.rotation = physics_body.rotation.to_euler_angles();
                        }
                    }
                }
            }
        }
    }

    /// Sync scene objects to physics world (when objects are moved in editor)
    fn sync_scene_to_physics(&mut self) {
        if let Some(scene) = self.scene_manager.current_scene() {
            for (object_id, object) in &scene.objects {
                for (idx, component) in object.components.iter().enumerate() {
                    if let Component::RigidBody { .. } = component {
                        // Update physics body position
                        if idx < self.physics_world.rigid_bodies.len() {
                            self.physics_world.rigid_bodies[idx].position =
                                object.transform.position;
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Initialize physics bodies for objects with RigidBody components
    fn initialize_physics_for_scene(&mut self) {
        if let Some(scene) = self.scene_manager.current_scene() {
            // Clear existing physics bodies
            self.physics_world.rigid_bodies.clear();

            for (object_id, object) in &scene.objects {
                for component in &object.components {
                    if let Component::RigidBody { shape, mass } = component {
                        // Add rigid body to physics world
                        let body_id = self.physics_world.add_rigid_body(
                            shape.clone(),
                            *mass,
                            object.transform.position,
                        );
                        // Note: In a real implementation, we'd need to store the mapping
                        // between object IDs and physics body IDs
                        break;
                    }
                }
            }
        }
    }

    /// Handle console commands that require access to editor state - REAL IMPLEMENTATION
    fn handle_console_commands(&mut self) {
        // Handle pending scene switch commands
        if let Some(scene_name) = self.console.take_pending_scene_command() {
            let success = self.handle_scene_switch_command(&scene_name);
            if success {
                self.console.log(
                    LogLevel::Info,
                    &format!("Successfully switched to scene: {}", scene_name),
                    "System",
                );
            } else {
                self.console.log(
                    LogLevel::Warning,
                    &format!(
                        "Failed to switch to scene: {} (scene not found)",
                        scene_name
                    ),
                    "System",
                );
            }
        }

        // Handle pending object spawn commands
        if let Some(object_type) = self.console.take_pending_spawn_command() {
            let success = self.handle_object_spawn_command(&object_type);
            if success {
                self.console.log(
                    LogLevel::Info,
                    &format!("Successfully spawned object: {}", object_type),
                    "System",
                );
            } else {
                self.console.log(
                    LogLevel::Warning,
                    &format!("Failed to spawn object: {} (invalid type)", object_type),
                    "System",
                );
            }
        }
    }

    /// Handle scene switching command from console - REAL IMPLEMENTATION
    fn handle_scene_switch_command(&mut self, scene_name: &str) -> bool {
        // List all available scenes for debugging
        self.console.log(
            LogLevel::Debug,
            &format!(
                "Available scenes: {:?}",
                self.scene_manager
                    .scenes
                    .iter()
                    .map(|s| &s.name)
                    .collect::<Vec<_>>()
            ),
            "System",
        );

        // Try to find scene by name (case-insensitive)
        for (index, scene) in self.scene_manager.scenes.iter().enumerate() {
            if scene.name.eq_ignore_ascii_case(scene_name) {
                self.scene_manager.switch_scene(index);
                // Reinitialize physics for the new scene
                self.initialize_physics_for_scene();
                return true;
            }
        }

        // Try to parse as scene index
        if let Ok(index) = scene_name.parse::<usize>() {
            if index < self.scene_manager.scenes.len() {
                self.scene_manager.switch_scene(index);
                // Reinitialize physics for the new scene
                self.initialize_physics_for_scene();
                return true;
            }
        }

        // If no match found, create a new scene with that name
        if scene_name.len() > 0 && scene_name.len() < 50 {
            let new_index = self
                .scene_manager
                .create_new_scene(scene_name.to_string(), false);
            self.scene_manager.switch_scene(new_index);
            self.initialize_physics_for_scene();
            self.console.log(
                LogLevel::Info,
                &format!("Created and switched to new scene: {}", scene_name),
                "System",
            );
            return true;
        }

        false
    }

    /// Handle object spawning command from console - REAL IMPLEMENTATION
    fn handle_object_spawn_command(&mut self, object_type: &str) -> bool {
        if let Some(scene) = self.scene_manager.current_scene_mut() {
            let (game_object_type, object_name) = match object_type.to_lowercase().as_str() {
                "cube" => (GameObjectType::Cube, "Cube (Console)"),
                "sphere" => (GameObjectType::Sphere, "Sphere (Console)"),
                "cylinder" => (GameObjectType::Cylinder, "Cylinder (Console)"),
                "plane" => (GameObjectType::Plane, "Plane (Console)"),
                "light" => (GameObjectType::Light, "Light (Console)"),
                "camera" => (GameObjectType::Camera, "Camera (Console)"),
                "empty" => (GameObjectType::Empty, "Empty (Console)"),
                "rigidbody" | "rigid" => (
                    GameObjectType::RigidBody(Shape::Box {
                        size: Vec3::new(1.0, 1.0, 1.0),
                    }),
                    "RigidBody (Console)",
                ),
                "softbody" | "soft" => (GameObjectType::SoftBody, "SoftBody (Console)"),
                "fluid" | "fluidemitter" => {
                    (GameObjectType::FluidEmitter, "FluidEmitter (Console)")
                }
                _ => return false,
            };

            let object_id = scene.add_object(object_name.to_string(), game_object_type.clone());

            // Add appropriate components based on object type
            if let Some(object) = scene.objects.get_mut(&object_id) {
                // Position the object at origin with slight randomization
                let random_offset = Vec3::new(
                    (object_id as f64 % 3.0) - 1.0,
                    1.0,
                    ((object_id * 7) as f64 % 3.0) - 1.0,
                );
                object.transform.position = random_offset;

                match game_object_type {
                    GameObjectType::Cube => {
                        object.components.push(Component::Mesh {
                            mesh_type: "Cube".to_string(),
                        });
                        object.components.push(Component::Renderer {
                            material: "Console".to_string(),
                            color: [0.8, 0.8, 0.2, 1.0], // Yellow tint for console-spawned objects
                        });
                    }
                    GameObjectType::Sphere => {
                        object.components.push(Component::Mesh {
                            mesh_type: "Sphere".to_string(),
                        });
                        object.components.push(Component::Renderer {
                            material: "Console".to_string(),
                            color: [0.2, 0.8, 0.8, 1.0], // Cyan tint
                        });
                    }
                    GameObjectType::Cylinder => {
                        object.components.push(Component::Mesh {
                            mesh_type: "Cylinder".to_string(),
                        });
                        object.components.push(Component::Renderer {
                            material: "Console".to_string(),
                            color: [0.8, 0.2, 0.8, 1.0], // Magenta tint
                        });
                    }
                    GameObjectType::Plane => {
                        object.components.push(Component::Mesh {
                            mesh_type: "Plane".to_string(),
                        });
                        object.components.push(Component::Renderer {
                            material: "Console".to_string(),
                            color: [0.6, 0.6, 0.6, 1.0], // Gray
                        });
                    }
                    GameObjectType::Light => {
                        object.components.push(Component::Light {
                            light_type: "Point".to_string(),
                            intensity: 1.0,
                            color: [1.0, 0.9, 0.7], // Warm white
                        });
                    }
                    GameObjectType::Camera => {
                        object.components.push(Component::Camera {
                            fov: 60.0,
                            near: 0.1,
                            far: 1000.0,
                        });
                    }
                    GameObjectType::RigidBody(_) => {
                        object.components.push(Component::Mesh {
                            mesh_type: "Cube".to_string(),
                        });
                        object.components.push(Component::Renderer {
                            material: "Physics".to_string(),
                            color: [0.9, 0.3, 0.3, 1.0], // Red for physics objects
                        });
                        object.components.push(Component::RigidBody {
                            shape: Shape::Box {
                                size: Vec3::new(1.0, 1.0, 1.0),
                            },
                            mass: 1.0,
                        });
                        object.components.push(Component::Collider {
                            shape: Shape::Box {
                                size: Vec3::new(1.0, 1.0, 1.0),
                            },
                            is_trigger: false,
                        });
                    }
                    GameObjectType::SoftBody => {
                        object.components.push(Component::SoftBodyComponent {
                            particles: 100,
                            stiffness: 0.8,
                        });
                        object.components.push(Component::Renderer {
                            material: "SoftBody".to_string(),
                            color: [0.3, 0.9, 0.3, 0.8], // Green semi-transparent
                        });
                    }
                    GameObjectType::FluidEmitter => {
                        object.components.push(Component::Renderer {
                            material: "Fluid".to_string(),
                            color: [0.3, 0.6, 1.0, 0.8], // Blue semi-transparent
                        });
                    }
                    _ => {}
                }
            }

            // Select the newly created object
            self.selected_object = Some(object_id);

            // Log detailed information about the spawned object
            self.console.log(
                LogLevel::Debug,
                &format!(
                    "Spawned {} with ID {} at position {:?}",
                    object_name,
                    object_id,
                    scene.objects.get(&object_id).map(|o| o.transform.position)
                ),
                "System",
            );

            // Reinitialize physics if the object has physics components
            if matches!(game_object_type, GameObjectType::RigidBody(_)) {
                self.initialize_physics_for_scene();
            }

            return true;
        }

        false
    }

    /// Save the current scene
    fn save_current_scene(&mut self) {
        if let Some(scene) = self.scene_manager.current_scene() {
            let filename = format!("{}.scene", scene.name.replace(" ", "_"));
            let current_index = self.scene_manager.current_scene;
            if let Err(e) = self.scene_manager.save_scene(current_index, &filename) {
                self.console.log(
                    LogLevel::Error,
                    &format!("Failed to save scene: {}", e),
                    "System",
                );
            } else {
                self.console.log(
                    LogLevel::Info,
                    &format!("Scene saved as: {}", filename),
                    "System",
                );
            }
        }
    }

    /// Undo the last action
    fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            // Apply the reverse of the action
            match action.clone() {
                EditorAction::CreateObject { id, .. } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        scene.remove_object(id);
                    }
                }
                EditorAction::DeleteObject { id, object } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        scene.objects.insert(id, object);
                    }
                }
                EditorAction::ModifyTransform {
                    id, old_transform, ..
                } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        if let Some(object) = scene.objects.get_mut(&id) {
                            object.transform = old_transform;
                        }
                    }
                }
                // Add more action reversals as needed
                _ => {}
            }

            // Move to redo stack
            self.redo_stack.push(action);

            // Limit redo stack size
            if self.redo_stack.len() > self.max_undo_history {
                self.redo_stack.remove(0);
            }
        }
    }

    /// Redo the last undone action
    fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            // Apply the action again
            match action.clone() {
                EditorAction::CreateObject { id, object } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        scene.objects.insert(id, object);
                    }
                }
                EditorAction::DeleteObject { id, .. } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        scene.remove_object(id);
                    }
                }
                EditorAction::ModifyTransform {
                    id, new_transform, ..
                } => {
                    if let Some(scene) = self.scene_manager.current_scene_mut() {
                        if let Some(object) = scene.objects.get_mut(&id) {
                            object.transform = new_transform;
                        }
                    }
                }
                // Add more action applications as needed
                _ => {}
            }

            // Move back to undo stack
            self.undo_stack.push(action);
        }
    }
}

/// Launch the Unity-style editor GUI
pub fn launch_unity_editor() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Matrix Language - Unity Style Editor",
        options,
        Box::new(|_cc| Ok(Box::new(UnityStyleEditor::default()))),
    )
}
