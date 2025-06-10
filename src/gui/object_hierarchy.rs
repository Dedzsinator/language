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

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        scene: &mut Scene,
        selected_object: &mut Option<u32>,
    ) {
        egui::SidePanel::left("hierarchy_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.show_ui_content(ui, scene, selected_object);
            });
    }

    pub fn show_ui(
        &mut self,
        ui: &mut egui::Ui,
        scene: &mut Scene,
        selected_object: &mut Option<u32>,
    ) -> Option<u32> {
        self.show_ui_content(ui, scene, selected_object);
        *selected_object
    }

    fn show_ui_content(
        &mut self,
        ui: &mut egui::Ui,
        scene: &mut Scene,
        selected_object: &mut Option<u32>,
    ) {
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
                                obj.components.push(Component::Mesh {
                                    mesh_type: "Cube".to_string(),
                                });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Sphere").clicked() {
                            let id = scene.add_object("Sphere".to_string(), GameObjectType::Sphere);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh {
                                    mesh_type: "Sphere".to_string(),
                                });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Cylinder").clicked() {
                            let id =
                                scene.add_object("Cylinder".to_string(), GameObjectType::Cylinder);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh {
                                    mesh_type: "Cylinder".to_string(),
                                });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Plane").clicked() {
                            let id = scene.add_object("Plane".to_string(), GameObjectType::Plane);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh {
                                    mesh_type: "Plane".to_string(),
                                });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Physics", |ui| {
                        if ui.button("Rigid Body Sphere").clicked() {
                            let id = scene.add_object(
                                "RigidBody".to_string(),
                                GameObjectType::RigidBody(Shape::Sphere { radius: 1.0 }),
                            );
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Mesh {
                                    mesh_type: "Sphere".to_string(),
                                });
                                obj.components.push(Component::Renderer {
                                    material: "Default".to_string(),
                                    color: [0.8, 0.3, 0.3, 1.0],
                                });
                                obj.components.push(Component::RigidBody {
                                    shape: Shape::Sphere { radius: 1.0 },
                                    mass: 1.0,
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Soft Body").clicked() {
                            let id =
                                scene.add_object("SoftBody".to_string(), GameObjectType::SoftBody);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::SoftBodyComponent {
                                    particles: 100,
                                    stiffness: 0.8,
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Fluid Emitter").clicked() {
                            let _id = scene.add_object(
                                "FluidEmitter".to_string(),
                                GameObjectType::FluidEmitter,
                            );
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Lighting", |ui| {
                        if ui.button("Directional Light").clicked() {
                            let id = scene
                                .add_object("Directional Light".to_string(), GameObjectType::Light);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Light {
                                    light_type: "Directional".to_string(),
                                    intensity: 1.0,
                                    color: [1.0, 1.0, 1.0],
                                });
                            }
                            ui.close_menu();
                        }
                        if ui.button("Point Light").clicked() {
                            let id =
                                scene.add_object("Point Light".to_string(), GameObjectType::Light);
                            if let Some(obj) = scene.objects.get_mut(&id) {
                                obj.components.push(Component::Light {
                                    light_type: "Point".to_string(),
                                    intensity: 1.0,
                                    color: [1.0, 1.0, 1.0],
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
                                far: 1000.0,
                            });
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    ui.menu_button("Presets", |ui| {
                        if ui.button("Aquarium 3D").clicked() {
                            self.create_aquarium_3d(scene);
                            ui.close_menu();
                        }
                        if ui.button("Aquarium 2D").clicked() {
                            self.create_aquarium_2d(scene);
                            ui.close_menu();
                        }
                        if ui.button("Physics Playground").clicked() {
                            self.create_physics_playground(scene);
                            ui.close_menu();
                        }
                        if ui.button("Solar System").clicked() {
                            self.create_solar_system(scene);
                            ui.close_menu();
                        }
                        if ui.button("Particle System").clicked() {
                            self.create_particle_system(scene);
                            ui.close_menu();
                        }
                    });
                });

                ui.separator();

                // Object list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Get root objects (objects with no parent)
                    let root_objects: Vec<u32> = scene
                        .objects
                        .values()
                        .filter(|obj| obj.parent.is_none())
                        .map(|obj| obj.id)
                        .collect();

                    for object_id in root_objects {
                        self.show_object_tree(ui, scene, object_id, selected_object, 0);
                    }
                });
    }

    fn show_object_tree(
        &mut self,
        ui: &mut egui::Ui,
        scene: &mut Scene,
        object_id: u32,
        selected_object: &mut Option<u32>,
        depth: usize,
    ) {
        // Get object properties first to avoid borrowing conflicts
        let object_info = if let Some(object) = scene.objects.get(&object_id) {
            Some((
                object.name.clone(),
                object.enabled,
                object.visible,
                object.children.clone(),
            ))
        } else {
            None
        };

        if let Some((name, enabled, visible, children)) = object_info {
            // Filter check
            if !self.filter_text.is_empty()
                && !name
                    .to_lowercase()
                    .contains(&self.filter_text.to_lowercase())
            {
                return;
            }

            // Show inactive check
            if !self.show_inactive && !enabled {
                return;
            }

            let indent = (depth as f32) * 20.0;
            ui.indent(format!("object_{}", object_id), |ui| {
                ui.allocate_ui_with_layout(
                    [ui.available_width() - indent, 20.0].into(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // Expand/collapse triangle for objects with children
                        if !children.is_empty() {
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
                        let mut current_visible = visible;
                        if ui.checkbox(&mut current_visible, "").changed() {
                            if let Some(obj) = scene.objects.get_mut(&object_id) {
                                obj.visible = current_visible;
                            }
                        }

                        // Object name (selectable)
                        let is_selected = *selected_object == Some(object_id);
                        let response = ui.selectable_label(is_selected, &name);

                        if response.clicked() {
                            *selected_object = Some(object_id);
                        }

                        // Store actions to perform after UI rendering
                        let mut duplicate_requested = false;
                        let mut delete_requested = false;
                        let mut add_child_requested = false;

                        response.context_menu(|ui| {
                            if ui.button("Duplicate").clicked() {
                                duplicate_requested = true;
                                ui.close_menu();
                            }
                            if ui.button("Delete").clicked() {
                                delete_requested = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Add Child").clicked() {
                                add_child_requested = true;
                                ui.close_menu();
                            }
                        });

                        // Handle actions after UI
                        if duplicate_requested {
                            self.duplicate_object(scene, object_id);
                        }
                        if delete_requested {
                            scene.remove_object(object_id);
                            if *selected_object == Some(object_id) {
                                *selected_object = None;
                            }
                        }
                        if add_child_requested {
                            let child_id =
                                scene.add_object("Child".to_string(), GameObjectType::Empty);
                            scene.set_parent(child_id, Some(object_id));
                        }
                    },
                );
            });

            // Show children if expanded
            if self.expanded_objects.contains(&object_id) {
                for &child_id in &children {
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

    /// Create a 3D aquarium (box without top)
    fn create_aquarium_3d(&mut self, scene: &mut Scene) {
        // Create aquarium container
        let container_id = scene.add_object("Aquarium".to_string(), GameObjectType::Empty);

        // Bottom
        let bottom_id = scene.add_object("Bottom".to_string(), GameObjectType::Cube);
        if let Some(bottom) = scene.objects.get_mut(&bottom_id) {
            bottom.transform.position = Vec3::new(0.0, -2.5, 0.0);
            bottom.transform.scale = Vec3::new(10.0, 0.5, 10.0);
            bottom.components.push(Component::Mesh { mesh_type: "Cube".to_string() });
            bottom.components.push(Component::Renderer {
                material: "Sand".to_string(),
                color: [0.8, 0.7, 0.5, 1.0]
            });
        }
        scene.set_parent(bottom_id, Some(container_id));

        // Walls
        let wall_positions = [
            (Vec3::new(-5.0, 0.0, 0.0), Vec3::new(0.5, 5.0, 10.0)), // Left
            (Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.5, 5.0, 10.0)),  // Right
            (Vec3::new(0.0, 0.0, -5.0), Vec3::new(10.0, 5.0, 0.5)), // Back
            (Vec3::new(0.0, 0.0, 5.0), Vec3::new(10.0, 5.0, 0.5)),  // Front
        ];

        for (i, (pos, scale)) in wall_positions.iter().enumerate() {
            let wall_id = scene.add_object(format!("Wall_{}", i), GameObjectType::Cube);
            if let Some(wall) = scene.objects.get_mut(&wall_id) {
                wall.transform.position = *pos;
                wall.transform.scale = *scale;
                wall.components.push(Component::Mesh { mesh_type: "Cube".to_string() });
                wall.components.push(Component::Renderer {
                    material: "Glass".to_string(),
                    color: [0.7, 0.9, 1.0, 0.3],
                });
            }
            scene.set_parent(wall_id, Some(container_id));
        }

        // Add some fish
        for i in 0..5 {
            let fish_id = scene.add_object(format!("Fish_{}", i), GameObjectType::Sphere);
            if let Some(fish) = scene.objects.get_mut(&fish_id) {
                fish.transform.position = Vec3::new(
                    (i as f64 - 2.0) * 1.5,
                    1.0 + (i as f64 * 0.5),
                    (i as f64 - 2.0) * 0.8,
                );
                fish.transform.scale = Vec3::new(0.3, 0.2, 0.5);
                fish.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                fish.components.push(Component::Renderer {
                    material: "Fish".to_string(),
                    color: [1.0, 0.5, 0.0, 1.0],
                });
                fish.components.push(Component::RigidBody {
                    shape: Shape::Sphere { radius: 0.3 },
                    mass: 0.1,
                });
            }
            scene.set_parent(fish_id, Some(container_id));
        }
    }

    /// Create a 2D aquarium
    fn create_aquarium_2d(&mut self, scene: &mut Scene) {
        let container_id = scene.add_object("Aquarium 2D".to_string(), GameObjectType::Empty);

        // Bottom
        let bottom_id = scene.add_object("Bottom".to_string(), GameObjectType::Plane);
        if let Some(bottom) = scene.objects.get_mut(&bottom_id) {
            bottom.transform.position = Vec3::new(0.0, -3.0, 0.0);
            bottom.transform.scale = Vec3::new(8.0, 1.0, 1.0);
            bottom.components.push(Component::Renderer {
                material: "Sand".to_string(),
                color: [0.8, 0.7, 0.5, 1.0],
            });
        }
        scene.set_parent(bottom_id, Some(container_id));

        // Side walls
        let left_wall = scene.add_object("Left Wall".to_string(), GameObjectType::Plane);
        if let Some(wall) = scene.objects.get_mut(&left_wall) {
            wall.transform.position = Vec3::new(-4.0, 0.0, 0.0);
            wall.transform.scale = Vec3::new(1.0, 6.0, 1.0);
            wall.components.push(Component::Renderer {
                material: "Glass".to_string(),
                color: [0.7, 0.9, 1.0, 0.5],
            });
        }
        scene.set_parent(left_wall, Some(container_id));

        let right_wall = scene.add_object("Right Wall".to_string(), GameObjectType::Plane);
        if let Some(wall) = scene.objects.get_mut(&right_wall) {
            wall.transform.position = Vec3::new(4.0, 0.0, 0.0);
            wall.transform.scale = Vec3::new(1.0, 6.0, 1.0);
            wall.components.push(Component::Renderer {
                material: "Glass".to_string(),
                color: [0.7, 0.9, 1.0, 0.5],
            });
        }
        scene.set_parent(right_wall, Some(container_id));

        // Add fish
        for i in 0..3 {
            let fish_id = scene.add_object(format!("Fish_{}", i), GameObjectType::Sphere);
            if let Some(fish) = scene.objects.get_mut(&fish_id) {
                fish.transform.position = Vec3::new((i as f64 - 1.0) * 2.0, 0.0, 0.0);
                fish.transform.scale = Vec3::new(0.4, 0.3, 0.3);
                fish.components.push(Component::Renderer {
                    material: "Fish".to_string(),
                    color: [0.0, 0.8, 1.0, 1.0],
                });
            }
            scene.set_parent(fish_id, Some(container_id));
        }
    }

    /// Create a physics playground
    fn create_physics_playground(&mut self, scene: &mut Scene) {
        let container_id = scene.add_object("Physics Playground".to_string(), GameObjectType::Empty);

        // Ground plane
        let ground_id = scene.add_object("Ground".to_string(), GameObjectType::Plane);
        if let Some(ground) = scene.objects.get_mut(&ground_id) {
            ground.transform.position = Vec3::new(0.0, -5.0, 0.0);
            ground.transform.scale = Vec3::new(20.0, 1.0, 20.0);
            ground.components.push(Component::Renderer {
                material: "Ground".to_string(),
                color: [0.2, 0.8, 0.2, 1.0],
            });
            ground.components.push(Component::RigidBody {
                shape: Shape::Box { size: Vec3::new(20.0, 1.0, 20.0) },
                mass: 0.0, // Static body
            });
        }
        scene.set_parent(ground_id, Some(container_id));

        // Stack of cubes
        for i in 0..5 {
            let cube_id = scene.add_object(format!("Cube_{}", i), GameObjectType::Cube);
            if let Some(cube) = scene.objects.get_mut(&cube_id) {
                cube.transform.position = Vec3::new(0.0, i as f64 * 2.1, 0.0);
                cube.transform.scale = Vec3::new(1.0, 1.0, 1.0);
                cube.components.push(Component::Mesh { mesh_type: "Cube".to_string() });
                cube.components.push(Component::Renderer {
                    material: "Metal".to_string(),
                    color: [0.8, 0.8, 0.9, 1.0],
                });
                cube.components.push(Component::RigidBody {
                    shape: Shape::Box { size: Vec3::new(1.0, 1.0, 1.0) },
                    mass: 1.0,
                });
            }
            scene.set_parent(cube_id, Some(container_id));
        }

        // Rolling spheres
        for i in 0..3 {
            let sphere_id = scene.add_object(format!("Sphere_{}", i), GameObjectType::Sphere);
            if let Some(sphere) = scene.objects.get_mut(&sphere_id) {
                sphere.transform.position = Vec3::new((i as f64 - 1.0) * 3.0, 10.0, 5.0);
                sphere.transform.scale = Vec3::new(0.8, 0.8, 0.8);
                sphere.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                sphere.components.push(Component::Renderer {
                    material: "Rubber".to_string(),
                    color: [1.0, 0.2, 0.2, 1.0],
                });
                sphere.components.push(Component::RigidBody {
                    shape: Shape::Sphere { radius: 0.8 },
                    mass: 0.5,
                });
            }
            scene.set_parent(sphere_id, Some(container_id));
        }
    }

    /// Create a simple solar system
    fn create_solar_system(&mut self, scene: &mut Scene) {
        let system_id = scene.add_object("Solar System".to_string(), GameObjectType::Empty);

        // Sun
        let sun_id = scene.add_object("Sun".to_string(), GameObjectType::Sphere);
        if let Some(sun) = scene.objects.get_mut(&sun_id) {
            sun.transform.position = Vec3::new(0.0, 0.0, 0.0);
            sun.transform.scale = Vec3::new(2.0, 2.0, 2.0);
            sun.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
            sun.components.push(Component::Renderer {
                material: "Sun".to_string(),
                color: [1.0, 0.8, 0.0, 1.0],
            });
            sun.components.push(Component::Light {
                light_type: "Point".to_string(),
                intensity: 2.0,
                color: [1.0, 0.9, 0.7],
            });
        }
        scene.set_parent(sun_id, Some(system_id));

        // Planets
        let planet_data = [
            ("Mercury", 3.0, 0.3, [0.7, 0.7, 0.7, 1.0]),
            ("Venus", 4.5, 0.4, [1.0, 0.8, 0.4, 1.0]),
            ("Earth", 6.0, 0.5, [0.2, 0.6, 1.0, 1.0]),
            ("Mars", 8.0, 0.4, [0.9, 0.3, 0.2, 1.0]),
        ];

        for (name, distance, size, color) in planet_data.iter() {
            let planet_id = scene.add_object(name.to_string(), GameObjectType::Sphere);
            if let Some(planet) = scene.objects.get_mut(&planet_id) {
                planet.transform.position = Vec3::new(*distance, 0.0, 0.0);
                planet.transform.scale = Vec3::new(*size, *size, *size);
                planet.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                planet.components.push(Component::Renderer {
                    material: "Planet".to_string(),
                    color: *color,
                });
            }
            scene.set_parent(planet_id, Some(system_id));
        }
    }

    /// Create a particle system demonstration
    fn create_particle_system(&mut self, scene: &mut Scene) {
        let system_id = scene.add_object("Particle System".to_string(), GameObjectType::Empty);

        // Emitter
        let emitter_id = scene.add_object("Emitter".to_string(), GameObjectType::FluidEmitter);
        if let Some(emitter) = scene.objects.get_mut(&emitter_id) {
            emitter.transform.position = Vec3::new(0.0, 5.0, 0.0);
            emitter.transform.scale = Vec3::new(0.5, 0.5, 0.5);
            emitter.components.push(Component::Renderer {
                material: "Emitter".to_string(),
                color: [1.0, 1.0, 0.0, 1.0],
            });
        }
        scene.set_parent(emitter_id, Some(system_id));

        // Create some sample particles
        for i in 0..20 {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / 20.0;
            let radius = 2.0;
            let particle_id = scene.add_object(format!("Particle_{}", i), GameObjectType::Sphere);
            if let Some(particle) = scene.objects.get_mut(&particle_id) {
                particle.transform.position = Vec3::new(
                    angle.cos() * radius,
                    3.0 + (i as f64 * 0.1),
                    angle.sin() * radius,
                );
                particle.transform.scale = Vec3::new(0.1, 0.1, 0.1);
                particle.components.push(Component::Mesh { mesh_type: "Sphere".to_string() });
                particle.components.push(Component::Renderer {
                    material: "Particle".to_string(),
                    color: [1.0, 0.5, 1.0, 0.8],
                });
                particle.components.push(Component::RigidBody {
                    shape: Shape::Sphere { radius: 0.1 },
                    mass: 0.01,
                });
            }
            scene.set_parent(particle_id, Some(system_id));
        }
    }

    // ...existing methods...
}

impl Default for ObjectHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
