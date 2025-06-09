// Fluid Simulation - Position-Based Fluids (PBF) and SPH Implementation
use super::math::*;
use super::spatial::*;
use std::collections::HashMap;

/// Fluid particle for SPH/PBF simulation
#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub position: Vec3,
    pub old_position: Vec3,
    pub velocity: Vec3,
    pub mass: f64,
    pub density: f64,
    pub pressure: f64,
    pub lambda: f64, // Lagrange multiplier for PBF
    pub delta_position: Vec3,
    pub radius: f64,
    pub id: usize,
}

impl FluidParticle {
    pub fn new(position: Vec3, mass: f64, radius: f64, id: usize) -> Self {
        Self {
            position,
            old_position: position,
            velocity: Vec3::zero(),
            mass,
            density: 0.0,
            pressure: 0.0,
            lambda: 0.0,
            delta_position: Vec3::zero(),
            radius,
            id,
        }
    }

    pub fn integrate(&mut self, dt: f64, gravity: Vec3) {
        // External forces
        let acceleration = gravity;

        // Predict position using symplectic Euler
        self.velocity = self.velocity + acceleration * dt;
        self.old_position = self.position;
        self.position = self.position + self.velocity * dt;
    }
}

/// SPH kernel functions for fluid simulation
pub struct SPHKernel;

impl SPHKernel {
    /// Poly6 kernel for density calculation
    pub fn poly6(r: f64, h: f64) -> f64 {
        if r >= h || r < 0.0 {
            return 0.0;
        }

        let h2 = h * h;
        let h9 = h2 * h2 * h2 * h2 * h;
        let r2 = r * r;
        let diff = h2 - r2;

        315.0 / (64.0 * std::f64::consts::PI * h9) * diff * diff * diff
    }

    /// Gradient of Poly6 kernel
    pub fn poly6_gradient(r_vec: Vec3, h: f64) -> Vec3 {
        let r = r_vec.magnitude();
        if r >= h || r <= 1e-6 {
            return Vec3::zero();
        }

        let h2 = h * h;
        let h9 = h2 * h2 * h2 * h2 * h;
        let r2 = r * r;
        let diff = h2 - r2;

        let coeff = -945.0 / (32.0 * std::f64::consts::PI * h9) * diff * diff;
        r_vec * coeff
    }

    /// Spiky kernel gradient for pressure forces
    pub fn spiky_gradient(r_vec: Vec3, h: f64) -> Vec3 {
        let r = r_vec.magnitude();
        if r >= h || r <= 1e-6 {
            return Vec3::zero();
        }

        let h6 = h * h * h * h * h * h;
        let diff = h - r;
        let coeff = -45.0 / (std::f64::consts::PI * h6) * diff * diff / r;

        r_vec * coeff
    }

    /// Viscosity laplacian kernel
    pub fn viscosity_laplacian(r: f64, h: f64) -> f64 {
        if r >= h || r < 0.0 {
            return 0.0;
        }

        let h6 = h * h * h * h * h * h;
        45.0 / (std::f64::consts::PI * h6) * (h - r)
    }

    /// Cohesion kernel for surface tension
    pub fn cohesion(r: f64, h: f64) -> f64 {
        if r >= h || r < 0.0 {
            return 0.0;
        }

        let h9 = h * h * h * h * h * h * h * h * h;
        let r_over_h = r / h;

        if r <= h / 2.0 {
            32.0 / (std::f64::consts::PI * h9)
                * (2.0 * (h - r) * (h - r) * (h - r) * r * r * r - h * h * h * h * h * h / 64.0)
        } else {
            32.0 / (std::f64::consts::PI * h9) * (h - r) * (h - r) * (h - r) * r * r * r
        }
    }
}

/// Fluid system using Position-Based Fluids (PBF) method
#[derive(Debug, Clone)]
pub struct FluidSystem {
    pub particles: Vec<FluidParticle>,
    pub rest_density: f64,
    pub smoothing_radius: f64,
    pub stiffness: f64,
    pub viscosity: f64,
    pub surface_tension: f64,
    pub damping: f64,
    pub epsilon: f64, // Relaxation parameter for PBF
    pub neighbor_search: SpatialHash,
    pub iterations: usize, // PBF constraint iterations
    pub bounds: AABB,
}

