// filepath: /home/deginandor/Documents/Programming/language/src/gui/game_view.rs
// use super::*;
use crate::ecs::World;
// use crate::physics::math::Vec3;
use eframe::egui;
use std::time::{Duration, Instant};

/// Display mode for the Game View
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayMode {
    Display1,
    Display2,
    Custom,
}

/// Game View panel for runtime preview of animation and physics
pub struct GameView {
    // Display settings
    pub display_mode: DisplayMode,
    pub device_simulator_enabled: bool,
    pub zoom_scale: f32,
    pub show_fps: bool,
    pub mute_audio: bool,

    // Playback controls
    pub is_playing: bool,
    pub is_paused: bool,

    // Performance tracking
    pub fps_counter: FPSCounter,
    pub frame_time_ms: f32,

    // Rendering info
    pub camera_entity: Option<usize>,
    pub max_view: bool,

    // Step frame control
    pub step_requested: bool,
    pub custom_time_scale: f32,

    // Viewport size
    pub viewport_rect: Option<egui::Rect>,

    // Current scene
    pub current_scene_name: String,
}

/// FPS Counter helper
pub struct FPSCounter {
    frames: Vec<Instant>,
    last_update: Instant,
    current_fps: f32,
}

impl FPSCounter {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(100),
            last_update: Instant::now(),
            current_fps: 0.0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        // Add current frame
        self.frames.push(now);

        // Remove old frames (older than 1 second)
        self.frames
            .retain(|&t| now.duration_since(t) < Duration::from_secs(1));

        // Calculate FPS
        if now.duration_since(self.last_update) >= Duration::from_millis(500) {
            self.current_fps = self.frames.len() as f32;
            self.last_update = now;
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.current_fps
    }
}

impl Default for GameView {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Display1,
            device_simulator_enabled: false,
            zoom_scale: 1.0,
            show_fps: true,
            mute_audio: false,
            is_playing: false,
            is_paused: false,
            fps_counter: FPSCounter::new(),
            frame_time_ms: 0.0,
            camera_entity: None,
            max_view: false,
            step_requested: false,
            custom_time_scale: 1.0,
            viewport_rect: None,
            current_scene_name: "Main Scene".to_string(),
        }
    }
}

impl GameView {
    /// Create a new Game View panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the Game View state
    pub fn update(&mut self, frame_time: f32) {
        if self.is_playing && !self.is_paused {
            self.frame_time_ms = frame_time;
            self.fps_counter.update();
        }
    }

