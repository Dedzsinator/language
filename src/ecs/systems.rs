// ECS Systems for Physics Engine
use bevy_ecs::prelude::*;
use crate::physics::math::*;
use crate::physics::spatial::SpatialObject;
use super::components::*;
use super::resources::*;
use rayon::prelude::*;

/// Spatial indexing system - updates spatial hash for collision detection
pub fn spatial_indexing_system(
    mut spatial_index: ResMut<SpatialIndex>,
    query: Query<(Entity, &PhysicsTransform, &PhysicsObject), Or<(With<RigidBodyComponent>, With<SoftBodyComponent>, With<FluidComponent>)>>,
) {
    if !spatial_index.dirty {
        return;
    }

    spatial_index.clear();

    for (entity, transform, physics_object) in query.iter() {
        let aabb = match physics_object {
            PhysicsObject::RigidBody => {
                // Create AABB for rigid body
                AABB::from_point(transform.position, 1.0) // Simplified - should use actual shape
            }
            PhysicsObject::SoftBody => {
                // Create AABB for soft body
                AABB::from_point(transform.position, 1.0)
            }
            PhysicsObject::Fluid => {
                // Create AABB for fluid particle
                AABB::from_point(transform.position, 0.5)
            }
            _ => continue,
        };

        spatial_index.spatial_hash.insert(
            SpatialObject::Entity(entity.index() as usize),
            aabb,
        );
    }
}

/// Rigid body integration system
pub fn rigid_body_integration_system(
    config: Res<PhysicsConfig>,
    time: Res<Time>,
    mut query: Query<(&mut PhysicsTransform, &mut VelocityComponent, &mut RigidBodyComponent)>,
) {
    if time.paused {
        return;
    }

    query.par_iter_mut().for_each(|(mut transform, mut velocity, mut rigid_body)| {
        if rigid_body.is_static {
            return;
        }

        let dt = time.delta;

        // Apply gravity
        let gravity = config.gravity;
        let mass = rigid_body.mass;
        rigid_body.apply_force(gravity * mass);

        // Calculate acceleration from forces
        let acceleration = rigid_body.force_accumulator * rigid_body.inv_mass;
        let damping = rigid_body.damping;

        // Semi-implicit Euler integration
        velocity.linear = velocity.linear + acceleration * dt;
        velocity.linear = velocity.linear * damping; // Apply damping

        // Clamp velocity
        let speed = velocity.linear.magnitude();
        if speed > config.max_velocity {
            velocity.linear = velocity.linear.normalized() * config.max_velocity;
        }

        // Update position
        transform.translate(velocity.linear * dt);

        // Clear force accumulator
        rigid_body.clear_forces();
    });
}

/// Soft body physics system using Position-Based Dynamics
pub fn soft_body_system(
    config: Res<PhysicsConfig>,
    time: Res<Time>,
    mut query: Query<&mut SoftBodyComponent>,
) {
    if time.paused {
        return;
    }

    let dt = time.delta;

    for mut soft_body in query.iter_mut() {
        // Integrate forces (Verlet integration)
        for particle in &mut soft_body.particles {
            if particle.pinned {
                continue;
            }

            let acceleration = config.gravity;
            let new_position = particle.position * 2.0 - particle.old_position + acceleration * (dt * dt);
            particle.old_position = particle.position;
            particle.position = new_position;
            particle.velocity = (particle.position - particle.old_position) / dt;
        }

        // Solve constraints
        for _ in 0..soft_body.iterations {
            // Clone constraints to avoid borrowing issues
            let constraints = soft_body.constraints.clone();
            for constraint in &constraints {
                match constraint {
                    SoftBodyConstraint::Distance { particle_a, particle_b, rest_length, stiffness } => {
                        if *particle_a >= soft_body.particles.len() || *particle_b >= soft_body.particles.len() {
                            continue;
                        }

                        let p1 = soft_body.particles[*particle_a].position;
                        let p2 = soft_body.particles[*particle_b].position;
                        let distance = p1.distance_to(p2);

                        if distance > 0.0 {
                            let direction = (p2 - p1).normalized();
                            let constraint_force = (distance - rest_length) * stiffness * 0.5;

                            if !soft_body.particles[*particle_a].pinned {
                                soft_body.particles[*particle_a].position = 
                                    soft_body.particles[*particle_a].position + direction * constraint_force;
                            }
                            if !soft_body.particles[*particle_b].pinned {
                                soft_body.particles[*particle_b].position = 
                                    soft_body.particles[*particle_b].position - direction * constraint_force;
                            }
                        }
                    }
                    SoftBodyConstraint::Bend { .. } => {
                        // TODO: Implement bending constraints
                    }
                    SoftBodyConstraint::Volume { .. } => {
                        // TODO: Implement volume constraints
                    }
                }
            }
        }

        // Apply damping
        let damping = soft_body.damping;
        for particle in &mut soft_body.particles {
            particle.velocity = particle.velocity * damping;
            particle.position = particle.old_position + particle.velocity * dt;
        }
    }
}

