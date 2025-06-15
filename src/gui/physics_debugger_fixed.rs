// Physics Debugger Implementation
use super::*;
use crate::physics::math::Vec3;
use crate::physics::PhysicsWorld;
use crate::ecs::World;

/// Physics debugging and visualization panel
pub struct PhysicsDebugger {
    /// Show collision shapes
    show_colliders: bool,
    /// Show velocity vectors
    show_velocities: bool,
    /// Show force vectors
    show_forces: bool,
    /// Show contact points
    show_contacts: bool,
    /// Show center of mass
    show_center_of_mass: bool,
    /// Show bounding boxes
    show_bounding_boxes: bool,
    /// Wireframe mode for physics shapes
    wireframe_physics: bool,
    /// Physics simulation settings
    gravity: Vec3,
    time_scale: f32,
    substeps: u32,
    /// Performance metrics
    physics_fps: f32,
    collision_count: u32,
    rigid_body_count: u32,
    /// Collision filtering
    selected_layer: Option<u32>,
    layer_names: std::collections::HashMap<u32, String>,
    /// Force application
    manual_force: Vec3,
    force_position: Vec3,
    applying_force: bool,
}

impl PhysicsDebugger {
    pub fn new() -> Self {
        Self {
            show_colliders: true,
            show_velocities: false,
            show_forces: false,
            show_contacts: true,
            show_center_of_mass: false,
            show_bounding_boxes: false,
            wireframe_physics: false,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_scale: 1.0,
            substeps: 4,
            physics_fps: 60.0,
            collision_count: 0,
            rigid_body_count: 0,
            selected_layer: None,
            layer_names: std::collections::HashMap::new(),
            manual_force: Vec3::new(0.0, 0.0, 0.0),
            force_position: Vec3::new(0.0, 0.0, 0.0),
            applying_force: false,
        }
    }

    /// Main UI method called by the Unity Layout
    pub fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        // Get physics world from ECS world
        if let Some(physics_world) = world.get_physics_world_mut() {
            self.show_ui(ui, physics_world);

            // Update physics debug information
            self.update(physics_world, 1.0 / 60.0); // Use fixed time step for now
        } else {
            ui.heading("🔧 Physics Debugger");
            ui.label("No physics world available");

            if ui.button("Create Physics World").clicked() {
                // Create a new physics world and add it to the ECS world
                world.create_physics_world();
            }
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui, physics_world: &mut PhysicsWorld) {
        ui.heading("🔧 Physics Debugger");

        // Physics visualization options
        self.show_visualization_options(ui);

        ui.separator();

        // Physics simulation settings
        self.show_simulation_settings(ui, physics_world);

        ui.separator();

        // Performance metrics
        self.show_performance_metrics(ui);

        ui.separator();

        // Collision layers
        self.show_collision_layers(ui);

        ui.separator();

        // Manual force application
        self.show_force_controls(ui);

        ui.separator();

        // Rigid body list
        self.show_rigid_body_list(ui, physics_world);
    }

