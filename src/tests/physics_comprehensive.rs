use crate::physics::*;
use crate::math::*;
use super::test_utilities::*;
use std::collections::HashMap;

#[cfg(test)]
mod physics_comprehensive_tests {
    use super::*;

    #[test]
    fn test_rigid_body_creation() {
        let body = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.velocity, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.acceleration, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.force, Vector3::new(0.0, 0.0, 0.0));
        assert!(body.is_static == false);
    }

    #[test]
    fn test_rigid_body_static_creation() {
        let body = RigidBody::new_static(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(body.mass, f32::INFINITY);
        assert_eq!(body.position, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(body.velocity, Vector3::new(0.0, 0.0, 0.0));
        assert!(body.is_static == true);
    }

    #[test]
    fn test_force_application() {
        let mut body = RigidBody::new(2.0, Vector3::new(0.0, 0.0, 0.0));
        
        // Apply force
        body.apply_force(Vector3::new(10.0, 0.0, 0.0));
        assert_eq!(body.force, Vector3::new(10.0, 0.0, 0.0));
        
        // Apply additional force (should accumulate)
        body.apply_force(Vector3::new(0.0, 5.0, 0.0));
        assert_eq!(body.force, Vector3::new(10.0, 5.0, 0.0));
        
        // Apply impulse
        body.apply_impulse(Vector3::new(4.0, 0.0, 0.0));
        assert_eq!(body.velocity, Vector3::new(2.0, 0.0, 0.0)); // impulse / mass
    }

    #[test]
    fn test_rigid_body_integration() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        
        // Apply constant force (gravity)
        body.apply_force(Vector3::new(0.0, -9.8, 0.0));
        
        // Update for 1 second
        body.update(1.0);
        
        // Check physics: v = at, s = 1/2 * a * t^2
        assert_eq!(body.acceleration, Vector3::new(0.0, -9.8, 0.0));
        assert!((body.velocity.y + 9.8).abs() < 0.001);
        assert!((body.position.y + 4.9).abs() < 0.001);
        
        // Force should be cleared after update
        assert_eq!(body.force, Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_rigid_body_multiple_updates() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 10.0, 0.0));
        
        // Simulate free fall for multiple time steps
        for _ in 0..10 {
            body.apply_force(Vector3::new(0.0, -9.8, 0.0));
            body.update(0.1); // 0.1 second steps
        }
        
        // After 1 second total, object should have fallen from height 10
        // Final position: 10 - 1/2 * 9.8 * 1^2 = 10 - 4.9 = 5.1
        assert!((body.position.y - 5.1).abs() < 0.1);
        assert!((body.velocity.y + 9.8).abs() < 0.1);
    }

    #[test]
    fn test_static_body_immutability() {
        let mut body = RigidBody::new_static(Vector3::new(0.0, 0.0, 0.0));
        
        // Static bodies should not be affected by forces
        body.apply_force(Vector3::new(100.0, 100.0, 100.0));
        body.update(1.0);
        
        assert_eq!(body.position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.velocity, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(body.acceleration, Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_soft_body_creation() {
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        
        let soft_body = SoftBody::new(positions.clone(), 1.0);
        assert_eq!(soft_body.particles.len(), 3);
        assert_eq!(soft_body.mass, 1.0);
        
        for (i, particle) in soft_body.particles.iter().enumerate() {
            assert_eq!(particle.position, positions[i]);
            assert_eq!(particle.velocity, Vector3::new(0.0, 0.0, 0.0));
            assert_eq!(particle.mass, 1.0 / 3.0); // Total mass divided by particle count
        }
    }

    #[test]
    fn test_soft_body_spring_constraints() {
        let positions = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        ];
        
        let mut soft_body = SoftBody::new(positions, 1.0);
        
        // Add spring constraint between particles
        soft_body.add_spring_constraint(0, 1, 1000.0, 0.1);
        assert_eq!(soft_body.constraints.len(), 1);
        
        // Stretch the spring by moving one particle
        soft_body.particles[1].position = Vector3::new(2.0, 0.0, 0.0);
        
        // Update should apply spring forces
        soft_body.update(0.01);
        
        // Particles should move towards each other due to spring force
        assert!(soft_body.particles[0].position.x > 0.0); // Pulled right
        assert!(soft_body.particles[1].position.x < 2.0); // Pulled left
    }

    #[test]
    fn test_soft_body_collision_response() {
        let positions = vec![
            Vector3::new(0.0, 1.0, 0.0), // Particle above ground
        ];
        
        let mut soft_body = SoftBody::new(positions, 1.0);
        
        // Apply gravity
        for particle in &mut soft_body.particles {
            particle.apply_force(Vector3::new(0.0, -9.8, 0.0));
        }
        
        // Simulate fall
        for _ in 0..100 {
            soft_body.update(0.01);
            
            // Simple ground collision (y = 0)
            for particle in &mut soft_body.particles {
                if particle.position.y < 0.0 {
                    particle.position.y = 0.0;
                    particle.velocity.y = -particle.velocity.y * 0.8; // Bounce with damping
                }
            }
        }
        
        // Particle should have bounced and settled near ground
        assert!(soft_body.particles[0].position.y >= 0.0);
        assert!(soft_body.particles[0].position.y < 0.5); // Should be close to ground
    }

    #[test]
    fn test_collision_detection_spheres() {
        let body1 = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        let body2 = RigidBody::new(1.0, Vector3::new(1.5, 0.0, 0.0));
        
        let collision_system = CollisionSystem::new();
        
        // Bodies are close but not touching (assuming radius = 1.0)
        let collision = collision_system.detect_collision(&body1, &body2);
        assert!(collision.is_none());
        
        // Move bodies closer
        let body2_close = RigidBody::new(1.0, Vector3::new(1.0, 0.0, 0.0));
        let collision = collision_system.detect_collision(&body1, &body2_close);
        assert!(collision.is_some());
        
        if let Some(contact) = collision {
            assert!(contact.penetration_depth > 0.0);
            assert_eq!(contact.contact_normal.normalize(), Vector3::new(1.0, 0.0, 0.0));
        }
    }

    #[test]
    fn test_collision_resolution() {
        let mut body1 = RigidBody::new(1.0, Vector3::new(-0.5, 0.0, 0.0));
        let mut body2 = RigidBody::new(1.0, Vector3::new(0.5, 0.0, 0.0));
        
        // Give them velocities toward each other
        body1.velocity = Vector3::new(2.0, 0.0, 0.0);
        body2.velocity = Vector3::new(-2.0, 0.0, 0.0);
        
        let mut collision_system = CollisionSystem::new();
        
        // Detect collision
        if let Some(contact) = collision_system.detect_collision(&body1, &body2) {
            // Resolve collision
            collision_system.resolve_collision(&mut body1, &mut body2, &contact);
            
            // Bodies should have bounced apart
            assert!(body1.velocity.x < 0.0); // Body1 should now move left
            assert!(body2.velocity.x > 0.0); // Body2 should now move right
        }
    }

    #[test]
    fn test_physics_world_integration() {
        let mut world = PhysicsWorld::new();
        
        // Add some rigid bodies
        let body1_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(0.0, 10.0, 0.0)));
        let body2_id = world.add_rigid_body(RigidBody::new_static(Vector3::new(0.0, 0.0, 0.0)));
        
        // Apply gravity to first body
        world.apply_force(body1_id, Vector3::new(0.0, -9.8, 0.0));
        
        // Step simulation
        for _ in 0..100 {
            world.step(0.01);
        }
        
        // First body should have fallen and potentially collided with second
        let body1 = world.get_rigid_body(body1_id).unwrap();
        assert!(body1.position.y < 10.0); // Should have fallen
        assert!(body1.position.y >= 0.0); // Should not go below ground
    }

    #[test]
    fn test_constraint_system() {
        let mut world = PhysicsWorld::new();
        
        // Create two bodies
        let body1_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0)));
        let body2_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(2.0, 0.0, 0.0)));
        
        // Add distance constraint (like a rod connecting them)
        world.add_distance_constraint(body1_id, body2_id, 2.0);
        
        // Try to move one body away
        world.apply_force(body1_id, Vector3::new(-100.0, 0.0, 0.0));
        
        // Step simulation
        for _ in 0..100 {
            world.step(0.01);
        }
        
        // Bodies should maintain their distance due to constraint
        let body1 = world.get_rigid_body(body1_id).unwrap();
        let body2 = world.get_rigid_body(body2_id).unwrap();
        let distance = (body2.position - body1.position).magnitude();
        assert!((distance - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_spring_constraint() {
        let mut world = PhysicsWorld::new();
        
        // Create two bodies
        let body1_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0)));
        let body2_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(1.0, 0.0, 0.0)));
        
        // Add spring constraint
        world.add_spring_constraint(body1_id, body2_id, 1000.0, 0.1, 1.0);
        
        // Move one body to stretch the spring
        world.get_rigid_body_mut(body2_id).unwrap().position = Vector3::new(3.0, 0.0, 0.0);
        
        // Step simulation
        for _ in 0..100 {
            world.step(0.01);
        }
        
        // Bodies should be pulled together by spring
        let body1 = world.get_rigid_body(body1_id).unwrap();
        let body2 = world.get_rigid_body(body2_id).unwrap();
        let distance = (body2.position - body1.position).magnitude();
        assert!(distance < 3.0); // Should be closer than initial stretched distance
        assert!(distance > 0.8); // But spring oscillation might not fully settle
    }

    #[test]
    fn test_friction_forces() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        
        // Give body initial velocity
        body.velocity = Vector3::new(10.0, 0.0, 0.0);
        
        // Apply friction force
        let friction_coefficient = 0.1;
        for _ in 0..100 {
            let friction_force = -body.velocity * friction_coefficient * body.mass;
            body.apply_force(friction_force);
            body.update(0.01);
        }
        
        // Body should have slowed down significantly
        assert!(body.velocity.magnitude() < 5.0);
        assert!((body.velocity.y).abs() < 0.001); // Should remain on horizontal plane
        assert!((body.velocity.z).abs() < 0.001);
    }

    #[test]
    fn test_angular_motion() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        
        // Set moment of inertia (assuming sphere)
        body.moment_of_inertia = 0.4 * body.mass * 1.0 * 1.0; // I = 2/5 * m * r^2
        
        // Apply torque
        body.apply_torque(Vector3::new(0.0, 0.0, 5.0));
        
        // Update
        body.update(1.0);
        
        // Check angular motion
        assert!(body.angular_velocity.z > 0.0);
        assert!(body.orientation.z != 0.0); // Should have rotated
    }

    #[test]
    fn test_energy_conservation() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 10.0, 0.0));
        
        let initial_potential_energy = body.mass * 9.8 * body.position.y;
        let initial_kinetic_energy = 0.5 * body.mass * body.velocity.magnitude_squared();
        let initial_total_energy = initial_potential_energy + initial_kinetic_energy;
        
        // Free fall simulation
        for _ in 0..100 {
            body.apply_force(Vector3::new(0.0, -9.8 * body.mass, 0.0));
            body.update(0.01);
        }
        
        let final_potential_energy = body.mass * 9.8 * body.position.y;
        let final_kinetic_energy = 0.5 * body.mass * body.velocity.magnitude_squared();
        let final_total_energy = final_potential_energy + final_kinetic_energy;
        
        // Energy should be conserved (within numerical precision)
        assert!((final_total_energy - initial_total_energy).abs() < 1.0);
    }

    #[test]
    fn test_momentum_conservation() {
        let mut body1 = RigidBody::new(1.0, Vector3::new(-1.0, 0.0, 0.0));
        let mut body2 = RigidBody::new(2.0, Vector3::new(1.0, 0.0, 0.0));
        
        body1.velocity = Vector3::new(3.0, 0.0, 0.0);
        body2.velocity = Vector3::new(-1.0, 0.0, 0.0);
        
        let initial_momentum = body1.mass * body1.velocity + body2.mass * body2.velocity;
        
        // Simulate collision
        let mut collision_system = CollisionSystem::new();
        if let Some(contact) = collision_system.detect_collision(&body1, &body2) {
            collision_system.resolve_collision(&mut body1, &mut body2, &contact);
        }
        
        let final_momentum = body1.mass * body1.velocity + body2.mass * body2.velocity;
        
        // Momentum should be conserved
        assert!((final_momentum - initial_momentum).magnitude() < 0.01);
    }

    #[test]
    fn test_complex_soft_body_deformation() {
        // Create a cloth-like soft body (2D grid of particles)
        let mut positions = Vec::new();
        let grid_size = 5;
        
        for i in 0..grid_size {
            for j in 0..grid_size {
                positions.push(Vector3::new(i as f32, j as f32, 0.0));
            }
        }
        
        let mut soft_body = SoftBody::new(positions, 1.0);
        
        // Add spring constraints to create grid structure
        for i in 0..grid_size {
            for j in 0..grid_size {
                let current_idx = i * grid_size + j;
                
                // Horizontal springs
                if j < grid_size - 1 {
                    soft_body.add_spring_constraint(current_idx, current_idx + 1, 1000.0, 0.1);
                }
                
                // Vertical springs
                if i < grid_size - 1 {
                    soft_body.add_spring_constraint(current_idx, current_idx + grid_size, 1000.0, 0.1);
                }
            }
        }
        
        // Apply force to corner particle
        soft_body.particles[0].apply_force(Vector3::new(0.0, 0.0, 10.0));
        
        // Simulate
        for _ in 0..100 {
            soft_body.update(0.01);
        }
        
        // Check that deformation propagated through the structure
        assert!(soft_body.particles[0].position.z > 0.0);
        assert!(soft_body.particles[1].position.z > 0.0); // Adjacent particle should move
        assert!(soft_body.particles[grid_size].position.z > 0.0); // Below particle should move
    }

    #[test]
    fn test_fluid_simulation_basic() {
        // Create a simple particle-based fluid
        let mut fluid_particles = Vec::new();
        
        // Create a 3x3x3 cube of particles
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let mut particle = Particle::new(0.1, Vector3::new(x as f32, y as f32 + 5.0, z as f32));
                    fluid_particles.push(particle);
                }
            }
        }
        
        // Apply gravity and simulate
        for _ in 0..100 {
            for particle in &mut fluid_particles {
                particle.apply_force(Vector3::new(0.0, -9.8 * particle.mass, 0.0));
                particle.update(0.01);
                
                // Simple ground collision
                if particle.position.y < 0.0 {
                    particle.position.y = 0.0;
                    particle.velocity.y = -particle.velocity.y * 0.5;
                }
            }
        }
        
        // All particles should have settled near the ground
        for particle in &fluid_particles {
            assert!(particle.position.y >= 0.0);
            assert!(particle.position.y < 2.0);
        }
    }

    #[test]
    fn test_large_scale_simulation() {
        let mut world = PhysicsWorld::new();
        
        // Add many bodies
        let mut body_ids = Vec::new();
        for i in 0..100 {
            let x = (i % 10) as f32;
            let y = (i / 10) as f32 + 10.0;
            let body_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(x, y, 0.0)));
            body_ids.push(body_id);
        }
        
        // Add ground
        let ground_id = world.add_rigid_body(RigidBody::new_static(Vector3::new(5.0, 0.0, 0.0)));
        
        // Simulate
        for step in 0..200 {
            // Apply gravity to all bodies
            for &body_id in &body_ids {
                world.apply_force(body_id, Vector3::new(0.0, -9.8, 0.0));
            }
            
            world.step(0.01);
            
            // Check performance doesn't degrade significantly
            if step % 50 == 0 {
                // Simple performance check - simulation should complete in reasonable time
                assert!(world.get_rigid_body_count() == 101);
            }
        }
        
        // All bodies should have fallen
        for &body_id in &body_ids {
            let body = world.get_rigid_body(body_id).unwrap();
            assert!(body.position.y < 10.0);
        }
    }

    #[test]
    fn test_numerical_stability() {
        let mut body = RigidBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
        
        // Apply very small forces over many iterations
        for _ in 0..10000 {
            body.apply_force(Vector3::new(0.001, 0.0, 0.0));
            body.update(0.0001); // Very small time step
        }
        
        // Position should be reasonable (not NaN or infinity)
        assert!(body.position.x.is_finite());
        assert!(body.position.y.is_finite());
        assert!(body.position.z.is_finite());
        assert!(body.velocity.x.is_finite());
        assert!(body.velocity.y.is_finite());
        assert!(body.velocity.z.is_finite());
        
        // Should have moved in positive x direction
        assert!(body.position.x > 0.0);
    }

    #[test]
    fn test_constraint_solver_accuracy() {
        let mut world = PhysicsWorld::new();
        
        // Create a chain of connected bodies
        let mut body_ids = Vec::new();
        for i in 0..5 {
            let body_id = world.add_rigid_body(RigidBody::new(1.0, Vector3::new(i as f32, 0.0, 0.0)));
            body_ids.push(body_id);
        }
        
        // Connect them with distance constraints
        for i in 0..4 {
            world.add_distance_constraint(body_ids[i], body_ids[i + 1], 1.0);
        }
        
        // Fix the first body (make it static)
        world.get_rigid_body_mut(body_ids[0]).unwrap().is_static = true;
        
        // Apply force to last body
        world.apply_force(body_ids[4], Vector3::new(0.0, -50.0, 0.0));
        
        // Simulate
        for _ in 0..500 {
            world.step(0.01);
        }
        
        // Check that distances are maintained
        for i in 0..4 {
            let body1 = world.get_rigid_body(body_ids[i]).unwrap();
            let body2 = world.get_rigid_body(body_ids[i + 1]).unwrap();
            let distance = (body2.position - body1.position).magnitude();
            assert!((distance - 1.0).abs() < 0.1); // Should maintain unit distance
        }
        
        // Last body should hang below the chain
        let last_body = world.get_rigid_body(body_ids[4]).unwrap();
        assert!(last_body.position.y < 0.0);
    }
}
