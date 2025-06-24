// use super::*;
use crate::ecs::World;
use crate::physics::math::Vec3;
use eframe::egui;
use std::collections::HashMap;

/// Animation clip data structure
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub length: f32,
    pub keyframes: HashMap<String, Vec<Keyframe>>,
    pub loop_mode: LoopMode,
    pub frame_rate: u32,
    pub events: Vec<AnimationEvent>,
}

/// Individual keyframe for animation properties
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: AnimationValue,
    pub interpolation: InterpolationType,
    pub in_tangent: f32,
    pub out_tangent: f32,
    pub is_broken_tangent: bool,
}

/// Animation custom event
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub time: f32,
    pub function_name: String,
    pub parameters: HashMap<String, String>,
}

/// Animation value types
#[derive(Debug, Clone)]
pub enum AnimationValue {
    Float(f32),
    Vec3(Vec3),
    Bool(bool),
    Color([f32; 4]),
    String(String),
    Enum(String, i32),
}

/// Animation interpolation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationType {
    Linear,
    Bezier,
    Step,
    Constant,
}

/// Animation loop modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
    ClampForever,
}

/// Animation view dope sheet modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationViewMode {
    Dopesheet,
    Curves,
    Both,
}

/// Animation View panel for creating and editing animations (Unity-style)
pub struct AnimationView {
    // Clip Management
    pub current_clip: Option<AnimationClip>,
    pub clips: Vec<AnimationClip>,
    pub new_clip_name: String,
    pub show_clip_create_dialog: bool,

    // Playback controls
    pub is_playing: bool,
    pub is_looping: bool,
    pub current_time: f32,
    pub current_frame: u32,
    pub play_speed: f32,

    // Timeline view controls
    pub timeline_zoom: f32,
    pub timeline_scroll: f32,
    pub view_mode: AnimationViewMode,
    pub auto_key: bool,

    // Selection state
    pub selected_keyframe: Option<(String, usize)>,
    pub selected_property: Option<String>,
    pub selected_track_expanded: bool,

    // Property filters
    pub show_position: bool,
    pub show_rotation: bool,
    pub show_scale: bool,
    pub show_visibility: bool,
    pub show_color: bool,
    pub show_rigidbody: bool,
    pub show_custom: bool,

    // Recording
    pub is_recording: bool,
    pub record_frame_interval: u32,

    // Preview settings
    pub show_onion_skin: bool,
    pub onion_skin_frames: u32,
    pub background_color: [f32; 4],

    // UI State
    pub track_heights: HashMap<String, f32>,
    pub show_event_markers: bool,
    pub show_property_add_menu: bool,
    pub show_curves_for_property: Option<String>,

    // Key editing
    pub begin_drag_frame: Option<u32>,
    pub dragging_keyframes: Vec<(String, usize)>,
    pub keyframe_selection: HashMap<(String, usize), bool>,
    pub copied_keyframes: Vec<(String, Keyframe)>,

    // Context menu
    pub show_context_menu: bool,
    pub context_menu_pos: egui::Pos2,
    pub context_menu_property: Option<String>,
    pub context_menu_frame: Option<u32>,

    // Search
    pub property_filter: String,

    // Animation preview
    pub preview_entity: Option<usize>,
}

impl Default for AnimationView {
    fn default() -> Self {
        Self {
            current_clip: None,
            clips: Vec::new(),
            new_clip_name: String::new(),
            show_clip_create_dialog: false,
            is_playing: false,
            is_looping: true,
            current_time: 0.0,
            current_frame: 0,
            play_speed: 1.0,
            timeline_zoom: 1.0,
            timeline_scroll: 0.0,
            view_mode: AnimationViewMode::Dopesheet,
            auto_key: false,
            selected_keyframe: None,
            selected_property: None,
            selected_track_expanded: false,
            show_position: true,
            show_rotation: true,
            show_scale: true,
            show_visibility: true,
            show_color: true,
            show_rigidbody: true,
            show_custom: true,
            is_recording: false,
            record_frame_interval: 1,
            show_onion_skin: false,
            onion_skin_frames: 3,
            background_color: [0.15, 0.15, 0.15, 1.0],
            track_heights: HashMap::new(),
            show_event_markers: true,
            show_property_add_menu: false,
            show_curves_for_property: None,
            begin_drag_frame: None,
            dragging_keyframes: Vec::new(),
            keyframe_selection: HashMap::new(),
            copied_keyframes: Vec::new(),
            show_context_menu: false,
            context_menu_pos: egui::Pos2::new(0.0, 0.0),
            context_menu_property: None,
            context_menu_frame: None,
            property_filter: String::new(),
            preview_entity: None,
        }
    }
}

