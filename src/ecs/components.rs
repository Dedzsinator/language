// ECS Components for Physics Engine
use bevy_ecs::prelude::*;
use crate::physics::math::*;
use crate::physics::constraints::Constraint;
use crate::physics::rigid_body::Shape;
use std::collections::HashMap;

/// PhysicsTransform component - position, rotation, scale
#[derive(Component, Debug, Clone)]
pub struct PhysicsTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl PhysicsTransform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::identity(),
            scale: Vec3::one(),
        }
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position = self.position + translation;
    }
}

/// Velocity component - linear and angular velocity
#[derive(Component, Debug, Clone)]
pub struct VelocityComponent {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Default for VelocityComponent {
    fn default() -> Self {
        Self {
            linear: Vec3::zero(),
            angular: Vec3::zero(),
        }
    }
}

/// Rigid body physics component
#[derive(Component, Debug, Clone)]
pub struct RigidBodyComponent {
    pub mass: f64,
    pub inv_mass: f64,
    pub shape: Shape,
    pub is_static: bool,
    pub restitution: f64,
    pub friction: f64,
    pub damping: f64,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
}

impl RigidBodyComponent {
    pub fn new(mass: f64, shape: Shape) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            mass,
            inv_mass,
            shape,
            is_static: mass == 0.0,
            restitution: 0.5,
            friction: 0.6,
            damping: 0.99,
            force_accumulator: Vec3::zero(),
            torque_accumulator: Vec3::zero(),
        }
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.force_accumulator = self.force_accumulator + force;
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        self.torque_accumulator = self.torque_accumulator + torque;
    }

    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec3::zero();
        self.torque_accumulator = Vec3::zero();
    }
}

/// Soft body particle
#[derive(Debug, Clone)]
pub struct SoftBodyParticle {
    pub position: Vec3,
    pub old_position: Vec3,
    pub velocity: Vec3,
    pub mass: f64,
    pub radius: f64,
    pub pinned: bool,
}

impl SoftBodyParticle {
    pub fn new(position: Vec3, mass: f64, radius: f64) -> Self {
        Self {
            position,
            old_position: position,
            velocity: Vec3::zero(),
            mass,
            radius,
            pinned: false,
        }
    }
}

/// Soft body physics component
#[derive(Component, Debug, Clone)]
pub struct SoftBodyComponent {
    pub particles: Vec<SoftBodyParticle>,
    pub constraints: Vec<SoftBodyConstraint>,
    pub stiffness: f64,
    pub damping: f64,
    pub iterations: usize,
}

impl SoftBodyComponent {
    pub fn new(particles: Vec<SoftBodyParticle>) -> Self {
        Self {
            particles,
            constraints: Vec::new(),
            stiffness: 0.8,
            damping: 0.99,
            iterations: 4,
        }
    }

    pub fn add_distance_constraint(&mut self, particle_a: usize, particle_b: usize, stiffness: f64) {
        if particle_a < self.particles.len() && particle_b < self.particles.len() {
            let rest_length = self.particles[particle_a].position
                .distance_to(self.particles[particle_b].position);
            
            self.constraints.push(SoftBodyConstraint::Distance {
                particle_a,
                particle_b,
                rest_length,
                stiffness,
            });
        }
    }
}

/// Soft body constraints
#[derive(Debug, Clone)]
pub enum SoftBodyConstraint {
    Distance {
        particle_a: usize,
        particle_b: usize,
        rest_length: f64,
        stiffness: f64,
    },
    Bend {
        particles: [usize; 4],
        rest_angle: f64,
        stiffness: f64,
    },
    Volume {
        particles: Vec<usize>,
        rest_volume: f64,
        stiffness: f64,
    },
}

/// Fluid particle
#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f64,
    pub density: f64,
    pub pressure: f64,
    pub neighbors: Vec<usize>,
}

impl FluidParticle {
    pub fn new(position: Vec3, mass: f64) -> Self {
        Self {
            position,
            velocity: Vec3::zero(),
            mass,
            density: 0.0,
            pressure: 0.0,
            neighbors: Vec::new(),
        }
    }
}

/// Fluid system component (SPH - Smoothed Particle Hydrodynamics)
#[derive(Component, Debug, Clone)]
pub struct FluidComponent {
    pub particles: Vec<FluidParticle>,
    pub rest_density: f64,
    pub gas_constant: f64,
    pub viscosity: f64,
    pub smoothing_radius: f64,
    pub surface_tension: f64,
}

impl FluidComponent {
    pub fn new(particles: Vec<FluidParticle>) -> Self {
        Self {
            particles,
            rest_density: 1000.0,
            gas_constant: 7.0,
            viscosity: 0.1,
            smoothing_radius: 1.0,
            surface_tension: 0.0728,
        }
    }
}

/// Constraint component for connecting entities
#[derive(Component, Debug, Clone)]
pub struct ConstraintComponent {
    pub constraints: Vec<Constraint>,
}

impl ConstraintComponent {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
}

impl Default for ConstraintComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics object type marker
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsObject {
    RigidBody,
    SoftBody,
    Fluid,
    Static,
}

/// Collision shape component
#[derive(Component, Debug, Clone)]
pub struct ColliderComponent {
    pub shape: Shape,
    pub is_trigger: bool,
    pub collision_groups: u32,
    pub collision_mask: u32,
}

impl ColliderComponent {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            is_trigger: false,
            collision_groups: 0xFFFFFFFF,
            collision_mask: 0xFFFFFFFF,
        }
    }
}

/// Material properties component
#[derive(Component, Debug, Clone)]
pub struct MaterialComponent {
    pub density: f64,
    pub restitution: f64,
    pub friction: f64,
    pub damping: f64,
    pub thermal_conductivity: f64,
    pub electrical_resistance: f64,
}

impl Default for MaterialComponent {
    fn default() -> Self {
        Self {
            density: 1.0,
            restitution: 0.5,
            friction: 0.6,
            damping: 0.99,
            thermal_conductivity: 0.0,
            electrical_resistance: f64::INFINITY,
        }
    }
}

/// Physics scripting hooks
#[derive(Component, Debug, Clone)]
pub struct ScriptHooksComponent {
    pub on_collision: Option<String>, // Script function name
    pub on_update: Option<String>,
    pub on_spawn: Option<String>,
    pub custom_force: Option<String>,
}

impl Default for ScriptHooksComponent {
    fn default() -> Self {
        Self {
            on_collision: None,
            on_update: None,
            on_spawn: None,
            custom_force: None,
        }
    }
}

/// Tag for entities that should be rendered
#[derive(Component, Debug, Clone)]
pub struct RenderableComponent {
    pub color: [f32; 4],
    pub visible: bool,
    pub wireframe: bool,
}

impl Default for RenderableComponent {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            visible: true,
            wireframe: false,
        }
    }
}
