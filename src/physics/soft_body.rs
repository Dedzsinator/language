// Soft Body Physics - Position-Based Dynamics (PBD) Implementation
use super::constraints;
use super::math::*;
use std::collections::HashMap;

/// Particle in a soft body system
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub old_position: Vec3,
    pub velocity: Vec3,
    pub mass: f64,
    pub radius: f64,
    pub pinned: bool,
}

impl Particle {
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

    pub fn integrate(&mut self, dt: f64, gravity: Vec3) {
        if self.pinned {
            return;
        }

        let acceleration = gravity + Vec3::new(0.0, 0.0, 0.0); // Add external forces here

        // Verlet integration
        let new_position = self.position * 2.0 - self.old_position + acceleration * (dt * dt);
        self.old_position = self.position;
        self.position = new_position;

        // Update velocity for damping and other systems
        self.velocity = (self.position - self.old_position) / dt;
    }
}

/// Constraint types for soft body physics
#[derive(Debug, Clone)]
pub enum SoftConstraint {
    Distance {
        particle_a: usize,
        particle_b: usize,
        rest_length: f64,
        stiffness: f64,
    },
    Bend {
        particles: [usize; 4], // Four particles for bending constraint
        rest_angle: f64,
        stiffness: f64,
    },
    Volume {
        particles: Vec<usize>, // Tetrahedron or more complex volume
        rest_volume: f64,
        stiffness: f64,
    },
    Surface {
        particles: [usize; 3], // Triangle surface constraint
        rest_area: f64,
        stiffness: f64,
    },
}

impl SoftConstraint {
    /// Apply constraint using Position-Based Dynamics
    pub fn solve(&self, particles: &mut [Particle], dt: f64) {
        match self {
            SoftConstraint::Distance {
                particle_a,
                particle_b,
                rest_length,
                stiffness,
            } => {
                self.solve_distance_constraint(
                    particles,
                    *particle_a,
                    *particle_b,
                    *rest_length,
                    *stiffness,
                    dt,
                );
            }
            SoftConstraint::Bend {
                particles: indices,
                rest_angle,
                stiffness,
            } => {
                self.solve_bend_constraint(particles, *indices, *rest_angle, *stiffness, dt);
            }
            SoftConstraint::Volume {
                particles: indices,
                rest_volume,
                stiffness,
            } => {
                self.solve_volume_constraint(particles, indices, *rest_volume, *stiffness, dt);
            }
            SoftConstraint::Surface {
                particles: indices,
                rest_area,
                stiffness,
            } => {
                self.solve_surface_constraint(particles, *indices, *rest_area, *stiffness, dt);
            }
        }
    }

    fn solve_distance_constraint(
        &self,
        particles: &mut [Particle],
        a: usize,
        b: usize,
        rest_length: f64,
        stiffness: f64,
        dt: f64,
    ) {
        let pos_a = particles[a].position;
        let pos_b = particles[b].position;
        let mass_a = particles[a].mass;
        let mass_b = particles[b].mass;

        let diff = pos_b - pos_a;
        let current_length = diff.magnitude();

        if current_length == 0.0 {
            return;
        }

        let constraint_force = (current_length - rest_length) / current_length;
        let total_mass = mass_a + mass_b;

        if total_mass == 0.0 {
            return;
        }

        let correction = diff * constraint_force * stiffness;
        let w_a = mass_b / total_mass;
        let w_b = mass_a / total_mass;

        if !particles[a].pinned {
            particles[a].position = particles[a].position + correction * w_a;
        }
        if !particles[b].pinned {
            particles[b].position = particles[b].position - correction * w_b;
        }
    }

    fn solve_bend_constraint(
        &self,
        particles: &mut [Particle],
        indices: [usize; 4],
        rest_angle: f64,
        stiffness: f64,
        dt: f64,
    ) {
        // Implement dihedral angle constraint for cloth bending
        let [p1, p2, p3, p4] = indices;

        let pos1 = particles[p1].position;
        let pos2 = particles[p2].position;
        let pos3 = particles[p3].position;
        let pos4 = particles[p4].position;

        // Calculate dihedral angle
        let e = pos3 - pos2; // Shared edge
        let n1 = (pos1 - pos2).cross(pos3 - pos2).normalized(); // Normal of triangle 1
        let n2 = (pos4 - pos2).cross(pos3 - pos2).normalized(); // Normal of triangle 2

        let current_angle = n1.dot(n2).acos();
        let angle_diff = current_angle - rest_angle;

        if angle_diff.abs() < 1e-6 {
            return;
        }

        // Apply correction forces (simplified version)
        let correction_magnitude = angle_diff * stiffness * 0.1;
        let correction_dir = n1.cross(n2).normalized();

        if !particles[p1].pinned {
            particles[p1].position = particles[p1].position + correction_dir * correction_magnitude;
        }
        if !particles[p4].pinned {
            particles[p4].position = particles[p4].position - correction_dir * correction_magnitude;
        }
    }

