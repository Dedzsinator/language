use crate::ecs::World;
use crate::physics::math::*;
// use crate::physics::rigid_body::{RigidBody, Shape};
use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke};

use super::GizmoMode;
use std::collections::HashMap;

/// Scene View camera control mode
#[derive(Debug, Clone, PartialEq)]
pub enum CameraMode {
    Orbit,
    Flythrough,
    Pan,
}

/// Scene view orientation for quick snapping
#[derive(Debug, Clone, PartialEq)]
pub enum ViewOrientation {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    Perspective,
}

/// Rendering style for the 3D view
#[derive(Debug, Clone, PartialEq)]
pub enum RenderStyle {
    Shaded,
    Wireframe,
    ShadedWireframe,
    Unlit,
}

/// Scene View panel for 3D scene editing - similar to Unity's Scene view
pub struct SceneView {
    // Camera controls
    pub camera_position: Vec3,
    pub camera_rotation: Vec3,
    pub camera_fov: f32,
    pub camera_mode: CameraMode,
    pub near_clip: f32,
    pub far_clip: f32,
    pub is_orthographic: bool,
    pub orthographic_size: f32,

    // Viewport settings
    pub show_grid: bool,
    pub show_gizmos: bool,
    pub show_wireframe: bool,
    pub show_lighting: bool,
    pub show_audio_sources: bool,
    pub show_colliders: bool,
    pub show_rigidbodies: bool,
    pub show_joints: bool,
    pub render_style: RenderStyle,

    // Display settings
    pub background_color: [f32; 4],

    // Gizmo settings
    pub gizmo_mode: GizmoMode,
    pub gizmo_space: bool, // true = world space, false = local space
    pub selected_object_id: Option<usize>,
    pub selected_component_id: Option<usize>,

    // Grid settings
    pub grid_size: f32,
    pub grid_subdivisions: u32,
    pub grid_color: Color32,
    pub grid_secondary_color: Color32,

    // Mouse interaction
    pub last_mouse_pos: Option<Pos2>,
    pub is_dragging: bool,
    pub mouse_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub key_shortcuts: HashMap<String, String>,

    // View orientation gizmo
    pub orientation_gizmo_size: f32,
    pub show_orientation_gizmo: bool,
    pub current_orientation: ViewOrientation,

    // Physics debugging
    pub draw_contacts: bool,
    pub draw_bounds: bool,
    pub draw_forces: bool,

    // Last frame interaction info
    pub hovered_object_id: Option<usize>,
    pub last_clicked_position: Option<Vec3>,

    // Viewport rectangle in screen space
    pub viewport_rect: Option<Rect>,
}

impl Default for SceneView {
    fn default() -> Self {
        Self {
            camera_position: Vec3::new(5.0, 5.0, 5.0),
            camera_rotation: Vec3::new(-30.0, 45.0, 0.0),
            camera_fov: 60.0,
            camera_mode: CameraMode::Orbit,
            near_clip: 0.1,
            far_clip: 1000.0,
            is_orthographic: false,
            orthographic_size: 5.0,

            show_grid: true,
            show_gizmos: true,
            show_wireframe: false,
            show_lighting: true,
            show_audio_sources: false,
            show_colliders: true,
            show_rigidbodies: true,
            show_joints: true,
            render_style: RenderStyle::Shaded,
            background_color: [0.2, 0.2, 0.2, 1.0],

            gizmo_mode: GizmoMode::Translate,
            gizmo_space: false,
            selected_object_id: None,
            selected_component_id: None,

            grid_size: 1.0,
            grid_subdivisions: 10,
            grid_color: Color32::from_gray(80),
            grid_secondary_color: Color32::from_gray(60),

            last_mouse_pos: None,
            is_dragging: false,
            mouse_sensitivity: 0.5,
            zoom_sensitivity: 0.1,
            key_shortcuts: Self::default_shortcuts(),

            orientation_gizmo_size: 80.0,
            show_orientation_gizmo: true,
            current_orientation: ViewOrientation::Perspective,

            draw_contacts: false,
            draw_bounds: true,
            draw_forces: false,

            hovered_object_id: None,
            last_clicked_position: None,

            viewport_rect: None,
        }
    }
}

