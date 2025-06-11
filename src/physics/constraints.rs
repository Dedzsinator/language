// Constraint Solver - XPBD (Extended Position-Based Dynamics) Implementation
use super::math::*;
use super::rigid_body::*;
use super::soft_body::*;

/// Different types of constraints in the physics system
#[derive(Debug, Clone)]
pub enum Constraint {
    Distance {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        rest_length: f64,
        stiffness: f64,
        damping: f64,
        lambda: f64, // Lagrange multiplier
    },
    Hinge {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        axis_a: Vec3,
        axis_b: Vec3,
        stiffness: f64,
        lambda: Vec3,
    },
    Fixed {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        stiffness: f64,
        lambda: Vec3,
    },
    Contact {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        contact_point: Vec3,
        contact_normal: Vec3,
        penetration_depth: f64,
        friction: f64,
        restitution: f64,
        lambda_normal: f64,
        lambda_tangent: Vec3,
    },
    Spring {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        rest_length: f64,
        spring_constant: f64,
        damping_constant: f64,
        lambda: f64,
    },
    Angular {
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        rest_angle: f64,
        axis: Vec3,
        stiffness: f64,
        lambda: f64,
    },
}

/// Reference to a body in the constraint system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintBody {
    RigidBody(usize),
    SoftBodyParticle(usize, usize), // (soft_body_index, particle_index)
    StaticPoint(Vec3),
}

