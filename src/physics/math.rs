// High-performance 3D math for physics simulation
use crate::eval::interpreter::{RuntimeResult, Value};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// 3D Vector with SIMD-optimized operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub const fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn magnitude_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn magnitude(self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag > f64::EPSILON {
            self / mag
        } else {
            Self::zero()
        }
    }

    pub fn distance_to(self, other: Self) -> f64 {
        (self - other).magnitude()
    }

    pub fn distance_squared_to(self, other: Self) -> f64 {
        (self - other).magnitude_squared()
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn to_value(self) -> RuntimeResult<Value> {
        Ok(Value::Array(vec![
            Value::Float(self.x),
            Value::Float(self.y),
            Value::Float(self.z),
        ]))
    }

    pub fn min_component_wise(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    pub fn max_component_wise(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    /// Essential math functions for physics calculations
    pub fn pow(self, exponent: f64) -> Self {
        Self::new(
            self.x.powf(exponent),
            self.y.powf(exponent),
            self.z.powf(exponent),
        )
    }

    pub fn exp(self) -> Self {
        Self::new(self.x.exp(), self.y.exp(), self.z.exp())
    }

    pub fn ln(self) -> Self {
        Self::new(self.x.ln(), self.y.ln(), self.z.ln())
    }

    pub fn log10(self) -> Self {
        Self::new(self.x.log10(), self.y.log10(), self.z.log10())
    }

    pub fn sqrt(self) -> Self {
        Self::new(self.x.sqrt(), self.y.sqrt(), self.z.sqrt())
    }

    pub fn cbrt(self) -> Self {
        Self::new(self.x.cbrt(), self.y.cbrt(), self.z.cbrt())
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round(), self.z.round())
    }

    pub fn sin(self) -> Self {
        Self::new(self.x.sin(), self.y.sin(), self.z.sin())
    }

    pub fn cos(self) -> Self {
        Self::new(self.x.cos(), self.y.cos(), self.z.cos())
    }

    pub fn tan(self) -> Self {
        Self::new(self.x.tan(), self.y.tan(), self.z.tan())
    }

    pub fn asin(self) -> Self {
        Self::new(self.x.asin(), self.y.asin(), self.z.asin())
    }

    pub fn acos(self) -> Self {
        Self::new(self.x.acos(), self.y.acos(), self.z.acos())
    }

    pub fn atan(self) -> Self {
        Self::new(self.x.atan(), self.y.atan(), self.z.atan())
    }

    pub fn atan2(self, other: Self) -> Self {
        Self::new(
            self.x.atan2(other.x),
            self.y.atan2(other.y),
            self.z.atan2(other.z),
        )
    }

    pub fn sinh(self) -> Self {
        Self::new(self.x.sinh(), self.y.sinh(), self.z.sinh())
    }

    pub fn cosh(self) -> Self {
        Self::new(self.x.cosh(), self.y.cosh(), self.z.cosh())
    }

    pub fn tanh(self) -> Self {
        Self::new(self.x.tanh(), self.y.tanh(), self.z.tanh())
    }

    // Physics-specific functions
    pub fn force_from_acceleration(self, mass: f64) -> Self {
        self * mass
    }

    pub fn velocity_from_force(self, mass: f64, dt: f64) -> Self {
        self * (dt / mass)
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
            self.z.clamp(min.z, max.z),
        )
    }

    pub fn clamp_magnitude(self, max_magnitude: f64) -> Self {
        let mag = self.magnitude();
        if mag > max_magnitude {
            self.normalized() * max_magnitude
        } else {
            self
        }
    }

    /// Essential physics utility functions
    pub fn gravitational_force(
        pos1: Self,
        mass1: f64,
        pos2: Self,
        mass2: f64,
        g_constant: f64,
    ) -> Self {
        let direction = pos2 - pos1;
        let distance_sq = direction.magnitude_squared().max(0.01); // Prevent division by zero
        let force_magnitude = g_constant * mass1 * mass2 / distance_sq;
        direction.normalized() * force_magnitude
    }

    /// Calculate spring force (Hooke's law)
    pub fn spring_force(pos1: Self, pos2: Self, rest_length: f64, spring_constant: f64) -> Self {
        let direction = pos2 - pos1;
        let distance = direction.magnitude();
        let displacement = distance - rest_length;
        direction.normalized() * (spring_constant * displacement)
    }

    /// Calculate damping force
    pub fn damping_force(velocity: Self, damping_coefficient: f64) -> Self {
        velocity * (-damping_coefficient)
    }

    /// Calculate drag force
    pub fn drag_force(
        velocity: Self,
        drag_coefficient: f64,
        fluid_density: f64,
        cross_sectional_area: f64,
    ) -> Self {
        let speed = velocity.magnitude();
        if speed > 0.0 {
            let drag_magnitude =
                0.5 * drag_coefficient * fluid_density * cross_sectional_area * speed * speed;
            velocity.normalized() * (-drag_magnitude)
        } else {
            Self::zero()
        }
    }

    /// Calculate buoyancy force
    pub fn buoyancy_force(fluid_density: f64, volume: f64, gravity: Self) -> Self {
        gravity * (-fluid_density * volume)
    }

    /// Calculate centripetal acceleration
    pub fn centripetal_acceleration(velocity: Self, radius: f64) -> Self {
        if radius > 0.0 {
            let speed = velocity.magnitude();
            let centripetal_magnitude = speed * speed / radius;
            // Direction is toward center (perpendicular to velocity)
            velocity.normalized() * centripetal_magnitude
        } else {
            Self::zero()
        }
    }

    /// Calculate orbital velocity for circular orbit
    pub fn orbital_velocity(central_mass: f64, orbital_radius: f64, g_constant: f64) -> f64 {
        if orbital_radius > 0.0 {
            (g_constant * central_mass / orbital_radius).sqrt()
        } else {
            0.0
        }
    }

    /// Calculate escape velocity
    pub fn escape_velocity(mass: f64, radius: f64, g_constant: f64) -> f64 {
        if radius > 0.0 {
            (2.0 * g_constant * mass / radius).sqrt()
        } else {
            0.0
        }
    }

    /// Apply impulse to change velocity instantly
    pub fn apply_impulse(velocity: Self, impulse: Self, mass: f64) -> Self {
        if mass > 0.0 {
            velocity + (impulse / mass)
        } else {
            velocity
        }
    }

    /// Calculate kinetic energy
    pub fn kinetic_energy(velocity: Self, mass: f64) -> f64 {
        0.5 * mass * velocity.magnitude_squared()
    }

    /// Calculate potential energy in gravity field
    pub fn gravitational_potential_energy(position: Self, mass: f64, gravity: Self) -> f64 {
        mass * gravity.magnitude() * position.y
    }

    /// Calculate elastic potential energy
    pub fn elastic_potential_energy(displacement: f64, spring_constant: f64) -> f64 {
        0.5 * spring_constant * displacement * displacement
    }

    /// Project vector onto another vector
    pub fn project_onto(self, other: Self) -> Self {
        let other_normalized = other.normalized();
        other_normalized * self.dot(other_normalized)
    }

    /// Reject vector from another vector (component perpendicular to other)
    pub fn reject_from(self, other: Self) -> Self {
        self - self.project_onto(other)
    }

    /// Calculate angle between two vectors in radians
    pub fn angle_between(self, other: Self) -> f64 {
        let dot_product = self.dot(other);
        let magnitudes = self.magnitude() * other.magnitude();
        if magnitudes > 0.0 {
            (dot_product / magnitudes).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        }
    }

    /// Rotate vector around axis by angle in radians
    pub fn rotate_around_axis(self, axis: Self, angle: f64) -> Self {
        let axis_normalized = axis.normalized();
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Rodrigues' rotation formula
        self * cos_angle
            + axis_normalized.cross(self) * sin_angle
            + axis_normalized * axis_normalized.dot(self) * (1.0 - cos_angle)
    }

    /// Calculate momentum
    pub fn momentum(velocity: Self, mass: f64) -> Self {
        velocity * mass
    }

    /// Calculate angular momentum for point mass
    pub fn angular_momentum(position: Self, velocity: Self, mass: f64) -> Self {
        position.cross(velocity * mass)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar;
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// Quaternion for efficient 3D rotations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quat {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }

    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        let half_angle = angle * 0.5;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();
        let normalized_axis = axis.normalized();

        Self::new(
            cos_half,
            normalized_axis.x * sin_half,
            normalized_axis.y * sin_half,
            normalized_axis.z * sin_half,
        )
    }

    pub fn from_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Self::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )
    }

    pub fn magnitude_squared(self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn magnitude(self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let mag = self.magnitude();
        if mag > f64::EPSILON {
            Self::new(self.w / mag, self.x / mag, self.y / mag, self.z / mag)
        } else {
            Self::identity()
        }
    }

    pub fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    pub fn inverse(self) -> Self {
        let mag_sq = self.magnitude_squared();
        if mag_sq > f64::EPSILON {
            let conj = self.conjugate();
            Self::new(
                conj.w / mag_sq,
                conj.x / mag_sq,
                conj.y / mag_sq,
                conj.z / mag_sq,
            )
        } else {
            Self::identity()
        }
    }

    pub fn rotate_vector(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v + (uv * self.w + uuv) * 2.0
    }

    pub fn to_rotation_matrix(self) -> Mat3 {
        let q = self.normalized();
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        Mat3::new([
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
            [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
            [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
        ])
    }

    pub fn slerp(self, other: Self, t: f64) -> Self {
        let dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;

        let (other, dot) = if dot < 0.0 {
            (Self::new(-other.w, -other.x, -other.y, -other.z), -dot)
        } else {
            (other, dot)
        };

        if dot > 0.9995 {
            // Linear interpolation for very close quaternions
            let result = Self::new(
                self.w + t * (other.w - self.w),
                self.x + t * (other.x - self.x),
                self.y + t * (other.y - self.y),
                self.z + t * (other.z - self.z),
            );
            result.normalized()
        } else {
            let theta_0 = dot.acos();
            let theta = theta_0 * t;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
            let s1 = sin_theta / sin_theta_0;

            Self::new(
                s0 * self.w + s1 * other.w,
                s0 * self.x + s1 * other.x,
                s0 * self.y + s1 * other.y,
                s0 * self.z + s1 * other.z,
            )
        }
    }

    pub fn to_axis_angle(self) -> (Vec3, f64) {
        let normalized = self.normalized();

        // Handle the case where w = 1 (no rotation)
        if normalized.w >= 1.0 {
            return (Vec3::up(), 0.0);
        }

        let angle = 2.0 * normalized.w.clamp(-1.0, 1.0).acos();
        let sin_half_angle = (1.0 - normalized.w * normalized.w).sqrt();

        if sin_half_angle < 1e-6 {
            // Near zero rotation
            return (Vec3::up(), 0.0);
        }

        let axis = Vec3::new(
            normalized.x / sin_half_angle,
            normalized.y / sin_half_angle,
            normalized.z / sin_half_angle,
        );

        (axis, angle)
    }
}

impl Mul for Quat {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        )
    }
}

