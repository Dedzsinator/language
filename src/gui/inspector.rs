use super::*;

/// Inspector panel similar to Unity's Inspector window
pub struct Inspector {
    // Component editing state
    editing_component: Option<usize>,
    temp_values: std::collections::HashMap<String, String>,
    // Tags system
    available_tags: Vec<String>,
    new_tag_name: String,
    show_script_dialog: bool,
    show_script_editor: bool,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            editing_component: None,
            temp_values: std::collections::HashMap::new(),
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
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, scene: &mut Scene, selected_object: Option<u32>) {
        egui::SidePanel::right("inspector_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");

                if let Some(object_id) = selected_object {
                    if let Some(object) = scene.objects.get_mut(&object_id) {
                        self.show_object_inspector(ui, object);
                    } else {
                        ui.label("Selected object not found");
                    }
                } else {
                    ui.label("No object selected");
                }

                // Show dialogs if requested
                if self.show_script_dialog {
                    self.show_script_dialog(ui);
                }

                if self.show_script_editor {
                    self.show_script_editor(ui);
                }
            });
    }

    pub fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        scene: &mut Scene,
        selected_object: Option<u32>,
    ) -> bool {
        ui.heading("Inspector");

        let mut transform_changed = false;

        if let Some(object_id) = selected_object {
            if let Some(object) = scene.objects.get_mut(&object_id) {
                transform_changed = self.show_object_inspector(ui, object);
            } else {
                ui.label("Selected object not found");
            }
        } else {
            ui.label("No object selected");
        }

        transform_changed
    }

    fn show_object_inspector(&mut self, ui: &mut egui::Ui, object: &mut GameObject) -> bool {
        let mut transform_changed = false;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Object header
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut object.enabled, "");
                    ui.text_edit_singleline(&mut object.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Tag:");
                    egui::ComboBox::from_id_salt("tag_combo")
                        .selected_text(&object.tag)
                        .show_ui(ui, |ui| {
                            for tag in &self.available_tags {
                                ui.selectable_value(&mut object.tag, tag.clone(), tag);
                            }

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.new_tag_name);
                                if ui.button("Add Tag").clicked() && !self.new_tag_name.is_empty() {
                                    if !self.available_tags.contains(&self.new_tag_name) {
                                        self.available_tags.push(self.new_tag_name.clone());
                                    }
                                    object.tag = self.new_tag_name.clone();
                                    self.new_tag_name.clear();
                                }
                            });
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Layer:");
                    egui::ComboBox::from_id_salt("layer_combo")
                        .selected_text("Default")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut 0, 0, "Default");
                            ui.selectable_value(&mut 1, 1, "UI");
                            ui.selectable_value(&mut 2, 2, "Physics");
                        });
                });
            });

            ui.separator();

            // Transform component (always present)
            transform_changed = self.show_transform_component(ui, &mut object.transform);

            ui.separator();

            // Other components
            let mut components_to_remove = Vec::new();
            for (index, component) in object.components.iter_mut().enumerate() {
                if self.show_component(ui, component, index) {
                    components_to_remove.push(index);
                }
                ui.separator();
            }

            // Remove components marked for deletion
            for &index in components_to_remove.iter().rev() {
                object.components.remove(index);
            }

            // Add Component button
            if ui.button("Add Component").clicked() {
                self.show_add_component_menu(ui, object);
            }
        });

        transform_changed
    }

    fn show_transform_component(&mut self, ui: &mut egui::Ui, transform: &mut Transform) -> bool {
        let mut changed = false;

        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].label("Position");
                    columns[1].horizontal(|ui| {
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.position.x)
                                    .prefix("X: ")
                                    .speed(0.1),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.position.y)
                                    .prefix("Y: ")
                                    .speed(0.1),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.position.z)
                                    .prefix("Z: ")
                                    .speed(0.1),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].label("Rotation");
                    columns[1].horizontal(|ui| {
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.rotation.x)
                                    .prefix("X: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.rotation.y)
                                    .prefix("Y: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.rotation.z)
                                    .prefix("Z: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].label("Scale");
                    columns[1].horizontal(|ui| {
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.scale.x)
                                    .prefix("X: ")
                                    .speed(0.01)
                                    .range(0.001..=100.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.scale.y)
                                    .prefix("Y: ")
                                    .speed(0.01)
                                    .range(0.001..=100.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::DragValue::new(&mut transform.scale.z)
                                    .prefix("Z: ")
                                    .speed(0.01)
                                    .range(0.001..=100.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    });
                });
            });

        changed
    }

    fn show_component(
        &mut self,
        ui: &mut egui::Ui,
        component: &mut Component,
        index: usize,
    ) -> bool {
        let mut should_remove = false;

        match component {
            Component::Mesh { mesh_type } => {
                // Use temp_values for validation feedback
                let validation_key = format!("mesh_validation_{}", index);

                egui::CollapsingHeader::new("Mesh")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mesh Type:");

                            egui::ComboBox::from_id_salt(format!("mesh_type_{}", index))
                                .selected_text(mesh_type.as_str())
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(mesh_type, "Cube".to_string(), "Cube")
                                        .clicked()
                                    {
                                        self.temp_values.insert(
                                            validation_key.clone(),
                                            "Mesh type changed to Cube".to_string(),
                                        );
                                    }
                                    if ui
                                        .selectable_value(mesh_type, "Sphere".to_string(), "Sphere")
                                        .clicked()
                                    {
                                        self.temp_values.insert(
                                            validation_key.clone(),
                                            "Mesh type changed to Sphere".to_string(),
                                        );
                                    }
                                    if ui
                                        .selectable_value(
                                            mesh_type,
                                            "Cylinder".to_string(),
                                            "Cylinder",
                                        )
                                        .clicked()
                                    {
                                        self.temp_values.insert(
                                            validation_key.clone(),
                                            "Mesh type changed to Cylinder".to_string(),
                                        );
                                    }
                                    if ui
                                        .selectable_value(mesh_type, "Plane".to_string(), "Plane")
                                        .clicked()
                                    {
                                        self.temp_values.insert(
                                            validation_key.clone(),
                                            "Mesh type changed to Plane".to_string(),
                                        );
                                    }
                                });

                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        // Show validation message if available
                        if let Some(message) = self.temp_values.get(&validation_key) {
                            ui.colored_label(egui::Color32::GREEN, message);
                        }
                    });
            }

            Component::Renderer { material, color } => {
                let validation_key = format!("renderer_validation_{}", index);
                let is_editing = self.editing_component == Some(index);

                egui::CollapsingHeader::new("Renderer")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Material:");

                            if is_editing {
                                // Edit mode with validation
                                let temp_key = format!("material_temp_{}", index);
                                let current_material = self
                                    .temp_values
                                    .get(&temp_key)
                                    .cloned()
                                    .unwrap_or_else(|| material.clone());

                                let mut temp_material = current_material.clone();
                                if ui.text_edit_singleline(&mut temp_material).changed() {
                                    self.temp_values.insert(temp_key.clone(), temp_material);
                                }

                                if ui.small_button("✓").clicked() {
                                    if let Some(new_material) = self.temp_values.get(&temp_key) {
                                        *material = new_material.clone();
                                        self.temp_values.insert(
                                            validation_key.clone(),
                                            "Material updated successfully".to_string(),
                                        );
                                    }
                                    self.editing_component = None;
                                    self.temp_values.remove(&temp_key);
                                }
                                if ui.small_button("✗").clicked() {
                                    self.editing_component = None;
                                    self.temp_values.remove(&temp_key);
                                }
                            } else {
                                ui.text_edit_singleline(material);
                                if ui.small_button("✏").clicked() {
                                    self.editing_component = Some(index);
                                    let temp_key = format!("material_temp_{}", index);
                                    self.temp_values.insert(temp_key, material.clone());
                                }
                            }

                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            if ui.color_edit_button_rgba_unmultiplied(color).changed() {
                                self.temp_values
                                    .insert(validation_key.clone(), "Color updated".to_string());
                            }
                        });

                        // Show validation message if available
                        if let Some(message) = self.temp_values.get(&validation_key) {
                            ui.colored_label(egui::Color32::GREEN, message);
                        }
                    });
            }

            Component::RigidBody { shape, mass } => {
                egui::CollapsingHeader::new("Rigid Body")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mass:");
                            ui.add(egui::DragValue::new(mass).speed(0.1).range(0.001..=1000.0));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Shape:");
                        match shape {
                            Shape::Sphere { radius } => {
                                ui.horizontal(|ui| {
                                    ui.label("Sphere Radius:");
                                    ui.add(
                                        egui::DragValue::new(radius)
                                            .speed(0.1)
                                            .range(0.001..=100.0),
                                    );
                                });
                            }
                            Shape::Box { size } => {
                                ui.horizontal(|ui| {
                                    ui.label("Box Size:");
                                    ui.add(
                                        egui::DragValue::new(&mut size.x).prefix("X: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut size.y).prefix("Y: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut size.z).prefix("Z: ").speed(0.1),
                                    );
                                });
                            }
                            Shape::Cylinder { radius, height } => {
                                ui.horizontal(|ui| {
                                    ui.label("Cylinder:");
                                    ui.add(egui::DragValue::new(radius).prefix("R: ").speed(0.1));
                                    ui.add(egui::DragValue::new(height).prefix("H: ").speed(0.1));
                                });
                            }
                            Shape::Capsule { radius, height } => {
                                ui.horizontal(|ui| {
                                    ui.label("Capsule:");
                                    ui.add(egui::DragValue::new(radius).prefix("R: ").speed(0.1));
                                    ui.add(egui::DragValue::new(height).prefix("H: ").speed(0.1));
                                });
                            }
                            Shape::ConvexHull { vertices: _ } => {
                                ui.label("Convex Hull - Vertex editing not yet implemented");
                            }
                            Shape::TriangleMesh {
                                vertices: _,
                                indices: _,
                            } => {
                                ui.label("Triangle Mesh - Mesh editing not yet implemented");
                            }
                        }
                    });
            }

            Component::SoftBodyComponent {
                particles,
                stiffness,
            } => {
                egui::CollapsingHeader::new("Soft Body")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Particles:");
                            ui.add(egui::DragValue::new(particles).range(10..=1000));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Stiffness:");
                            ui.add(egui::Slider::new(stiffness, 0.0..=1.0));
                        });
                    });
            }

            Component::Script { script_path, code } => {
                egui::CollapsingHeader::new("Script")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Script:");
                            ui.text_edit_singleline(script_path);
                            if ui.button("Browse").clicked() {
                                self.show_script_dialog = true;
                            }
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Code Preview:");
                        ui.add(
                            egui::TextEdit::multiline(code)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY),
                        );

                        if ui.button("Edit Script").clicked() {
                            self.show_script_editor = true;
                        }
                    });
            }

            Component::Light {
                light_type,
                intensity,
                color,
            } => {
                egui::CollapsingHeader::new("Light")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt(format!("light_type_{}", index))
                                .selected_text(light_type.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        light_type,
                                        "Directional".to_string(),
                                        "Directional",
                                    );
                                    ui.selectable_value(light_type, "Point".to_string(), "Point");
                                    ui.selectable_value(light_type, "Spot".to_string(), "Spot");
                                });
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(intensity).speed(0.1).range(0.0..=10.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_rgb(color);
                        });
                    });
            }

            Component::Camera { fov, near, far } => {
                egui::CollapsingHeader::new("Camera")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Field of View:");
                            ui.add(
                                egui::DragValue::new(fov)
                                    .speed(1.0)
                                    .range(1.0..=179.0)
                                    .suffix("°"),
                            );
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Near Plane:");
                            ui.add(egui::DragValue::new(near).speed(0.01).range(0.001..=1000.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Far Plane:");
                            ui.add(egui::DragValue::new(far).speed(1.0).range(1.0..=10000.0));
                        });
                    });
            }

            Component::Collider { shape, is_trigger } => {
                egui::CollapsingHeader::new("Collider")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(is_trigger, "Is Trigger");
                            ui.label(format!("Shape: {:?}", shape));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Shape: (same as RigidBody)");
                        // Shape editing would be similar to RigidBody
                    });
            }
        }

        should_remove
    }

    fn show_add_component_menu(&mut self, ui: &mut egui::Ui, object: &mut GameObject) {
        ui.menu_button("Add Component", |ui| {
            if ui.button("Renderer").clicked() {
                object.components.push(Component::Renderer {
                    material: "Default".to_string(),
                    color: [1.0, 1.0, 1.0, 1.0],
                });
                ui.close_menu();
            }

            if ui.button("Rigid Body").clicked() {
                object.components.push(Component::RigidBody {
                    shape: Shape::Box {
                        size: Vec3::new(1.0, 1.0, 1.0),
                    },
                    mass: 1.0,
                });
                ui.close_menu();
            }

            if ui.button("Collider").clicked() {
                object.components.push(Component::Collider {
                    shape: Shape::Box {
                        size: Vec3::new(1.0, 1.0, 1.0),
                    },
                    is_trigger: false,
                });
                ui.close_menu();
            }

            if ui.button("Script").clicked() {
                object.components.push(Component::Script {
                    script_path: "script.matrix".to_string(),
                    code: "// Add your Matrix Language code here\nlet x = 5".to_string(),
                });
                ui.close_menu();
            }

            if ui.button("Light").clicked() {
                object.components.push(Component::Light {
                    light_type: "Point".to_string(),
                    intensity: 1.0,
                    color: [1.0, 1.0, 1.0],
                });
                ui.close_menu();
            }

            if ui.button("Camera").clicked() {
                object.components.push(Component::Camera {
                    fov: 60.0,
                    near: 0.1,
                    far: 1000.0,
                });
                ui.close_menu();
            }
        });
    }

    /// Show script file browser dialog
    fn show_script_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Select Script File")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.label("Select a Matrix Language script file:");
                ui.separator();

                // Simple file browser for script files
                if let Ok(entries) = std::fs::read_dir(".") {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for entry in entries.flatten() {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.ends_with(".matrix") || name.ends_with(".ml") {
                                        if ui.selectable_label(false, name).clicked() {
                                            // This would set the script path in the component
                                            // For now, just close the dialog
                                            self.show_script_dialog = false;
                                        }
                                    }
                                }
                            }
                        });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_script_dialog = false;
                    }
                });
            });
    }

    /// Show script editor window
    fn show_script_editor(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Script Editor")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ui.ctx(), |ui| {
                ui.label("Matrix Language Script Editor");
                ui.separator();

                // Basic script editor interface
                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        // Save script logic here
                        println!("Save script");
                    }
                    if ui.button("▶ Run").clicked() {
                        // Run script logic here
                        println!("Run script");
                    }
                    if ui.button("🐛 Debug").clicked() {
                        // Debug script logic here
                        println!("Debug script");
                    }
                });

                ui.separator();

                // Script content area
                let mut script_content =
                    "// Matrix Language Script\nlet x = 5\nlet y = x * 2\nprint(y)".to_string();
                ui.add(
                    egui::TextEdit::multiline(&mut script_content)
                        .desired_rows(20)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.show_script_editor = false;
                    }
                });
            });
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
