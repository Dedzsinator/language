// Unity-Style Layout Manager for Animation-Physics Workflows
use super::*;
use crate::ecs::World;
// use crate::physics::*;
use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
// use std::collections::HashMap;

/// Main view tab selection
#[derive(Debug, Clone, PartialEq)]
pub enum MainViewTab {
    Scene,
    Game,
}

/// Side panel tab selection
#[derive(Debug, Clone, PartialEq)]
pub enum SceneViewTab {
    Hierarchy,
    Project,
}

/// Bottom panel tab selection
#[derive(Debug, Clone, PartialEq)]
pub enum BottomPanelTab {
    Console,
    Animation,
    Physics,
}

/// Tab types that can be used in the dock area
#[derive(Debug, Clone, PartialEq)]
pub enum EditorTab {
    SceneView,
    GameView,
    Inspector,
    Hierarchy,
    Project,
    Console,
    Animation,
    PhysicsDebugger,
}

/// Unity-style layout manager that recreates the Unity interface
pub struct UnityLayoutManager {
    // Layout state
    pub scene_view_tab: SceneViewTab,
    pub main_view_tab: MainViewTab,
    pub bottom_panel_tab: BottomPanelTab,

    // Panel dimensions and positions
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,

    // Panel visibility flags
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_project: bool,
    pub show_console: bool,
    pub show_animation: bool,
    pub show_physics: bool,

    // Maximized view
    pub maximized_view: Option<EditorTab>,

    // Panel instances
    pub scene_view: scene_view::SceneView,
    pub game_view: game_view::GameView,
    pub inspector: inspector::Inspector,
    pub hierarchy: object_hierarchy::ObjectHierarchy,
    pub project_browser: project_browser::ProjectBrowser,
    pub console: console::Console,
    pub animation_view: animation_view::AnimationView,
    pub physics_debugger: physics_debugger::PhysicsDebugger,

    // Dock state for flexible layout
    pub dock_state: DockState<EditorTab>,

    // Selected entity tracking
    pub selected_entity: Option<usize>,
    pub hovering_entity: Option<usize>,

    // Editor state
    pub is_playing: bool,
    pub is_paused: bool,
    pub show_grid: bool,
    pub show_gizmos: bool,
    pub gizmo_mode: GizmoMode,
}

impl Default for UnityLayoutManager {
    fn default() -> Self {
        // Create dock state with initial Unity-like layout
        let mut dock_state = DockState::new(vec![EditorTab::SceneView]);

        // Split into main area and right panel (inspector)
        let [center, _right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.75,
            vec![EditorTab::Inspector],
        );

        // Split left side for hierarchy/project
        let [_left, main] = dock_state.main_surface_mut().split_left(
            center,
            0.2,
            vec![EditorTab::Hierarchy, EditorTab::Project],
        );

        // Split bottom for console/animation/physics
        let [_, _bottom] = dock_state.main_surface_mut().split_below(
            main,
            0.7,
            vec![
                EditorTab::Console,
                EditorTab::Animation,
                EditorTab::PhysicsDebugger,
            ],
        );

        // Add game view as a tab in the main area
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(EditorTab::GameView);

        Self {
            scene_view_tab: SceneViewTab::Hierarchy,
            main_view_tab: MainViewTab::Scene,
            bottom_panel_tab: BottomPanelTab::Console,

            left_panel_width: 300.0,
            right_panel_width: 300.0,
            bottom_panel_height: 200.0,

            show_hierarchy: true,
            show_inspector: true,
            show_project: true,
            show_console: true,
            show_animation: true,
            show_physics: true,

            maximized_view: None,

            scene_view: scene_view::SceneView::new(),
            game_view: game_view::GameView::new(),
            inspector: inspector::Inspector::new(),
            hierarchy: object_hierarchy::ObjectHierarchy::new(),
            project_browser: project_browser::ProjectBrowser::new(),
            console: console::Console::new(),
            animation_view: animation_view::AnimationView::new(),
            physics_debugger: physics_debugger::PhysicsDebugger::new(),

            dock_state,

            selected_entity: None,
            hovering_entity: None,

            is_playing: false,
            is_paused: false,
            show_grid: true,
            show_gizmos: true,
            gizmo_mode: GizmoMode::Translate,
        }
    }
}

impl UnityLayoutManager {
    /// Create a new Unity-style layout manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the layout manager state
    pub fn update(&mut self, _world: &mut World, delta_time: f32) {
        // Update game view
        if self.is_playing && !self.is_paused {
            self.game_view.update(delta_time * 1000.0);
        }

        // Update animation view
        self.animation_view.update(delta_time);

        // Sync playing state between different panels
        if self.game_view.is_playing != self.is_playing {
            self.is_playing = self.game_view.is_playing;
        }

        if self.game_view.is_paused != self.is_paused {
            self.is_paused = self.game_view.is_paused;
        }
    }