/// 3x3 Matrix for rotations and inertia tensors
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    pub data: [[f64; 3]; 3],
}

impl Mat3 {
    pub fn new(data: [[f64; 3]; 3]) -> Self {
        Self { data }
    }

    pub fn identity() -> Self {
        Self::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn zero() -> Self {
        Self::new([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])
    }

    pub fn from_diagonal(x: f64, y: f64, z: f64) -> Self {
        Self::new([[x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, z]])
    }

    pub fn transpose(self) -> Self {
        Self::new([
            [self.data[0][0], self.data[1][0], self.data[2][0]],
            [self.data[0][1], self.data[1][1], self.data[2][1]],
            [self.data[0][2], self.data[1][2], self.data[2][2]],
        ])
    }

    pub fn determinant(self) -> f64 {
        let m = &self.data;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    pub fn inverse(self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f64::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let m = &self.data;

        Some(Self::new([
            [
                (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
                (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
                (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
            ],
            [
                (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
                (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
                (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
            ],
            [
                (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
                (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
                (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
            ],
        ]))
    }

    pub fn transform_vector(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.data[0][0] * v.x + self.data[0][1] * v.y + self.data[0][2] * v.z,
            self.data[1][0] * v.x + self.data[1][1] * v.y + self.data[1][2] * v.z,
            self.data[2][0] * v.x + self.data[2][1] * v.y + self.data[2][2] * v.z,
        )
    }
}

impl Mul for Mat3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut result = Self::zero();
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        result
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.transform_vector(v)
    }
}

impl Mul<f64> for Mat3 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        let mut result = self;
        for i in 0..3 {
            for j in 0..3 {
                result.data[i][j] *= scalar;
            }
        }
        result
    }
}

/// Transform combining translation, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self::new(Vec3::zero(), Quat::identity(), Vec3::one())
    }

    pub fn from_position(position: Vec3) -> Self {
        Self::new(position, Quat::identity(), Vec3::one())
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self::new(Vec3::zero(), rotation, Vec3::one())
    }

    pub fn transform_point(self, point: Vec3) -> Vec3 {
        let scaled = Vec3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );
        let rotated = self.rotation.rotate_vector(scaled);
        rotated + self.position
    }

    pub fn transform_vector(self, vector: Vec3) -> Vec3 {
        let scaled = Vec3::new(
            vector.x * self.scale.x,
            vector.y * self.scale.y,
            vector.z * self.scale.z,
        );
        self.rotation.rotate_vector(scaled)
    }

    pub fn inverse(self) -> Self {
        let inv_rotation = self.rotation.conjugate();
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        let inv_position = inv_rotation.rotate_vector(-self.position);

        Self::new(
            Vec3::new(
                inv_position.x * inv_scale.x,
                inv_position.y * inv_scale.y,
                inv_position.z * inv_scale.z,
            ),
            inv_rotation,
            inv_scale,
        )
    }
}

/// Axis-Aligned Bounding Box for collision detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self::new(center - half_size, center + half_size)
    }

    pub fn from_point(point: Vec3, radius: f64) -> Self {
        let offset = Vec3::new(radius, radius, radius);
        Self::new(point - offset, point + offset)
    }

    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn expand(self, point: Vec3) -> Self {
        Self::new(
            Vec3::new(
                self.min.x.min(point.x),
                self.min.y.min(point.y),
                self.min.z.min(point.z),
            ),
            Vec3::new(
                self.max.x.max(point.x),
                self.max.y.max(point.y),
                self.max.z.max(point.z),
            ),
        )
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(
            Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));

        assert_eq!(a.dot(b), 32.0);
        assert_eq!(a.cross(b), Vec3::new(-3.0, 6.0, -3.0));

        let normalized = Vec3::new(1.0, 0.0, 0.0).normalized();
        assert!((normalized.magnitude() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quaternion_operations() {
        let q = Quat::from_axis_angle(Vec3::up(), std::f64::consts::PI / 2.0);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q.rotate_vector(v);

        // Should rotate 90 degrees around Y axis
        assert!((rotated.x - 0.0).abs() < 1e-10);
        assert!((rotated.z - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_operations() {
        let m = Mat3::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(m * v, v);

        let det = Mat3::new([[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [5.0, 6.0, 0.0]]).determinant();
        assert_eq!(det, 1.0);
    }

    #[test]
    fn test_aabb_operations() {
        let aabb1 = AABB::new(Vec3::zero(), Vec3::one());
        let aabb2 = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));

        assert!(aabb1.intersects(aabb2));
        assert!(aabb1.contains_point(Vec3::new(0.5, 0.5, 0.5)));

        let center = aabb1.center();
        assert_eq!(center, Vec3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_transform_operations() {
        let transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::zero();
        let transformed = transform.transform_point(point);
        assert_eq!(transformed, Vec3::new(1.0, 2.0, 3.0));

        let inverse = transform.inverse();
        let back = inverse.transform_point(transformed);
        assert!((back - point).magnitude() < f64::EPSILON);
    }
}
