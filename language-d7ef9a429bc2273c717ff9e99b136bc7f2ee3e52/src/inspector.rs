use super::{Component, GameObject, Scene};
use crate::ecs::World;
use crate::physics::math::Vec3;
use crate::physics::rigid_body::Shape;
use eframe::egui;
use std::collections::HashMap;

/// Inspector panel similar to Unity's Inspector window
pub struct Inspector {
    // Component editing state
    editing_component: Option<usize>,
    temp_values: HashMap<String, String>,

    // Tags system
    available_tags: Vec<String>,
    new_tag_name: String,
    show_script_dialog: bool,
    show_script_editor: bool,

    // Component addition
    show_add_component: bool,
    component_search: String,

    // Collapsible sections
    transform_expanded: bool,
    rigidbody_expanded: bool,
    collider_expanded: bool,

    // Drag handles
    drag_start_position: Option<egui::Pos2>,
    drag_start_value: Option<f64>,
    dragging_field: Option<String>,
}

/// Represents different types of physics colliders
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderType {
    Box,
    Sphere,
    Capsule,
    Mesh,
    Convex,
}

impl Default for Inspector {
    fn default() -> Self {
        Self {
            editing_component: None,
            temp_values: HashMap::new(),
            available_tags: vec![
                "Untagged".to_string(),
                "Player".to_string(),
                "Enemy".to_string(),
                "Environment".to_string(),
                "UI".to_string(),
                "Pickup".to_string(),
            ],
            new_tag_name: String::new(),
            show_script_dialog: false,
            show_script_editor: false,
            show_add_component: false,
            component_search: String::new(),
            transform_expanded: true,
            rigidbody_expanded: true,
            collider_expanded: true,
            drag_start_position: None,
            drag_start_value: None,
            dragging_field: None,
        }
    }
}