/// Fluid simulation system using SPH (Smoothed Particle Hydrodynamics)
pub fn fluid_system(
    config: Res<PhysicsConfig>,
    time: Res<Time>,
    spatial_index: Res<SpatialIndex>,
    mut query: Query<&mut FluidComponent>,
) {
    if time.paused {
        return;
    }

    let dt = time.delta;

    for mut fluid in query.iter_mut() {
        // Find neighbors for each particle
        for i in 0..fluid.particles.len() {
            fluid.particles[i].neighbors.clear();
            let particle_aabb = AABB::from_point(fluid.particles[i].position, fluid.smoothing_radius);
            
            let nearby_objects = spatial_index.spatial_hash.query(particle_aabb);
            for obj in nearby_objects {
                if let SpatialObject::Entity(entity_index) = obj {
                    // Find particles within smoothing radius
                    for j in 0..fluid.particles.len() {
                        if i != j {
                            let distance = fluid.particles[i].position.distance_to(fluid.particles[j].position);
                            if distance < fluid.smoothing_radius {
                                fluid.particles[i].neighbors.push(j);
                            }
                        }
                    }
                }
            }
        }

        // Calculate density and pressure
        for i in 0..fluid.particles.len() {
            let mut density = 0.0;
            
            // Self contribution
            density += fluid.particles[i].mass * poly6_kernel(0.0, fluid.smoothing_radius);
            
            // Neighbor contributions
            for &neighbor_idx in &fluid.particles[i].neighbors {
                let distance = fluid.particles[i].position.distance_to(fluid.particles[neighbor_idx].position);
                density += fluid.particles[neighbor_idx].mass * poly6_kernel(distance, fluid.smoothing_radius);
            }
            
            fluid.particles[i].density = density;
            fluid.particles[i].pressure = fluid.gas_constant * (density - fluid.rest_density);
        }

        // Calculate forces and integrate
        for i in 0..fluid.particles.len() {
            let mut pressure_force = Vec3::zero();
            let mut viscosity_force = Vec3::zero();

            for &neighbor_idx in &fluid.particles[i].neighbors {
                let r = fluid.particles[i].position - fluid.particles[neighbor_idx].position;
                let distance = r.magnitude();
                
                if distance > 0.0 {
                    let direction = r.normalized();
                    
                    // Pressure force
                    let pressure_magnitude = fluid.particles[neighbor_idx].mass * 
                        (fluid.particles[i].pressure + fluid.particles[neighbor_idx].pressure) / 
                        (2.0 * fluid.particles[neighbor_idx].density) * 
                        spiky_kernel_gradient(distance, fluid.smoothing_radius);
                    pressure_force = pressure_force + direction * pressure_magnitude;
                    
                    // Viscosity force
                    let velocity_diff = fluid.particles[neighbor_idx].velocity - fluid.particles[i].velocity;
                    let viscosity_magnitude = fluid.viscosity * fluid.particles[neighbor_idx].mass * 
                        velocity_diff.dot(direction) / fluid.particles[neighbor_idx].density * 
                        viscosity_kernel_laplacian(distance, fluid.smoothing_radius);
                    viscosity_force = viscosity_force + direction * viscosity_magnitude;
                }
            }

            // Total acceleration
            let total_force = pressure_force + viscosity_force + config.gravity * fluid.particles[i].mass;
            let acceleration = total_force / fluid.particles[i].mass;

            // Integration
            fluid.particles[i].velocity = fluid.particles[i].velocity + acceleration * dt;
            fluid.particles[i].position = fluid.particles[i].position + fluid.particles[i].velocity * dt;
        }
    }
}