impl FluidSystem {
    pub fn new(rest_density: f64, smoothing_radius: f64, bounds: AABB) -> Self {
        Self {
            particles: Vec::new(),
            rest_density,
            smoothing_radius,
            stiffness: 1000.0,
            viscosity: 0.01,
            surface_tension: 0.1,
            damping: 0.99,
            epsilon: 100.0,
            neighbor_search: SpatialHash::new(smoothing_radius),
            iterations: 4,
            bounds,
        }
    }

    /// Add a particle to the fluid system
    pub fn add_particle(&mut self, position: Vec3, mass: f64) {
        let id = self.particles.len();
        self.particles.push(FluidParticle::new(
            position,
            mass,
            self.smoothing_radius * 0.1,
            id,
        ));
    }

    /// Create a fluid block
    pub fn create_fluid_block(&mut self, min: Vec3, max: Vec3, spacing: f64, mass: f64) {
        let x_count = ((max.x - min.x) / spacing) as usize + 1;
        let y_count = ((max.y - min.y) / spacing) as usize + 1;
        let z_count = ((max.z - min.z) / spacing) as usize + 1;

        for z in 0..z_count {
            for y in 0..y_count {
                for x in 0..x_count {
                    let position = Vec3::new(
                        min.x + x as f64 * spacing,
                        min.y + y as f64 * spacing,
                        min.z + z as f64 * spacing,
                    );
                    self.add_particle(position, mass);
                }
            }
        }
    }

    /// Update fluid simulation using Position-Based Fluids
    pub fn update(&mut self, dt: f64, gravity: Vec3) {
        // 1. Apply external forces and predict positions
        self.apply_forces_and_predict(dt, gravity);

        // 2. Update neighbor search
        self.update_neighbor_search();

        // 3. PBF constraint iterations
        for _ in 0..self.iterations {
            // Calculate densities and constraint function
            self.calculate_densities();

            // Calculate lambda (Lagrange multipliers)
            self.calculate_lambda();

            // Calculate position corrections
            self.calculate_position_corrections();

            // Apply position corrections
            self.apply_position_corrections();
        }

        // 4. Update velocities and apply additional forces
        self.update_velocities(dt);

        // 5. Apply viscosity, surface tension, and other forces
        self.apply_viscosity();
        self.apply_surface_tension();

        // 6. Handle boundary conditions
        self.handle_boundaries();

        // 7. Update positions
        self.finalize_positions(dt);
    }

    fn apply_forces_and_predict(&mut self, dt: f64, gravity: Vec3) {
        for particle in &mut self.particles {
            particle.integrate(dt, gravity);
        }
    }

    fn update_neighbor_search(&mut self) {
        self.neighbor_search.clear();

        for (i, particle) in self.particles.iter().enumerate() {
            let aabb = AABB::from_point(particle.position, self.smoothing_radius);
            self.neighbor_search
                .insert(SpatialObject::FluidParticle(i), aabb);
        }
    }

    fn calculate_densities(&mut self) {
        for i in 0..self.particles.len() {
            let particle_pos = self.particles[i].position;
            let query_aabb = AABB::from_point(particle_pos, self.smoothing_radius);
            let neighbors = self.neighbor_search.query(query_aabb);

            let mut density = 0.0;

            for neighbor in neighbors {
                if let SpatialObject::FluidParticle(j) = neighbor {
                    let neighbor_pos = self.particles[j].position;
                    let r = (particle_pos - neighbor_pos).magnitude();

                    if r < self.smoothing_radius {
                        density +=
                            self.particles[j].mass * SPHKernel::poly6(r, self.smoothing_radius);
                    }
                }
            }

            self.particles[i].density = density;
        }
    }

