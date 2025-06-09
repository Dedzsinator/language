use super::*;

/// Object hierarchy panel similar to Unity's Hierarchy window
pub struct ObjectHierarchy {
    filter_text: String,
    show_inactive: bool,
    expanded_objects: std::collections::HashSet<u32>,
}

impl ObjectHierarchy {
    pub fn new() -> Self {
        Self {
            filter_text: String::new(),
            show_inactive: true,
            expanded_objects: std::collections::HashSet::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, scene: &mut Scene, selected_object: &mut Option<u32>) {
        egui::SidePanel::left("hierarchy_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Hierarchy");

                // Toolbar
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.filter_text);
                    ui.checkbox(&mut self.show_inactive, "Show Inactive");
                });

                ui.separator();

                // Create object menu
                ui.menu_button("Create", |ui| {
                    if ui.button("Empty GameObject").clicked() {
                        scene.add_object("GameObject".to_string(), GameObjectType::Empty);
                        ui.close_menu();
                    }

                    ui.separator();

                    ui.menu_button("3D Objects", |ui| {
                        if ui.button("Cube").clicked() {
                            let id = scene.add_object("Cube".to_string(), GameObjectType::Cube);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh { mesh_type: "Cube".to_string() });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Sphere").clicked() {
                            let id = scene.add_object("Sphere".to_string(), GameObjectType::Sphere);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Cylinder").clicked() {
                            let id = scene.add_object("Cylinder".to_string(), GameObjectType::Cylinder);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh { mesh_type: "Cylinder".to_string() });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Plane").clicked() {
                            let id = scene.add_object("Plane".to_string(), GameObjectType::Plane);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh { mesh_type: "Plane".to_string() });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Physics", |ui| {
                        if ui.button("Rigid Body Sphere").clicked() {
                            let id = scene.add_object("RigidBody".to_string(),
                                GameObjectType::RigidBody(Shape::Sphere { radius: 1.0 }));
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [0.8, 0.3, 0.3, 1.0]
                                });
                                obj.components.push(Component::RigidBody {
                                    shape: Shape::Sphere { radius: 1.0 },
                                    mass: 1.0
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Soft Body").clicked() {
                            let id = scene.add_object("SoftBody".to_string(), GameObjectType::SoftBody);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::SoftBodyComponent {
                                    particles: 100,
                                    stiffness: 0.8
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Fluid Emitter").clicked() {
                            let id = scene.add_object("FluidEmitter".to_string(), GameObjectType::FluidEmitter);
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Lighting", |ui| {
                        if ui.button("Directional Light").clicked() {
                            let id = scene.add_object("Directional Light".to_string(), GameObjectType::Light);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Light {
                                    light_type: "Directional".to_string(),
                                    intensity: 1.0,
                                    color: [1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Point Light").clicked() {
                            let id = scene.add_object("Point Light".to_string(), GameObjectType::Light);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Light {
                                    light_type: "Point".to_string(),
                                    intensity: 1.0,
                                    color: [1.0, 1.0, 1.0]
                                });
                            }
                            ui.close_menu();
                        }
                    });

                    if ui.button("Camera").clicked() {
                        let id = scene.add_object("Camera".to_string(), GameObjectType::Camera);
                        if let Some(obj) = scene.objects.get_mut(&id) {
                            obj.components.push(Component::Camera {
                                fov: 60.0,
                                near: 0.1,
                                far: 1000.0
                            });
                        }
                        ui.close_menu();
                    }
                });

                ui.separator();

                // Object list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Get root objects (objects with no parent)
                    let root_objects: Vec<u32> = scene.objects.values()
                        .filter(|obj| obj.parent.is_none())
                        .map(|obj| obj.id)
                        .collect();

                    for object_id in root_objects {
                        self.show_object_tree(ui, scene, object_id, selected_object, 0);
                    }
                });
            });
    }

    fn show_object_tree(&mut self, ui: &mut egui::Ui, scene: &mut Scene, object_id: u32,
                       selected_object: &mut Option<u32>, depth: usize) {
        if let Some(object) = scene.objects.get(&object_id) {
            // Filter check
            if !self.filter_text.is_empty() &&
               !object.name.to_lowercase().contains(&self.filter_text.to_lowercase()) {
                return;
            }

            // Show inactive check
            if !self.show_inactive && !object.enabled {
                return;
            }

            let indent = (depth as f32) * 20.0;
            ui.indent(format!("object_{}", object_id), |ui| {
                ui.allocate_ui_with_layout(
                    [ui.available_width() - indent, 20.0].into(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // Expand/collapse triangle for objects with children
                        if !object.children.is_empty() {
                            let expanded = self.expanded_objects.contains(&object_id);
                            let triangle = if expanded { "▼" } else { "▶" };
                            if ui.small_button(triangle).clicked() {
                                if expanded {
                                    self.expanded_objects.remove(&object_id);
                                } else {
                                    self.expanded_objects.insert(object_id);
                                }
                            }
                        } else {
                            ui.add_space(20.0); // Space for alignment
                        }

                        // Visibility toggle
                        let mut visible = object.visible;
                        if ui.checkbox(&mut visible, "").changed() {
                            if let Some(obj) = scene.objects.get_mut(&object_id) {
                                obj.visible = visible;
                            }
                        }

                        // Object name (selectable)
                        let is_selected = *selected_object == Some(object_id);
                        let response = ui.selectable_label(is_selected, &object.name);

                        if response.clicked() {
                            *selected_object = Some(object_id);
                        }

                        // Context menu - temporarily disabled to fix borrow checker
                        // TODO: Fix borrow checker issues properly
                        /*
                        response.context_menu(|ui| {
                            if ui.button("Duplicate").clicked() {
                                self.duplicate_object(scene, object_id);
                                ui.close_menu();
                            }
                            if ui.button("Delete").clicked() {
                                scene.remove_object(object_id);
                                if *selected_object == Some(object_id) {
                                    *selected_object = None;
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Add Child").clicked() {
                                let child_id = scene.add_object("Child".to_string(), GameObjectType::Empty);
                                scene.set_parent(child_id, Some(object_id));
                                ui.close_menu();
                            }
                        });
                        */
                    }
                );
            });

            // Show children if expanded
            if self.expanded_objects.contains(&object_id) {
                for &child_id in &object.children.clone() {
                    self.show_object_tree(ui, scene, child_id, selected_object, depth + 1);
                }
            }
        }
    }

    fn duplicate_object(&mut self, scene: &mut Scene, object_id: u32) {
        if let Some(original) = scene.objects.get(&object_id).cloned() {
            let new_id = scene.next_id;
            scene.next_id += 1;

            let mut duplicate = original;
            duplicate.id = new_id;
            duplicate.name = format!("{} Copy", duplicate.name);
            duplicate.children.clear();

            scene.objects.insert(new_id, duplicate);
        }
    }
}

impl Default for ObjectHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
