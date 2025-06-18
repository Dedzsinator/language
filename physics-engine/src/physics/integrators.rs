// High-performance numerical integrators for physics simulation

use super::math::Vec3;

/// Different integration schemes for physics simulation
#[derive(Debug, Clone, Copy)]
pub enum IntegratorType {
    ExplicitEuler,
    SemiImplicitEuler,
    Verlet,
    LeapFrog,
    RungeKutta4,
    RungeKuttaFehlberg45,
}

/// State for a particle/body in simulation
#[derive(Debug, Clone)]
pub struct ParticleState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub mass: f64,
    pub inv_mass: f64,
}

impl ParticleState {
    pub fn new(position: Vec3, velocity: Vec3, mass: f64) -> Self {
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        Self {
            position,
            velocity,
            acceleration: Vec3::zero(),
            mass,
            inv_mass,
        }
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.acceleration = self.acceleration + force * self.inv_mass;
    }

    pub fn clear_forces(&mut self) {
        self.acceleration = Vec3::zero();
    }
}

/// High-performance integrator for physics simulation
#[derive(Debug)]
pub struct Integrator {
    integrator_type: IntegratorType,
    damping: f64,
}

impl Integrator {
    pub fn new(integrator_type: IntegratorType, damping: f64) -> Self {
        Self {
            integrator_type,
            damping,
        }
    }

    /// Integrate a single particle state forward by dt
    pub fn integrate(&self, state: &mut ParticleState, dt: f64) {
        match self.integrator_type {
            IntegratorType::ExplicitEuler => self.explicit_euler(state, dt),
            IntegratorType::SemiImplicitEuler => self.semi_implicit_euler(state, dt),
            IntegratorType::Verlet => self.verlet(state, dt),
            IntegratorType::LeapFrog => self.leapfrog(state, dt),
            IntegratorType::RungeKutta4 => self.runge_kutta4(state, dt),
            IntegratorType::RungeKuttaFehlberg45 => self.rkf45(state, dt),
        }
    }

    /// Explicit Euler method - simple but less stable
    fn explicit_euler(&self, state: &mut ParticleState, dt: f64) {
        // x(t+dt) = x(t) + v(t) * dt
        // v(t+dt) = v(t) + a(t) * dt
        state.position = state.position + state.velocity * dt;
        state.velocity = state.velocity * self.damping + state.acceleration * dt;
    }

    /// Semi-implicit Euler (Symplectic Euler) - better energy conservation
    fn semi_implicit_euler(&self, state: &mut ParticleState, dt: f64) {
        // v(t+dt) = v(t) + a(t) * dt
        // x(t+dt) = x(t) + v(t+dt) * dt
        state.velocity = state.velocity * self.damping + state.acceleration * dt;
        state.position = state.position + state.velocity * dt;
    }

    /// Velocity Verlet - excellent for conservative systems
    fn verlet(&self, state: &mut ParticleState, dt: f64) {
        // This is a simplified Verlet - in practice you'd store previous acceleration
        // x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt^2
        // v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
        
        let old_acceleration = state.acceleration;
        state.position = state.position + state.velocity * dt + old_acceleration * (0.5 * dt * dt);
        
        // For now, assume acceleration doesn't change significantly
        // In full implementation, you'd recalculate forces here
        state.velocity = state.velocity * self.damping + old_acceleration * dt;
    }

    /// Leapfrog integration - good for orbital mechanics
    fn leapfrog(&self, state: &mut ParticleState, dt: f64) {
        // This is velocity Verlet in leapfrog form
        // Similar to Verlet but with half-step velocity updates
        let half_dt = dt * 0.5;
        
        // Update velocity by half step
        state.velocity = state.velocity + state.acceleration * half_dt;
        
        // Update position by full step
        state.position = state.position + state.velocity * dt;
        
        // Update velocity by another half step (would need new acceleration)
        state.velocity = (state.velocity + state.acceleration * half_dt) * self.damping;
    }