    /// Draw the layout UI
    pub fn ui(&mut self, ctx: &egui::Context, world: &mut World) {
        // Draw the main menu bar
        self.draw_main_menu_bar(ctx, world);

        // Draw the toolbar
        self.draw_toolbar(ctx, world);

        // If we're in maximized mode, only show that view
        if let Some(maximized_tab) = &self.maximized_view {
            match maximized_tab {
                EditorTab::SceneView => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.scene_view.show(ui, world);
                    });
                }
                EditorTab::GameView => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.game_view.ui(ui, world);
                    });
                }
                _ => {
                    // Exit maximized mode for other tabs
                    self.maximized_view = None;
                }
            }

            // Show exit maximized button
            egui::Window::new("Maximized")
                .title_bar(false)
                .fixed_size([40.0, 40.0])
                .fixed_pos([
                    ctx.screen_rect().right() - 50.0,
                    ctx.screen_rect().top() + 50.0,
                ])
                .show(ctx, |ui| {
                    if ui.button("⊟").clicked() {
                        self.maximized_view = None;
                    }
                });

            return;
        }

        // Draw the dockable layout
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a clone of the dock state
            let mut dock_state_clone = self.dock_state.clone();

            // Use a viewer that doesn't borrow self mutably
            let mut viewer = EditorTabViewer {
                manager: self,
                world,
            };

            // Show the dock area
            DockArea::new(&mut dock_state_clone).show_inside(ui, &mut viewer);

            // Update our dock state with the modified clone
            self.dock_state = dock_state_clone;
        });
    }

    /// Draw the main menu bar
    fn draw_main_menu_bar(&mut self, ctx: &egui::Context, _world: &mut World) {
        egui::TopBottomPanel::top("main_menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        // Create new scene
                        ui.close_menu();
                    }
                    if ui.button("Open Scene...").clicked() {
                        // Open scene
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        // Save scene
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        // Save scene as
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        // Exit application
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        // Undo
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        // Redo
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut").clicked() {
                        // Cut
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        // Copy
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        // Paste
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Project Settings...").clicked() {
                        // Project settings
                        ui.close_menu();
                    }
                });

                ui.menu_button("GameObject", |ui| {
                    if ui.button("Create Empty").clicked() {
                        // Create empty object
                        ui.close_menu();
                    }

                    ui.menu_button("3D Object", |ui| {
                        if ui.button("Cube").clicked() {
                            // Create cube
                            ui.close_menu();
                        }
                        if ui.button("Sphere").clicked() {
                            // Create sphere
                            ui.close_menu();
                        }
                        if ui.button("Capsule").clicked() {
                            // Create capsule
                            ui.close_menu();
                        }
                        if ui.button("Plane").clicked() {
                            // Create plane
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Light", |ui| {
                        if ui.button("Directional Light").clicked() {
                            // Create directional light
                            ui.close_menu();
                        }
                        if ui.button("Point Light").clicked() {
                            // Create point light
                            ui.close_menu();
                        }
                        if ui.button("Spot Light").clicked() {
                            // Create spot light
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Component", |ui| {
                    ui.menu_button("Physics", |ui| {
                        if ui.button("Rigidbody").clicked() {
                            // Add rigidbody component
                            ui.close_menu();
                        }
                        if ui.button("Box Collider").clicked() {
                            // Add box collider
                            ui.close_menu();
                        }
                        if ui.button("Sphere Collider").clicked() {
                            // Add sphere collider
                            ui.close_menu();
                        }
                        if ui.button("Capsule Collider").clicked() {
                            // Add capsule collider
                            ui.close_menu();
                        }
                        if ui.button("Mesh Collider").clicked() {
                            // Add mesh collider
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Window", |ui| {
                    if ui.checkbox(&mut self.show_hierarchy, "Hierarchy").clicked() {
                        // Toggle hierarchy visibility
                    }
                    if ui.checkbox(&mut self.show_inspector, "Inspector").clicked() {
                        // Toggle inspector visibility
                    }
                    if ui
                        .checkbox(&mut self.show_project, "Project Browser")
                        .clicked()
                    {
                        // Toggle project browser visibility
                    }
                    if ui.checkbox(&mut self.show_console, "Console").clicked() {
                        // Toggle console visibility
                    }
                    if ui.checkbox(&mut self.show_animation, "Animation").clicked() {
                        // Toggle animation view visibility
                    }
                    if ui
                        .checkbox(&mut self.show_physics, "Physics Debugger")
                        .clicked()
                    {
                        // Toggle physics debugger visibility
                    }
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        // Reset layout to default
                        *self = Self::default();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        // Open documentation
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        // Show about dialog
                        ui.close_menu();
                    }
                });

                // Right-aligned play controls
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        // Play/pause buttons
                        if ui.button(if self.is_playing { "■" } else { "▶" }).clicked() {
                            self.is_playing = !self.is_playing;
                            self.game_view.toggle_play();
                        }

                        if ui.button(if self.is_paused { "▶" } else { "❚❚" }).clicked() {
                            self.is_paused = !self.is_paused;
                            self.game_view.toggle_pause();
                        }

                        if ui.button("⏭").clicked() {
                            // Step frame
                            self.game_view.step_frame();
                        }
                    });
                });
            });
        });
    }

    /// Draw the toolbar
    fn draw_toolbar(&mut self, ctx: &egui::Context, _world: &mut World) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Transform tools
                ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Translate, "🔄 Move");
                ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Rotate, "↻ Rotate");
                ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Scale, "⇲ Scale");

                ui.separator();

                // Toggle buttons for grid and gizmos
                ui.toggle_value(&mut self.show_grid, "Grid");
                ui.toggle_value(&mut self.show_gizmos, "Gizmos");

                // Sync scene view settings
                self.scene_view.show_grid = self.show_grid;
                self.scene_view.show_gizmos = self.show_gizmos;
                self.scene_view.gizmo_mode = self.gizmo_mode;
            });
        });
    }

    /// Select an entity in the scene
    pub fn select_entity(&mut self, entity_id: Option<usize>) {
        self.selected_entity = entity_id;
    }

    /// Toggle maximized view for a panel
    pub fn toggle_maximize(&mut self, tab: EditorTab) {
        if self.maximized_view == Some(tab.clone()) {
            self.maximized_view = None;
        } else {
            self.maximized_view = Some(tab);
        }
    }

    /// Create a simple default scene with physics objects
    pub fn create_default_scene(&mut self, _world: &mut World) {
        // In a real implementation, this would populate the world
        // with some default entities to get started

        // Add a floor

        // Add a few physics objects

        // Add a light

        // Add a camera
    }
}

