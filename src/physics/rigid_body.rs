// Modern Rigid Body Physics using impulse-based dynamics
use super::math::{Vec3, Quat, Mat3, Transform, AABB};
use super::constraints::{Constraint, ConstraintBody};
use crate::eval::interpreter::{Value, RuntimeResult};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Rigid body shapes for collision detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Sphere { radius: f64 },
    Box { size: Vec3 },
    Capsule { radius: f64, height: f64 },
    Cylinder { radius: f64, height: f64 },
    ConvexHull { vertices: Vec<Vec3> },
    TriangleMesh { vertices: Vec<Vec3>, indices: Vec<[usize; 3]> },
}

impl Shape {
    /// Calculate inertia tensor for the shape
    pub fn inertia_tensor(&self, mass: f64) -> Mat3 {
        match self {
            Shape::Sphere { radius } => {
                let inertia = 0.4 * mass * radius * radius;
                Mat3::from_diagonal(inertia, inertia, inertia)
            },
            Shape::Box { size } => {
                let x2 = size.x * size.x;
                let y2 = size.y * size.y;
                let z2 = size.z * size.z;
                let factor = mass / 12.0;
                Mat3::from_diagonal(
                    factor * (y2 + z2),
                    factor * (x2 + z2),
                    factor * (x2 + y2),
                )
            },
            Shape::Capsule { radius, height } => {
                // Approximate as cylinder + hemisphere caps
                let cylinder_mass = mass * height / (height + 4.0 * radius / 3.0);
                let sphere_mass = mass - cylinder_mass;

                let cyl_inertia_y = 0.5 * cylinder_mass * radius * radius;
                let cyl_inertia_xz = cylinder_mass * (3.0 * radius * radius + height * height) / 12.0;

                let sphere_inertia = 0.4 * sphere_mass * radius * radius;
                let sphere_offset = (height / 2.0 + radius * 0.375).powi(2);

                Mat3::from_diagonal(
                    cyl_inertia_xz + sphere_inertia + sphere_mass * sphere_offset,
                    cyl_inertia_y + sphere_inertia,
                    cyl_inertia_xz + sphere_inertia + sphere_mass * sphere_offset,
                )
            },
            Shape::Cylinder { radius, height } => {
                let inertia_y = 0.5 * mass * radius * radius;
                let inertia_xz = mass * (3.0 * radius * radius + height * height) / 12.0;
                Mat3::from_diagonal(inertia_xz, inertia_y, inertia_xz)
            },
            Shape::ConvexHull { vertices } => {
                // Approximate using bounding box
                if vertices.is_empty() {
                    return Mat3::identity();
                }

                let mut min = vertices[0];
                let mut max = vertices[0];
                for &vertex in vertices {
                    min = Vec3::new(min.x.min(vertex.x), min.y.min(vertex.y), min.z.min(vertex.z));
                    max = Vec3::new(max.x.max(vertex.x), max.y.max(vertex.y), max.z.max(vertex.z));
                }

                let size = max - min;
                Shape::Box { size }.inertia_tensor(mass)
            },
            Shape::TriangleMesh { .. } => {
                // For triangle meshes, use bounding box approximation
                // In a full implementation, would compute actual inertia tensor
                Mat3::identity() * mass
            },
        }
    }

