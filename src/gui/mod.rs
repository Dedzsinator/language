use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use crate::physics::*;
use crate::physics::math::*;
use crate::physics::rigid_body::*;
use crate::physics::soft_body::*;
use crate::physics::fluid::*;
use crate::physics::constraints::*;

/// Main application structure for the physics visualization GUI
pub struct PhysicsVisualizationApp {
    physics_world: PhysicsWorld,
    is_simulating: bool,
    time_step: f64,
    time: f64,
    trace_points: Vec<(f64, f64)>, // For plotting trajectories
    _selected_object: Option<usize>,
    show_vectors: bool,
    show_constraints: bool,
    gravity: [f64; 3],
}

impl Default for PhysicsVisualizationApp {
    fn default() -> Self {
        Self {
            physics_world: PhysicsWorld::new(),
            is_simulating: false,
            time_step: 0.016, // ~60 FPS
            time: 0.0,
            trace_points: Vec::new(),
            _selected_object: None,
            show_vectors: true,
            show_constraints: true,
            gravity: [0.0, -9.81, 0.0],
        }
    }
}

impl eframe::App for PhysicsVisualizationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for smooth animation
        ctx.request_repaint();

        // Update physics simulation
        if self.is_simulating {
            self.physics_world.step();
            self.time += self.time_step;
            
            // Update trace points for plotting
            self.update_trace_points();
        }

        // Create the main UI
        self.create_control_panel(ctx);
        self.create_physics_view(ctx);
        self.create_properties_panel(ctx);
    }
}