    /// Update debug information from physics world
    pub fn update(&mut self, physics_world: &mut PhysicsWorld, dt: f32) {
        // Update performance metrics
        self.physics_fps = 1.0 / dt;
        self.collision_count = physics_world.get_collision_count();
        self.rigid_body_count = physics_world.get_rigid_bodies().len() as u32;

        // Apply manual force if needed
        if self.applying_force {
            // Find the closest rigid body to apply force to
            if let Some((closest_idx, _)) = physics_world.get_rigid_bodies().iter().enumerate()
                .min_by(|(_, a), (_, b)| {
                    let dist_a = a.position.distance_to(self.force_position);
                    let dist_b = b.position.distance_to(self.force_position);
                    dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                // Apply the force to the rigid body
                if let Some(body) = physics_world.rigid_bodies.get_mut(closest_idx) {
                    body.apply_force_at_point(self.manual_force, self.force_position);
                }
            }
            self.applying_force = false;
        }
    }

    fn show_visualization_options(&mut self, ui: &mut egui::Ui) {
        ui.heading("Visualization");

        ui.checkbox(&mut self.show_colliders, "Show Colliders");
        ui.checkbox(&mut self.show_velocities, "Show Velocity Vectors");
        ui.checkbox(&mut self.show_forces, "Show Force Vectors");
        ui.checkbox(&mut self.show_contacts, "Show Contact Points");
        ui.checkbox(&mut self.show_center_of_mass, "Show Center of Mass");
        ui.checkbox(&mut self.show_bounding_boxes, "Show Bounding Boxes");
        ui.checkbox(&mut self.wireframe_physics, "Wireframe Physics Shapes");

        // Color settings for debug visualization
        ui.horizontal(|ui| {
            ui.label("Debug Colors:");
            if ui.small_button("Reset").clicked() {
                // Reset to default debug colors
            }
        });
    }

    fn show_simulation_settings(&mut self, ui: &mut egui::Ui, physics_world: &mut PhysicsWorld) {
        ui.heading("Simulation Settings");

        // Gravity controls
        ui.horizontal(|ui| {
            ui.label("Gravity:");
            ui.add(
                egui::DragValue::new(&mut self.gravity.x)
                    .prefix("X: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.gravity.y)
                    .prefix("Y: ")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.gravity.z)
                    .prefix("Z: ")
                    .speed(0.1),
            );
        });

        // Quick gravity presets
        ui.horizontal(|ui| {
            if ui.button("Earth").clicked() {
                self.gravity = Vec3::new(0.0, -9.81, 0.0);
            }
            if ui.button("Moon").clicked() {
                self.gravity = Vec3::new(0.0, -1.62, 0.0);
            }
            if ui.button("Mars").clicked() {
                self.gravity = Vec3::new(0.0, -3.71, 0.0);
            }
            if ui.button("Zero G").clicked() {
                self.gravity = Vec3::new(0.0, 0.0, 0.0);
            }
        });

        // Time scale
        ui.horizontal(|ui| {
            ui.label("Time Scale:");
            ui.add(egui::Slider::new(&mut self.time_scale, 0.0..=5.0));
            if ui.button("Reset").clicked() {
                self.time_scale = 1.0;
            }
        });

        // Substeps
        ui.horizontal(|ui| {
            ui.label("Substeps:");
            ui.add(egui::DragValue::new(&mut self.substeps).range(1..=10));
        });

        // Apply settings to physics world
        physics_world.set_gravity(self.gravity);
        physics_world.set_time_scale(self.time_scale.into());
    }

    fn show_performance_metrics(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance");

        ui.horizontal(|ui| {
            ui.label(format!("Physics FPS: {:.1}", self.physics_fps));
            ui.separator();
            ui.label(format!("Rigid Bodies: {}", self.rigid_body_count));
            ui.separator();
            ui.label(format!("Collisions: {}", self.collision_count));
        });

        // Performance graph would go here
        ui.label("📊 Performance graphs coming soon...");
    }

    fn show_collision_layers(&mut self, ui: &mut egui::Ui) {
        ui.heading("Collision Layers");

        // Layer selection
        ui.horizontal(|ui| {
            ui.label("Filter by layer:");
            egui::ComboBox::from_id_salt("collision_layer")
                .selected_text(
                    self.selected_layer
                        .and_then(|layer| self.layer_names.get(&layer))
                        .map(|name| name.as_str())
                        .unwrap_or("All Layers"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.selected_layer.is_none(), "All Layers")
                        .clicked()
                    {
                        self.selected_layer = None;
                    }

                    for (&layer_id, layer_name) in &self.layer_names {
                        if ui
                            .selectable_label(self.selected_layer == Some(layer_id), layer_name)
                            .clicked()
                        {
                            self.selected_layer = Some(layer_id);
                        }
                    }
                });
        });

        // Layer management
        ui.horizontal(|ui| {
            if ui.button("Add Layer").clicked() {
                let new_id = self.layer_names.len() as u32;
                self.layer_names.insert(new_id, format!("Layer {}", new_id));
            }

            if ui.button("Edit Layers").clicked() {
                // Open layer editor dialog
            }
        });

        // Collision matrix visualization
        if !self.layer_names.is_empty() {
            ui.label("Collision Matrix:");
            egui::Grid::new("collision_matrix").show(ui, |ui| {
                // Header row
                ui.label("");
                for layer_name in self.layer_names.values() {
                    ui.label(layer_name);
                }
                ui.end_row();

                // Matrix rows
                for (_layer_id, layer_name) in &self.layer_names {
                    ui.label(layer_name);
                    for _other_layer_id in self.layer_names.keys() {
                        let mut collides = true; // Get from physics world
                        if ui.checkbox(&mut collides, "").clicked() {
                            // Update collision matrix in physics world
                        }
                    }
                    ui.end_row();
                }
            });
        }
    }