    /// Get bounding box for the shape
    pub fn bounding_box(&self, transform: &Transform) -> AABB {
        match self {
            Shape::Sphere { radius } => {
                AABB::from_point(transform.position, *radius)
            },
            Shape::Box { size } => {
                // Transform all 8 corners of the box
                let half_size = *size * 0.5;
                let corners = [
                    Vec3::new(-half_size.x, -half_size.y, -half_size.z),
                    Vec3::new( half_size.x, -half_size.y, -half_size.z),
                    Vec3::new(-half_size.x,  half_size.y, -half_size.z),
                    Vec3::new( half_size.x,  half_size.y, -half_size.z),
                    Vec3::new(-half_size.x, -half_size.y,  half_size.z),
                    Vec3::new( half_size.x, -half_size.y,  half_size.z),
                    Vec3::new(-half_size.x,  half_size.y,  half_size.z),
                    Vec3::new( half_size.x,  half_size.y,  half_size.z),
                ];

                let transformed_corners: Vec<Vec3> = corners.iter()
                    .map(|&corner| transform.transform_point(corner))
                    .collect();

                let mut min = transformed_corners[0];
                let mut max = transformed_corners[0];
                for &corner in &transformed_corners[1..] {
                    min = Vec3::new(min.x.min(corner.x), min.y.min(corner.y), min.z.min(corner.z));
                    max = Vec3::new(max.x.max(corner.x), max.y.max(corner.y), max.z.max(corner.z));
                }

                AABB::new(min, max)
            },
            Shape::Capsule { radius, height } => {
                let half_height = height * 0.5;
                let extent = *radius + half_height;
                AABB::from_point(transform.position, extent)
            },
            Shape::Cylinder { radius, height } => {
                let half_height = height * 0.5;
                let extent = radius.max(half_height);
                AABB::from_point(transform.position, extent)
            },
            Shape::ConvexHull { vertices } => {
                if vertices.is_empty() {
                    return AABB::from_point(transform.position, 0.0);
                }

                let transformed_vertices: Vec<Vec3> = vertices.iter()
                    .map(|&vertex| transform.transform_point(vertex))
                    .collect();

                let mut min = transformed_vertices[0];
                let mut max = transformed_vertices[0];
                for &vertex in &transformed_vertices[1..] {
                    min = Vec3::new(min.x.min(vertex.x), min.y.min(vertex.y), min.z.min(vertex.z));
                    max = Vec3::new(max.x.max(vertex.x), max.y.max(vertex.y), max.z.max(vertex.z));
                }

                AABB::new(min, max)
            },
            Shape::TriangleMesh { vertices, .. } => {
                if vertices.is_empty() {
                    return AABB::from_point(transform.position, 0.0);
                }

                let transformed_vertices: Vec<Vec3> = vertices.iter()
                    .map(|&vertex| transform.transform_point(vertex))
                    .collect();

                let mut min = transformed_vertices[0];
                let mut max = transformed_vertices[0];
                for &vertex in &transformed_vertices[1..] {
                    min = Vec3::new(min.x.min(vertex.x), min.y.min(vertex.y), min.z.min(vertex.z));
                    max = Vec3::new(max.x.max(vertex.x), max.y.max(vertex.y), max.z.max(vertex.z));
                }

                AABB::new(min, max)
            },
        }
    }
}

/// Rigid body with modern dynamics
#[derive(Debug, Clone)]
pub struct RigidBody {
    // Transform
    pub position: Vec3,
    pub rotation: Quat,

    // Previous transform for Verlet integration
    pub prev_position: Vec3,
    pub prev_rotation: Quat,

    // Linear motion
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub force: Vec3,
    pub mass: f64,
    pub inv_mass: f64,

    // Angular motion
    pub angular_velocity: Vec3,
    pub angular_acceleration: Vec3,
    pub torque: Vec3,
    pub inertia_tensor: Mat3,
    pub inv_inertia_tensor: Mat3,
    pub world_inv_inertia_tensor: Mat3,

    // Material properties
    pub restitution: f64,
    pub friction: f64,
    pub density: f64,

    // Collision shape
    pub shape: Shape,

    // State flags
    pub is_static: bool,
    pub is_kinematic: bool,
    pub is_sleeping: bool,
    pub gravity_scale: f64,

    // Linear and angular damping
    pub linear_damping: f64,
    pub angular_damping: f64,

    // For constraint solving
    pub constraint_force: Vec3,
    pub constraint_torque: Vec3,
}

impl RigidBody {
    pub fn new(shape: Shape, mass: f64, position: Vec3) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let inertia_tensor = shape.inertia_tensor(mass);
        let inv_inertia_tensor = if mass > 0.0 {
            inertia_tensor.inverse().unwrap_or(Mat3::identity())
        } else {
            Mat3::zero()
        };