    fn solve_volume_constraint(
        &self,
        particles: &mut [Particle],
        indices: &[usize],
        rest_volume: f64,
        stiffness: f64,
        dt: f64,
    ) {
        if indices.len() < 4 {
            return;
        }

        // Simplified tetrahedral volume constraint
        let pos0 = particles[indices[0]].position;
        let pos1 = particles[indices[1]].position;
        let pos2 = particles[indices[2]].position;
        let pos3 = particles[indices[3]].position;

        let current_volume = (pos1 - pos0).dot((pos2 - pos0).cross(pos3 - pos0)) / 6.0;
        let volume_diff = current_volume - rest_volume;

        if volume_diff.abs() < 1e-6 {
            return;
        }

        // Calculate gradients and apply corrections
        let grad0 = (pos2 - pos1).cross(pos3 - pos1) / 6.0;
        let grad1 = (pos3 - pos0).cross(pos2 - pos0) / 6.0;
        let grad2 = (pos0 - pos1).cross(pos3 - pos1) / 6.0;
        let grad3 = (pos1 - pos0).cross(pos2 - pos0) / 6.0;

        let total_mass = particles[indices[0]].mass
            + particles[indices[1]].mass
            + particles[indices[2]].mass
            + particles[indices[3]].mass;

        if total_mass == 0.0 {
            return;
        }

        let lambda = -volume_diff * stiffness
            / (grad0.magnitude_squared() / particles[indices[0]].mass
                + grad1.magnitude_squared() / particles[indices[1]].mass
                + grad2.magnitude_squared() / particles[indices[2]].mass
                + grad3.magnitude_squared() / particles[indices[3]].mass);

        if !particles[indices[0]].pinned {
            particles[indices[0]].position =
                particles[indices[0]].position + grad0 * (lambda / particles[indices[0]].mass);
        }
        if !particles[indices[1]].pinned {
            particles[indices[1]].position =
                particles[indices[1]].position + grad1 * (lambda / particles[indices[1]].mass);
        }
        if !particles[indices[2]].pinned {
            particles[indices[2]].position =
                particles[indices[2]].position + grad2 * (lambda / particles[indices[2]].mass);
        }
        if !particles[indices[3]].pinned {
            particles[indices[3]].position =
                particles[indices[3]].position + grad3 * (lambda / particles[indices[3]].mass);
        }
    }

    fn solve_surface_constraint(
        &self,
        particles: &mut [Particle],
        indices: [usize; 3],
        rest_area: f64,
        stiffness: f64,
        dt: f64,
    ) {
        let [p1, p2, p3] = indices;

        let pos1 = particles[p1].position;
        let pos2 = particles[p2].position;
        let pos3 = particles[p3].position;

        let current_area = (pos2 - pos1).cross(pos3 - pos1).magnitude() * 0.5;
        let area_diff = current_area - rest_area;

        if area_diff.abs() < 1e-6 {
            return;
        }

        // Calculate area gradients
        let edge1 = pos2 - pos1;
        let edge2 = pos3 - pos1;
        let normal = edge1.cross(edge2);
        let normal_magnitude = normal.magnitude();

        if normal_magnitude == 0.0 {
            return;
        }

        let normalized_normal = normal / normal_magnitude;

        // Simplified area preservation
        let correction_magnitude = area_diff * stiffness * 0.1;
        let correction = normalized_normal * correction_magnitude;

        let total_mass = particles[p1].mass + particles[p2].mass + particles[p3].mass;
        if total_mass == 0.0 {
            return;
        }

        if !particles[p1].pinned {
            particles[p1].position =
                particles[p1].position + correction * (particles[p1].mass / total_mass);
        }
        if !particles[p2].pinned {
            particles[p2].position =
                particles[p2].position + correction * (particles[p2].mass / total_mass);
        }
        if !particles[p3].pinned {
            particles[p3].position =
                particles[p3].position + correction * (particles[p3].mass / total_mass);
        }
    }
}

