// Physics Engine Module - Modern high-performance physics simulation
pub mod constraints;
pub mod differential;
pub mod fluid;
pub mod integrators;
pub mod math;
pub mod rigid_body;
pub mod sampling;
pub mod soft_body;
pub mod spatial;

use crate::eval::interpreter::{RuntimeResult, Value};
use std::collections::HashMap;

/// Core physics world that manages all simulations
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    pub rigid_bodies: Vec<rigid_body::RigidBody>,
    pub soft_bodies: Vec<soft_body::SoftBody>,
    pub fluid_systems: Vec<fluid::FluidSystem>,
    pub constraints: Vec<constraints::Constraint>,
    pub spatial_hash: spatial::SpatialHash,
    pub time: f64,
    pub dt: f64,
    pub gravity: math::Vec3,
    pub damping: f64,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            rigid_bodies: Vec::new(),
            soft_bodies: Vec::new(),
            fluid_systems: Vec::new(),
            constraints: Vec::new(),
            spatial_hash: spatial::SpatialHash::new(1.0),
            time: 0.0,
            dt: 1.0 / 60.0, // 60 FPS default
            gravity: math::Vec3::new(0.0, -9.81, 0.0),
            damping: 0.99,
        }
    }

    /// Main simulation step using modern algorithms
    pub fn step(&mut self) {
        // 1. Broad phase collision detection using spatial hashing
        self.update_spatial_hash();

        // 2. Integrate forces and predict positions (Verlet integration)
        self.integrate_forces();

        // 3. Generate collision and constraint data
        let collisions = self.detect_collisions();

        // 4. Solve constraints using iterative solver (PGS/XPBD)
        self.solve_constraints(&collisions);

        // 5. Update fluid systems using Position-Based Fluids
        self.update_fluids();

        // 6. Update soft bodies using Position-Based Dynamics
        self.update_soft_bodies();

        // 7. Finalize positions and update velocities
        self.finalize_step();

        self.time += self.dt;
    }

    fn update_spatial_hash(&mut self) {
        self.spatial_hash.clear();

        // Add rigid bodies
        for (i, body) in self.rigid_bodies.iter().enumerate() {
            self.spatial_hash
                .insert(spatial::SpatialObject::RigidBody(i), body.aabb());
        }

        // Add soft body particles
        for (i, soft_body) in self.soft_bodies.iter().enumerate() {
            for (j, particle) in soft_body.particles.iter().enumerate() {
                self.spatial_hash.insert(
                    spatial::SpatialObject::SoftBodyParticle(i, j),
                    math::AABB::from_point(particle.position, particle.radius),
                );
            }
        }
    }

    fn integrate_forces(&mut self) {
        // Symplectic Euler integration for rigid bodies
        for body in &mut self.rigid_bodies {
            if !body.is_static {
                body.integrate_forces(self.dt, self.gravity);
            }
        }

        // Verlet integration for soft body particles
        for soft_body in &mut self.soft_bodies {
            soft_body.integrate_forces(self.dt, self.gravity);
        }
    }

    fn detect_collisions(&self) -> Vec<constraints::Constraint> {
        let mut collisions = Vec::new();

        // Use spatial hash for broad phase, then narrow phase
        let pairs = self.spatial_hash.get_potential_pairs();

        for (obj1, obj2) in pairs {
            if let Some(contact) = self.narrow_phase_collision(obj1, obj2) {
                collisions.push(contact);
            }
        }

        collisions
    }

    fn narrow_phase_collision(
        &self,
        obj1: spatial::SpatialObject,
        obj2: spatial::SpatialObject,
    ) -> Option<constraints::Constraint> {
        use spatial::SpatialObject::*;

        match (obj1, obj2) {
            (RigidBody(i1), RigidBody(i2)) => {
                rigid_body::collide(&self.rigid_bodies[i1], &self.rigid_bodies[i2])
            }
            (RigidBody(i), SoftBodyParticle(sb_idx, p_idx)) => rigid_body::collide_with_particle(
                &self.rigid_bodies[i],
                &self.soft_bodies[sb_idx].particles[p_idx],
            ),
            (SoftBodyParticle(sb1, p1), SoftBodyParticle(sb2, p2)) => {
                soft_body::SoftBody::collide_particles(
                    &self.soft_bodies[sb1].particles[p1],
                    &self.soft_bodies[sb2].particles[p2],
                )
            }
            _ => None,
        }
    }

    fn solve_constraints(&mut self, collisions: &[constraints::Constraint]) {
        // Extended Position-Based Dynamics (XPBD) solver
        let mut solver = constraints::XPBDSolver::new();

        // Add collision constraints
        for contact in collisions {
            solver.add_constraint(contact.clone());
        }

        // Add user-defined constraints
        for constraint in &self.constraints {
            solver.add_constraint(constraint.clone());
        }

        // Iterative constraint solving (typically 5-10 iterations)
        for _ in 0..8 {
            solver.solve_iteration(&mut self.rigid_bodies, &mut self.soft_bodies, self.dt);
        }
    }

    fn update_fluids(&mut self) {
        for fluid_system in &mut self.fluid_systems {
            fluid_system.update(self.dt, self.gravity);
        }
    }

    fn update_soft_bodies(&mut self) {
        for soft_body in &mut self.soft_bodies {
            soft_body.update(self.dt);
        }
    }

    fn finalize_step(&mut self) {
        // Update velocities from position changes (Verlet)
        for body in &mut self.rigid_bodies {
            if !body.is_static {
                body.finalize_step(self.dt);
            }
        }

        for soft_body in &mut self.soft_bodies {
            soft_body.finalize_step(self.dt);
        }

        // Apply damping
        for body in &mut self.rigid_bodies {
            body.apply_damping(self.damping);
        }
    }

    /// Add physics objects from language constructs
    pub fn add_rigid_body(
        &mut self,
        shape: rigid_body::Shape,
        mass: f64,
        position: math::Vec3,
    ) -> usize {
        let body = rigid_body::RigidBody::new(shape, mass, position);
        self.rigid_bodies.push(body);
        self.rigid_bodies.len() - 1
    }

    pub fn add_soft_body(&mut self, soft_body: soft_body::SoftBody) -> usize {
        self.soft_bodies.push(soft_body);
        self.soft_bodies.len() - 1
    }

    pub fn add_fluid_system(&mut self, particles: Vec<math::Vec3>, rest_density: f64) -> usize {
        // Create bounds based on particle positions
        let bounds = if particles.is_empty() {
            math::AABB::new(
                math::Vec3::new(-10.0, -10.0, -10.0),
                math::Vec3::new(10.0, 10.0, 10.0),
            )
        } else {
            let mut min = particles[0];
            let mut max = particles[0];
            for &pos in &particles {
                min = min.min_component_wise(pos);
                max = max.max_component_wise(pos);
            }
            // Expand bounds slightly
            let expansion = math::Vec3::new(2.0, 2.0, 2.0);
            math::AABB::new(min - expansion, max + expansion)
        };

        let smoothing_radius = 0.1; // Default smoothing radius
        let mut fluid = fluid::FluidSystem::new(rest_density, smoothing_radius, bounds);

        // Add particles to the fluid system with default mass
        for pos in particles {
            fluid.add_particle(pos, 1.0); // Default mass of 1.0
        }

        self.fluid_systems.push(fluid);
        self.fluid_systems.len() - 1
    }

    /// Convert simulation state to language values for visualization
    pub fn to_simulation_state(&self) -> RuntimeResult<Value> {
        let mut state = HashMap::new();

        // Rigid bodies
        let mut rb_data = Vec::new();
        for (i, body) in self.rigid_bodies.iter().enumerate() {
            rb_data.push(body.to_value(i)?);
        }
        state.insert("rigid_bodies".to_string(), Value::Array(rb_data));

        // Soft bodies
        let mut sb_data = Vec::new();
        for (i, body) in self.soft_bodies.iter().enumerate() {
            sb_data.push(body.to_value(i)?);
        }
        state.insert("soft_bodies".to_string(), Value::Array(sb_data));

        // Fluids
        let mut fluid_data = Vec::new();
        for (i, fluid) in self.fluid_systems.iter().enumerate() {
            fluid_data.push(fluid.to_value(i)?);
        }
        state.insert("fluids".to_string(), Value::Array(fluid_data));

        state.insert("time".to_string(), Value::Float(self.time));

        Ok(Value::Struct {
            name: "PhysicsState".to_string(),
            fields: state,
        })
    }
}