        Self {
            position,
            rotation: Quat::identity(),
            prev_position: position,
            prev_rotation: Quat::identity(),
            velocity: Vec3::zero(),
            acceleration: Vec3::zero(),
            force: Vec3::zero(),
            mass,
            inv_mass,
            angular_velocity: Vec3::zero(),
            angular_acceleration: Vec3::zero(),
            torque: Vec3::zero(),
            inertia_tensor,
            inv_inertia_tensor,
            world_inv_inertia_tensor: inv_inertia_tensor,
            restitution: 0.6,
            friction: 0.7,
            density: 1.0,
            shape,
            is_static: mass == 0.0,
            is_kinematic: false,
            is_sleeping: false,
            gravity_scale: 1.0,
            linear_damping: 0.01,
            angular_damping: 0.05,
            constraint_force: Vec3::zero(),
            constraint_torque: Vec3::zero(),
        }
    }

    /// Update world-space inverse inertia tensor
    pub fn update_world_inertia(&mut self) {
        if self.is_static {
            self.world_inv_inertia_tensor = Mat3::zero();
        } else {
            let rotation_matrix = self.rotation.to_rotation_matrix();
            self.world_inv_inertia_tensor = rotation_matrix * self.inv_inertia_tensor * rotation_matrix.transpose();
        }
    }

    /// Apply force at center of mass
    pub fn apply_force(&mut self, force: Vec3) {
        if !self.is_static {
            self.force += force;
        }
    }

    /// Apply force at a world point (generates torque)
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        if !self.is_static {
            self.force += force;
            let r = point - self.position;
            self.torque += r.cross(force);
        }
    }

    /// Apply impulse at center of mass
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_static {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Apply impulse at a world point
    pub fn apply_impulse_at_point(&mut self, impulse: Vec3, point: Vec3) {
        if !self.is_static {
            self.velocity += impulse * self.inv_mass;
            let r = point - self.position;
            self.angular_velocity += self.world_inv_inertia_tensor.transform_vector(r.cross(impulse));
        }
    }

    /// Apply torque
    pub fn apply_torque(&mut self, torque: Vec3) {
        if !self.is_static {
            self.torque += torque;
        }
    }

    /// Get velocity at a world point
    pub fn velocity_at_point(&self, point: Vec3) -> Vec3 {
        let r = point - self.position;
        self.velocity + self.angular_velocity.cross(r)
    }

    /// Integration step using Verlet integration
    pub fn integrate_forces(&mut self, dt: f64, gravity: Vec3) {
        if self.is_static || self.is_kinematic {
            return;
        }

        // Store previous state for Verlet
        self.prev_position = self.position;
        self.prev_rotation = self.rotation;

        // Apply gravity
        if self.gravity_scale != 0.0 {
            self.force += gravity * self.mass * self.gravity_scale;
        }

        // Integrate linear motion
        self.acceleration = self.force * self.inv_mass;

        // Semi-implicit Euler for stability
        self.velocity += self.acceleration * dt;
        self.velocity *= 1.0 - self.linear_damping * dt; // Linear damping
        self.position += self.velocity * dt;

        // Integrate angular motion
        self.update_world_inertia();
        self.angular_acceleration = self.world_inv_inertia_tensor.transform_vector(self.torque);

        self.angular_velocity += self.angular_acceleration * dt;
        self.angular_velocity *= 1.0 - self.angular_damping * dt; // Angular damping

        // Integrate rotation using quaternion
        if self.angular_velocity.magnitude_squared() > 0.0 {
            let angular_speed = self.angular_velocity.magnitude();
            let axis = self.angular_velocity.normalized();
            let delta_rotation = Quat::from_axis_angle(axis, angular_speed * dt);
            self.rotation = delta_rotation * self.rotation;
            self.rotation = self.rotation.normalized();
        }

        // Clear forces for next frame
        self.force = Vec3::zero();
        self.torque = Vec3::zero();
    }

    /// Finalize integration step
    pub fn finalize_step(&mut self, dt: f64) {
        if self.is_static || self.is_kinematic {
            return;
        }

        // Update velocity from position change (for constraint solving)
        self.velocity = (self.position - self.prev_position) / dt;

        // Update angular velocity from rotation change
        if self.prev_rotation != self.rotation {
            let delta_rotation = self.rotation * self.prev_rotation.conjugate();
            if delta_rotation.magnitude_squared() > 0.0 {
                let angle = 2.0 * delta_rotation.w.acos();
                let axis = Vec3::new(delta_rotation.x, delta_rotation.y, delta_rotation.z).normalized();
                self.angular_velocity = axis * (angle / dt);
            }
        }
    }

    /// Apply damping
    pub fn apply_damping(&mut self, damping: f64) {
        if !self.is_static {
            self.velocity *= damping;
            self.angular_velocity *= damping;
        }
    }

    /// Get current transform
    pub fn transform(&self) -> Transform {
        Transform::new(self.position, self.rotation, Vec3::one())
    }

    /// Get AABB for collision detection
    pub fn aabb(&self) -> AABB {
        self.shape.bounding_box(&self.transform())
    }

    /// Convert to language value for visualization
    pub fn to_value(&self, id: usize) -> RuntimeResult<Value> {
        let mut fields = HashMap::new();

        fields.insert("id".to_string(), Value::Int(id as i64));
        fields.insert("position".to_string(), self.position.to_value()?);
        fields.insert("rotation".to_string(), Value::Array(vec![
            Value::Float(self.rotation.w),
            Value::Float(self.rotation.x),
            Value::Float(self.rotation.y),
            Value::Float(self.rotation.z),
        ]));
        fields.insert("velocity".to_string(), self.velocity.to_value()?);
        fields.insert("angular_velocity".to_string(), self.angular_velocity.to_value()?);
        fields.insert("mass".to_string(), Value::Float(self.mass));
        fields.insert("is_static".to_string(), Value::Bool(self.is_static));

        // Shape info
        let shape_data = match &self.shape {
            Shape::Sphere { radius } => {
                let mut shape_fields = HashMap::new();
                shape_fields.insert("type".to_string(), Value::String("sphere".to_string()));
                shape_fields.insert("radius".to_string(), Value::Float(*radius));
                Value::Struct { name: "Shape".to_string(), fields: shape_fields }
            },
            Shape::Box { size } => {
                let mut shape_fields = HashMap::new();
                shape_fields.insert("type".to_string(), Value::String("box".to_string()));
                shape_fields.insert("size".to_string(), size.to_value()?);
                Value::Struct { name: "Shape".to_string(), fields: shape_fields }
            },
            _ => Value::String("complex_shape".to_string()),
        };
        fields.insert("shape".to_string(), shape_data);

        Ok(Value::Struct {
            name: "RigidBody".to_string(),
            fields,
        })
    }
}