    fn calculate_lambda(&mut self) {
        for i in 0..self.particles.len() {
            let particle_pos = self.particles[i].position;
            let density_i = self.particles[i].density;
            let constraint = density_i / self.rest_density - 1.0;

            let query_aabb = AABB::from_point(particle_pos, self.smoothing_radius);
            let neighbors = self.neighbor_search.query(query_aabb);

            let mut gradient_sum = 0.0;
            let mut gradient_i = Vec3::zero();

            for neighbor in neighbors {
                if let SpatialObject::FluidParticle(j) = neighbor {
                    if i == j {
                        continue;
                    }

                    let neighbor_pos = self.particles[j].position;
                    let r_vec = particle_pos - neighbor_pos;
                    let r = r_vec.magnitude();

                    if r < self.smoothing_radius && r > 1e-6 {
                        let gradient_j = SPHKernel::poly6_gradient(r_vec, self.smoothing_radius)
                            * (self.particles[j].mass / self.rest_density);
                        gradient_i = gradient_i + gradient_j;
                        gradient_sum += gradient_j.magnitude_squared();
                    }
                }
            }

            gradient_sum += gradient_i.magnitude_squared();

            if gradient_sum > 1e-6 {
                self.particles[i].lambda = -constraint / (gradient_sum + self.epsilon);
            } else {
                self.particles[i].lambda = 0.0;
            }
        }
    }

    fn calculate_position_corrections(&mut self) {
        for i in 0..self.particles.len() {
            self.particles[i].delta_position = Vec3::zero();
        }

        for i in 0..self.particles.len() {
            let particle_pos = self.particles[i].position;
            let lambda_i = self.particles[i].lambda;

            let query_aabb = AABB::from_point(particle_pos, self.smoothing_radius);
            let neighbors = self.neighbor_search.query(query_aabb);

            for neighbor in neighbors {
                if let SpatialObject::FluidParticle(j) = neighbor {
                    if i == j {
                        continue;
                    }

                    let neighbor_pos = self.particles[j].position;
                    let lambda_j = self.particles[j].lambda;
                    let r_vec = particle_pos - neighbor_pos;
                    let r = r_vec.magnitude();

                    if r < self.smoothing_radius && r > 1e-6 {
                        let gradient = SPHKernel::poly6_gradient(r_vec, self.smoothing_radius)
                            * (self.particles[j].mass / self.rest_density);

                        let correction = gradient * (lambda_i + lambda_j);
                        self.particles[i].delta_position =
                            self.particles[i].delta_position + correction;
                    }
                }
            }
        }
    }

    fn apply_position_corrections(&mut self) {
        for particle in &mut self.particles {
            particle.position = particle.position + particle.delta_position;
        }
    }

    fn update_velocities(&mut self, dt: f64) {
        for particle in &mut self.particles {
            particle.velocity = (particle.position - particle.old_position) / dt;
        }
    }

    fn apply_viscosity(&mut self) {
        for i in 0..self.particles.len() {
            let particle_pos = self.particles[i].position;
            let particle_vel = self.particles[i].velocity;

            let query_aabb = AABB::from_point(particle_pos, self.smoothing_radius);
            let neighbors = self.neighbor_search.query(query_aabb);

            let mut viscosity_force = Vec3::zero();

            for neighbor in neighbors {
                if let SpatialObject::FluidParticle(j) = neighbor {
                    if i == j {
                        continue;
                    }

                    let neighbor_pos = self.particles[j].position;
                    let neighbor_vel = self.particles[j].velocity;
                    let r = (particle_pos - neighbor_pos).magnitude();

                    if r < self.smoothing_radius {
                        let vel_diff = neighbor_vel - particle_vel;
                        let laplacian = SPHKernel::viscosity_laplacian(r, self.smoothing_radius);
                        viscosity_force = viscosity_force
                            + vel_diff
                                * (self.particles[j].mass * laplacian / self.particles[j].density);
                    }
                }
            }

            self.particles[i].velocity = self.particles[i].velocity
                + viscosity_force * (self.viscosity / self.particles[i].mass);
        }
    }