    /// Fourth-order Runge-Kutta - high accuracy
    fn runge_kutta4(&self, state: &mut ParticleState, dt: f64) {
        let initial_pos = state.position;
        let initial_vel = state.velocity;
        let acceleration = state.acceleration;
        
        // k1 = f(t, y)
        let k1_vel = acceleration;
        let k1_pos = initial_vel;
        
        // k2 = f(t + dt/2, y + k1*dt/2)
        let k2_vel = acceleration; // Assuming constant acceleration for simplicity
        let k2_pos = initial_vel + k1_vel * (dt * 0.5);
        
        // k3 = f(t + dt/2, y + k2*dt/2)
        let k3_vel = acceleration;
        let k3_pos = initial_vel + k2_vel * (dt * 0.5);
        
        // k4 = f(t + dt, y + k3*dt)
        let k4_vel = acceleration;
        let k4_pos = initial_vel + k3_vel * dt;
        
        // Combine using RK4 formula
        state.velocity = initial_vel + (k1_vel + k2_vel * 2.0 + k3_vel * 2.0 + k4_vel) * (dt / 6.0);
        state.position = initial_pos + (k1_pos + k2_pos * 2.0 + k3_pos * 2.0 + k4_pos) * (dt / 6.0);
        
        // Apply damping
        state.velocity = state.velocity * self.damping;
    }

    /// Runge-Kutta-Fehlberg with adaptive stepping
    fn rkf45(&self, state: &mut ParticleState, dt: f64) {
        // Simplified RKF45 - in practice would include error estimation and adaptive stepping
        // For now, just use RK4 with damping
        self.runge_kutta4(state, dt);
    }

    /// Integrate multiple particles efficiently
    pub fn integrate_system(&self, states: &mut [ParticleState], dt: f64) {
        for state in states.iter_mut() {
            self.integrate(state, dt);
        }
    }

    /// Integrate with constraint projection (for constrained systems)
    pub fn integrate_with_constraints<F>(&self, state: &mut ParticleState, dt: f64, constraint_fn: F)
    where
        F: Fn(&mut ParticleState),
    {
        // Store original state
        let original_position = state.position;
        
        // Integrate normally
        self.integrate(state, dt);
        
        // Apply constraints
        constraint_fn(state);
        
        // Update velocity based on constrained position change
        if dt > 0.0 {
            let corrected_velocity = (state.position - original_position) / dt;
            state.velocity = corrected_velocity * self.damping;
        }
    }
}

/// Adaptive timestep controller for variable timestep integration
#[derive(Debug)]
pub struct AdaptiveTimestepper {
    min_dt: f64,
    max_dt: f64,
    tolerance: f64,
    safety_factor: f64,
}

impl AdaptiveTimestepper {
    pub fn new(min_dt: f64, max_dt: f64, tolerance: f64) -> Self {
        Self {
            min_dt,
            max_dt,
            tolerance,
            safety_factor: 0.9,
        }
    }

    /// Calculate adaptive timestep based on error estimation
    pub fn calculate_timestep(&self, current_dt: f64, error: f64) -> f64 {
        if error <= 0.0 {
            return self.max_dt;
        }

        // Calculate new timestep based on error
        let factor = self.safety_factor * (self.tolerance / error).powf(0.2);
        let new_dt = current_dt * factor.clamp(0.1, 5.0);
        
        new_dt.clamp(self.min_dt, self.max_dt)
    }

    /// Estimate integration error (simplified version)
    pub fn estimate_error(&self, state1: &ParticleState, state2: &ParticleState) -> f64 {
        let pos_error = (state1.position - state2.position).magnitude();
        let vel_error = (state1.velocity - state2.velocity).magnitude();
        (pos_error + vel_error).max(1e-10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::math::Vec3;

    #[test]
    fn test_particle_state_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let vel = Vec3::new(0.1, 0.2, 0.3);
        let mass = 2.0;
        
        let state = ParticleState::new(pos, vel, mass);
        
        assert_eq!(state.position, pos);
        assert_eq!(state.velocity, vel);
        assert_eq!(state.mass, mass);
        assert_eq!(state.inv_mass, 0.5);
        assert_eq!(state.acceleration, Vec3::zero());
    }

    #[test]
    fn test_zero_mass_particle() {
        let state = ParticleState::new(Vec3::zero(), Vec3::zero(), 0.0);
        assert_eq!(state.inv_mass, 0.0);
    }

    #[test]
    fn test_apply_force() {
        let mut state = ParticleState::new(Vec3::zero(), Vec3::zero(), 2.0);
        let force = Vec3::new(4.0, 0.0, 0.0);
        
        state.apply_force(force);
        
        // a = F/m = 4.0/2.0 = 2.0
        assert_eq!(state.acceleration.x, 2.0);
    }

    #[test]
    fn test_explicit_euler_integration() {
        let integrator = Integrator::new(IntegratorType::ExplicitEuler, 1.0);
        let mut state = ParticleState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            1.0
        );
        state.acceleration = Vec3::new(2.0, 0.0, 0.0);
        
        integrator.integrate(&mut state, 0.1);
        
        // x = x0 + v*dt = 0 + 1*0.1 = 0.1
        // v = v0 + a*dt = 1 + 2*0.1 = 1.2
        assert!((state.position.x - 0.1).abs() < 1e-10);
        assert!((state.velocity.x - 1.2).abs() < 1e-10);
    }