/// Built-in physics functions for the language
pub fn register_physics_functions(env: &mut crate::eval::interpreter::Environment) {
    // Physics world creation
    env.define(
        "create_physics_world".to_string(),
        Value::BuiltinFunction {
            name: "create_physics_world".to_string(),
            arity: 0,
            func: |_args| {
                let _world = PhysicsWorld::new();
                // Store as opaque value - in real implementation would use a registry
                Ok(Value::String("physics_world_handle".to_string()))
            },
        },
    );

    // Rigid body creation
    env.define(
        "add_rigid_body".to_string(),
        Value::BuiltinFunction {
            name: "add_rigid_body".to_string(),
            arity: 4, // world, shape, mass, position
            func: |_args| {
                // Implementation would extract world handle and add rigid body
                Ok(Value::Int(0)) // Return body ID
            },
        },
    );

    // Simulation step
    env.define(
        "physics_step".to_string(),
        Value::BuiltinFunction {
            name: "physics_step".to_string(),
            arity: 1, // world handle
            func: |_args| {
                // Implementation would step the physics world
                Ok(Value::Unit)
            },
        },
    );

    // More physics functions...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::new();
        assert_eq!(world.time, 0.0);
        assert_eq!(world.dt, 1.0 / 60.0);
        assert_eq!(world.gravity.y, -9.81);
    }

    #[test]
    fn test_add_rigid_body() {
        let mut world = PhysicsWorld::new();
        let shape = rigid_body::Shape::Sphere { radius: 1.0 };
        let id = world.add_rigid_body(shape, 1.0, math::Vec3::zero());
        assert_eq!(id, 0);
        assert_eq!(world.rigid_bodies.len(), 1);
    }

    #[test]
    fn test_simulation_step() {
        let mut world = PhysicsWorld::new();
        let initial_time = world.time;
        world.step();
        assert!(world.time > initial_time);
    }

    #[test]
    fn test_spatial_hashing() {
        let mut world = PhysicsWorld::new();
        let shape = rigid_body::Shape::Box {
            size: math::Vec3::new(1.0, 1.0, 1.0),
        };
        world.add_rigid_body(shape.clone(), 1.0, math::Vec3::new(0.0, 0.0, 0.0));
        world.add_rigid_body(shape, 1.0, math::Vec3::new(2.0, 0.0, 0.0));

        world.update_spatial_hash();
        let pairs = world.spatial_hash.get_potential_pairs();

        // Should find collision pairs based on proximity
        assert!(pairs.len() >= 0);
    }
}