impl SceneView {
    /// Create default key shortcuts mapping
    fn default_shortcuts() -> HashMap<String, String> {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("frame_selected".to_string(), "f".to_string());
        shortcuts.insert("toggle_gizmo_mode".to_string(), "q".to_string());
        shortcuts.insert("translate_tool".to_string(), "w".to_string());
        shortcuts.insert("rotate_tool".to_string(), "e".to_string());
        shortcuts.insert("scale_tool".to_string(), "r".to_string());
        shortcuts.insert("toggle_perspective".to_string(), "5".to_string());
        shortcuts.insert("top_view".to_string(), "7".to_string());
        shortcuts.insert("front_view".to_string(), "1".to_string());
        shortcuts.insert("side_view".to_string(), "3".to_string());
        shortcuts
    }

    /// Create a new Scene View panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current gizmo mode
    pub fn set_gizmo_mode(&mut self, mode: GizmoMode) {
        self.gizmo_mode = mode;
    }

    /// Toggle between world and local space for gizmos
    pub fn toggle_gizmo_space(&mut self) {
        self.gizmo_space = !self.gizmo_space;
    }

    /// Frame the selected object
    pub fn frame_selected(&mut self, _world: &World) {
        if let Some(id) = self.selected_object_id {
            // In a real implementation, you would find the object position in the world
            // and adjust camera position/rotation to frame it
            // This is a placeholder implementation
            self.camera_position = Vec3::new(id as f64 + 5.0, 5.0, 5.0);
        }
    }

    /// Toggle between perspective and orthographic projection
    pub fn toggle_projection(&mut self) {
        self.is_orthographic = !self.is_orthographic;
    }

    /// Set view orientation to a specific preset
    pub fn set_orientation(&mut self, orientation: ViewOrientation) {
        // Clone the orientation for the match statement since the original is moved
        let orientation_copy = orientation.clone();
        self.current_orientation = orientation;

        // Adjust the camera position and rotation based on the orientation
        match orientation_copy {
            ViewOrientation::Front => {
                self.camera_position = Vec3::new(0.0, 0.0, 10.0);
                self.camera_rotation = Vec3::new(0.0, 0.0, 0.0);
            }
            ViewOrientation::Back => {
                self.camera_position = Vec3::new(0.0, 0.0, -10.0);
                self.camera_rotation = Vec3::new(0.0, 180.0, 0.0);
            }
            ViewOrientation::Left => {
                self.camera_position = Vec3::new(-10.0, 0.0, 0.0);
                self.camera_rotation = Vec3::new(0.0, -90.0, 0.0);
            }
            ViewOrientation::Right => {
                self.camera_position = Vec3::new(10.0, 0.0, 0.0);
                self.camera_rotation = Vec3::new(0.0, 90.0, 0.0);
            }
            ViewOrientation::Top => {
                self.camera_position = Vec3::new(0.0, 10.0, 0.0);
                self.camera_rotation = Vec3::new(-90.0, 0.0, 0.0);
            }
            ViewOrientation::Bottom => {
                self.camera_position = Vec3::new(0.0, -10.0, 0.0);
                self.camera_rotation = Vec3::new(90.0, 0.0, 0.0);
            }
            ViewOrientation::Perspective => {
                self.camera_position = Vec3::new(5.0, 5.0, 5.0);
                self.camera_rotation = Vec3::new(-30.0, 45.0, 0.0);
            }
        }
    }

    /// Draw the scene view UI
    pub fn show(&mut self, ui: &mut egui::Ui, world: &mut World) {
        // Save viewport rect for interaction calculations
        self.viewport_rect = Some(ui.available_rect_before_wrap());

        // Top toolbar
        ui.horizontal(|ui| {
            // Gizmo mode selection
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Translate, "🔄 Translate");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Rotate, "↻ Rotate");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Scale, "⇲ Scale");

            // Gizmo space toggle
            if ui
                .button(if self.gizmo_space { "World" } else { "Local" })
                .clicked()
            {
                self.toggle_gizmo_space();
            }

            ui.separator();

            // Toggle buttons for various view settings
            ui.selectable_value(&mut self.is_orthographic, true, "Ortho");
            ui.selectable_value(&mut self.is_orthographic, false, "Persp");

            ui.separator();