    /// Toggle play mode
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        self.is_paused = false;
    }

    /// Toggle pause mode
    pub fn toggle_pause(&mut self) {
        if self.is_playing {
            self.is_paused = !self.is_paused;
        }
    }

    /// Request a single frame step
    pub fn step_frame(&mut self) {
        if self.is_playing && self.is_paused {
            self.step_requested = true;
        }
    }

    /// Set camera entity for game view
    pub fn set_camera_entity(&mut self, entity_id: usize) {
        self.camera_entity = Some(entity_id);
    }

    /// Toggle maximum view mode
    pub fn toggle_max_view(&mut self) {
        self.max_view = !self.max_view;
    }

    /// Draw Game View UI
    pub fn ui(&mut self, ui: &mut egui::Ui, _world: &mut World) {
        // Top toolbar
        ui.horizontal(|ui| {
            // Display selection
            ui.menu_button("Display", |ui| {
                if ui
                    .selectable_label(self.display_mode == DisplayMode::Display1, "Display 1")
                    .clicked()
                {
                    self.display_mode = DisplayMode::Display1;
                    ui.close_menu();
                }
                if ui
                    .selectable_label(self.display_mode == DisplayMode::Display2, "Display 2")
                    .clicked()
                {
                    self.display_mode = DisplayMode::Display2;
                    ui.close_menu();
                }
                if ui
                    .selectable_label(self.display_mode == DisplayMode::Custom, "Custom")
                    .clicked()
                {
                    self.display_mode = DisplayMode::Custom;
                    ui.close_menu();
                }
            });

            // Device simulator toggle
            ui.checkbox(&mut self.device_simulator_enabled, "Device Simulator");

            // Zoom scale slider
            ui.add(egui::Slider::new(&mut self.zoom_scale, 0.1..=5.0).text("Zoom"));

            // FPS toggle
            ui.checkbox(&mut self.show_fps, "Show Stats");

            // Audio mute toggle
            ui.checkbox(&mut self.mute_audio, "Mute");

            ui.separator();

            // Playback controls
            if ui.button(if self.is_playing { "■" } else { "▶" }).clicked() {
                self.toggle_play();
            }
            if ui.button(if self.is_paused { "▶" } else { "❚❚" }).clicked() {
                self.toggle_pause();
            }
            if ui.button("⏭").clicked() {
                self.step_frame();
            }

            // Time scale slider
            ui.add(egui::Slider::new(&mut self.custom_time_scale, 0.01..=10.0).text("Speed"));

            // Maximize button
            if ui.button(if self.max_view { "⊟" } else { "⊞" }).clicked() {
                self.toggle_max_view();
            }
        });

        // Game view area
        let (rect, _response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        // Store current viewport rect
        self.viewport_rect = Some(rect);

        // Render the game view
        let painter = ui.painter_at(rect);

        // Fill background
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 30));

        // Sample render - in a real implementation, this would be your game rendering
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            if self.is_playing {
                if self.is_paused {
                    "PAUSED"
                } else {
                    "PLAYING"
                }
            } else {
                "STOPPED"
            },
            egui::FontId::proportional(24.0),
            egui::Color32::WHITE,
        );

        // Draw the current scene name
        painter.text(
            rect.left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            &self.current_scene_name,
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );

        // Display FPS counter if enabled
        if self.show_fps {
            painter.text(
                rect.right_top() + egui::vec2(-10.0, 10.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.1} FPS", self.fps_counter.get_fps()),
                egui::FontId::proportional(14.0),
                egui::Color32::GREEN,
            );

            painter.text(
                rect.right_top() + egui::vec2(-10.0, 30.0),
                egui::Align2::RIGHT_TOP,
                format!("{:.2} ms", self.frame_time_ms),
                egui::FontId::proportional(12.0),
                egui::Color32::LIGHT_GREEN,
            );
        }

        // Device simulator border
        if self.device_simulator_enabled {
            let device_margin = egui::vec2(20.0, 40.0);
            let device_rect =
                egui::Rect::from_min_max(rect.min + device_margin, rect.max - device_margin);
            painter.rect_stroke(
                device_rect,
                8.0,
                egui::Stroke::new(2.0, egui::Color32::DARK_GRAY),
                egui::StrokeKind::Outside,
            );

            // Draw device notch
            let notch_width = 60.0;
            let notch_height = 20.0;
            let notch_rect = egui::Rect::from_center_size(
                egui::pos2(device_rect.center().x, device_rect.min.y),
                egui::vec2(notch_width, notch_height),
            );
            painter.rect_filled(notch_rect, 8.0, egui::Color32::DARK_GRAY);
        }
    }
}

// Unit tests for GameView
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_view_default() {
        let game_view = GameView::default();
        assert!(!game_view.is_playing);
        assert!(!game_view.is_paused);
        assert_eq!(game_view.zoom_scale, 1.0);
    }

    #[test]
    fn test_toggle_play() {
        let mut game_view = GameView::default();
        game_view.toggle_play();
        assert!(game_view.is_playing);
        assert!(!game_view.is_paused);

        game_view.toggle_play();
        assert!(!game_view.is_playing);
    }

    #[test]
    fn test_toggle_pause() {
        let mut game_view = GameView::default();

        // Should not pause when not playing
        game_view.toggle_pause();
        assert!(!game_view.is_paused);

        // Start playing then pause
        game_view.is_playing = true;
        game_view.toggle_pause();
        assert!(game_view.is_paused);

        // Unpause
        game_view.toggle_pause();
        assert!(!game_view.is_paused);
    }
}