/// Constraint solving system
pub fn constraint_solving_system(
    config: Res<PhysicsConfig>,
    time: Res<Time>,
    mut constraints_query: Query<&mut ConstraintComponent>,
    mut rigid_bodies: Query<(&mut PhysicsTransform, &mut VelocityComponent, &RigidBodyComponent)>,
) {
    if time.paused {
        return;
    }

    for mut constraint_comp in constraints_query.iter_mut() {
        for constraint in &mut constraint_comp.constraints {
            // TODO: Implement constraint solving using XPBD
            // This would solve position and velocity constraints between entities
        }
    }
}

/// Collision detection system
pub fn collision_detection_system(
    spatial_index: Res<SpatialIndex>,
    mut events: ResMut<PhysicsEvents>,
    query: Query<(Entity, &PhysicsTransform, &ColliderComponent)>,
) {
    let objects: Vec<_> = query.iter().collect();
    
    // Broad phase - get potential collision pairs from spatial hash
    let mut collision_pairs = Vec::new();
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            let (entity_a, transform_a, collider_a) = objects[i];
            let (entity_b, transform_b, collider_b) = objects[j];
            
            // Check collision groups
            if (collider_a.collision_groups & collider_b.collision_mask) == 0 ||
               (collider_b.collision_groups & collider_a.collision_mask) == 0 {
                continue;
            }
            
            // Simple AABB test for now
            let aabb_a = AABB::from_point(transform_a.position, 1.0);
            let aabb_b = AABB::from_point(transform_b.position, 1.0);
            
            if aabb_a.intersects(aabb_b) {
                collision_pairs.push((entity_a, entity_b, transform_a.position, transform_b.position));
            }
        }
    }
    
    // Narrow phase - detailed collision detection
    for (entity_a, entity_b, pos_a, pos_b) in collision_pairs {
        let contact_point = (pos_a + pos_b) * 0.5;
        let normal = (pos_b - pos_a).normalized();
        let impulse = 1.0; // Simplified
        
        events.push(PhysicsEvent::Collision {
            entity_a,
            entity_b,
            contact_point,
            normal,
            impulse,
        });
    }
}

/// Time update system
pub fn time_update_system(mut time: ResMut<Time>) {
    let real_delta = 1.0 / 60.0; // Simplified - should use actual frame time
    time.update(real_delta);
}

/// Statistics collection system
pub fn stats_system(
    mut stats: ResMut<PhysicsStats>,
    rigid_bodies: Query<&RigidBodyComponent>,
    soft_bodies: Query<&SoftBodyComponent>,
    fluids: Query<&FluidComponent>,
    constraints: Query<&ConstraintComponent>,
) {
    stats.new_frame();
    stats.rigid_body_count = rigid_bodies.iter().count();
    stats.soft_body_count = soft_bodies.iter().count();
    stats.fluid_particle_count = fluids.iter().map(|f| f.particles.len()).sum();
    stats.constraint_count = constraints.iter().map(|c| c.constraints.len()).sum();
}

// SPH kernel functions
fn poly6_kernel(distance: f64, smoothing_radius: f64) -> f64 {
    if distance >= smoothing_radius {
        return 0.0;
    }
    let h_sq = smoothing_radius * smoothing_radius;
    let r_sq = distance * distance;
    let factor = 315.0 / (64.0 * std::f64::consts::PI * h_sq.powf(4.5));
    factor * (h_sq - r_sq).powf(3.0)
}

fn spiky_kernel_gradient(distance: f64, smoothing_radius: f64) -> f64 {
    if distance >= smoothing_radius {
        return 0.0;
    }
    let factor = -45.0 / (std::f64::consts::PI * smoothing_radius.powf(6.0));
    factor * (smoothing_radius - distance).powf(2.0)
}

fn viscosity_kernel_laplacian(distance: f64, smoothing_radius: f64) -> f64 {
    if distance >= smoothing_radius {
        return 0.0;
    }
    let factor = 45.0 / (std::f64::consts::PI * smoothing_radius.powf(6.0));
    factor * (smoothing_radius - distance)
}