/// Constraint solver using XPBD (Extended Position-Based Dynamics)
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    pub constraints: Vec<Constraint>,
    pub iterations: usize,
    pub relaxation: f64, // SOR (Successive Over-Relaxation) parameter
    pub tolerance: f64,  // Convergence tolerance
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            iterations: 10,
            relaxation: 1.8, // Optimal for most cases is between 1.0 and 2.0
            tolerance: 1e-6,
        }
    }

    /// Add a constraint to the solver
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Clear all constraints
    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
    }

    /// Solve all constraints using XPBD
    pub fn solve(&mut self, rigid_bodies: &mut [RigidBody], soft_bodies: &mut [SoftBody], dt: f64) {
        let dt2 = dt * dt;

        for _iteration in 0..self.iterations {
            let mut max_error: f64 = 0.0;

            // Clone constraints to avoid borrowing conflicts
            let mut constraints = self.constraints.clone();
            for constraint in &mut constraints {
                let error = self.solve_constraint(constraint, rigid_bodies, soft_bodies, dt2);
                max_error = max_error.max(error.abs());
            }
            // Update the solver's constraints
            self.constraints = constraints;

            // Early termination if converged
            if max_error < self.tolerance {
                break;
            }
        }
    }

    fn solve_constraint(
        &self,
        constraint: &mut Constraint,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        match constraint {
            Constraint::Distance {
                body_a,
                body_b,
                rest_length,
                stiffness,
                damping,
                lambda,
            } => self.solve_distance_constraint(
                *body_a,
                *body_b,
                *rest_length,
                *stiffness,
                *damping,
                lambda,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
            Constraint::Hinge {
                body_a,
                body_b,
                anchor_a,
                anchor_b,
                axis_a,
                axis_b,
                stiffness,
                lambda,
            } => self.solve_hinge_constraint(
                *body_a,
                *body_b,
                *anchor_a,
                *anchor_b,
                *axis_a,
                *axis_b,
                *stiffness,
                lambda,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
            Constraint::Fixed {
                body_a,
                body_b,
                anchor_a,
                anchor_b,
                stiffness,
                lambda,
            } => self.solve_fixed_constraint(
                *body_a,
                *body_b,
                *anchor_a,
                *anchor_b,
                *stiffness,
                lambda,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
            Constraint::Contact {
                body_a,
                body_b,
                contact_point,
                contact_normal,
                penetration_depth,
                friction,
                restitution,
                lambda_normal,
                lambda_tangent,
            } => self.solve_contact_constraint(
                *body_a,
                *body_b,
                *contact_point,
                *contact_normal,
                *penetration_depth,
                *friction,
                *restitution,
                lambda_normal,
                lambda_tangent,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
            Constraint::Spring {
                body_a,
                body_b,
                anchor_a,
                anchor_b,
                rest_length,
                spring_constant,
                damping_constant,
                lambda,
            } => self.solve_spring_constraint(
                *body_a,
                *body_b,
                *anchor_a,
                *anchor_b,
                *rest_length,
                *spring_constant,
                *damping_constant,
                lambda,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
            Constraint::Angular {
                body_a,
                body_b,
                rest_angle,
                axis,
                stiffness,
                lambda,
            } => self.solve_angular_constraint(
                *body_a,
                *body_b,
                *rest_angle,
                *axis,
                *stiffness,
                lambda,
                rigid_bodies,
                soft_bodies,
                dt2,
            ),
        }
    }

    fn solve_distance_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        rest_length: f64,
        stiffness: f64,
        _damping: f64,
        lambda: &mut f64,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        let (pos_a, mass_a) = self.get_position_and_mass(body_a, rigid_bodies, soft_bodies);
        let (pos_b, mass_b) = self.get_position_and_mass(body_b, rigid_bodies, soft_bodies);

        let diff = pos_b - pos_a;
        let current_length = diff.magnitude();

        if current_length < 1e-6 || (mass_a == 0.0 && mass_b == 0.0) {
            return 0.0;
        }

        let constraint_value = current_length - rest_length;
        let direction = diff / current_length;

        // XPBD compliance (inverse stiffness)
        let alpha = 1.0 / (stiffness * dt2);
        let total_mass = mass_a + mass_b;

        if total_mass == 0.0 {
            return constraint_value.abs();
        }

        let denominator = total_mass + alpha;
        let delta_lambda = (-constraint_value - alpha * *lambda) / denominator;
        *lambda += delta_lambda * self.relaxation;

        let impulse = direction * delta_lambda;

        // Apply position corrections
        if mass_a > 0.0 {
            let correction_a = impulse * (-1.0 / mass_a);
            self.apply_position_correction(body_a, correction_a, rigid_bodies, soft_bodies);
        }

        if mass_b > 0.0 {
            let correction_b = impulse * (1.0 / mass_b);
            self.apply_position_correction(body_b, correction_b, rigid_bodies, soft_bodies);
        }

        constraint_value.abs()
    }

    fn solve_hinge_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        _axis_a: Vec3,
        _axis_b: Vec3,
        stiffness: f64,
        lambda: &mut Vec3,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        // For simplicity, implementing as a fixed joint with angular constraint
        // Full hinge constraint would require more complex angular calculations
        let mut lambda_array = [lambda.x, lambda.y, lambda.z];
        let fixed_error = self.solve_fixed_constraint_impl(
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            stiffness,
            &mut lambda_array,
            rigid_bodies,
            soft_bodies,
            dt2,
        );
        lambda.x = lambda_array[0];
        lambda.y = lambda_array[1];
        lambda.z = lambda_array[2];

        // Add angular constraint for hinge axis
        // For a hinge joint, we need to constrain rotation around two axes perpendicular to the hinge axis
        let hinge_axis = Vec3::new(0.0, 1.0, 0.0); // Default Y-axis hinge
        let perp1 = Vec3::new(1.0, 0.0, 0.0); // X axis
        let perp2 = Vec3::new(0.0, 0.0, 1.0); // Z axis

        // Apply angular constraints for both perpendicular axes
        let mut angular_lambda1 = 0.0;
        let mut angular_lambda2 = 0.0;

        let angular_error1 = self.solve_angular_constraint(
            body_a,
            body_b,
            0.0,
            perp1,
            stiffness * 0.1,
            &mut angular_lambda1,
            rigid_bodies,
            soft_bodies,
            dt2,
        );
        let angular_error2 = self.solve_angular_constraint(
            body_a,
            body_b,
            0.0,
            perp2,
            stiffness * 0.1,
            &mut angular_lambda2,
            rigid_bodies,
            soft_bodies,
            dt2,
        );

        fixed_error + angular_error1 + angular_error2
    }

    fn solve_fixed_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        stiffness: f64,
        lambda: &mut Vec3,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        let mut lambda_array = [lambda.x, lambda.y, lambda.z];
        let result = self.solve_fixed_constraint_impl(
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            stiffness,
            &mut lambda_array,
            rigid_bodies,
            soft_bodies,
            dt2,
        );
        lambda.x = lambda_array[0];
        lambda.y = lambda_array[1];
        lambda.z = lambda_array[2];
        result
    }

    fn solve_fixed_constraint_impl(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        stiffness: f64,
        lambda: &mut [f64; 3],
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        let world_anchor_a = self.get_world_point(body_a, anchor_a, rigid_bodies, soft_bodies);
        let world_anchor_b = self.get_world_point(body_b, anchor_b, rigid_bodies, soft_bodies);

        let constraint_vector = world_anchor_b - world_anchor_a;
        let constraint_magnitude = constraint_vector.magnitude();

        if constraint_magnitude < 1e-6 {
            return 0.0;
        }

        let (_, mass_a) = self.get_position_and_mass(body_a, rigid_bodies, soft_bodies);
        let (_, mass_b) = self.get_position_and_mass(body_b, rigid_bodies, soft_bodies);

        let alpha = 1.0 / (stiffness * dt2);
        let total_mass = mass_a + mass_b;

        if total_mass == 0.0 {
            return constraint_magnitude;
        }

        // Solve for each axis separately
        let constraint_components = [
            constraint_vector.x,
            constraint_vector.y,
            constraint_vector.z,
        ];
        for i in 0..3 {
            let constraint_value = constraint_components[i];
            let denominator = total_mass + alpha;
            let delta_lambda = (-constraint_value - alpha * lambda[i]) / denominator;
            lambda[i] += delta_lambda * self.relaxation;

            let impulse = match i {
                0 => Vec3::new(delta_lambda, 0.0, 0.0),
                1 => Vec3::new(0.0, delta_lambda, 0.0),
                2 => Vec3::new(0.0, 0.0, delta_lambda),
                _ => Vec3::zero(),
            };

            // Apply corrections
            if mass_a > 0.0 {
                let correction_a = impulse * (-1.0 / mass_a);
                self.apply_position_correction(body_a, correction_a, rigid_bodies, soft_bodies);
            }

            if mass_b > 0.0 {
                let correction_b = impulse * (1.0 / mass_b);
                self.apply_position_correction(body_b, correction_b, rigid_bodies, soft_bodies);
            }
        }

        constraint_magnitude
    }

    fn solve_contact_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        _contact_point: Vec3,
        contact_normal: Vec3,
        penetration_depth: f64,
        _friction: f64,
        _restitution: f64,
        lambda_normal: &mut f64,
        _lambda_tangent: &mut Vec3,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        _dt2: f64,
    ) -> f64 {
        if penetration_depth <= 0.0 {
            return 0.0;
        }

        let (_, mass_a) = self.get_position_and_mass(body_a, rigid_bodies, soft_bodies);
        let (_, mass_b) = self.get_position_and_mass(body_b, rigid_bodies, soft_bodies);

        let total_mass = mass_a + mass_b;
        if total_mass == 0.0 {
            return penetration_depth;
        }

        // Normal constraint (separation)
        let constraint_value = penetration_depth;
        let denominator = total_mass; // No compliance for contacts
        let delta_lambda_normal = -constraint_value / denominator;

        // Non-negative constraint for normal impulse
        let old_lambda = *lambda_normal;
        *lambda_normal = (*lambda_normal + delta_lambda_normal).max(0.0);
        let actual_delta_lambda = *lambda_normal - old_lambda;

        let normal_impulse = contact_normal * actual_delta_lambda;

        // Apply normal impulse
        if mass_a > 0.0 {
            let correction_a = normal_impulse * (-1.0 / mass_a);
            self.apply_position_correction(body_a, correction_a, rigid_bodies, soft_bodies);
        }

        if mass_b > 0.0 {
            let correction_b = normal_impulse * (1.0 / mass_b);
            self.apply_position_correction(body_b, correction_b, rigid_bodies, soft_bodies);
        }

        // Add friction constraint (tangential)
        // Calculate relative velocity at contact point using contact point as anchor
        let world_anchor_a = self.get_world_point(body_a, Vec3::zero(), rigid_bodies, soft_bodies);
        let world_anchor_b = self.get_world_point(body_b, Vec3::zero(), rigid_bodies, soft_bodies);

        let vel_a = self.get_velocity_at_point(body_a, world_anchor_a, rigid_bodies, soft_bodies);
        let vel_b = self.get_velocity_at_point(body_b, world_anchor_b, rigid_bodies, soft_bodies);
        let relative_velocity = vel_b - vel_a;

        // Calculate tangential velocity (perpendicular to contact normal)
        let normal = contact_normal.normalized();
        let normal_velocity = normal * relative_velocity.dot(normal);
        let tangential_velocity = relative_velocity - normal_velocity;

        if tangential_velocity.magnitude() > 1e-6 {
            let friction_coefficient = 0.3; // Default friction coefficient
            let max_friction_force = friction_coefficient * normal_impulse.magnitude();

            let tangent_direction = tangential_velocity.normalized();
            let friction_impulse =
                -tangent_direction * max_friction_force.min(tangential_velocity.magnitude() * 0.1);

            // Apply friction impulse
            if mass_a > 0.0 {
                let friction_correction_a = friction_impulse * (1.0 / mass_a);
                self.apply_velocity_correction(
                    body_a,
                    friction_correction_a,
                    rigid_bodies,
                    soft_bodies,
                );
            }
            if mass_b > 0.0 {
                let friction_correction_b = -friction_impulse * (1.0 / mass_b);
                self.apply_velocity_correction(
                    body_b,
                    friction_correction_b,
                    rigid_bodies,
                    soft_bodies,
                );
            }
        }

        constraint_value.abs()
    }

    fn solve_spring_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        rest_length: f64,
        spring_constant: f64,
        _damping_constant: f64,
        lambda: &mut f64,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        let world_anchor_a = self.get_world_point(body_a, anchor_a, rigid_bodies, soft_bodies);
        let world_anchor_b = self.get_world_point(body_b, anchor_b, rigid_bodies, soft_bodies);

        let diff = world_anchor_b - world_anchor_a;
        let current_length = diff.magnitude();

        if current_length < 1e-6 {
            return 0.0;
        }

        let constraint_value = current_length - rest_length;
        let direction = diff / current_length;

        // Spring stiffness in XPBD is handled through compliance
        let stiffness = spring_constant; // Convert spring constant to stiffness
        let alpha = 1.0 / (stiffness * dt2);

        let (_, mass_a) = self.get_position_and_mass(body_a, rigid_bodies, soft_bodies);
        let (_, mass_b) = self.get_position_and_mass(body_b, rigid_bodies, soft_bodies);
        let total_mass = mass_a + mass_b;

        if total_mass == 0.0 {
            return constraint_value.abs();
        }

        let denominator = total_mass + alpha;
        let delta_lambda = (-constraint_value - alpha * *lambda) / denominator;
        *lambda += delta_lambda;

        let impulse = direction * delta_lambda;

        // Apply position corrections
        if mass_a > 0.0 {
            let correction_a = impulse * (-1.0 / mass_a);
            self.apply_position_correction(body_a, correction_a, rigid_bodies, soft_bodies);
        }

        if mass_b > 0.0 {
            let correction_b = impulse * (1.0 / mass_b);
            self.apply_position_correction(body_b, correction_b, rigid_bodies, soft_bodies);
        }

        constraint_value.abs()
    }

    fn solve_angular_constraint(
        &self,
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        rest_angle: f64,
        axis: Vec3,
        stiffness: f64,
        lambda: &mut f64,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt2: f64,
    ) -> f64 {
        // Implement proper angular constraint with quaternion differences
        let (orientation_a, inertia_a) =
            self.get_orientation_and_inertia(body_a, rigid_bodies, soft_bodies);
        let (orientation_b, inertia_b) =
            self.get_orientation_and_inertia(body_b, rigid_bodies, soft_bodies);

        // Calculate relative rotation between bodies
        let relative_rotation = orientation_b * orientation_a.conjugate();

        // Convert to axis-angle representation
        let (rel_axis, rel_angle) = relative_rotation.to_axis_angle();

        // Project the relative angle onto the constraint axis
        let projected_angle = if rel_axis.magnitude() > 1e-6 {
            rel_angle * rel_axis.normalized().dot(axis.normalized())
        } else {
            0.0
        };

        // Calculate constraint violation
        let angle_error = projected_angle - rest_angle;

        if angle_error.abs() < 1e-6 {
            return 0.0;
        }

        // Calculate constraint impulse
        let constraint_axis = axis.normalized();
        let effective_inertia_a = if inertia_a > 0.0 {
            1.0 / inertia_a
        } else {
            0.0
        };
        let effective_inertia_b = if inertia_b > 0.0 {
            1.0 / inertia_b
        } else {
            0.0
        };
        let total_inertia = effective_inertia_a + effective_inertia_b;

        if total_inertia > 1e-6 {
            let impulse_magnitude = -stiffness * angle_error / total_inertia;
            let angular_impulse = constraint_axis * impulse_magnitude;

            // Apply angular impulses
            if inertia_a > 0.0 {
                self.apply_angular_impulse(body_a, angular_impulse, rigid_bodies, soft_bodies);
            }
            if inertia_b > 0.0 {
                self.apply_angular_impulse(body_b, -angular_impulse, rigid_bodies, soft_bodies);
            }

            *lambda += impulse_magnitude;
        }

        angle_error.abs()
    }

    fn get_position_and_mass(
        &self,
        body: ConstraintBody,
        rigid_bodies: &[RigidBody],
        soft_bodies: &[SoftBody],
    ) -> (Vec3, f64) {
        match body {
            ConstraintBody::RigidBody(index) => {
                (rigid_bodies[index].position, rigid_bodies[index].mass)
            }
            ConstraintBody::SoftBodyParticle(body_index, particle_index) => {
                let particle = &soft_bodies[body_index].particles[particle_index];
                (particle.position, particle.mass)
            }
            ConstraintBody::StaticPoint(position) => {
                (position, 0.0) // Static points have infinite mass (represented as 0)
            }
        }
    }

    fn get_world_point(
        &self,
        body: ConstraintBody,
        local_point: Vec3,
        rigid_bodies: &[RigidBody],
        soft_bodies: &[SoftBody],
    ) -> Vec3 {
        match body {
            ConstraintBody::RigidBody(index) => {
                let rigid_body = &rigid_bodies[index];
                rigid_body.position + rigid_body.rotation.rotate_vector(local_point)
            }
            ConstraintBody::SoftBodyParticle(body_index, particle_index) => {
                // For particles, local_point is typically zero (particle center)
                soft_bodies[body_index].particles[particle_index].position + local_point
            }
            ConstraintBody::StaticPoint(position) => position + local_point,
        }
    }

    fn apply_position_correction(
        &self,
        body: ConstraintBody,
        correction: Vec3,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
    ) {
        match body {
            ConstraintBody::RigidBody(index) => {
                rigid_bodies[index].position = rigid_bodies[index].position + correction;
            }
            ConstraintBody::SoftBodyParticle(body_index, particle_index) => {
                if !soft_bodies[body_index].particles[particle_index].pinned {
                    soft_bodies[body_index].particles[particle_index].position =
                        soft_bodies[body_index].particles[particle_index].position + correction;
                }
            }
            ConstraintBody::StaticPoint(_) => {
                // Static points don't move
            }
        }
    }

    // Helper methods for physics constraints
    fn get_velocity_at_point(
        &self,
        body: ConstraintBody,
        point: Vec3,
        rigid_bodies: &[RigidBody],
        soft_bodies: &[SoftBody],
    ) -> Vec3 {
        match body {
            ConstraintBody::RigidBody(index) => {
                if let Some(rb) = rigid_bodies.get(index) {
                    rb.velocity + rb.angular_velocity.cross(point - rb.position)
                } else {
                    Vec3::zero()
                }
            }
            ConstraintBody::SoftBodyParticle(body_idx, particle_idx) => {
                if let Some(sb) = soft_bodies.get(body_idx) {
                    if let Some(particle) = sb.particles.get(particle_idx) {
                        particle.velocity
                    } else {
                        Vec3::zero()
                    }
                } else {
                    Vec3::zero()
                }
            }
            ConstraintBody::StaticPoint(_) => Vec3::zero(),
        }
    }

    fn apply_velocity_correction(
        &self,
        body: ConstraintBody,
        correction: Vec3,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
    ) {
        match body {
            ConstraintBody::RigidBody(index) => {
                if let Some(rb) = rigid_bodies.get_mut(index) {
                    rb.velocity += correction;
                }
            }
            ConstraintBody::SoftBodyParticle(body_idx, particle_idx) => {
                if let Some(sb) = soft_bodies.get_mut(body_idx) {
                    if let Some(particle) = sb.particles.get_mut(particle_idx) {
                        particle.velocity += correction;
                    }
                }
            }
            ConstraintBody::StaticPoint(_) => {} // Static points don't move
        }
    }

    fn get_orientation_and_inertia(
        &self,
        body: ConstraintBody,
        rigid_bodies: &[RigidBody],
        _soft_bodies: &[SoftBody],
    ) -> (Quat, f64) {
        match body {
            ConstraintBody::RigidBody(index) => {
                if let Some(rb) = rigid_bodies.get(index) {
                    (rb.rotation, rb.inertia_tensor.data[0][0]) // Use first diagonal element of inertia tensor
                } else {
                    (Quat::identity(), 0.0)
                }
            }
            ConstraintBody::SoftBodyParticle(_, _) => {
                // Soft body particles don't have orientation
                (Quat::identity(), 0.0)
            }
            ConstraintBody::StaticPoint(_) => (Quat::identity(), 0.0),
        }
    }

    fn apply_angular_impulse(
        &self,
        body: ConstraintBody,
        impulse: Vec3,
        rigid_bodies: &mut [RigidBody],
        _soft_bodies: &mut [SoftBody],
    ) {
        match body {
            ConstraintBody::RigidBody(index) => {
                if let Some(rb) = rigid_bodies.get_mut(index) {
                    let inertia_x = rb.inertia_tensor.data[0][0];
                    if inertia_x > 0.0 {
                        rb.angular_velocity += impulse / inertia_x;
                    }
                }
            }
            ConstraintBody::SoftBodyParticle(_, _) => {
                // Soft body particles don't have angular velocity
            }
            ConstraintBody::StaticPoint(_) => {
                // Static points don't move
            }
        }
    }

    /// Create a distance constraint between two bodies
    pub fn create_distance_constraint(
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        rest_length: f64,
        stiffness: f64,
    ) -> Constraint {
        Constraint::Distance {
            body_a,
            body_b,
            rest_length,
            stiffness,
            damping: 0.1,
            lambda: 0.0,
        }
    }

    /// Create a fixed joint constraint
    pub fn create_fixed_constraint(
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        stiffness: f64,
    ) -> Constraint {
        Constraint::Fixed {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            stiffness,
            lambda: Vec3::zero(),
        }
    }

    /// Create a contact constraint for collision resolution
    pub fn create_contact_constraint(
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        contact_point: Vec3,
        contact_normal: Vec3,
        penetration_depth: f64,
        friction: f64,
        restitution: f64,
    ) -> Constraint {
        Constraint::Contact {
            body_a,
            body_b,
            contact_point,
            contact_normal,
            penetration_depth,
            friction,
            restitution,
            lambda_normal: 0.0,
            lambda_tangent: Vec3::zero(),
        }
    }

    /// Create a spring constraint
    pub fn create_spring_constraint(
        body_a: ConstraintBody,
        body_b: ConstraintBody,
        anchor_a: Vec3,
        anchor_b: Vec3,
        rest_length: f64,
        spring_constant: f64,
        damping_constant: f64,
    ) -> Constraint {
        Constraint::Spring {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            rest_length,
            spring_constant,
            damping_constant,
            lambda: 0.0,
        }
    }
}