    fn apply_surface_tension(&mut self) {
        for i in 0..self.particles.len() {
            let particle_pos = self.particles[i].position;

            let query_aabb = AABB::from_point(particle_pos, self.smoothing_radius);
            let neighbors = self.neighbor_search.query(query_aabb);

            let mut surface_force = Vec3::zero();

            for neighbor in neighbors {
                if let SpatialObject::FluidParticle(j) = neighbor {
                    if i == j {
                        continue;
                    }

                    let neighbor_pos = self.particles[j].position;
                    let r_vec = particle_pos - neighbor_pos;
                    let r = r_vec.magnitude();

                    if r < self.smoothing_radius && r > 1e-6 {
                        let cohesion = SPHKernel::cohesion(r, self.smoothing_radius);
                        surface_force = surface_force
                            - r_vec.normalized()
                                * (self.particles[j].mass * cohesion / self.particles[j].density);
                    }
                }
            }

            self.particles[i].velocity = self.particles[i].velocity
                + surface_force * (self.surface_tension / self.particles[i].mass);
        }
    }

    fn handle_boundaries(&mut self) {
        for particle in &mut self.particles {
            // X boundaries
            if particle.position.x < self.bounds.min.x {
                particle.position.x = self.bounds.min.x;
                particle.velocity.x = -particle.velocity.x * self.damping;
            } else if particle.position.x > self.bounds.max.x {
                particle.position.x = self.bounds.max.x;
                particle.velocity.x = -particle.velocity.x * self.damping;
            }

            // Y boundaries
            if particle.position.y < self.bounds.min.y {
                particle.position.y = self.bounds.min.y;
                particle.velocity.y = -particle.velocity.y * self.damping;
            } else if particle.position.y > self.bounds.max.y {
                particle.position.y = self.bounds.max.y;
                particle.velocity.y = -particle.velocity.y * self.damping;
            }

            // Z boundaries
            if particle.position.z < self.bounds.min.z {
                particle.position.z = self.bounds.min.z;
                particle.velocity.z = -particle.velocity.z * self.damping;
            } else if particle.position.z > self.bounds.max.z {
                particle.position.z = self.bounds.max.z;
                particle.velocity.z = -particle.velocity.z * self.damping;
            }
        }
    }

    fn finalize_positions(&mut self, dt: f64) {
        for particle in &mut self.particles {
            particle.position = particle.position + particle.velocity * dt;
        }
    }

    /// Get axis-aligned bounding box of all particles
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

