// ECS-based Physics Engine Architecture
pub mod components;
pub mod resources;
pub mod systems;

pub use bevy_ecs::prelude::*;
pub use components::*;
pub use resources::*;
pub use systems::*;
use crate::physics::rigid_body::Shape;

/// Main ECS-based physics world
pub struct PhysicsECS {
    pub world: World,
    pub schedule: Schedule,
}

impl PhysicsECS {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Initialize resources
        world.insert_resource(PhysicsConfig::default());
        world.insert_resource(Time::default());
        world.insert_resource(SpatialIndex::new(1.0));

        // Add physics systems
        schedule.add_systems((
            spatial_indexing_system,
            rigid_body_integration_system,
            soft_body_system,
            fluid_system,
            constraint_solving_system,
            collision_detection_system,
        ));

        Self { world, schedule }
    }

    pub fn step(&mut self) {
        self.schedule.run(&mut self.world);
    }

    pub fn spawn_rigid_body(&mut self, shape: Shape, mass: f64, position: crate::physics::math::Vec3) -> Entity {
        self.world.spawn((
            PhysicsTransform::from_position(position),
            RigidBodyComponent::new(mass, shape),
            VelocityComponent::default(),
            PhysicsObject::RigidBody,
        )).id()
    }

    pub fn spawn_soft_body(&mut self, particles: Vec<SoftBodyParticle>) -> Entity {
        self.world.spawn((
            SoftBodyComponent::new(particles),
            PhysicsObject::SoftBody,
        )).id()
    }

    pub fn spawn_fluid_system(&mut self, particles: Vec<FluidParticle>) -> Entity {
        self.world.spawn((
            FluidComponent::new(particles),
            PhysicsObject::Fluid,
        )).id()
    }
}

impl Default for PhysicsECS {
    fn default() -> Self {
        Self::new()
    }
}