impl PhysicsVisualizationApp {
    /// Create the control panel with simulation controls
    fn create_control_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("control_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Simulation controls
                if ui.button(if self.is_simulating { "⏸ Pause" } else { "▶ Play" }).clicked() {
                    self.is_simulating = !self.is_simulating;
                }
                
                if ui.button("⏹ Stop").clicked() {
                    self.is_simulating = false;
                    self.time = 0.0;
                    self.trace_points.clear();
                    // Reset physics world (could implement reset method)
                }
                
                if ui.button("⏮ Step").clicked() {
                    self.physics_world.step();
                    self.time += self.time_step;
                    self.update_trace_points();
                }

                ui.separator();

                // Time step control
                ui.label("Time Step:");
                ui.add(egui::Slider::new(&mut self.time_step, 0.001..=0.1).text("s"));

                ui.separator();

                // Visualization options
                ui.checkbox(&mut self.show_vectors, "Show Vectors");
                ui.checkbox(&mut self.show_constraints, "Show Constraints");

                ui.separator();

                ui.label(format!("Time: {:.2}s", self.time));
            });
        });
    }

    /// Create the main physics visualization view
    fn create_physics_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Physics Simulation");
            
            // Create a 2D plot for physics visualization
            Plot::new("physics_plot")
                .view_aspect(1.0)
                .data_aspect(1.0)
                .show(ui, |plot_ui| {
                    // Draw rigid bodies
                    self.draw_rigid_bodies(plot_ui);
                    
                    // Draw soft bodies
                    self.draw_soft_bodies(plot_ui);
                    
                    // Draw fluid particles
                    self.draw_fluid_particles(plot_ui);
                    
                    // Draw constraints if enabled
                    if self.show_constraints {
                        self.draw_constraints(plot_ui);
                    }
                    
                    // Draw trajectory traces
                    if !self.trace_points.is_empty() {
                        let trace_line = Line::new(PlotPoints::from(
                            self.trace_points.iter().map(|(x, y)| [*x, *y]).collect::<Vec<_>>()
                        ))
                        .color(egui::Color32::RED);
                        plot_ui.line(trace_line);
                    }
                });
        });
    }

    /// Create the properties panel for object details
    fn create_properties_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("properties_panel").show(ctx, |ui| {
            ui.heading("Properties");
            
            // Gravity controls
            ui.group(|ui| {
                ui.label("Gravity");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut self.gravity[0]).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.gravity[1]).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut self.gravity[2]).speed(0.1));
                });
                
                if ui.button("Apply Gravity").clicked() {
                    // Apply gravity to physics world
                    // This would require implementing a method to update gravity
                }
            });

            ui.separator();

            // Object management
            ui.group(|ui| {
                ui.label("Add Objects");
                
                if ui.button("Add Rigid Body").clicked() {
                    self.add_sample_rigid_body();
                }
                
                if ui.button("Add Soft Body").clicked() {
                    self.add_sample_soft_body();
                }
                
                if ui.button("Add Fluid System").clicked() {
                    self.add_sample_fluid_system();
                }
            });

            ui.separator();

            // Statistics
            ui.group(|ui| {
                ui.label("Statistics");
                ui.label(format!("Rigid Bodies: {}", self.physics_world.rigid_bodies.len()));
                ui.label(format!("Soft Bodies: {}", self.physics_world.soft_bodies.len()));
                ui.label(format!("Fluid Systems: {}", self.physics_world.fluid_systems.len()));
                ui.label(format!("Constraints: {}", self.physics_world.constraints.len()));
            });
        });
    }

    /// Draw rigid bodies on the plot
    fn draw_rigid_bodies(&self, plot_ui: &mut egui_plot::PlotUi) {
        for (i, body) in self.physics_world.rigid_bodies.iter().enumerate() {
            let pos = body.position;
            let points = PlotPoints::from(vec![[pos.x, pos.y]]);
            
            // Draw as a circle (representing the body)
            plot_ui.points(
                egui_plot::Points::new(points)
                    .color(egui::Color32::BLUE)
                    .radius(5.0)
                    .name(format!("RigidBody_{}", i))
            );

            // Draw velocity vector if enabled
            if self.show_vectors {
                let vel_end = [pos.x + body.velocity.x * 0.1, pos.y + body.velocity.y * 0.1];
                let velocity_line = Line::new(PlotPoints::from(vec![[pos.x, pos.y], vel_end]))
                    .color(egui::Color32::GREEN);
                plot_ui.line(velocity_line);
            }
        }
    }

    /// Draw soft bodies on the plot
    fn draw_soft_bodies(&self, plot_ui: &mut egui_plot::PlotUi) {
        for (i, soft_body) in self.physics_world.soft_bodies.iter().enumerate() {
            // Draw particles
            let points: Vec<[f64; 2]> = soft_body.particles.iter()
                .map(|p| [p.position.x, p.position.y])
                .collect();
            
            if !points.is_empty() {
                plot_ui.points(
                    egui_plot::Points::new(PlotPoints::from(points))
                        .color(egui::Color32::YELLOW)
                        .radius(3.0)
                        .name(format!("SoftBody_{}", i))
                );
            }

            // Draw edges/constraints connecting particles
            if self.show_constraints {
                // This would need to iterate through the soft body's internal constraints
                // For now, we'll skip this as it requires more detailed soft body structure
            }
        }
    }

    /// Draw fluid particles on the plot
    fn draw_fluid_particles(&self, plot_ui: &mut egui_plot::PlotUi) {
        for (i, fluid_system) in self.physics_world.fluid_systems.iter().enumerate() {
            let points: Vec<[f64; 2]> = fluid_system.particles.iter()
                .map(|p| [p.position.x, p.position.y])
                .collect();
            
            if !points.is_empty() {
                plot_ui.points(
                    egui_plot::Points::new(PlotPoints::from(points))
                        .color(egui::Color32::LIGHT_BLUE)
                        .radius(2.0)
                        .name(format!("FluidSystem_{}", i))
                );
            }
        }
    }

    /// Draw constraints between objects
    fn draw_constraints(&self, plot_ui: &mut egui_plot::PlotUi) {
        for constraint in &self.physics_world.constraints {
            match constraint {
                Constraint::Distance { body_a, body_b, .. } => {
                    if let (Some(pos_a), Some(pos_b)) = 
                        (self.get_body_position(body_a), self.get_body_position(body_b)) {
                        let constraint_line = Line::new(PlotPoints::from(vec![
                            [pos_a.x, pos_a.y],
                            [pos_b.x, pos_b.y]
                        ]))
                        .color(egui::Color32::GRAY);
                        plot_ui.line(constraint_line);
                    }
                }
                _ => {
                    // Handle other constraint types as needed
                }
            }
        }
    }

    /// Get the position of a constraint body
    fn get_body_position(&self, body: &ConstraintBody) -> Option<Vec3> {
        match body {
            ConstraintBody::RigidBody(index) => {
                self.physics_world.rigid_bodies.get(*index).map(|b| b.position)
            }
            ConstraintBody::SoftBodyParticle(body_index, particle_index) => {
                self.physics_world.soft_bodies.get(*body_index)
                    .and_then(|sb| sb.particles.get(*particle_index))
                    .map(|p| p.position)
            }
            ConstraintBody::StaticPoint(pos) => Some(*pos),
        }
    }

    /// Update trajectory trace points
    fn update_trace_points(&mut self) {
        // Add current position of first rigid body to trace (if any)
        if let Some(first_body) = self.physics_world.rigid_bodies.first() {
            self.trace_points.push((first_body.position.x, first_body.position.y));
            
            // Limit trace length to prevent memory issues
            if self.trace_points.len() > 1000 {
                self.trace_points.remove(0);
            }
        }
    }

    /// Add a sample rigid body for testing
    fn add_sample_rigid_body(&mut self) {
        let position = Vec3::new(
            (self.physics_world.rigid_bodies.len() as f64) * 2.0,
            5.0,
            0.0
        );
        let shape = Shape::Sphere { radius: 1.0 };
        let mut body = RigidBody::new(shape, 1.0, position);
        body.velocity = Vec3::new(0.0, 0.0, 0.0);
        
        self.physics_world.rigid_bodies.push(body);
    }

    /// Add a sample soft body for testing
    fn add_sample_soft_body(&mut self) {
        let soft_body = SoftBody::create_cloth(10, 10, 1.0, 1.0);
        
        self.physics_world.soft_bodies.push(soft_body);
    }

    /// Add a sample fluid system for testing
    fn add_sample_fluid_system(&mut self) {
        let mut fluid_system = FluidSystem::new(
            1000.0, // Density
            0.1,    // Smoothing length
            math::AABB::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0))
        );
        
        // Add some particles
        for i in 0..10 {
            for j in 0..10 {
                fluid_system.add_particle(
                    Vec3::new(i as f64 * 0.2 - 1.0, j as f64 * 0.2 + 1.0, 0.0),
                    0.1 // Mass
                );
            }
        }
        
        self.physics_world.fluid_systems.push(fluid_system);
    }
}

/// Launch the physics visualization GUI
pub fn launch_physics_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Physics Engine Visualization",
        options,
        Box::new(|_cc| Box::new(PhysicsVisualizationApp::default())),
    )
}
