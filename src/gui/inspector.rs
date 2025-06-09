use super::*;

/// Inspector panel similar to Unity's Inspector window
pub struct Inspector {
    // Component editing state
    editing_component: Option<usize>,
    temp_values: std::collections::HashMap<String, String>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            editing_component: None,
            temp_values: std::collections::HashMap::new(),
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
            });
    }

    fn show_object_inspector(&mut self, ui: &mut egui::Ui, object: &mut GameObject) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Object header
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut object.enabled, "");
                    ui.text_edit_singleline(&mut object.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Tag:");
                    ui.text_edit_singleline(&mut String::from("Untagged")); // TODO: Add tags system
                });

                ui.horizontal(|ui| {
                    ui.label("Layer:");
                    egui::ComboBox::from_id_source("layer_combo")
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
            self.show_transform_component(ui, &mut object.transform);

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
    }

    fn show_transform_component(&mut self, ui: &mut egui::Ui, transform: &mut Transform) {
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].label("Position");
                    columns[1].horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut transform.position.x).prefix("X: ").speed(0.1));
                        ui.add(egui::DragValue::new(&mut transform.position.y).prefix("Y: ").speed(0.1));
                        ui.add(egui::DragValue::new(&mut transform.position.z).prefix("Z: ").speed(0.1));
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].label("Rotation");
                    columns[1].horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut transform.rotation.x).prefix("X: ").speed(1.0).suffix("°"));
                        ui.add(egui::DragValue::new(&mut transform.rotation.y).prefix("Y: ").speed(1.0).suffix("°"));
                        ui.add(egui::DragValue::new(&mut transform.rotation.z).prefix("Z: ").speed(1.0).suffix("°"));
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].label("Scale");
                    columns[1].horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut transform.scale.x).prefix("X: ").speed(0.01).clamp_range(0.001..=100.0));
                        ui.add(egui::DragValue::new(&mut transform.scale.y).prefix("Y: ").speed(0.01).clamp_range(0.001..=100.0));
                        ui.add(egui::DragValue::new(&mut transform.scale.z).prefix("Z: ").speed(0.01).clamp_range(0.001..=100.0));
                    });
                });
            });
    }

    fn show_component(&mut self, ui: &mut egui::Ui, component: &mut Component, index: usize) -> bool {
        let mut should_remove = false;

        match component {
            Component::Mesh { mesh_type } => {
                egui::CollapsingHeader::new("Mesh")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mesh Type:");
                            egui::ComboBox::from_id_source(format!("mesh_type_{}", index))
                                .selected_text(mesh_type.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(mesh_type, "Cube".to_string(), "Cube");
                                    ui.selectable_value(mesh_type, "Sphere".to_string(), "Sphere");
                                    ui.selectable_value(mesh_type, "Cylinder".to_string(), "Cylinder");
                                    ui.selectable_value(mesh_type, "Plane".to_string(), "Plane");
                                });

                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });
                    });
            },

            Component::Renderer { material, color } => {
                egui::CollapsingHeader::new("Renderer")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Material:");
                            ui.text_edit_singleline(material);
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_rgba_unmultiplied(color);
                        });
                    });
            },

            Component::RigidBody { shape, mass } => {
                egui::CollapsingHeader::new("Rigid Body")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mass:");
                            ui.add(egui::DragValue::new(mass).speed(0.1).clamp_range(0.001..=1000.0));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Shape:");
                        match shape {
                            Shape::Sphere { radius } => {
                                ui.horizontal(|ui| {
                                    ui.label("Sphere Radius:");
                                    ui.add(egui::DragValue::new(radius).speed(0.1).clamp_range(0.001..=100.0));
                                });
                            },
                            Shape::Box { size } => {
                                ui.horizontal(|ui| {
                                    ui.label("Box Size:");
                                    ui.add(egui::DragValue::new(&mut size.x).prefix("X: ").speed(0.1));
                                    ui.add(egui::DragValue::new(&mut size.y).prefix("Y: ").speed(0.1));
                                    ui.add(egui::DragValue::new(&mut size.z).prefix("Z: ").speed(0.1));
                                });
                            },
                            Shape::Cylinder { radius, height } => {
                                ui.horizontal(|ui| {
                                    ui.label("Cylinder:");
                                    ui.add(egui::DragValue::new(radius).prefix("R: ").speed(0.1));
                                    ui.add(egui::DragValue::new(height).prefix("H: ").speed(0.1));
                                });
                            },
                            Shape::Capsule { radius, height } => {
                                ui.horizontal(|ui| {
                                    ui.label("Capsule:");
                                    ui.add(egui::DragValue::new(radius).prefix("R: ").speed(0.1));
                                    ui.add(egui::DragValue::new(height).prefix("H: ").speed(0.1));
                                });
                            },
                            Shape::ConvexHull { vertices: _ } => {
                                ui.label("Convex Hull - Vertex editing not yet implemented");
                            },
                            Shape::TriangleMesh { vertices: _, indices: _ } => {
                                ui.label("Triangle Mesh - Mesh editing not yet implemented");
                            },
                        }
                    });
            },

            Component::SoftBodyComponent { particles, stiffness } => {
                egui::CollapsingHeader::new("Soft Body")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Particles:");
                            ui.add(egui::DragValue::new(particles).clamp_range(10..=1000));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Stiffness:");
                            ui.add(egui::Slider::new(stiffness, 0.0..=1.0));
                        });
                    });
            },

            Component::Script { script_path, code } => {
                egui::CollapsingHeader::new("Script")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Script:");
                            ui.text_edit_singleline(script_path);
                            if ui.button("Browse").clicked() {
                                // TODO: Open file dialog
                            }
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Code Preview:");
                        ui.add(egui::TextEdit::multiline(code)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY));

                        if ui.button("Edit Script").clicked() {
                            // TODO: Open script editor
                        }
                    });
            },

            Component::Light { light_type, intensity, color } => {
                egui::CollapsingHeader::new("Light")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_source(format!("light_type_{}", index))
                                .selected_text(light_type.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(light_type, "Directional".to_string(), "Directional");
                                    ui.selectable_value(light_type, "Point".to_string(), "Point");
                                    ui.selectable_value(light_type, "Spot".to_string(), "Spot");
                                });
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(intensity).speed(0.1).clamp_range(0.0..=10.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_rgb(color);
                        });
                    });
            },

            Component::Camera { fov, near, far } => {
                egui::CollapsingHeader::new("Camera")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Field of View:");
                            ui.add(egui::DragValue::new(fov).speed(1.0).clamp_range(1.0..=179.0).suffix("°"));
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Near Plane:");
                            ui.add(egui::DragValue::new(near).speed(0.01).clamp_range(0.001..=1000.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Far Plane:");
                            ui.add(egui::DragValue::new(far).speed(1.0).clamp_range(1.0..=10000.0));
                        });
                    });
            },

            Component::Collider { shape, is_trigger } => {
                egui::CollapsingHeader::new("Collider")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(is_trigger, "Is Trigger");
                            if ui.small_button("🗑").clicked() {
                                should_remove = true;
                            }
                        });

                        ui.label("Shape: (same as RigidBody)");
                        // Shape editing would be similar to RigidBody
                    });
            },
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
                    shape: Shape::Box { size: Vec3::new(1.0, 1.0, 1.0) },
                    mass: 1.0,
                });
                ui.close_menu();
            }

            if ui.button("Collider").clicked() {
                object.components.push(Component::Collider {
                    shape: Shape::Box { size: Vec3::new(1.0, 1.0, 1.0) },
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
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