            // View visibility toggles
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_gizmos, "Gizmos");
            ui.checkbox(&mut self.show_wireframe, "Wireframe");

            ui.separator();

            // Physics debugging
            ui.checkbox(&mut self.show_colliders, "Colliders");
            ui.checkbox(&mut self.show_rigidbodies, "Rigidbody");
            ui.checkbox(&mut self.draw_forces, "Forces");
        });

        // Main viewport area
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        // Draw orientation gizmo in the top-right corner
        if self.show_orientation_gizmo {
            self.draw_orientation_gizmo(ui, rect);
        }

        // Handle mouse input for camera control
        self.handle_camera_controls(response, rect);

        // Draw the scene content
        self.draw_scene(ui, rect, world);
    }

    /// Draw the 3D scene content
    fn draw_scene(&mut self, ui: &mut egui::Ui, rect: Rect, world: &World) {
        let painter = ui.painter_at(rect);

        // Fill with background color
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_premultiplied(
                (self.background_color[0] * 255.0) as u8,
                (self.background_color[1] * 255.0) as u8,
                (self.background_color[2] * 255.0) as u8,
                (self.background_color[3] * 255.0) as u8,
            ),
        );

        // Draw grid
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // In a real implementation, you would render 3D objects here
        // This is just a placeholder
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "3D Scene View",
            egui::FontId::proportional(24.0),
            Color32::WHITE,
        );

        // Draw physics gizmos if needed
        if self.show_colliders || self.show_rigidbodies {
            self.draw_physics_gizmos(&painter, rect, world);
        }

        // Draw manipulation gizmo for selected object
        if let Some(obj_id) = self.selected_object_id {
            self.draw_manipulation_gizmo(&painter, rect, obj_id, world);
        }
    }

    /// Draw the grid in the scene
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        // This is a simplified 2D grid - in a real 3D application,
        // you'd project 3D grid lines onto the 2D viewport

        // Grid center and size
        let center = rect.center();
        let size = rect.width().min(rect.height()) * 0.8;
        let half_size = size / 2.0;

        // Calculate grid spacing based on the grid_size and zoom level
        // This is simplified - would depend on actual 3D view transformation
        let grid_spacing = 20.0; // pixels

        // Draw main grid lines
        for i in -10..=10 {
            let offset = i as f32 * grid_spacing;

            // Horizontal line
            painter.line_segment(
                [
                    egui::pos2(center.x - half_size, center.y + offset),
                    egui::pos2(center.x + half_size, center.y + offset),
                ],
                Stroke::new(1.0, self.grid_color),
            );

            // Vertical line
            painter.line_segment(
                [
                    egui::pos2(center.x + offset, center.y - half_size),
                    egui::pos2(center.x + offset, center.y + half_size),
                ],
                Stroke::new(1.0, self.grid_color),
            );
        }

        // Draw coordinate axes with stronger stroke
        // X-axis (red)
        painter.line_segment(
            [center, egui::pos2(center.x + half_size, center.y)],
            Stroke::new(2.0, Color32::RED),
        );

        // Y-axis (green)
        painter.line_segment(
            [center, egui::pos2(center.x, center.y - half_size)],
            Stroke::new(2.0, Color32::GREEN),
        );

        // Z-axis (blue) - project Z into screen
        painter.line_segment(
            [
                center,
                egui::pos2(center.x + half_size * 0.5, center.y + half_size * 0.5),
            ],
            Stroke::new(2.0, Color32::BLUE),
        );
    }

    /// Draw the orientation gizmo cube in the corner
    fn draw_orientation_gizmo(&self, ui: &mut egui::Ui, rect: Rect) {
        // Position orientation gizmo in the top-right corner
        let gizmo_rect = Rect::from_min_size(
            rect.right_top() + egui::vec2(-self.orientation_gizmo_size - 10.0, 10.0),
            egui::vec2(self.orientation_gizmo_size, self.orientation_gizmo_size),
        );

        // Draw gizmo background
        ui.painter()
            .rect_filled(gizmo_rect, 5.0, Color32::from_black_alpha(100));

        // Draw simplified 3D cube faces
        let center = gizmo_rect.center();
        let size = self.orientation_gizmo_size * 0.4;
        let half_size = size / 2.0;

        // Simple face colors
        let top_color = Color32::from_rgb(100, 200, 100); // Green for Y+
        let right_color = Color32::from_rgb(200, 100, 100); // Red for X+
        let front_color = Color32::from_rgb(100, 100, 200); // Blue for Z+

        // Simplified cube drawing (not actual 3D projection)
        // Front face (Z+)
        ui.painter().rect_filled(
            Rect::from_center_size(
                center + egui::vec2(0.0, half_size * 0.5),
                egui::vec2(size, size * 0.8),
            ),
            0.0,
            front_color,
        );

        // Right face (X+)
        ui.painter().rect_filled(
            Rect::from_center_size(
                center + egui::vec2(half_size * 0.5, 0.0),
                egui::vec2(size * 0.8, size),
            ),
            0.0,
            right_color,
        );

        // Top face (Y+)
        ui.painter().rect_filled(
            Rect::from_center_size(
                center + egui::vec2(0.0, -half_size * 0.5),
                egui::vec2(size, size * 0.8),
            ),
            0.0,
            top_color,
        );

        // Draw labels
        let label_font = egui::FontId::proportional(12.0);
        ui.painter().text(
            center + egui::vec2(size * 0.7, 0.0),
            egui::Align2::CENTER_CENTER,
            "X",
            label_font.clone(),
            Color32::WHITE,
        );

        ui.painter().text(
            center + egui::vec2(0.0, -size * 0.7),
            egui::Align2::CENTER_CENTER,
            "Y",
            label_font.clone(),
            Color32::WHITE,
        );

        ui.painter().text(
            center + egui::vec2(0.0, size * 0.7),
            egui::Align2::CENTER_CENTER,
            "Z",
            label_font,
            Color32::WHITE,
        );
    }

    /// Draw physics gizmos for rigidbodies and colliders
    fn draw_physics_gizmos(&self, painter: &egui::Painter, rect: Rect, _world: &World) {
        // This is a simplified version - in a real application you would:
        // 1. Get all rigidbodies from the world
        // 2. Project their 3D position/shape to 2D screen coordinates
        // 3. Draw appropriate visualization

        // Simplified example - draw a placeholder for a rigidbody
        let center = rect.center();

        // Draw a box collider shape
        if self.show_colliders {
            painter.rect_stroke(
                Rect::from_center_size(center, egui::vec2(100.0, 100.0)),
                5.0,
                Stroke::new(2.0, Color32::from_rgb(0, 255, 0)),
                egui::StrokeKind::Outside,
            );
        }

        // Draw rigidbody icon
        if self.show_rigidbodies {
            painter.circle_stroke(
                center,
                15.0,
                Stroke::new(2.0, Color32::from_rgb(255, 165, 0)),
            );

            // Draw mass center dot
            painter.circle_filled(center, 4.0, Color32::from_rgb(255, 165, 0));

            // In a real application, you would draw forces if enabled
            if self.draw_forces {
                let force_dir = egui::vec2(50.0, -30.0);
                // Draw a line and then an arrow head
                painter.line_segment(
                    [center, center + force_dir],
                    Stroke::new(2.0, Color32::from_rgb(255, 0, 0)),
                );
                // Arrow uses a different signature with proper parameters
                painter.arrow(
                    center,
                    force_dir,
                    Stroke::new(2.0, Color32::from_rgb(255, 0, 0)),
                );
            }
        }
    }

    /// Draw manipulation gizmo for the selected object
    fn draw_manipulation_gizmo(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        _object_id: usize,
        _world: &World,
    ) {
        // In a real application, you would:
        // 1. Get the object's world position
        // 2. Project it to screen coordinates
        // 3. Draw the appropriate gizmo based on the current gizmo_mode

        // This is a simplified placeholder
        let center = rect.center();

        match self.gizmo_mode {
            GizmoMode::Translate => {
                // Draw translation arrows
                // X axis (red)
                painter.arrow(
                    center,
                    egui::vec2(50.0, 0.0),
                    Stroke::new(3.0, Color32::RED),
                );

                // Y axis (green)
                painter.arrow(
                    center,
                    egui::vec2(0.0, -50.0),
                    Stroke::new(3.0, Color32::GREEN),
                );

                // Z axis (blue)
                painter.arrow(
                    center,
                    egui::vec2(-35.0, 35.0),
                    Stroke::new(3.0, Color32::BLUE),
                );
            }
            GizmoMode::Rotate => {
                // Draw rotation circles
                painter.circle_stroke(
                    center,
                    40.0,
                    Stroke::new(3.0, Color32::RED.linear_multiply(0.7)),
                );
                painter.circle_stroke(
                    center,
                    35.0,
                    Stroke::new(3.0, Color32::GREEN.linear_multiply(0.7)),
                );
                painter.circle_stroke(
                    center,
                    30.0,
                    Stroke::new(3.0, Color32::BLUE.linear_multiply(0.7)),
                );
            }
            GizmoMode::Scale => {
                // Draw scale handles
                let handle_size = 8.0;

                // X axis (red)
                painter.line_segment(
                    [center, center + egui::vec2(40.0, 0.0)],
                    Stroke::new(3.0, Color32::RED),
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        center + egui::vec2(40.0, 0.0),
                        egui::vec2(handle_size, handle_size),
                    ),
                    0.0,
                    Color32::RED,
                );

                // Y axis (green)
                painter.line_segment(
                    [center, center + egui::vec2(0.0, -40.0)],
                    Stroke::new(3.0, Color32::GREEN),
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        center + egui::vec2(0.0, -40.0),
                        egui::vec2(handle_size, handle_size),
                    ),
                    0.0,
                    Color32::GREEN,
                );

                // Z axis (blue) - represented as diagonal
                painter.line_segment(
                    [center, center + egui::vec2(-28.0, 28.0)],
                    Stroke::new(3.0, Color32::BLUE),
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        center + egui::vec2(-28.0, 28.0),
                        egui::vec2(handle_size, handle_size),
                    ),
                    0.0,
                    Color32::BLUE,
                );
            }
        }
    }

    /// Handle camera navigation controls based on mouse input
    fn handle_camera_controls(&mut self, response: egui::Response, _rect: Rect) {
        // Handle mouse drag for camera rotation/movement
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = response.drag_delta();
            match self.camera_mode {
                CameraMode::Orbit => {
                    // Orbit camera around target
                    self.camera_rotation.y += (delta.x * self.mouse_sensitivity) as f64;
                    self.camera_rotation.x -= (delta.y * self.mouse_sensitivity) as f64;

                    // Clamp vertical rotation to prevent gimbal lock
                    self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                }
                CameraMode::Pan => {
                    // Pan the camera
                    // In a real implementation, this would take camera orientation into account
                    let pan_speed = 0.05;
                    self.camera_position.x -= delta.x as f64 * pan_speed;
                    self.camera_position.y += delta.y as f64 * pan_speed;
                }
                CameraMode::Flythrough => {
                    // Modify camera orientation
                    self.camera_rotation.y += (delta.x * self.mouse_sensitivity) as f64;
                    self.camera_rotation.x -= (delta.y * self.mouse_sensitivity) as f64;

                    // Clamp vertical rotation
                    self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
                }
            }
        }

        // Alt + Right mouse button for orbiting around a point
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            self.camera_rotation.y += (delta.x * self.mouse_sensitivity) as f64;
            self.camera_rotation.x -= (delta.y * self.mouse_sensitivity) as f64;

            // Clamp vertical rotation
            self.camera_rotation.x = self.camera_rotation.x.clamp(-89.0, 89.0);
        }

        // Forward/backward movement with scroll (simplified)
        let scroll_delta = response.ctx.input(|i| i.raw_scroll_delta);
        if scroll_delta.y != 0.0 {
            // Move camera forward/backward based on scroll
            // In a real 3D implementation, this would move along the view direction
            let zoom_step = scroll_delta.y * self.zoom_sensitivity;

            // If orthographic, adjust orthographic size, otherwise adjust position
            if self.is_orthographic {
                self.orthographic_size -= zoom_step;
                // Ensure the orthographic size doesn't go negative
                self.orthographic_size = self.orthographic_size.max(0.1);
            } else {
                self.camera_position.z -= zoom_step as f64;
            }
        }

        // Reset the drag state when mouse is released
        if !response.dragged() {
            self.is_dragging = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_view_default() {
        let scene_view = SceneView::default();
        assert_eq!(scene_view.camera_mode, CameraMode::Orbit);
        assert!(scene_view.show_grid);
        assert!(scene_view.show_gizmos);
    }

    #[test]
    fn test_gizmo_mode_setting() {
        let mut scene_view = SceneView::default();
        scene_view.set_gizmo_mode(GizmoMode::Rotate);
        assert_eq!(scene_view.gizmo_mode, GizmoMode::Rotate);
    }

    #[test]
    fn test_toggle_gizmo_space() {
        let mut scene_view = SceneView::default();
        let initial_state = scene_view.gizmo_space;
        scene_view.toggle_gizmo_space();
        assert_eq!(scene_view.gizmo_space, !initial_state);
    }

    #[test]
    fn test_set_orientation() {
        let mut scene_view = SceneView::default();
        scene_view.set_orientation(ViewOrientation::Top);
        assert_eq!(scene_view.current_orientation, ViewOrientation::Top);
        assert_eq!(scene_view.camera_rotation.x, -90.0);
    }
}