impl AnimationView {
    /// Create a new Animation View panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current animation clip
    pub fn set_clip(&mut self, clip: AnimationClip) {
        self.current_clip = Some(clip);
        self.current_time = 0.0;
        self.current_frame = 0;
    }

    /// Reset the playback to the beginning
    pub fn reset_playback(&mut self) {
        self.current_time = 0.0;
        self.current_frame = 0;
    }

    /// Toggle play/pause
    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Toggle recording mode
    pub fn toggle_recording(&mut self) {
        self.is_recording = !self.is_recording;
    }

    /// Add a new keyframe
    pub fn add_keyframe(&mut self, property: &str, time: f32, value: AnimationValue) {
        if let Some(clip) = &mut self.current_clip {
            let keyframe = Keyframe {
                time,
                value,
                interpolation: InterpolationType::Linear,
                in_tangent: 0.0,
                out_tangent: 0.0,
                is_broken_tangent: false,
            };

            clip.keyframes
                .entry(property.to_string())
                .or_insert_with(Vec::new)
                .push(keyframe);
        }
    }

    /// Create a new animation clip
    pub fn create_new_clip(&mut self, name: &str, length: f32, frame_rate: u32) -> AnimationClip {
        AnimationClip {
            name: name.to_string(),
            length,
            keyframes: HashMap::new(),
            loop_mode: LoopMode::Loop,
            frame_rate,
            events: Vec::new(),
        }
    }

    /// Update animation timing
    pub fn update(&mut self, delta_time: f32) {
        if self.is_playing {
            if let Some(clip) = &self.current_clip {
                self.current_time += delta_time * self.play_speed;

                // Handle looping
                if self.is_looping && self.current_time > clip.length {
                    self.current_time %= clip.length;
                } else if !self.is_looping && self.current_time > clip.length {
                    self.current_time = clip.length;
                    self.is_playing = false;
                }

                // Update current frame
                self.current_frame = (self.current_time * clip.frame_rate as f32) as u32;
            }
        }
    }

    /// Draw the animation view UI
    pub fn ui(&mut self, ui: &mut egui::Ui, _world: &mut World) {
        // Top controls - Animation clip selector and playback controls
        self.draw_top_controls(ui);

        ui.separator();

        // Main content area with timeline and property tracks
        egui::SidePanel::left("animation_properties")
            .resizable(true)
            .min_width(150.0)
            .default_width(200.0)
            .show_inside(ui, |ui| {
                self.draw_property_list(ui);
            });

        ui.vertical(|ui| {
            // Timeline ruler
            self.draw_timeline_ruler(ui);

            // Key area
            self.draw_keyframe_area(ui);

            ui.separator();

            // Bottom controls - Frame counter and navigation
            self.draw_bottom_controls(ui);
        });
    }