/// Soft body object using Position-Based Dynamics
#[derive(Debug, Clone)]
pub struct SoftBody {
    pub particles: Vec<Particle>,
    pub constraints: Vec<SoftConstraint>,
    pub damping: f64,
    pub iterations: usize, // PBD solver iterations
}

impl SoftBody {
    pub fn new(particles: Vec<Particle>, constraints: Vec<SoftConstraint>) -> Self {
        Self {
            particles,
            constraints,
            damping: 0.99,
            iterations: 4, // Default PBD iterations
        }
    }

    /// Create a cloth mesh
    pub fn create_cloth(width: usize, height: usize, spacing: f64, mass: f64) -> Self {
        let mut particles = Vec::new();
        let mut constraints = Vec::new();

        // Create particles in a grid
        for y in 0..height {
            for x in 0..width {
                let position = Vec3::new(x as f64 * spacing, y as f64 * spacing, 0.0);
                particles.push(Particle::new(position, mass, 0.05));
            }
        }

        // Pin top corners
        particles[0].pinned = true;
        particles[width - 1].pinned = true;

        // Create distance constraints
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                // Horizontal constraints
                if x < width - 1 {
                    constraints.push(SoftConstraint::Distance {
                        particle_a: index,
                        particle_b: index + 1,
                        rest_length: spacing,
                        stiffness: 0.9,
                    });
                }

                // Vertical constraints
                if y < height - 1 {
                    constraints.push(SoftConstraint::Distance {
                        particle_a: index,
                        particle_b: index + width,
                        rest_length: spacing,
                        stiffness: 0.9,
                    });
                }

                // Diagonal constraints for shear resistance
                if x < width - 1 && y < height - 1 {
                    constraints.push(SoftConstraint::Distance {
                        particle_a: index,
                        particle_b: index + width + 1,
                        rest_length: spacing * 1.414,
                        stiffness: 0.5,
                    });

                    constraints.push(SoftConstraint::Distance {
                        particle_a: index + 1,
                        particle_b: index + width,
                        rest_length: spacing * 1.414,
                        stiffness: 0.5,
                    });
                }

                // Bending constraints
                if x < width - 2 && y < height - 2 {
                    constraints.push(SoftConstraint::Bend {
                        particles: [index, index + 1, index + width, index + width + 1],
                        rest_angle: 0.0, // Flat cloth
                        stiffness: 0.1,
                    });
                }
            }
        }

        Self::new(particles, constraints)
    }

    /// Create a soft body sphere
    pub fn create_sphere(center: Vec3, radius: f64, resolution: usize, mass: f64) -> Self {
        let mut particles = Vec::new();
        let mut constraints = Vec::new();

        // Create particles on sphere surface using icosphere subdivision
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // Golden ratio

        // Icosahedron vertices
        let vertices = vec![
            Vec3::new(-1.0, phi, 0.0).normalized() * radius + center,
            Vec3::new(1.0, phi, 0.0).normalized() * radius + center,
            Vec3::new(-1.0, -phi, 0.0).normalized() * radius + center,
            Vec3::new(1.0, -phi, 0.0).normalized() * radius + center,
            Vec3::new(0.0, -1.0, phi).normalized() * radius + center,
            Vec3::new(0.0, 1.0, phi).normalized() * radius + center,
            Vec3::new(0.0, -1.0, -phi).normalized() * radius + center,
            Vec3::new(0.0, 1.0, -phi).normalized() * radius + center,
            Vec3::new(phi, 0.0, -1.0).normalized() * radius + center,
            Vec3::new(phi, 0.0, 1.0).normalized() * radius + center,
            Vec3::new(-phi, 0.0, -1.0).normalized() * radius + center,
            Vec3::new(-phi, 0.0, 1.0).normalized() * radius + center,
        ];

        for vertex in vertices {
            particles.push(Particle::new(vertex, mass, 0.05));
        }

        // Create distance constraints between neighboring vertices
        let edges = vec![
            (0, 11),
            (0, 5),
            (0, 1),
            (0, 7),
            (0, 10),
            (1, 5),
            (5, 11),
            (11, 10),
            (10, 7),
            (7, 1),
            (3, 9),
            (3, 4),
            (3, 2),
            (3, 6),
            (3, 8),
            (4, 9),
            (9, 8),
            (8, 6),
            (6, 2),
            (2, 4),
            (1, 9),
            (5, 4),
            (11, 2),
            (10, 6),
            (7, 8),
        ];

        for (a, b) in edges {
            let rest_length = (particles[a].position - particles[b].position).magnitude();
            constraints.push(SoftConstraint::Distance {
                particle_a: a,
                particle_b: b,
                rest_length,
                stiffness: 0.8,
            });
        }

        Self::new(particles, constraints)
    }

    /// Integrate forces using Verlet integration
    pub fn integrate_forces(&mut self, dt: f64, gravity: Vec3) {
        for particle in &mut self.particles {
            particle.integrate(dt, gravity);
        }
    }

    /// Solve all constraints using Position-Based Dynamics
    pub fn solve_constraints(&mut self, dt: f64) {
        for _ in 0..self.iterations {
            for constraint in &self.constraints {
                constraint.solve(&mut self.particles, dt);
            }
        }
    }

    /// Apply damping to reduce oscillations
    pub fn apply_damping(&mut self, dt: f64) {
        for particle in &mut self.particles {
            particle.velocity = particle.velocity * self.damping;
            particle.position = particle.old_position + particle.velocity * dt;
        }
    }

    /// Get axis-aligned bounding box
    pub fn aabb(&self) -> AABB {
        if self.particles.is_empty() {
            return AABB::new(Vec3::zero(), Vec3::zero());
        }

        let mut min = self.particles[0].position;
        let mut max = self.particles[0].position;

        for particle in &self.particles {
            min = min.min_component_wise(particle.position);
            max = max.max_component_wise(particle.position);
        }

        AABB::new(min, max)
    }

    /// Collision detection between two soft body particles
    pub fn collide_particles(
        particle1: &Particle,
        particle2: &Particle,
    ) -> Option<constraints::Constraint> {
        let diff = particle2.position - particle1.position;
        let distance_squared = diff.dot(diff);
        let radius_sum = particle1.radius + particle2.radius;

        if distance_squared < radius_sum * radius_sum {
            let distance = distance_squared.sqrt();
            let normal = if distance > f64::EPSILON {
                diff / distance
            } else {
                Vec3::up()
            };

            let penetration = radius_sum - distance;
            let contact_point = particle1.position + normal * particle1.radius;

            Some(constraints::Constraint::Contact {
                body_a: constraints::ConstraintBody::SoftBodyParticle(0, 0), // Would need proper indices
                body_b: constraints::ConstraintBody::SoftBodyParticle(1, 0),
                contact_point,
                contact_normal: normal,
                penetration_depth: penetration,
                friction: 0.5,    // Default friction for particles
                restitution: 0.3, // Default restitution for particles
                lambda_normal: 0.0,
                lambda_tangent: Vec3::zero(),
            })
        } else {
            None
        }
    }

    /// Update soft body physics simulation
    pub fn update(&mut self, dt: f64) {
        // Apply gravity and integrate forces
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        self.integrate_forces(dt, gravity);

        // Solve constraints
        self.solve_constraints(dt);

        // Apply damping
        self.apply_damping(dt);
    }

    /// Finalize simulation step
    pub fn finalize_step(&mut self, _dt: f64) {
        // Update particle old positions for next frame
        for particle in &mut self.particles {
            particle.old_position = particle.position;
        }
    }

    /// Convert soft body to interpreter value
    pub fn to_value(
        &self,
        _index: usize,
    ) -> Result<crate::eval::interpreter::Value, crate::eval::interpreter::RuntimeError> {
        use crate::eval::interpreter::Value;
        use std::collections::HashMap;

        let mut fields = HashMap::new();

        // Convert particles to values
        let particles: Vec<Value> = self
            .particles
            .iter()
            .map(|p| {
                let mut particle_fields = HashMap::new();
                particle_fields.insert(
                    "position".to_string(),
                    Value::Array(vec![
                        Value::Float(p.position.x),
                        Value::Float(p.position.y),
                        Value::Float(p.position.z),
                    ]),
                );
                particle_fields.insert(
                    "velocity".to_string(),
                    Value::Array(vec![
                        Value::Float(p.velocity.x),
                        Value::Float(p.velocity.y),
                        Value::Float(p.velocity.z),
                    ]),
                );
                particle_fields.insert("mass".to_string(), Value::Float(p.mass));
                particle_fields.insert("radius".to_string(), Value::Float(p.radius));
                particle_fields.insert("pinned".to_string(), Value::Bool(p.pinned));
                Value::Struct {
                    name: "Particle".to_string(),
                    fields: particle_fields,
                }
            })
            .collect();

        fields.insert("particles".to_string(), Value::Array(particles));
        fields.insert("iterations".to_string(), Value::Int(self.iterations as i64));
        fields.insert("damping".to_string(), Value::Float(self.damping));

        Ok(Value::Struct {
            name: "SoftBody".to_string(),
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let particle = Particle::new(pos, 1.0, 0.1);

        assert_eq!(particle.position, pos);
        assert_eq!(particle.old_position, pos);
        assert_eq!(particle.velocity, Vec3::zero());
        assert_eq!(particle.mass, 1.0);
        assert_eq!(particle.radius, 0.1);
        assert!(!particle.pinned);
    }

    #[test]
    fn test_particle_integration() {
        let mut particle = Particle::new(Vec3::new(0.0, 10.0, 0.0), 1.0, 0.1);
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let dt = 1.0 / 60.0;

        let initial_y = particle.position.y;
        particle.integrate(dt, gravity);

        // Should move down due to gravity
        assert!(particle.position.y < initial_y);
    }

    #[test]
    fn test_distance_constraint() {
        let mut particles = vec![
            Particle::new(Vec3::new(0.0, 0.0, 0.0), 1.0, 0.1),
            Particle::new(Vec3::new(2.0, 0.0, 0.0), 1.0, 0.1),
        ];

        let constraint = SoftConstraint::Distance {
            particle_a: 0,
            particle_b: 1,
            rest_length: 1.0,
            stiffness: 1.0,
        };

        constraint.solve(&mut particles, 1.0 / 60.0);

        // Particles should move closer together
        let distance = (particles[1].position - particles[0].position).magnitude();
        assert!(distance < 2.0);
        assert!(distance > 1.0); // Won't reach exactly 1.0 in one iteration
    }

    #[test]
    fn test_cloth_creation() {
        let cloth = SoftBody::create_cloth(3, 3, 1.0, 1.0);

        assert_eq!(cloth.particles.len(), 9);
        assert!(cloth.constraints.len() > 0);
        assert!(cloth.particles[0].pinned); // Top-left corner
        assert!(cloth.particles[2].pinned); // Top-right corner
    }

    #[test]
    fn test_sphere_creation() {
        let center = Vec3::new(5.0, 5.0, 5.0);
        let radius = 2.0;
        let sphere = SoftBody::create_sphere(center, radius, 2, 1.0);

        assert_eq!(sphere.particles.len(), 12); // Icosahedron has 12 vertices
        assert!(sphere.constraints.len() > 0);

        // Check that all particles are approximately on the sphere surface
        for particle in &sphere.particles {
            let distance_from_center = (particle.position - center).magnitude();
            assert!((distance_from_center - radius).abs() < 0.1);
        }
    }

    #[test]
    fn test_soft_body_aabb() {
        let particles = vec![
            Particle::new(Vec3::new(-1.0, -1.0, -1.0), 1.0, 0.1),
            Particle::new(Vec3::new(1.0, 1.0, 1.0), 1.0, 0.1),
            Particle::new(Vec3::new(0.0, 2.0, 0.0), 1.0, 0.1),
        ];

        let soft_body = SoftBody::new(particles, vec![]);
        let aabb = soft_body.aabb();

        assert_eq!(aabb.min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 1.0));
    }

    #[test]
    fn test_constraint_solving() {
        let mut soft_body = SoftBody::create_cloth(2, 2, 1.0, 1.0);
        let dt = 1.0 / 60.0;

        // Apply some disturbance
        soft_body.particles[3].position = Vec3::new(5.0, 5.0, 5.0);

        // Solve constraints
        soft_body.solve_constraints(dt);

        // The disturbed particle should be pulled back towards rest configuration
        assert!(soft_body.particles[3].position.magnitude() < 5.0);
    }

    #[test]
    fn test_particle_collision() {
        let mut particle1 = Particle::new(Vec3::new(0.0, 0.0, 0.0), 1.0, 0.1);
        let mut particle2 = Particle::new(Vec3::new(0.5, 0.0, 0.0), 1.0, 0.1);

        // Initially, they should not collide
        assert!(SoftBody::collide_particles(&particle1, &particle2).is_none());

        // Move particle2 closer to particle1
        particle2.position = Vec3::new(0.3, 0.0, 0.0);

        // They should collide now
        let collision = SoftBody::collide_particles(&particle1, &particle2);
        assert!(collision.is_some());

        // Check collision properties
        let collision = collision.unwrap();
        assert_eq!(
            collision.body_a,
            constraints::ConstraintBody::SoftBodyParticle(0, 0)
        );
        assert_eq!(
            collision.body_b,
            constraints::ConstraintBody::SoftBodyParticle(1, 0)
        );
        assert!((collision.penetration_depth - 0.2).abs() < 1e-6);
    }
}
