// filepath: /home/deginandor/Documents/Programming/language/src/ecs/world.rs
use bevy_ecs::prelude::*;
use crate::physics::PhysicsWorld;

/// Trait to extend the functionality of World
pub trait WorldExt {
    /// Get mutable access to the physics world resource
    fn get_physics_world_mut(&mut self) -> Option<&mut PhysicsWorld>;

    /// Create and initialize a physics world resource
    fn create_physics_world(&mut self);
}

/// Implementation of WorldExt for the bevy_ecs World
impl WorldExt for World {
    fn get_physics_world_mut(&mut self) -> Option<&mut PhysicsWorld> {
        if let Some(mut res) = self.get_resource_mut::<PhysicsWorld>() {
            // Convert from Mut<PhysicsWorld> to &mut PhysicsWorld
            Some(unsafe {
                // This is safe because we have exclusive access to the resource
                // during the lifetime of the &mut self reference
                &mut *(res.as_mut() as *mut PhysicsWorld)
            })
        } else {
            None
        }
    }

    fn create_physics_world(&mut self) {
        // Create a new physics world with default settings
        let physics_world = PhysicsWorld::new();

        // Insert it as a resource into the ECS world
        self.insert_resource(physics_world);
    }
}