/// Collision detection between rigid bodies
pub fn collide(body1: &RigidBody, body2: &RigidBody) -> Option<Constraint> {
    // Broad phase - AABB check
    if !body1.aabb().intersects(body2.aabb()) {
        return None;
    }

    // Narrow phase collision detection
    match (&body1.shape, &body2.shape) {
        (Shape::Sphere { radius: r1 }, Shape::Sphere { radius: r2 }) => {
            sphere_sphere_collision(body1, body2, *r1, *r2)
        },
        (Shape::Sphere { radius }, Shape::Box { size }) => {
            sphere_box_collision(body1, body2, *radius, *size)
        },
        (Shape::Box { size }, Shape::Sphere { radius }) => {
            sphere_box_collision(body2, body1, *radius, *size)
        },
        (Shape::Box { size: size1 }, Shape::Box { size: size2 }) => {
            box_box_collision(body1, body2, *size1, *size2)
        },
        _ => {
            // For other shape combinations, use bounding box approximation
            None
        }
    }
}

fn sphere_sphere_collision(body1: &RigidBody, body2: &RigidBody, r1: f64, r2: f64) -> Option<Constraint> {
    let diff = body2.position - body1.position;
    let distance_squared = diff.magnitude_squared();
    let radius_sum = r1 + r2;

    if distance_squared < radius_sum * radius_sum {
        let distance = distance_squared.sqrt();
        let normal = if distance > f64::EPSILON {
            diff / distance
        } else {
            Vec3::up() // Arbitrary direction for coincident spheres
        };

        let penetration = radius_sum - distance;
        let contact_point = body1.position + normal * r1;

        Some(Constraint::Contact {
            body_a: ConstraintBody::RigidBody(0), // Would need proper indices
            body_b: ConstraintBody::RigidBody(1),
            contact_point,
            contact_normal: normal,
            penetration_depth: penetration,
            friction: body1.friction.max(body2.friction),
            restitution: body1.restitution.min(body2.restitution),
            lambda_normal: 0.0,
            lambda_tangent: Vec3::zero(),
        })
    } else {
        None
    }
}