/// Extended Position-Based Dynamics (XPBD) solver implementation
#[derive(Debug, Clone)]
pub struct XPBDSolver {
    pub constraints: Vec<Constraint>,
    pub iterations: usize,
    pub relaxation: f64,
    pub tolerance: f64,
}

impl XPBDSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            iterations: 10,
            relaxation: 1.8,
            tolerance: 1e-6,
        }
    }

    pub fn add_contact_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn solve(&mut self, dt: f64, rigid_bodies: &mut [RigidBody], soft_bodies: &mut [SoftBody]) {
        let dt2 = dt * dt;

        for _ in 0..self.iterations {
            // Clone constraints to avoid borrowing conflicts
            let mut constraints = self.constraints.clone();
            for constraint in &mut constraints {
                match constraint {
                    Constraint::Contact { .. } => {
                        Self::solve_contact_constraint_static(
                            constraint,
                            rigid_bodies,
                            soft_bodies,
                            dt2,
                        );
                    }
                    Constraint::Distance { .. } => {
                        Self::solve_distance_constraint_static(
                            constraint,
                            rigid_bodies,
                            soft_bodies,
                            dt2,
                        );
                    }
                    _ => {
                        // Handle other constraint types
                    }
                }
            }
            // Update the original constraints with any changes
            self.constraints = constraints;
        }
    }

    pub fn solve_iteration(
        &mut self,
        rigid_bodies: &mut [RigidBody],
        soft_bodies: &mut [SoftBody],
        dt: f64,
    ) {
        let dt2 = dt * dt;

        // Clone constraints to avoid borrowing conflicts
        let mut constraints = self.constraints.clone();
        for constraint in &mut constraints {
            match constraint {
                Constraint::Contact { .. } => {
                    Self::solve_contact_constraint_static(
                        constraint,
                        rigid_bodies,
                        soft_bodies,
                        dt2,
                    );
                }
                Constraint::Distance { .. } => {
                    Self::solve_distance_constraint_static(
                        constraint,
                        rigid_bodies,
                        soft_bodies,
                        dt2,
                    );
                }
                _ => {
                    // Handle other constraint types
                }
            }
        }
        // Update the original constraints with any changes
        self.constraints = constraints;
    }

    fn solve_contact_constraint_static(
        _constraint: &mut Constraint,
        _rigid_bodies: &mut [RigidBody],
        _soft_bodies: &mut [SoftBody],
        _dt2: f64,
    ) {
        // Implementation would go here
    }

    fn solve_distance_constraint_static(
        _constraint: &mut Constraint,
        _rigid_bodies: &mut [RigidBody],
        _soft_bodies: &mut [SoftBody],
        _dt2: f64,
    ) {
        // Implementation would go here
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_solver_creation() {
        let solver = ConstraintSolver::new();

        assert_eq!(solver.constraints.len(), 0);
        assert_eq!(solver.iterations, 10);
        assert!(solver.relaxation > 1.0 && solver.relaxation < 2.0);
        assert!(solver.tolerance > 0.0);
    }

    #[test]
    fn test_add_constraint() {
        let mut solver = ConstraintSolver::new();

        let constraint = ConstraintSolver::create_distance_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::RigidBody(1),
            1.0,
            1000.0,
        );

        solver.add_constraint(constraint);
        assert_eq!(solver.constraints.len(), 1);
    }

    #[test]
    fn test_clear_constraints() {
        let mut solver = ConstraintSolver::new();

        solver.add_constraint(ConstraintSolver::create_distance_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::RigidBody(1),
            1.0,
            1000.0,
        ));

        assert_eq!(solver.constraints.len(), 1);

        solver.clear_constraints();
        assert_eq!(solver.constraints.len(), 0);
    }

    #[test]
    fn test_distance_constraint_creation() {
        let constraint = ConstraintSolver::create_distance_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::RigidBody(1),
            2.0,
            500.0,
        );

        if let Constraint::Distance {
            rest_length,
            stiffness,
            ..
        } = constraint
        {
            assert_eq!(rest_length, 2.0);
            assert_eq!(stiffness, 500.0);
        } else {
            panic!("Expected Distance constraint");
        }
    }

    #[test]
    fn test_fixed_constraint_creation() {
        let constraint = ConstraintSolver::create_fixed_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::StaticPoint(Vec3::zero()),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::zero(),
            1000.0,
        );

        if let Constraint::Fixed {
            anchor_a,
            anchor_b,
            stiffness,
            ..
        } = constraint
        {
            assert_eq!(anchor_a, Vec3::new(1.0, 0.0, 0.0));
            assert_eq!(anchor_b, Vec3::zero());
            assert_eq!(stiffness, 1000.0);
        } else {
            panic!("Expected Fixed constraint");
        }
    }

    #[test]
    fn test_contact_constraint_creation() {
        let constraint = ConstraintSolver::create_contact_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::RigidBody(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            0.1,
            0.5,
            0.8,
        );

        if let Constraint::Contact {
            contact_normal,
            penetration_depth,
            friction,
            restitution,
            ..
        } = constraint
        {
            assert_eq!(contact_normal, Vec3::new(0.0, 1.0, 0.0));
            assert_eq!(penetration_depth, 0.1);
            assert_eq!(friction, 0.5);
            assert_eq!(restitution, 0.8);
        } else {
            panic!("Expected Contact constraint");
        }
    }

    #[test]
    fn test_spring_constraint_creation() {
        let constraint = ConstraintSolver::create_spring_constraint(
            ConstraintBody::RigidBody(0),
            ConstraintBody::RigidBody(1),
            Vec3::zero(),
            Vec3::zero(),
            1.0,
            100.0,
            5.0,
        );

        if let Constraint::Spring {
            rest_length,
            spring_constant,
            damping_constant,
            ..
        } = constraint
        {
            assert_eq!(rest_length, 1.0);
            assert_eq!(spring_constant, 100.0);
            assert_eq!(damping_constant, 5.0);
        } else {
            panic!("Expected Spring constraint");
        }
    }

    #[test]
    fn test_constraint_body_types() {
        let rigid_body = ConstraintBody::RigidBody(5);
        let particle = ConstraintBody::SoftBodyParticle(2, 10);
        let static_point = ConstraintBody::StaticPoint(Vec3::new(1.0, 2.0, 3.0));

        match rigid_body {
            ConstraintBody::RigidBody(index) => assert_eq!(index, 5),
            _ => panic!("Expected RigidBody"),
        }

        match particle {
            ConstraintBody::SoftBodyParticle(body_idx, particle_idx) => {
                assert_eq!(body_idx, 2);
                assert_eq!(particle_idx, 10);
            }
            _ => panic!("Expected SoftBodyParticle"),
        }

        match static_point {
            ConstraintBody::StaticPoint(pos) => assert_eq!(pos, Vec3::new(1.0, 2.0, 3.0)),
            _ => panic!("Expected StaticPoint"),
        }
    }

    #[test]
    fn test_solver_parameters() {
        let mut solver = ConstraintSolver::new();

        solver.iterations = 20;
        solver.relaxation = 1.5;
        solver.tolerance = 1e-8;

        assert_eq!(solver.iterations, 20);
        assert_eq!(solver.relaxation, 1.5);
        assert_eq!(solver.tolerance, 1e-8);
    }
}