    #[test]
    fn test_semi_implicit_euler() {
        let integrator = Integrator::new(IntegratorType::SemiImplicitEuler, 1.0);
        let mut state = ParticleState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            1.0
        );
        state.acceleration = Vec3::new(2.0, 0.0, 0.0);
        
        integrator.integrate(&mut state, 0.1);
        
        // v = v0 + a*dt = 1 + 2*0.1 = 1.2
        // x = x0 + v_new*dt = 0 + 1.2*0.1 = 0.12
        assert!((state.velocity.x - 1.2).abs() < 1e-10);
        assert!((state.position.x - 0.12).abs() < 1e-10);
    }

    #[test]
    fn test_verlet_integration() {
        let integrator = Integrator::new(IntegratorType::Verlet, 1.0);
        let mut state = ParticleState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            1.0
        );
        state.acceleration = Vec3::new(2.0, 0.0, 0.0);
        
        integrator.integrate(&mut state, 0.1);
        
        // Verlet: x = x0 + v*dt + 0.5*a*dt^2
        // x = 0 + 1*0.1 + 0.5*2*0.01 = 0.1 + 0.01 = 0.11
        assert!((state.position.x - 0.11).abs() < 1e-10);
    }

    #[test]
    fn test_runge_kutta4() {
        let integrator = Integrator::new(IntegratorType::RungeKutta4, 1.0);
        let mut state = ParticleState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            1.0
        );
        state.acceleration = Vec3::new(2.0, 0.0, 0.0);
        
        integrator.integrate(&mut state, 0.1);
        
        // RK4 should give high accuracy
        assert!(state.position.x > 0.0);
        assert!(state.velocity.x > 1.0);
    }

    #[test]
    fn test_damping() {
        let integrator = Integrator::new(IntegratorType::ExplicitEuler, 0.9);
        let mut state = ParticleState::new(
            Vec3::zero(),
            Vec3::new(1.0, 0.0, 0.0),
            1.0
        );
        
        integrator.integrate(&mut state, 0.1);
        
        // Velocity should be damped
        assert!(state.velocity.x < 1.0);
        assert!(state.velocity.x > 0.8); // Should be around 0.9
    }

    #[test]
    fn test_integrate_system() {
        let integrator = Integrator::new(IntegratorType::ExplicitEuler, 1.0);
        let mut states = vec![
            ParticleState::new(Vec3::zero(), Vec3::new(1.0, 0.0, 0.0), 1.0),
            ParticleState::new(Vec3::zero(), Vec3::new(0.0, 1.0, 0.0), 1.0),
        ];
        
        integrator.integrate_system(&mut states, 0.1);
        
        assert!((states[0].position.x - 0.1).abs() < 1e-10);
        assert!((states[1].position.y - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_adaptive_timestepper() {
        let stepper = AdaptiveTimestepper::new(0.001, 0.1, 1e-6);
        
        // Large error should decrease timestep
        let dt1 = stepper.calculate_timestep(0.01, 1e-3);
        assert!(dt1 < 0.01);
        
        // Small error should increase timestep
        let dt2 = stepper.calculate_timestep(0.01, 1e-9);
        assert!(dt2 > 0.01);
        
        // Should respect bounds
        let dt3 = stepper.calculate_timestep(0.01, 1e-12);
        assert!(dt3 <= 0.1);
    }

    #[test]
    fn test_error_estimation() {
        let stepper = AdaptiveTimestepper::new(0.001, 0.1, 1e-6);
        
        let state1 = ParticleState::new(Vec3::new(1.0, 0.0, 0.0), Vec3::zero(), 1.0);
        let state2 = ParticleState::new(Vec3::new(1.1, 0.0, 0.0), Vec3::zero(), 1.0);
        
        let error = stepper.estimate_error(&state1, &state2);
        assert!(error > 0.0);
        assert!((error - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_constraint_integration() {
        let integrator = Integrator::new(IntegratorType::ExplicitEuler, 1.0);
        let mut state = ParticleState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            1.0
        );
        
        // Constraint: keep on x-axis (y = 0)
        let constraint = |s: &mut ParticleState| {
            s.position.y = 0.0;
        };
        
        integrator.integrate_with_constraints(&mut state, 0.1, constraint);
        
        assert!((state.position.y).abs() < 1e-10);
        assert!(state.position.x > 0.0);
    }
}