    fn show_force_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Manual Forces");

        ui.horizontal(|ui| {
            ui.label("Force:");
            ui.add(egui::DragValue::new(&mut self.manual_force.x).prefix("X: "));
            ui.add(egui::DragValue::new(&mut self.manual_force.y).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut self.manual_force.z).prefix("Z: "));
        });

        ui.horizontal(|ui| {
            ui.label("Position:");
            ui.add(egui::DragValue::new(&mut self.force_position.x).prefix("X: "));
            ui.add(egui::DragValue::new(&mut self.force_position.y).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut self.force_position.z).prefix("Z: "));
        });

        ui.horizontal(|ui| {
            if ui.button("Apply Force").clicked() {
                self.applying_force = true;
            }

            if ui.button("Apply Impulse").clicked() {
                // Apply impulse to physics world
            }

            if ui.button("Clear Forces").clicked() {
                self.manual_force = Vec3::new(0.0, 0.0, 0.0);
            }
        });

        // Force presets
        ui.horizontal(|ui| {
            ui.label("Presets:");
            if ui.button("Upward").clicked() {
                self.manual_force = Vec3::new(0.0, 10.0, 0.0);
            }
            if ui.button("Forward").clicked() {
                self.manual_force = Vec3::new(0.0, 0.0, 10.0);
            }
            if ui.button("Explosion").clicked() {
                self.manual_force = Vec3::new(5.0, 5.0, 5.0);
            }
        });
    }

    fn show_rigid_body_list(&mut self, ui: &mut egui::Ui, physics_world: &PhysicsWorld) {
        ui.heading("Rigid Bodies");

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let bodies = physics_world.get_rigid_bodies();

                for (i, body) in bodies.iter().enumerate() {
                    egui::CollapsingHeader::new(format!("Body {}", i)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("Mass: {:.1}", body.mass));
                            ui.label(format!("Static: {}", body.is_static));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.label(format!("({:.1}, {:.1}, {:.1})",
                                            body.position.x, body.position.y, body.position.z));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Velocity:");
                            ui.label(format!("({:.1}, {:.1}, {:.1})",
                                            body.velocity.x, body.velocity.y, body.velocity.z));
                        });

                        ui.horizontal(|ui| {
                            ui.label(format!("Restitution: {:.2}", body.restitution));
                            ui.label(format!("Friction: {:.2}", body.friction));
                        });

                        match &body.shape {
                            crate::physics::rigid_body::Shape::Sphere { radius } => {
                                ui.label(format!("Shape: Sphere (radius: {:.2})", radius));
                            },
                            crate::physics::rigid_body::Shape::Box { size } => {
                                ui.label(format!("Shape: Box ({:.2} x {:.2} x {:.2})",
                                                size.x, size.y, size.z));
                            },
                            _ => {
                                ui.label("Shape: Other");
                            }
                        }
                    });
                }
            });
    }
}