/// Helper struct to implement TabViewer trait for egui_dock 0.16
pub struct EditorTabViewer<'a> {
    pub manager: &'a mut UnityLayoutManager,
    pub world: &'a mut World,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::SceneView => {
                if ui.button("⊞").clicked() {
                    self.manager.toggle_maximize(EditorTab::SceneView);
                }
                ui.separator();
                self.manager.scene_view.show(ui, self.world);
            }
            EditorTab::GameView => {
                if ui.button("⊞").clicked() {
                    self.manager.toggle_maximize(EditorTab::GameView);
                }
                ui.separator();
                self.manager.game_view.ui(ui, self.world);
            }
            EditorTab::Inspector => {
                self.manager
                    .inspector
                    .show_ui(ui, self.world, self.manager.selected_entity);
            }
            EditorTab::Hierarchy => {
                self.manager
                    .hierarchy
                    .show_ui_for_world(ui, self.world, &mut self.manager.selected_entity);
            }
            EditorTab::Project => {
                self.manager.project_browser.show_ui(ui);
            }
            EditorTab::Console => {
                self.manager.console.show_ui(ui);
            }
            EditorTab::Animation => {
                self.manager.animation_view.show_ui(ui, self.world);
            }
            EditorTab::PhysicsDebugger => {
                self.manager.physics_debugger.ui(ui, self.world);
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::SceneView => "Scene View".into(),
            EditorTab::GameView => "Game View".into(),
            EditorTab::Inspector => "Inspector".into(),
            EditorTab::Hierarchy => "Hierarchy".into(),
            EditorTab::Project => "Project".into(),
            EditorTab::Console => "Console".into(),
            EditorTab::Animation => "Animation".into(),
            EditorTab::PhysicsDebugger => "Physics Debugger".into(),
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        // Don't allow closing tabs for now
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unity_layout_default() {
        let layout = UnityLayoutManager::default();
        assert!(!layout.is_playing);
        assert!(!layout.is_paused);
        assert!(layout.show_grid);
        assert!(layout.show_gizmos);
    }

    #[test]
    fn test_toggle_maximize() {
        let mut layout = UnityLayoutManager::default();
        assert_eq!(layout.maximized_view, None);

        layout.toggle_maximize(EditorTab::SceneView);
        assert_eq!(layout.maximized_view, Some(EditorTab::SceneView));

        layout.toggle_maximize(EditorTab::SceneView);
        assert_eq!(layout.maximized_view, None);
    }

    #[test]
    fn test_select_entity() {
        let mut layout = UnityLayoutManager::default();
        assert_eq!(layout.selected_entity, None);

        layout.select_entity(Some(5));
        assert_eq!(layout.selected_entity, Some(5));

        layout.select_entity(None);
        assert_eq!(layout.selected_entity, None);
    }
}