    /// Show the animation view panel in a UI area
    pub fn show_ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        self.ui(ui, world);
    }

    /// Draw the top controls (clip selector and playback)
    fn draw_top_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Clip selector
            ui.label("Clip:");
            let current_clip_name = match &self.current_clip {
                Some(clip) => &clip.name,
                None => "None",
            };

            egui::ComboBox::from_id_salt("animation_clip_selector")
                .selected_text(current_clip_name)
                .show_ui(ui, |ui| {
                    // Collect clipnames and indices first to avoid the self borrow issue
                    let clip_data: Vec<(usize, String, bool)> = self.clips.iter().enumerate()
                        .map(|(idx, clip)| {
                            let is_selected = self.current_clip
                                .as_ref()
                                .map_or(false, |c| c.name == clip.name);
                            (idx, clip.name.clone(), is_selected)
                        })
                        .collect();

                    // Now use the collected data without borrowing self
                    for (idx, name, is_selected) in clip_data {
                        if ui.selectable_label(is_selected, &name).clicked() {
                            // Get the clip directly from the index
                            let clip_to_set = self.clips[idx].clone();
                            self.set_clip(clip_to_set);
                        }
                    }
                });

            // Create new clip button
            if ui.button("+").clicked() {
                self.show_clip_create_dialog = true;
            }

            ui.separator();

            // Playback controls
            if ui.button("⏮").clicked() {
                self.reset_playback();
            }

            if ui.button(if self.is_playing { "⏸" } else { "▶" }).clicked() {
                self.toggle_play();
            }

            if ui.button("⏭").clicked() {
                // Skip to next keyframe or end
            }

            // Loop toggle
            ui.toggle_value(&mut self.is_looping, "🔄");

            ui.separator();

            // Recording controls
            let recording_button = ui.toggle_value(&mut self.is_recording, "🔴");
            if recording_button.clicked() {
                self.toggle_recording();
            }

            // Auto-keyframing toggle
            ui.toggle_value(&mut self.auto_key, "Auto Key");

            ui.separator();

            // View mode
            ui.selectable_value(
                &mut self.view_mode,
                AnimationViewMode::Dopesheet,
                "Dopesheet",
            );
            ui.selectable_value(&mut self.view_mode, AnimationViewMode::Curves, "Curves");
        });

        // Dialog for creating a new clip
        if self.show_clip_create_dialog {
            egui::Window::new("Create Animation Clip")
                .fixed_size([300.0, 150.0])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_clip_name);
                    });

                    let mut length = 5.0;
                    ui.horizontal(|ui| {
                        ui.label("Length (seconds):");
                        ui.add(egui::Slider::new(&mut length, 0.1..=60.0));
                    });

                    let mut frame_rate = 30;
                    ui.horizontal(|ui| {
                        ui.label("Frame Rate:");
                        ui.add(egui::Slider::new(&mut frame_rate, 1..=120));
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_clip_create_dialog = false;
                        }

                        if ui.button("Create").clicked() && !self.new_clip_name.is_empty() {
                            // Store a clone of the name to avoid borrowing self in the closure
                            let clip_name = self.new_clip_name.clone();

                            // Create clip directly without calling self method
                            let new_clip = AnimationClip {
                                name: clip_name,
                                length,
                                frame_rate: frame_rate as u32,
                                keyframes: std::collections::HashMap::new(),
                                loop_mode: LoopMode::Loop,
                                events: Vec::new(),
                            };

                            self.clips.push(new_clip.clone());
                            self.set_clip(new_clip);
                            self.new_clip_name.clear();
                            self.show_clip_create_dialog = false;
                        }
                    });
                });
        }
    }

    /// Draw the property list (left panel)
    fn draw_property_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Properties");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+").clicked() {
                    self.show_property_add_menu = true;
                }

                if ui.text_edit_singleline(&mut self.property_filter).changed() {
                    // Filter properties
                }
            });
        });

        ui.separator();

        // Display property groups
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.show_position {
                self.draw_property_group(
                    ui,
                    "Position",
                    &["position.x", "position.y", "position.z"],
                );
            }

            if self.show_rotation {
                self.draw_property_group(
                    ui,
                    "Rotation",
                    &["rotation.x", "rotation.y", "rotation.z"],
                );
            }

            if self.show_scale {
                self.draw_property_group(ui, "Scale", &["scale.x", "scale.y", "scale.z"]);
            }

            if self.show_visibility {
                self.draw_property_group(ui, "Visibility", &["enabled"]);
            }

            if self.show_rigidbody {
                self.draw_property_group(
                    ui,
                    "Rigidbody",
                    &["rigidbody.isKinematic", "rigidbody.mass"],
                );
            }

            // Show all other properties in the current clip
            if let Some(clip) = &self.current_clip {
                for (property, _keyframes) in &clip.keyframes {
                    // Skip properties already shown in groups
                    if property.starts_with("position.")
                        || property.starts_with("rotation.")
                        || property.starts_with("scale.")
                        || property == "enabled"
                        || property.starts_with("rigidbody.")
                    {
                        continue;
                    }

                    // Draw standalone property
                    let is_selected = self
                        .selected_property
                        .as_ref()
                        .map_or(false, |p| p == property);
                    if ui.selectable_label(is_selected, property).clicked() {
                        self.selected_property = Some(property.to_string());
                    }
                }
            }
        });

        // Property add menu
        if self.show_property_add_menu {
            egui::Window::new("Add Property")
                .fixed_size([250.0, 300.0])
                .show(ui.ctx(), |ui| {
                    ui.heading("Select Property");

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Transform properties
                        ui.collapsing("Transform", |ui| {
                            if ui.selectable_label(false, "Position").clicked() {
                                // Add position tracks
                                self.show_property_add_menu = false;
                            }
                            if ui.selectable_label(false, "Rotation").clicked() {
                                // Add rotation tracks
                                self.show_property_add_menu = false;
                            }
                            if ui.selectable_label(false, "Scale").clicked() {
                                // Add scale tracks
                                self.show_property_add_menu = false;
                            }
                        });

                        // Rigidbody properties
                        ui.collapsing("Rigidbody", |ui| {
                            if ui.selectable_label(false, "Is Kinematic").clicked() {
                                // Add isKinematic track
                                self.show_property_add_menu = false;
                            }
                            if ui.selectable_label(false, "Mass").clicked() {
                                // Add mass track
                                self.show_property_add_menu = false;
                            }
                            if ui.selectable_label(false, "Use Gravity").clicked() {
                                // Add useGravity track
                                self.show_property_add_menu = false;
                            }
                        });

                        // Material properties
                        ui.collapsing("Material", |ui| {
                            if ui.selectable_label(false, "Color").clicked() {
                                // Add color track
                                self.show_property_add_menu = false;
                            }
                            if ui.selectable_label(false, "Emission").clicked() {
                                // Add emission track
                                self.show_property_add_menu = false;
                            }
                        });
                    });

                    ui.separator();

                    if ui.button("Cancel").clicked() {
                        self.show_property_add_menu = false;
                    }
                });
        }
    }

    /// Draw a group of related properties
    fn draw_property_group(&mut self, ui: &mut egui::Ui, group_name: &str, properties: &[&str]) {
        ui.collapsing(group_name, |ui| {
            for &property in properties {
                let is_selected = self
                    .selected_property
                    .as_ref()
                    .map_or(false, |p| p == property);
                if ui.selectable_label(is_selected, property).clicked() {
                    self.selected_property = Some(property.to_string());
                }
            }
        });
    }

    /// Draw the timeline ruler
    fn draw_timeline_ruler(&mut self, ui: &mut egui::Ui) {
        let clip_length = match &self.current_clip {
            Some(clip) => clip.length,
            None => 5.0, // Default length if no clip
        };

        let frame_rate = match &self.current_clip {
            Some(clip) => clip.frame_rate as f32,
            None => 30.0, // Default frame rate
        };

        let total_frames = (clip_length * frame_rate) as u32;
        let frame_width = 10.0 * self.timeline_zoom;

        let ruler_rect = ui.available_rect_before_wrap();
        let ruler_height = 20.0;
        let _ruler_width = (total_frames as f32) * frame_width;

        // Calculate visible range based on scroll position
        let visible_start = (self.timeline_scroll / frame_width) as u32;
        let visible_end = visible_start + (ruler_rect.width() / frame_width) as u32 + 1;
        let visible_end = visible_end.min(total_frames);

        // Allocate space for ruler
        let (ruler_rect, _) = ui.allocate_exact_size(
            egui::vec2(ruler_rect.width(), ruler_height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(ruler_rect);

        // Draw ruler background
        painter.rect_filled(ruler_rect, 0.0, egui::Color32::from_rgb(40, 40, 40));

        // Draw frame markers
        for frame in visible_start..=visible_end {
            let x = ruler_rect.left() + (frame as f32 * frame_width) - self.timeline_scroll;

            // Determine marker height
            let height = if frame % 10 == 0 {
                ruler_height * 0.5 // Major marker
            } else if frame % 5 == 0 {
                ruler_height * 0.3 // Medium marker
            } else {
                ruler_height * 0.2 // Minor marker
            };

            // Draw frame marker line
            painter.line_segment(
                [
                    egui::pos2(x, ruler_rect.bottom() - height),
                    egui::pos2(x, ruler_rect.bottom()),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 180)),
            );

            // Draw frame numbers for major markers
            if frame % 10 == 0 {
                painter.text(
                    egui::pos2(x, ruler_rect.top() + 2.0),
                    egui::Align2::CENTER_TOP,
                    format!("{}", frame),
                    egui::FontId::proportional(10.0),
                    egui::Color32::from_rgb(200, 200, 200),
                );
            }
        }

        // Draw current time marker
        let current_pos = ruler_rect.left() + ((self.current_time * frame_rate) * frame_width)
            - self.timeline_scroll;

        painter.line_segment(
            [
                egui::pos2(current_pos, ruler_rect.top()),
                egui::pos2(current_pos, ruler_rect.bottom()),
            ],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
        );
    }

    /// Draw the keyframe area (main timeline)
    fn draw_keyframe_area(&mut self, ui: &mut egui::Ui) {
        if self.current_clip.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No animation clip selected");
            });
            return;
        }

        let clip = self.current_clip.as_ref().unwrap();
        let frame_rate = clip.frame_rate as f32;
        let total_frames = (clip.length * frame_rate) as u32;
        let frame_width = 10.0 * self.timeline_zoom;

        let available_rect = ui.available_rect_before_wrap();
        let track_height = 18.0;

        // Space needed for all tracks
        let track_count = clip.keyframes.len().max(1);
        let _content_height = track_count as f32 * track_height;

        // Clone all data we need to avoid borrowing issues
        let clip_keyframes: Vec<(String, Vec<_>)> = clip.keyframes.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Create a scrollable area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut visible_properties = Vec::new();

                // Collect visible property names from clip keyframes
                for (property, _) in &clip_keyframes {
                    // Filter based on categories we're showing
                    let include = match property.as_str() {
                        p if p.starts_with("position.") => self.show_position,
                        p if p.starts_with("rotation.") => self.show_rotation,
                        p if p.starts_with("scale.") => self.show_scale,
                        "enabled" => self.show_visibility,
                        p if p.starts_with("rigidbody.") => self.show_rigidbody,
                        p if p.starts_with("material.color") => self.show_color,
                        _ => self.show_custom,
                    };

                    if include {
                        visible_properties.push(property);
                    }
                }

                // If we have no properties yet, show message
                if visible_properties.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No properties to display. Add a property to begin animating.");
                    });
                    return;
                }

                // Calculate visible range based on scroll position
                let visible_start = (self.timeline_scroll / frame_width) as u32;
                let visible_end = visible_start + (available_rect.width() / frame_width) as u32 + 1;
                let visible_end = visible_end.min(total_frames);

                // For each property, draw a track with its keyframes
                for property in &visible_properties {
                    if let Some((_, keyframes)) = clip_keyframes.iter().find(|(k, _)| k.as_str() == *property) {
                        // Allocate space for this track
                        let (track_rect, track_response) = ui.allocate_exact_size(
                            egui::vec2(available_rect.width(), track_height),
                            egui::Sense::click_and_drag(),
                        );

                        let painter = ui.painter_at(track_rect);

                        // Draw track background
                        let is_selected = self
                            .selected_property
                            .as_ref()
                            .map_or(false, |p| p.as_str() == *property);
                        let bg_color = if is_selected {
                            egui::Color32::from_rgb(60, 60, 80)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        };

                        painter.rect_filled(track_rect, 0.0, bg_color);

                        // Draw keyframes
                        for (idx, keyframe) in keyframes.iter().enumerate() {
                            let frame = (keyframe.time * frame_rate) as u32;
                            if frame >= visible_start && frame <= visible_end {
                                let x = track_rect.left() + (frame as f32 * frame_width)
                                    - self.timeline_scroll;
                                let y = track_rect.center().y;
                                let kf_size = 7.0;

                                // Is this keyframe selected?
                                let is_kf_selected = self
                                    .selected_keyframe
                                    .as_ref()
                                    .map_or(false, |&(ref p, i)| p.as_str() == *property && i == idx);

                                // Draw keyframe diamond
                                let keyframe_color = match keyframe.value {
                                    AnimationValue::Float(_) => {
                                        egui::Color32::from_rgb(150, 150, 255)
                                    }
                                    AnimationValue::Vec3(_) => {
                                        egui::Color32::from_rgb(255, 150, 150)
                                    }
                                    AnimationValue::Bool(_) => {
                                        egui::Color32::from_rgb(150, 255, 150)
                                    }
                                    AnimationValue::Color(_) => {
                                        egui::Color32::from_rgb(255, 255, 150)
                                    }
                                    AnimationValue::String(_) => {
                                        egui::Color32::from_rgb(200, 200, 200)
                                    }
                                    AnimationValue::Enum(_, _) => {
                                        egui::Color32::from_rgb(200, 150, 255)
                                    }
                                };

                                // Key points (diamond shape)
                                let points = [
                                    egui::pos2(x, y - kf_size),
                                    egui::pos2(x + kf_size, y),
                                    egui::pos2(x, y + kf_size),
                                    egui::pos2(x - kf_size, y),
                                ];

                                painter.add(egui::Shape::convex_polygon(
                                    points.to_vec(),
                                    keyframe_color,
                                    egui::Stroke::new(
                                        if is_kf_selected { 2.0 } else { 1.0 },
                                        if is_kf_selected {
                                            egui::Color32::WHITE
                                        } else {
                                            egui::Color32::BLACK
                                        },
                                    ),
                                ));

                                // Check for clicks on this keyframe
                                let kf_rect = egui::Rect::from_center_size(
                                    egui::pos2(x, y),
                                    egui::vec2(kf_size * 2.0, kf_size * 2.0),
                                );

                                if track_response.clicked()
                                    && track_response
                                        .hover_pos()
                                        .map_or(false, |pos| kf_rect.contains(pos))
                                {
                                    self.selected_keyframe = Some((property.to_string(), idx));
                                    self.selected_property = Some(property.to_string());
                                }
                            }
                        }

                        // Handle right-click for context menu
                        if track_response.clicked_by(egui::PointerButton::Secondary) {
                            if let Some(pos) = track_response.hover_pos() {
                                self.show_context_menu = true;
                                self.context_menu_pos = pos;
                                self.context_menu_property = Some(property.to_string());

                                // Calculate frame number at click position
                                let offset_x = pos.x - track_rect.left() + self.timeline_scroll;
                                let frame = (offset_x / frame_width) as u32;
                                self.context_menu_frame = Some(frame);
                            }
                        }
                    }
                }

                // Handle context menu
                if self.show_context_menu {
                    // Extract values before the closure to avoid borrowing conflicts
                    let context_property = self.context_menu_property.clone();
                    let context_frame = self.context_menu_frame;
                    let selected_keyframe = self.selected_keyframe.clone();
                    let copied_keyframes = self.copied_keyframes.clone();

                    egui::Window::new("Timeline Context Menu")
                        .fixed_size([180.0, 100.0])
                        .pivot(egui::Align2::CENTER_TOP)
                        .fixed_pos(self.context_menu_pos)
                        .title_bar(false)
                        .show(ui.ctx(), |ui| {
                            if ui.button("Add Key").clicked() {
                                if let (Some(property), Some(frame)) = (&context_property, context_frame) {
                                    let time = frame as f32 / frame_rate;

                                    // Add a keyframe with a default value based on type
                                    self.add_keyframe(
                                        property,
                                        time,
                                        AnimationValue::Float(0.0), // Default value
                                    );
                                }
                                self.show_context_menu = false;
                            }

                            if selected_keyframe.is_some() {
                                if ui.button("Delete Key").clicked() {
                                    if let Some((property, idx)) = &selected_keyframe {
                                        if let Some(clip) = &mut self.current_clip {
                                            if let Some(keyframes) =
                                                clip.keyframes.get_mut(property)
                                            {
                                                if *idx < keyframes.len() {
                                                    keyframes.remove(*idx);
                                                }
                                            }
                                        }
                                    }
                                    self.selected_keyframe = None;
                                    self.show_context_menu = false;
                                }

                                if ui.button("Copy").clicked() {
                                    if let Some((property, idx)) = &selected_keyframe {
                                        if let Some(clip) = &self.current_clip {
                                            if let Some(keyframes) = clip.keyframes.get(property.as_str()) {
                                                if *idx < keyframes.len() {
                                                    self.copied_keyframes = vec![(
                                                        property.to_string(),
                                                        keyframes[*idx].clone(),
                                                    )];
                                                }
                                            }
                                        }
                                    }
                                    self.show_context_menu = false;
                                }
                            }

                            if !copied_keyframes.is_empty() {
                                if ui.button("Paste").clicked() {
                                    if let Some(frame) = context_frame {
                                        let time = frame as f32 / frame_rate;

                                        // Paste keyframes at the new time
                                        if let Some(clip) = &mut self.current_clip {
                                            for (property, keyframe) in &copied_keyframes {
                                                let mut new_keyframe = keyframe.clone();
                                                new_keyframe.time = time;

                                                clip.keyframes
                                                    .entry(property.to_string())
                                                    .or_insert_with(Vec::new)
                                                    .push(new_keyframe);
                                            }
                                        }
                                    }
                                    self.show_context_menu = false;
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_context_menu = false;
                            }
                        });
                }
            });
    }

    /// Draw the bottom controls (frame counter and navigation)
    fn draw_bottom_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Frame counter
            if let Some(clip) = &self.current_clip {
                let frame_count = (clip.length * clip.frame_rate as f32) as u32;

                ui.label(format!(
                    "Frame: {}/{}  Time: {:.2}/{:.2}s",
                    self.current_frame, frame_count, self.current_time, clip.length
                ));
            } else {
                ui.label("No clip loaded");
            }

            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            let zoom_response =
                ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.1..=5.0).logarithmic(true));

            if zoom_response.changed() {
                // Adjust scroll position to maintain center when zooming
            }

            // Reset zoom
            if ui.button("1:1").clicked() {
                self.timeline_zoom = 1.0;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_view_default() {
        let anim_view = AnimationView::default();
        assert!(!anim_view.is_playing);
        assert!(anim_view.is_looping);
        assert_eq!(anim_view.timeline_zoom, 1.0);
    }

    #[test]
    fn test_create_clip() {
        let mut anim_view = AnimationView::default();
        let clip = anim_view.create_new_clip("TestClip", 5.0, 30);

        assert_eq!(clip.name, "TestClip");
        assert_eq!(clip.length, 5.0);
        assert_eq!(clip.frame_rate, 30);
        assert!(clip.keyframes.is_empty());
    }

    #[test]
    fn test_add_keyframe() {
        let mut anim_view = AnimationView::default();
        let clip = anim_view.create_new_clip("TestClip", 5.0, 30);
        anim_view.set_clip(clip);

        anim_view.add_keyframe("position.x", 0.5, AnimationValue::Float(10.0));

        if let Some(clip) = &anim_view.current_clip {
            if let Some(keyframes) = clip.keyframes.get("position.x") {
                assert_eq!(keyframes.len(), 1);
                assert_eq!(keyframes[0].time, 0.5);
                match &keyframes[0].value {
                    AnimationValue::Float(v) => assert_eq!(*v, 10.0),
                    _ => panic!("Expected Float animation value"),
                }
            } else {
                panic!("Expected keyframes for 'position.x'");
            }
        } else {
            panic!("Expected current_clip to be Some");
        }
    }
}