    /// Convert fluid system to interpreter value
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
                particle_fields.insert("density".to_string(), Value::Float(p.density));
                particle_fields.insert("pressure".to_string(), Value::Float(p.pressure));
                particle_fields.insert("id".to_string(), Value::Int(p.id as i64));
                Value::Struct {
                    name: "FluidParticle".to_string(),
                    fields: particle_fields,
                }
            })
            .collect();

        fields.insert("particles".to_string(), Value::Array(particles));
        fields.insert("rest_density".to_string(), Value::Float(self.rest_density));
        fields.insert(
            "smoothing_radius".to_string(),
            Value::Float(self.smoothing_radius),
        );
        fields.insert("stiffness".to_string(), Value::Float(self.stiffness));
        fields.insert("viscosity".to_string(), Value::Float(self.viscosity));
        fields.insert(
            "surface_tension".to_string(),
            Value::Float(self.surface_tension),
        );
        fields.insert("damping".to_string(), Value::Float(self.damping));
        fields.insert("iterations".to_string(), Value::Int(self.iterations as i64));

        Ok(Value::Struct {
            name: "FluidSystem".to_string(),
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluid_particle_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let particle = FluidParticle::new(pos, 1.0, 0.1, 0);

        assert_eq!(particle.position, pos);
        assert_eq!(particle.old_position, pos);
        assert_eq!(particle.velocity, Vec3::zero());
        assert_eq!(particle.mass, 1.0);
        assert_eq!(particle.radius, 0.1);
        assert_eq!(particle.id, 0);
    }

    #[test]
    fn test_fluid_particle_integration() {
        let mut particle = FluidParticle::new(Vec3::new(0.0, 10.0, 0.0), 1.0, 0.1, 0);
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let dt = 1.0 / 60.0;

        let initial_y = particle.position.y;
        particle.integrate(dt, gravity);

        // Should move down due to gravity
        assert!(particle.position.y < initial_y);
    }

    #[test]
    fn test_sph_kernels() {
        let h = 1.0;

        // Test poly6 kernel
        assert!(SPHKernel::poly6(0.0, h) > 0.0);
        assert!(SPHKernel::poly6(0.5, h) > 0.0);
        assert_eq!(SPHKernel::poly6(h, h), 0.0);
        assert_eq!(SPHKernel::poly6(h + 0.1, h), 0.0);

        // Test gradient
        let r_vec = Vec3::new(0.5, 0.0, 0.0);
        let grad = SPHKernel::poly6_gradient(r_vec, h);
        assert!(grad.magnitude() > 0.0);

        // Test spiky gradient
        let spiky_grad = SPHKernel::spiky_gradient(r_vec, h);
        assert!(spiky_grad.magnitude() > 0.0);

        // Test viscosity laplacian
        assert!(SPHKernel::viscosity_laplacian(0.5, h) > 0.0);
        assert_eq!(SPHKernel::viscosity_laplacian(h + 0.1, h), 0.0);
    }

    #[test]
    fn test_fluid_system_creation() {
        let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let mut fluid = FluidSystem::new(1000.0, 1.0, bounds);

        assert_eq!(fluid.particles.len(), 0);
        assert_eq!(fluid.rest_density, 1000.0);
        assert_eq!(fluid.smoothing_radius, 1.0);

        fluid.add_particle(Vec3::zero(), 1.0);
        assert_eq!(fluid.particles.len(), 1);
    }

    #[test]
    fn test_fluid_block_creation() {
        let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let mut fluid = FluidSystem::new(1000.0, 1.0, bounds);

        let min = Vec3::new(0.0, 0.0, 0.0);
        let max = Vec3::new(2.0, 2.0, 2.0);
        fluid.create_fluid_block(min, max, 1.0, 1.0);

        // Should create a 3x3x3 grid of particles
        assert_eq!(fluid.particles.len(), 27);
    }

    #[test]
    fn test_fluid_simulation_step() {
        let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let mut fluid = FluidSystem::new(1000.0, 0.5, bounds);

        // Create a small fluid block
        let min = Vec3::new(-1.0, 5.0, -1.0);
        let max = Vec3::new(1.0, 7.0, 1.0);
        fluid.create_fluid_block(min, max, 0.5, 1.0);

        let initial_particles = fluid.particles.len();
        let initial_position = fluid.particles[0].position;

        // Run simulation step
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        fluid.update(1.0 / 60.0, gravity);

        // Particles should still exist
        assert_eq!(fluid.particles.len(), initial_particles);

        // Particles should move due to gravity
        assert!(fluid.particles[0].position.y < initial_position.y);
    }

    #[test]
    fn test_boundary_conditions() {
        let bounds = AABB::new(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let mut fluid = FluidSystem::new(1000.0, 1.0, bounds);

        // Add particle outside bounds
        fluid.add_particle(Vec3::new(10.0, 10.0, 10.0), 1.0);
        fluid.particles[0].velocity = Vec3::new(1.0, 1.0, 1.0);

        fluid.handle_boundaries();

        // Particle should be inside bounds
        assert!(fluid.particles[0].position.x <= bounds.max.x);
        assert!(fluid.particles[0].position.y <= bounds.max.y);
        assert!(fluid.particles[0].position.z <= bounds.max.z);
    }

    #[test]
    fn test_fluid_aabb() {
        let bounds = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let mut fluid = FluidSystem::new(1000.0, 1.0, bounds);

        fluid.add_particle(Vec3::new(-2.0, -2.0, -2.0), 1.0);
        fluid.add_particle(Vec3::new(3.0, 3.0, 3.0), 1.0);

        let aabb = fluid.aabb();
        assert_eq!(aabb.min, Vec3::new(-2.0, -2.0, -2.0));
        assert_eq!(aabb.max, Vec3::new(3.0, 3.0, 3.0));
    }
}