impl Inspector {
    /// Create a new Inspector panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the inspector panel UI
    pub fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<usize>) {
        ui.heading("Inspector");

        if let Some(entity_id) = selected_entity {
            ui.horizontal(|ui| {
                ui.checkbox(&mut true, ""); // Entity enabled state
                ui.text_edit_singleline(&mut format!("Entity {}", entity_id));

                // The real implementation would allow renaming entities
            });

            ui.separator();

            // Show transform component
            self.show_transform_component(ui, entity_id, world);

            // Show rigidbody component if it exists
            self.show_rigidbody_component(ui, entity_id, world);

            // Show collider component if it exists
            self.show_collider_component(ui, entity_id, world);

            // Other components would go here...

            // Add Component button
            if ui.button("Add Component").clicked() {
                self.show_add_component = true;
            }

            // Add Component Dialog
            if self.show_add_component {
                self.show_add_component_dialog(ui, entity_id, world);
            }
        } else {
            ui.label("No entity selected");
        }
    }

    /// Show the inspector in a panel
    pub fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        selected_entity: Option<usize>,
    ) -> bool {
        self.ui(ui, world, selected_entity);
        false // Return true if any changes were made
    }

    /// Show the transform component UI
    fn show_transform_component(
        &mut self,
        ui: &mut egui::Ui,
        _entity_id: usize,
        _world: &mut World,
    ) {
        let transform_header = ui.collapsing("Transform", |_| {}).header_response;

        if transform_header.clicked() {
            self.transform_expanded = !self.transform_expanded;
        }

        if self.transform_expanded {
            egui::CollapsingHeader::new("Transform")
                .default_open(true)
                .show(ui, |ui| {
                    // NOTE: This is a simplified implementation
                    // In a real app, you'd get the actual transform component from the entity

                    let mut position = Vec3::new(0.0, 0.0, 0.0);
                    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
                    let mut scale = Vec3::new(1.0, 1.0, 1.0);

                    // Position
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        ui.add(
                            egui::DragValue::new(&mut position.x)
                                .speed(0.1)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut position.y)
                                .speed(0.1)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut position.z)
                                .speed(0.1)
                                .prefix("Z: "),
                        );
                    });

                    // Rotation (euler angles)
                    ui.horizontal(|ui| {
                        ui.label("Rotation");
                        ui.add(
                            egui::DragValue::new(&mut rotation.x)
                                .speed(1.0)
                                .prefix("X: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut rotation.y)
                                .speed(1.0)
                                .prefix("Y: "),
                        );
                        ui.add(
                            egui::DragValue::new(&mut rotation.z)
                                .speed(1.0)
                                .prefix("Z: "),
                        );
                    });

                    // Scale
                    ui.horizontal(|ui| {
                        ui.label("Scale   ");
                        ui.add(egui::DragValue::new(&mut scale.x).speed(0.1).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut scale.y).speed(0.1).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut scale.z).speed(0.1).prefix("Z: "));
                    });

                    // In a real implementation, you'd update the entity's transform component here
                });
        }
    }

    /// Show the rigidbody component UI (similar to Unity's Rigidbody inspector)
    fn show_rigidbody_component(
        &mut self,
        ui: &mut egui::Ui,
        _entity_id: usize,
        _world: &mut World,
    ) {
        // In a real implementation, you'd check if the entity has a rigidbody component
        let has_rigidbody = true; // Placeholder

        if has_rigidbody {
            let rb_header = ui.collapsing("Rigidbody", |_| {}).header_response;

            if rb_header.clicked() {
                self.rigidbody_expanded = !self.rigidbody_expanded;
            }

            if self.rigidbody_expanded {
                egui::CollapsingHeader::new("Rigidbody")
                    .default_open(true)
                    .show(ui, |ui| {
                        // NOTE: This is a simplified implementation
                        // In a real app, you'd get the actual rigidbody from the entity

                        let mut mass = 1.0;
                        let mut drag = 0.0;
                        let mut angular_drag = 0.05;
                        let mut use_gravity = true;
                        let mut is_kinematic = false;
                        let mut collision_detection = 0; // 0 = Discrete, 1 = Continuous, 2 = ContinuousDynamic

                        // Freeze position/rotation
                        let mut freeze_pos_x = false;
                        let mut freeze_pos_y = false;
                        let mut freeze_pos_z = false;
                        let mut freeze_rot_x = false;
                        let mut freeze_rot_y = false;
                        let mut freeze_rot_z = false;

                        // Mass
                        ui.horizontal(|ui| {
                            ui.label("Mass");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut mass)
                                        .speed(0.1)
                                        .range(0.001..=10000.0),
                                )
                                .changed()
                            {
                                // Update rigidbody mass here
                            }
                        });

                        // Drag
                        ui.horizontal(|ui| {
                            ui.label("Drag");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut drag)
                                        .speed(0.01)
                                        .range(0.0..=10.0),
                                )
                                .changed()
                            {
                                // Update drag here
                            }
                        });

                        // Angular Drag
                        ui.horizontal(|ui| {
                            ui.label("Angular Drag");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut angular_drag)
                                        .speed(0.01)
                                        .range(0.0..=10.0),
                                )
                                .changed()
                            {
                                // Update angular drag here
                            }
                        });

                        // Use Gravity
                        if ui.checkbox(&mut use_gravity, "Use Gravity").changed() {
                            // Update gravity usage here
                        }

                        // Is Kinematic
                        if ui.checkbox(&mut is_kinematic, "Is Kinematic").changed() {
                            // Update kinematic state here
                        }

                        // Collision Detection dropdown
                        ui.horizontal(|ui| {
                            ui.label("Collision Detection");
                            egui::ComboBox::from_label("")
                                .selected_text(match collision_detection {
                                    0 => "Discrete",
                                    1 => "Continuous",
                                    2 => "Continuous Dynamic",
                                    _ => "Discrete",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut collision_detection, 0, "Discrete");
                                    ui.selectable_value(&mut collision_detection, 1, "Continuous");
                                    ui.selectable_value(
                                        &mut collision_detection,
                                        2,
                                        "Continuous Dynamic",
                                    );
                                });
                        });

                        // Constraints section
                        ui.collapsing("Constraints", |ui| {
                            ui.label("Freeze Position");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut freeze_pos_x, "X");
                                ui.checkbox(&mut freeze_pos_y, "Y");
                                ui.checkbox(&mut freeze_pos_z, "Z");
                            });

                            ui.label("Freeze Rotation");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut freeze_rot_x, "X");
                                ui.checkbox(&mut freeze_rot_y, "Y");
                                ui.checkbox(&mut freeze_rot_z, "Z");
                            });
                        });
                    });
            }
        }
    }

    /// Show the collider component UI (similar to Unity's Collider inspector)
    fn show_collider_component(
        &mut self,
        ui: &mut egui::Ui,
        _entity_id: usize,
        _world: &mut World,
    ) {
        // In a real implementation, you'd check if the entity has a collider component
        let has_collider = true; // Placeholder
        let collider_type = ColliderType::Box; // Placeholder

        if has_collider {
            let collider_name = match collider_type {
                ColliderType::Box => "Box Collider",
                ColliderType::Sphere => "Sphere Collider",
                ColliderType::Capsule => "Capsule Collider",
                ColliderType::Mesh => "Mesh Collider",
                ColliderType::Convex => "Convex Mesh Collider",
            };

            let header = ui.collapsing(collider_name, |_| {}).header_response;

            if header.clicked() {
                self.collider_expanded = !self.collider_expanded;
            }

            if self.collider_expanded {
                egui::CollapsingHeader::new(collider_name)
                    .default_open(true)
                    .show(ui, |ui| {
                        // NOTE: This is a simplified implementation
                        // In a real app, you'd get the actual collider from the entity

                        let mut is_trigger = false;
                        let mut material_idx = 0;
                        let material_names = ["Default", "Bouncy", "Ice", "Metal", "Wood"];

                        // Is Trigger
                        if ui.checkbox(&mut is_trigger, "Is Trigger").changed() {
                            // Update trigger state here
                        }

                        // Material dropdown (Physics Material)
                        ui.horizontal(|ui| {
                            ui.label("Material");
                            egui::ComboBox::from_label("")
                                .selected_text(material_names[material_idx])
                                .show_ui(ui, |ui| {
                                    for (idx, name) in material_names.iter().enumerate() {
                                        ui.selectable_value(&mut material_idx, idx, *name);
                                    }
                                });
                        });

                        // Collider-specific parameters
                        match collider_type {
                            ColliderType::Box => {
                                // Box collider parameters
                                let mut size = Vec3::new(1.0, 1.0, 1.0);
                                let mut center = Vec3::new(0.0, 0.0, 0.0);

                                ui.horizontal(|ui| {
                                    ui.label("Center");
                                    ui.add(
                                        egui::DragValue::new(&mut center.x)
                                            .speed(0.01)
                                            .prefix("X: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.y)
                                            .speed(0.01)
                                            .prefix("Y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.z)
                                            .speed(0.01)
                                            .prefix("Z: "),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Size  ");
                                    ui.add(
                                        egui::DragValue::new(&mut size.x).speed(0.01).prefix("X: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut size.y).speed(0.01).prefix("Y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut size.z).speed(0.01).prefix("Z: "),
                                    );
                                });
                            }
                            ColliderType::Sphere => {
                                // Sphere collider parameters
                                let mut radius = 0.5;
                                let mut center = Vec3::new(0.0, 0.0, 0.0);

                                ui.horizontal(|ui| {
                                    ui.label("Center");
                                    ui.add(
                                        egui::DragValue::new(&mut center.x)
                                            .speed(0.01)
                                            .prefix("X: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.y)
                                            .speed(0.01)
                                            .prefix("Y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.z)
                                            .speed(0.01)
                                            .prefix("Z: "),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Radius");
                                    ui.add(egui::DragValue::new(&mut radius).speed(0.01));
                                });
                            }
                            ColliderType::Capsule => {
                                // Capsule collider parameters
                                let mut radius = 0.5;
                                let mut height = 2.0;
                                let mut center = Vec3::new(0.0, 0.0, 0.0);
                                let mut direction = 0; // 0 = Y, 1 = X, 2 = Z

                                ui.horizontal(|ui| {
                                    ui.label("Center");
                                    ui.add(
                                        egui::DragValue::new(&mut center.x)
                                            .speed(0.01)
                                            .prefix("X: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.y)
                                            .speed(0.01)
                                            .prefix("Y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut center.z)
                                            .speed(0.01)
                                            .prefix("Z: "),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Radius");
                                    ui.add(egui::DragValue::new(&mut radius).speed(0.01));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Height");
                                    ui.add(egui::DragValue::new(&mut height).speed(0.01));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Direction");
                                    egui::ComboBox::from_label("")
                                        .selected_text(match direction {
                                            0 => "Y-Axis",
                                            1 => "X-Axis",
                                            2 => "Z-Axis",
                                            _ => "Y-Axis",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut direction, 0, "Y-Axis");
                                            ui.selectable_value(&mut direction, 1, "X-Axis");
                                            ui.selectable_value(&mut direction, 2, "Z-Axis");
                                        });
                                });
                            }
                            ColliderType::Mesh => {
                                // Mesh collider parameters
                                let mut convex = false;
                                let mut mesh_path = "Default Mesh".to_string();

                                if ui.checkbox(&mut convex, "Convex").changed() {
                                    // Update convex state here
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Mesh");
                                    ui.text_edit_singleline(&mut mesh_path);
                                    if ui.button("Browse...").clicked() {
                                        // Open mesh selection dialog
                                    }
                                });
                            }
                            ColliderType::Convex => {
                                // Convex mesh collider parameters
                                let mut mesh_path = "Default Convex Mesh".to_string();

                                ui.horizontal(|ui| {
                                    ui.label("Mesh");
                                    ui.text_edit_singleline(&mut mesh_path);
                                    if ui.button("Browse...").clicked() {
                                        // Open mesh selection dialog
                                    }
                                });
                            }
                        }
                    });
            }
        }
    }

    /// Show the dialog for adding a component
    fn show_add_component_dialog(
        &mut self,
        ui: &mut egui::Ui,
        _entity_id: usize,
        _world: &mut World,
    ) {
        egui::Window::new("Add Component")
            .fixed_size([300.0, 400.0])
            .show(ui.ctx(), |ui| {
                // Search field
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.component_search);
                    if ui.button("×").clicked() {
                        self.component_search.clear();
                    }
                });

                ui.separator();

                // Component categories
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Physics category
                    ui.collapsing("Physics", |ui| {
                        if ui.selectable_label(false, "Rigidbody").clicked() {
                            // Add rigidbody component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Box Collider").clicked() {
                            // Add box collider component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Sphere Collider").clicked() {
                            // Add sphere collider component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Capsule Collider").clicked() {
                            // Add capsule collider component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Mesh Collider").clicked() {
                            // Add mesh collider component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Hinge Joint").clicked() {
                            // Add hinge joint component to entity
                            self.show_add_component = false;
                        }
                    });

                    // Animation category
                    ui.collapsing("Animation", |ui| {
                        if ui.selectable_label(false, "Animator").clicked() {
                            // Add animator component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Animation").clicked() {
                            // Add animation component to entity
                            self.show_add_component = false;
                        }
                    });

                    // Rendering category
                    ui.collapsing("Rendering", |ui| {
                        if ui.selectable_label(false, "Mesh Renderer").clicked() {
                            // Add mesh renderer component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Camera").clicked() {
                            // Add camera component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Light").clicked() {
                            // Add light component to entity
                            self.show_add_component = false;
                        }
                    });

                    // Audio category
                    ui.collapsing("Audio", |ui| {
                        if ui.selectable_label(false, "Audio Source").clicked() {
                            // Add audio source component to entity
                            self.show_add_component = false;
                        }

                        if ui.selectable_label(false, "Audio Listener").clicked() {
                            // Add audio listener component to entity
                            self.show_add_component = false;
                        }
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_add_component = false;
                    }
                });
            });
    }

    /// Show the inspector for a Scene (wrapper around show_ui)
    pub fn show_ui_for_scene(
        &mut self,
        ui: &mut egui::Ui,
        scene: &mut Scene,
        selected_object: Option<u32>,
    ) -> bool {
        ui.heading("Inspector");

        if let Some(object_id) = selected_object {
            // Get object data first to avoid borrowing conflicts
            let (object_name, object_enabled, object_transform) = {
                if let Some(object) = scene.objects.get(&object_id) {
                    (
                        object.name.clone(),
                        object.enabled,
                        object.transform.clone(),
                    )
                } else {
                    ui.label("Selected object not found!");
                    return false;
                }
            };

            // Object header with name and enable checkbox
            let mut name = object_name;
            let mut enabled = object_enabled;
            let mut name_changed = false;
            let mut enabled_changed = false;

            ui.horizontal(|ui| {
                if ui.checkbox(&mut enabled, "").changed() {
                    enabled_changed = true;
                }

                if ui.text_edit_singleline(&mut name).changed() {
                    name_changed = true;
                }
            });

            // Apply changes after the borrow ends
            if name_changed || enabled_changed {
                if let Some(obj) = scene.objects.get_mut(&object_id) {
                    if name_changed {
                        obj.name = name;
                    }
                    if enabled_changed {
                        obj.enabled = enabled;
                    }
                }
            }

            ui.separator();

            // Tag dropdown
            ui.horizontal(|ui| {
                ui.label("Tag:");
                egui::ComboBox::from_label("")
                    .selected_text("Untagged")
                    .show_ui(ui, |ui| {
                        for tag in &self.available_tags {
                            ui.selectable_value(&mut "Untagged".to_string(), tag.clone(), tag);
                        }
                    });
            });

            ui.separator();

            // Transform component (always present) - EDITABLE VERSION
            let mut transform_changed = false;
            let mut new_transform = object_transform.clone();

            ui.collapsing("Transform", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.position.x)
                                .speed(0.1)
                                .prefix("X: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.position.y)
                                .speed(0.1)
                                .prefix("Y: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.position.z)
                                .speed(0.1)
                                .prefix("Z: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.x)
                                .speed(1.0)
                                .prefix("X: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.y)
                                .speed(1.0)
                                .prefix("Y: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.rotation.z)
                                .speed(1.0)
                                .prefix("Z: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.scale.x)
                                .speed(0.1)
                                .prefix("X: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.scale.y)
                                .speed(0.1)
                                .prefix("Y: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                    if ui
                        .add(
                            egui::DragValue::new(&mut new_transform.scale.z)
                                .speed(0.1)
                                .prefix("Z: "),
                        )
                        .changed()
                    {
                        transform_changed = true;
                    }
                });
            });

            // Apply transform changes
            if transform_changed {
                if let Some(obj) = scene.objects.get_mut(&object_id) {
                    obj.transform = new_transform;
                }
            }

            // Show other components
            if let Some(object) = scene.objects.get(&object_id) {
                for component in &object.components {
                    match component {
                        Component::Mesh { mesh_type } => {
                            ui.collapsing("Mesh Filter", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Mesh:");
                                    ui.label(mesh_type);
                                });
                            });
                        }
                        Component::Renderer { material, color } => {
                            ui.collapsing("Mesh Renderer", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Material:");
                                    ui.label(material);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    let mut color_array = [color[0], color[1], color[2], color[3]];
                                    ui.color_edit_button_rgba_premultiplied(&mut color_array);
                                    // In a real implementation, you'd update the component here
                                });
                            });
                        }
                        Component::RigidBody { shape, mass } => {
                            ui.collapsing("Rigidbody", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Mass:");
                                    ui.label(format!("{:.2}", mass));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Shape:");
                                    match shape {
                                        Shape::Box { size } => {
                                            ui.label(format!(
                                                "Box ({:.2}, {:.2}, {:.2})",
                                                size.x, size.y, size.z
                                            ));
                                        }
                                        Shape::Sphere { radius } => {
                                            ui.label(format!("Sphere (r: {:.2})", radius));
                                        }
                                        Shape::Capsule { radius, height } => {
                                            ui.label(format!(
                                                "Capsule (r: {:.2}, h: {:.2})",
                                                radius, height
                                            ));
                                        }
                                        Shape::Cylinder { radius, height } => {
                                            ui.label(format!(
                                                "Cylinder (r: {:.2}, h: {:.2})",
                                                radius, height
                                            ));
                                        }
                                        Shape::ConvexHull { vertices } => {
                                            ui.label(format!(
                                                "ConvexHull ({} vertices)",
                                                vertices.len()
                                            ));
                                        }
                                        Shape::TriangleMesh { vertices, indices } => {
                                            ui.label(format!(
                                                "TriangleMesh ({} vertices, {} triangles)",
                                                vertices.len(),
                                                indices.len()
                                            ));
                                        }
                                    }
                                });
                            });
                        }
                        Component::Collider { shape, is_trigger } => {
                            ui.collapsing("Collider", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Is Trigger:");
                                    ui.label(if *is_trigger { "Yes" } else { "No" });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Shape:");
                                    match shape {
                                        Shape::Box { size } => {
                                            ui.label(format!(
                                                "Box ({:.2}, {:.2}, {:.2})",
                                                size.x, size.y, size.z
                                            ));
                                        }
                                        Shape::Sphere { radius } => {
                                            ui.label(format!("Sphere (r: {:.2})", radius));
                                        }
                                        Shape::Capsule { radius, height } => {
                                            ui.label(format!(
                                                "Capsule (r: {:.2}, h: {:.2})",
                                                radius, height
                                            ));
                                        }
                                        Shape::Cylinder { radius, height } => {
                                            ui.label(format!(
                                                "Cylinder (r: {:.2}, h: {:.2})",
                                                radius, height
                                            ));
                                        }
                                        Shape::ConvexHull { vertices } => {
                                            ui.label(format!(
                                                "ConvexHull ({} vertices)",
                                                vertices.len()
                                            ));
                                        }
                                        Shape::TriangleMesh { vertices, indices } => {
                                            ui.label(format!(
                                                "TriangleMesh ({} vertices, {} triangles)",
                                                vertices.len(),
                                                indices.len()
                                            ));
                                        }
                                    }
                                });
                            });
                        }
                        Component::Light {
                            light_type,
                            intensity,
                            color,
                        } => {
                            ui.collapsing("Light", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Type:");
                                    ui.label(light_type);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Intensity:");
                                    ui.label(format!("{:.2}", intensity));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    let mut color_array = [color[0], color[1], color[2]];
                                    ui.color_edit_button_rgb(&mut color_array);
                                });
                            });
                        }
                        Component::Camera { fov, near, far } => {
                            ui.collapsing("Camera", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("FOV:");
                                    ui.label(format!("{:.1}°", fov));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Near:");
                                    ui.label(format!("{:.2}", near));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Far:");
                                    ui.label(format!("{:.2}", far));
                                });
                            });
                        }
                        Component::SoftBodyComponent {
                            particles,
                            stiffness,
                        } => {
                            ui.collapsing("Soft Body", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Particles:");
                                    ui.label(format!("{}", particles));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Stiffness:");
                                    ui.label(format!("{:.2}", stiffness));
                                });
                            });
                        }
                        Component::Script { script_path, code } => {
                            ui.collapsing("Script", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Path:");
                                    ui.label(script_path);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Lines:");
                                    ui.label(format!("{}", code.lines().count()));
                                });
                            });
                        }
                    }
                }

                // Add Component button
                if ui.button("Add Component").clicked() {
                    self.show_add_component = true;
                }

                // Add Component Dialog
                if self.show_add_component {
                    self.show_add_component_dialog_for_scene(ui, object_id, scene);
                }
            }

            transform_changed // Return true if transform was changed
        } else {
            ui.label("No object selected");
            false
        }
    }

    /// Show transform component for scene objects
    fn show_scene_transform_component(&mut self, ui: &mut egui::Ui, object: &GameObject) {
        ui.collapsing("Transform", |ui| {
            let transform = &object.transform;

            // Position
            ui.horizontal(|ui| {
                ui.label("Position");
                ui.label(format!("X: {:.2}", transform.position.x));
                ui.label(format!("Y: {:.2}", transform.position.y));
                ui.label(format!("Z: {:.2}", transform.position.z));
            });

            // Rotation
            ui.horizontal(|ui| {
                ui.label("Rotation");
                ui.label(format!("X: {:.2}", transform.rotation.x));
                ui.label(format!("Y: {:.2}", transform.rotation.y));
                ui.label(format!("Z: {:.2}", transform.rotation.z));
            });

            // Scale
            ui.horizontal(|ui| {
                ui.label("Scale   ");
                ui.label(format!("X: {:.2}", transform.scale.x));
                ui.label(format!("Y: {:.2}", transform.scale.y));
                ui.label(format!("Z: {:.2}", transform.scale.z));
            });
        });
    }

    /// Show add component dialog for scene objects
    fn show_add_component_dialog_for_scene(
        &mut self,
        ui: &mut egui::Ui,
        _object_id: u32,
        _scene: &mut Scene,
    ) {
        egui::Window::new("Add Component")
            .fixed_size([300.0, 400.0])
            .show(ui.ctx(), |ui| {
                ui.label("Add Component functionality coming soon!");

                ui.separator();

                if ui.button("Cancel").clicked() {
                    self.show_add_component = false;
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_default() {
        let inspector = Inspector::default();
        assert!(inspector.transform_expanded);
        assert!(inspector.rigidbody_expanded);
        assert!(!inspector.show_add_component);
    }
}