fn sphere_box_collision(sphere_body: &RigidBody, box_body: &RigidBody, radius: f64, box_size: Vec3) -> Option<Constraint> {
    // Transform sphere center to box local space
    let box_transform = box_body.transform();
    let box_inv_transform = box_transform.inverse();
    let local_sphere_center = box_inv_transform.transform_point(sphere_body.position);

    let half_size = box_size * 0.5;

    // Find closest point on box to sphere center
    let closest_point = Vec3::new(
        local_sphere_center.x.clamp(-half_size.x, half_size.x),
        local_sphere_center.y.clamp(-half_size.y, half_size.y),
        local_sphere_center.z.clamp(-half_size.z, half_size.z),
    );

    let diff = local_sphere_center - closest_point;
    let distance_squared = diff.magnitude_squared();

    if distance_squared < radius * radius {
        let distance = distance_squared.sqrt();
        let normal = if distance > f64::EPSILON {
            box_transform.transform_vector(diff.normalized())
        } else {
            // Sphere center is inside box, find axis of minimum penetration
            let penetrations = [
                half_size.x - local_sphere_center.x.abs(),
                half_size.y - local_sphere_center.y.abs(),
                half_size.z - local_sphere_center.z.abs(),
            ];

            let min_axis = penetrations.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap().0;

            let mut normal_local = Vec3::zero();
            match min_axis {
                0 => normal_local.x = if local_sphere_center.x > 0.0 { 1.0 } else { -1.0 },
                1 => normal_local.y = if local_sphere_center.y > 0.0 { 1.0 } else { -1.0 },
                2 => normal_local.z = if local_sphere_center.z > 0.0 { 1.0 } else { -1.0 },
                _ => unreachable!(),
            }

            box_transform.transform_vector(normal_local)
        };

        let penetration = radius - distance;
        let world_closest_point = box_transform.transform_point(closest_point);

        Some(Constraint::Contact {
            body_a: ConstraintBody::RigidBody(0), // Would need proper indices
            body_b: ConstraintBody::RigidBody(1),
            contact_point: world_closest_point,
            contact_normal: normal,
            penetration_depth: penetration,
            friction: sphere_body.friction.max(box_body.friction),
            restitution: sphere_body.restitution.min(box_body.restitution),
            lambda_normal: 0.0,
            lambda_tangent: Vec3::zero(),
        })
    } else {
        None
    }
}

fn box_box_collision(body1: &RigidBody, body2: &RigidBody, size1: Vec3, size2: Vec3) -> Option<Constraint> {
    // Simplified SAT (Separating Axis Theorem) implementation
    // In a full implementation, would test all 15 potential separating axes

    let transform1 = body1.transform();
    let transform2 = body2.transform();

    // For simplicity, assume axis-aligned boxes
    let aabb1 = AABB::from_center_size(transform1.position, size1);
    let aabb2 = AABB::from_center_size(transform2.position, size2);

    if aabb1.intersects(aabb2) {
        // Find minimum penetration axis
        let overlap_x = (aabb1.max.x - aabb2.min.x).min(aabb2.max.x - aabb1.min.x);
        let overlap_y = (aabb1.max.y - aabb2.min.y).min(aabb2.max.y - aabb1.min.y);
        let overlap_z = (aabb1.max.z - aabb2.min.z).min(aabb2.max.z - aabb1.min.z);

        let min_overlap = overlap_x.min(overlap_y).min(overlap_z);
        let normal = if min_overlap == overlap_x {
            Vec3::new(if body2.position.x > body1.position.x { 1.0 } else { -1.0 }, 0.0, 0.0)
        } else if min_overlap == overlap_y {
            Vec3::new(0.0, if body2.position.y > body1.position.y { 1.0 } else { -1.0 }, 0.0)
        } else {
            Vec3::new(0.0, 0.0, if body2.position.z > body1.position.z { 1.0 } else { -1.0 })
        };

        let contact_point = (body1.position + body2.position) * 0.5;

        Some(Constraint::Contact {
            body_a: ConstraintBody::RigidBody(0),
            body_b: ConstraintBody::RigidBody(1),
            contact_point,
            contact_normal: normal,
            penetration_depth: min_overlap,
            friction: body1.friction.max(body2.friction),
            restitution: body1.restitution.min(body2.restitution),
            lambda_normal: 0.0,
            lambda_tangent: Vec3::zero(),
        })
    } else {
        None
    }
}

/// Collision between rigid body and particle (for soft body interaction)
pub fn collide_with_particle(body: &RigidBody, particle: &super::soft_body::Particle) -> Option<Constraint> {
    // Simplified collision with particle as sphere
    match &body.shape {
        Shape::Sphere { radius } => {
            let diff = particle.position - body.position;
            let distance_squared = diff.magnitude_squared();
            let radius_sum = radius + particle.radius;

            if distance_squared < radius_sum * radius_sum {
                let distance = distance_squared.sqrt();
                let normal = if distance > f64::EPSILON {
                    diff / distance
                } else {
                    Vec3::up()
                };

                let penetration = radius_sum - distance;

                Some(Constraint::Contact {
                    body_a: ConstraintBody::RigidBody(0),
                    body_b: ConstraintBody::SoftBodyParticle(0, 0), // Would need proper indices
                    contact_point: particle.position - normal * particle.radius,
                    contact_normal: normal,
                    penetration_depth: penetration,
                    friction: 0.7,
                    restitution: 0.6,
                    lambda_normal: 0.0,
                    lambda_tangent: Vec3::zero(),
                })
            } else {
                None
            }
        },
        _ => None, // Simplified for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_creation() {
        let shape = Shape::Sphere { radius: 1.0 };
        let body = RigidBody::new(shape, 1.0, Vec3::zero());

        assert_eq!(body.mass, 1.0);
        assert_eq!(body.inv_mass, 1.0);
        assert_eq!(body.position, Vec3::zero());
        assert!(!body.is_static);
    }

    #[test]
    fn test_static_body() {
        let shape = Shape::Box { size: Vec3::one() };
        let body = RigidBody::new(shape, 0.0, Vec3::zero());

        assert_eq!(body.mass, 0.0);
        assert_eq!(body.inv_mass, 0.0);
        assert!(body.is_static);
    }

    #[test]
    fn test_force_application() {
        let mut body = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::zero());
        body.apply_force(Vec3::new(10.0, 0.0, 0.0));

        assert_eq!(body.force, Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_impulse_application() {
        let mut body = RigidBody::new(Shape::Sphere { radius: 1.0 }, 2.0, Vec3::zero());
        body.apply_impulse(Vec3::new(4.0, 0.0, 0.0));

        assert_eq!(body.velocity, Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_sphere_collision() {
        let body1 = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::new(-0.5, 0.0, 0.0));
        let body2 = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::new(0.5, 0.0, 0.0));

        let contact = collide(&body1, &body2);
        assert!(contact.is_some());

        let contact = contact.unwrap();
        if let Constraint::Contact { penetration_depth, contact_normal, .. } = contact {
            assert!(penetration_depth > 0.0);
            assert!((contact_normal - Vec3::new(1.0, 0.0, 0.0)).magnitude() < f64::EPSILON);
        } else {
            panic!("Expected Contact constraint");
        }
    }

    #[test]
    fn test_no_collision_when_separated() {
        let body1 = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::new(-3.0, 0.0, 0.0));
        let body2 = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::new(3.0, 0.0, 0.0));

        let contact = collide(&body1, &body2);
        assert!(contact.is_none());
    }

    #[test]
    fn test_inertia_tensor_sphere() {
        let shape = Shape::Sphere { radius: 2.0 };
        let inertia = shape.inertia_tensor(10.0);
        let expected = 0.4 * 10.0 * 4.0; // 0.4 * mass * radius^2

        assert!((inertia.data[0][0] - expected).abs() < f64::EPSILON);
        assert!((inertia.data[1][1] - expected).abs() < f64::EPSILON);
        assert!((inertia.data[2][2] - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_velocity_at_point() {
        let mut body = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::zero());
        body.velocity = Vec3::new(1.0, 0.0, 0.0);
        body.angular_velocity = Vec3::new(0.0, 0.0, 1.0);

        let point = Vec3::new(0.0, 1.0, 0.0);
        let velocity = body.velocity_at_point(point);

        // Should be linear + angular velocity cross product
        assert_eq!(velocity, Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_integration() {
        let mut body = RigidBody::new(Shape::Sphere { radius: 1.0 }, 1.0, Vec3::zero());
        body.apply_force(Vec3::new(10.0, 0.0, 0.0));

        let dt = 1.0 / 60.0;
        let gravity = Vec3::new(0.0, -9.81, 0.0);

        body.integrate_forces(dt, gravity);

        assert!(body.velocity.x > 0.0);
        assert!(body.velocity.y < 0.0);
        assert!(body.position.x > 0.0);
    }
}
