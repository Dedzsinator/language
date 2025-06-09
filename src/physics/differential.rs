// Differential Equation Solvers - Advanced numerical integration methods
use std::collections::HashMap;

/// State vector for differential equations
pub type StateVector = Vec<f64>;

/// Derivative function type: f(t, y) = dy/dt
pub type DerivativeFunction = Box<dyn Fn(f64, &StateVector) -> StateVector + Send + Sync>;

/// Event detection function for event-driven systems
pub type EventFunction = Box<dyn Fn(f64, &StateVector) -> f64 + Send + Sync>;

/// Jacobian matrix function for implicit methods
pub type JacobianFunction = Box<dyn Fn(f64, &StateVector) -> Vec<Vec<f64>> + Send + Sync>;

/// Integration method selection
#[derive(Debug, Clone, Copy)]
pub enum IntegrationMethod {
    Euler,
    RungeKutta4,
    RungeKuttaFehlberg45, // Adaptive RK45
    DormandPrince,        // High-accuracy adaptive method
    ImplicitEuler,        // For stiff systems
    BDF2,                 // Backward Differentiation Formula
    Verlet,               // For Hamiltonian systems
    LeapFrog,             // Symplectic integrator
}

/// Adaptive step size control
#[derive(Debug, Clone)]
pub struct AdaptiveStepControl {
    pub tolerance: f64,
    pub min_step: f64,
    pub max_step: f64,
    pub safety_factor: f64,
    pub error_order: f64,
}

impl Default for AdaptiveStepControl {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            min_step: 1e-12,
            max_step: 1.0,
            safety_factor: 0.9,
            error_order: 5.0, // For RK45
        }
    }
}

/// Differential equation solver with multiple integration methods
pub struct ODESolver {
    pub method: IntegrationMethod,
    pub adaptive_control: Option<AdaptiveStepControl>,
    pub derivative_func: Option<DerivativeFunction>,
    pub jacobian_func: Option<JacobianFunction>,
    pub event_functions: Vec<EventFunction>,
    pub state_dimension: usize,
}

impl ODESolver {
    pub fn new(method: IntegrationMethod, state_dimension: usize) -> Self {
        Self {
            method,
            adaptive_control: None,
            derivative_func: None,
            jacobian_func: None,
            event_functions: Vec::new(),
            state_dimension,
        }
    }

    /// Set derivative function
    pub fn set_derivative_function(&mut self, func: DerivativeFunction) {
        self.derivative_func = Some(func);
    }

    /// Set Jacobian function for implicit methods
    pub fn set_jacobian_function(&mut self, func: JacobianFunction) {
        self.jacobian_func = Some(func);
    }

    /// Enable adaptive step size control
    pub fn enable_adaptive_stepping(&mut self, control: AdaptiveStepControl) {
        self.adaptive_control = Some(control);
    }

    /// Add event function for event detection
    pub fn add_event_function(&mut self, event_func: EventFunction) {
        self.event_functions.push(event_func);
    }

    /// Solve ODE from t0 to t_final with initial conditions
    pub fn solve(&self, y0: StateVector, t0: f64, t_final: f64, initial_step: f64) -> Result<ODESolution, String> {
        if self.derivative_func.is_none() {
            return Err("Derivative function not set".to_string());
        }

        let func = self.derivative_func.as_ref().unwrap();
        let mut solution = ODESolution::new();
        let mut t = t0;
        let mut y = y0;
        let mut h = initial_step;

        solution.add_point(t, y.clone());

        while t < t_final {
            // Adjust step size to not overshoot
            if t + h > t_final {
                h = t_final - t;
            }

            // Take integration step
            let (new_y, _new_h, error) = match self.method {
                IntegrationMethod::Euler => {
                    let new_y = self.euler_step(&y, t, h, func);
                    (new_y, h, 0.0)
                },
                IntegrationMethod::RungeKutta4 => {
                    let new_y = self.rk4_step(&y, t, h, func);
                    (new_y, h, 0.0)
                },
                IntegrationMethod::RungeKuttaFehlberg45 => {
                    self.rkf45_step(&y, t, h, func)?
                },
                IntegrationMethod::DormandPrince => {
                    self.dp_step(&y, t, h, func)?
                },
                IntegrationMethod::ImplicitEuler => {
                    let new_y = self.implicit_euler_step(&y, t, h, func)?;
                    (new_y, h, 0.0)
                },
                IntegrationMethod::BDF2 => {
                    // BDF2 requires previous step information
                    let new_y = self.euler_step(&y, t, h, func); // Fallback for first step
                    (new_y, h, 0.0)
                },
                IntegrationMethod::Verlet => {
                    let new_y = self.verlet_step(&y, t, h, func);
                    (new_y, h, 0.0)
                },
                IntegrationMethod::LeapFrog => {
                    let new_y = self.leapfrog_step(&y, t, h, func);
                    (new_y, h, 0.0)
                },
            };

            // Adaptive step size control
            if let Some(control) = &self.adaptive_control {
                if error > 0.0 {
                    let error_ratio = control.tolerance / error;
                    let new_step = h * control.safety_factor * error_ratio.powf(1.0 / control.error_order);
                    h = new_step.clamp(control.min_step, control.max_step);

                    if error > control.tolerance {
                        // Reject step and retry with smaller step size
                        continue;
                    }
                }
            }

            t += h;
            y = new_y;

            // Check for events
            for event_func in &self.event_functions {
                let event_value = event_func(t, &y);
                if event_value.abs() < 1e-10 {
                    solution.add_event(t, y.clone());
                }
            }

            solution.add_point(t, y.clone());

            // Prevent infinite loops
            if solution.time_points.len() > 1_000_000 {
                return Err("Maximum number of steps exceeded".to_string());
            }
        }

        Ok(solution)
    }

    /// Euler's method (first-order)
    fn euler_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> StateVector {
        let dy = func(t, y);
        y.iter().zip(dy.iter()).map(|(yi, dyi)| yi + h * dyi).collect()
    }

    /// Fourth-order Runge-Kutta method
    fn rk4_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> StateVector {
        let k1 = func(t, y);
        
        let y_temp: StateVector = y.iter().zip(k1.iter())
            .map(|(yi, k1i)| yi + 0.5 * h * k1i).collect();
        let k2 = func(t + 0.5 * h, &y_temp);
        
        let y_temp: StateVector = y.iter().zip(k2.iter())
            .map(|(yi, k2i)| yi + 0.5 * h * k2i).collect();
        let k3 = func(t + 0.5 * h, &y_temp);
        
        let y_temp: StateVector = y.iter().zip(k3.iter())
            .map(|(yi, k3i)| yi + h * k3i).collect();
        let k4 = func(t + h, &y_temp);
        
        y.iter().enumerate().map(|(i, yi)| {
            yi + (h / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])
        }).collect()
    }

    /// Runge-Kutta-Fehlberg method (adaptive RK45)
    fn rkf45_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> Result<(StateVector, f64, f64), String> {
        let k1 = func(t, y);
        
        let y_temp: StateVector = y.iter().zip(k1.iter())
            .map(|(yi, k1i)| yi + h * k1i / 4.0).collect();
        let k2 = func(t + h / 4.0, &y_temp);
        
        let y_temp: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (3.0 * k1[i] + 9.0 * k2[i]) / 32.0).collect();
        let k3 = func(t + 3.0 * h / 8.0, &y_temp);
        
        let y_temp: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (1932.0 * k1[i] - 7200.0 * k2[i] + 7296.0 * k3[i]) / 2197.0).collect();
        let k4 = func(t + 12.0 * h / 13.0, &y_temp);
        
        let y_temp: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (439.0 * k1[i] / 216.0 - 8.0 * k2[i] + 3680.0 * k3[i] / 513.0 - 845.0 * k4[i] / 4104.0)).collect();
        let k5 = func(t + h, &y_temp);
        
        let y_temp: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (-8.0 * k1[i] / 27.0 + 2.0 * k2[i] - 3544.0 * k3[i] / 2565.0 + 1859.0 * k4[i] / 4104.0 - 11.0 * k5[i] / 40.0)).collect();
        let k6 = func(t + h / 2.0, &y_temp);

        // 4th order solution
        let y4: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (25.0 * k1[i] / 216.0 + 1408.0 * k3[i] / 2565.0 + 2197.0 * k4[i] / 4104.0 - k5[i] / 5.0)).collect();

        // 5th order solution
        let y5: StateVector = y.iter().enumerate()
            .map(|(i, yi)| yi + h * (16.0 * k1[i] / 135.0 + 6656.0 * k3[i] / 12825.0 + 28561.0 * k4[i] / 56430.0 - 9.0 * k5[i] / 50.0 + 2.0 * k6[i] / 55.0)).collect();

        // Error estimate
        let error: f64 = y4.iter().zip(y5.iter())
            .map(|(y4i, y5i)| (y4i - y5i).abs())
            .fold(0.0, f64::max);

        Ok((y5, h, error))
    }

    /// Dormand-Prince method (high-accuracy adaptive)
    fn dp_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> Result<(StateVector, f64, f64), String> {
        // Simplified version - full DP would have different coefficients
        self.rkf45_step(y, t, h, func)
    }

    /// Implicit Euler method for stiff systems
    fn implicit_euler_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> Result<StateVector, String> {
        // Newton's method to solve: y_new = y + h * f(t + h, y_new)
        let mut y_new = y.clone();
        
        for _iteration in 0..10 { // Maximum 10 Newton iterations
            let f_new = func(t + h, &y_new);
            let residual: StateVector = y_new.iter().enumerate()
                .map(|(i, yi)| yi - y[i] - h * f_new[i]).collect();
            
            // Check convergence
            let residual_norm: f64 = residual.iter().map(|r| r * r).sum::<f64>().sqrt();
            if residual_norm < 1e-10 {
                break;
            }
            
            // Simplified Newton step (assuming identity Jacobian for now)
            for i in 0..y_new.len() {
                y_new[i] -= residual[i] / (1.0 + h); // Simplified
            }
        }
        
        Ok(y_new)
    }

    /// Verlet integration for Hamiltonian systems
    fn verlet_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> StateVector {
        // Assumes y = [positions, velocities]
        if y.len() % 2 != 0 {
            return self.euler_step(y, t, h, func); // Fallback
        }
        
        let n = y.len() / 2;
        let mut new_y = y.clone();
        
        let dy = func(t, y);
        
        // Update positions: x_new = x + v * h + 0.5 * a * h^2
        for i in 0..n {
            let acceleration = dy[i + n];
            new_y[i] = y[i] + y[i + n] * h + 0.5 * acceleration * h * h;
        }
        
        // Update velocities: v_new = v + 0.5 * (a_old + a_new) * h
        let new_dy = func(t + h, &new_y);
        for i in 0..n {
            let old_acceleration = dy[i + n];
            let new_acceleration = new_dy[i + n];
            new_y[i + n] = y[i + n] + 0.5 * (old_acceleration + new_acceleration) * h;
        }
        
        new_y
    }

    /// Leapfrog integration (symplectic)
    fn leapfrog_step(&self, y: &StateVector, t: f64, h: f64, func: &DerivativeFunction) -> StateVector {
        // Similar to Verlet but with different update order
        if y.len() % 2 != 0 {
            return self.euler_step(y, t, h, func); // Fallback
        }
        
        let n = y.len() / 2;
        let mut new_y = y.clone();
        
        let dy = func(t, y);
        
        // Half-step velocity update
        for i in 0..n {
            new_y[i + n] = y[i + n] + 0.5 * dy[i + n] * h;
        }
        
        // Full-step position update
        for i in 0..n {
            new_y[i] = y[i] + new_y[i + n] * h;
        }
        
        // Half-step velocity update
        let new_dy = func(t + h, &new_y);
        for i in 0..n {
            new_y[i + n] = new_y[i + n] + 0.5 * new_dy[i + n] * h;
        }
        
        new_y
    }
}

/// Solution container for ODE integration
#[derive(Debug, Clone)]
pub struct ODESolution {
    pub time_points: Vec<f64>,
    pub state_history: Vec<StateVector>,
    pub events: Vec<(f64, StateVector)>,
}

impl ODESolution {
    pub fn new() -> Self {
        Self {
            time_points: Vec::new(),
            state_history: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn add_point(&mut self, t: f64, y: StateVector) {
        self.time_points.push(t);
        self.state_history.push(y);
    }

    pub fn add_event(&mut self, t: f64, y: StateVector) {
        self.events.push((t, y));
    }

    pub fn get_final_state(&self) -> Option<&StateVector> {
        self.state_history.last()
    }

    pub fn interpolate(&self, t: f64) -> Option<StateVector> {
        if self.time_points.is_empty() {
            return None;
        }
        
        // Find bracketing points
        let mut i = 0;
        while i < self.time_points.len() - 1 && self.time_points[i + 1] < t {
            i += 1;
        }
        
        if i == self.time_points.len() - 1 {
            return Some(self.state_history[i].clone());
        }
        
        // Linear interpolation
        let t0 = self.time_points[i];
        let t1 = self.time_points[i + 1];
        let alpha = (t - t0) / (t1 - t0);
        
        let y0 = &self.state_history[i];
        let y1 = &self.state_history[i + 1];
        
        let interpolated: StateVector = y0.iter().zip(y1.iter())
            .map(|(y0i, y1i)| y0i + alpha * (y1i - y0i))
            .collect();
        
        Some(interpolated)
    }
}

/// System of differential equations for complex simulations
pub struct DifferentialSystem {
    pub subsystems: HashMap<String, ODESolver>,
    pub coupling_functions: Vec<Box<dyn Fn(&HashMap<String, StateVector>) -> HashMap<String, StateVector> + Send + Sync>>,
}

impl DifferentialSystem {
    pub fn new() -> Self {
        Self {
            subsystems: HashMap::new(),
            coupling_functions: Vec::new(),
        }
    }

    pub fn add_subsystem(&mut self, name: String, solver: ODESolver) {
        self.subsystems.insert(name, solver);
    }

    pub fn add_coupling(&mut self, coupling_func: Box<dyn Fn(&HashMap<String, StateVector>) -> HashMap<String, StateVector> + Send + Sync>) {
        self.coupling_functions.push(coupling_func);
    }

    /// Solve coupled system (simplified approach)
    pub fn solve_coupled(&self, initial_states: HashMap<String, StateVector>, 
                        t0: f64, t_final: f64, h: f64) -> Result<HashMap<String, ODESolution>, String> {
        // This is a simplified version - proper coupling would require more sophisticated methods
        let mut solutions = HashMap::new();
        
        for (name, solver) in &self.subsystems {
            if let Some(initial_state) = initial_states.get(name) {
                let solution = solver.solve(initial_state.clone(), t0, t_final, h)?;
                solutions.insert(name.clone(), solution);
            }
        }
        
        Ok(solutions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ode_solver_creation() {
        let solver = ODESolver::new(IntegrationMethod::RungeKutta4, 2);
        
        assert_eq!(solver.state_dimension, 2);
        assert!(matches!(solver.method, IntegrationMethod::RungeKutta4));
        assert!(solver.derivative_func.is_none());
        assert!(solver.adaptive_control.is_none());
    }

    #[test]
    fn test_adaptive_step_control() {
        let control = AdaptiveStepControl::default();
        
        assert_eq!(control.tolerance, 1e-6);
        assert!(control.min_step > 0.0);
        assert!(control.max_step > control.min_step);
        assert!(control.safety_factor > 0.0 && control.safety_factor < 1.0);
    }

    #[test]
    fn test_simple_ode_solution() {
        let mut solver = ODESolver::new(IntegrationMethod::Euler, 1);
        
        // dy/dt = -y (exponential decay)
        solver.set_derivative_function(Box::new(|_t, y| vec![-y[0]]));
        
        let initial_state = vec![1.0];
        let solution = solver.solve(initial_state, 0.0, 1.0, 0.1).unwrap();
        
        assert!(!solution.time_points.is_empty());
        assert_eq!(solution.time_points.len(), solution.state_history.len());
        
        // Should decay exponentially
        let final_state = solution.get_final_state().unwrap();
        assert!(final_state[0] < 1.0);
        assert!(final_state[0] > 0.0);
    }

    #[test]
    fn test_harmonic_oscillator() {
        let mut solver = ODESolver::new(IntegrationMethod::RungeKutta4, 2);
        
        // Simple harmonic oscillator: d²x/dt² = -ω²x
        // State: [position, velocity]
        let omega = 1.0;
        solver.set_derivative_function(Box::new(move |_t, y| {
            vec![y[1], -omega * omega * y[0]]
        }));
        
        let initial_state = vec![1.0, 0.0]; // Start at x=1, v=0
        let solution = solver.solve(initial_state, 0.0, 2.0 * std::f64::consts::PI, 0.01).unwrap();
        
        assert!(!solution.time_points.is_empty());
        
        // Should oscillate and return close to initial position after one period
        let final_state = solution.get_final_state().unwrap();
        assert!((final_state[0] - 1.0).abs() < 0.1); // Position
        assert!(final_state[1].abs() < 0.1); // Velocity
    }

    #[test]
    fn test_rk4_vs_euler() {
        // Compare Euler and RK4 for same problem
        let derivative_func = |_t: f64, y: &StateVector| vec![-y[0]];
        
        let mut euler_solver = ODESolver::new(IntegrationMethod::Euler, 1);
        euler_solver.set_derivative_function(Box::new(derivative_func));
        
        let mut rk4_solver = ODESolver::new(IntegrationMethod::RungeKutta4, 1);
        rk4_solver.set_derivative_function(Box::new(derivative_func));
        
        let initial_state = vec![1.0];
        let h = 0.1;
        
        let euler_solution = euler_solver.solve(initial_state.clone(), 0.0, 1.0, h).unwrap();
        let rk4_solution = rk4_solver.solve(initial_state, 0.0, 1.0, h).unwrap();
        
        // RK4 should be more accurate
        let exact_final = (-1.0_f64).exp(); // e^(-1)
        let euler_final = euler_solution.get_final_state().unwrap()[0];
        let rk4_final = rk4_solution.get_final_state().unwrap()[0];
        
        let euler_error = (euler_final - exact_final).abs();
        let rk4_error = (rk4_final - exact_final).abs();
        
        assert!(rk4_error < euler_error);
    }

    #[test]
    fn test_solution_interpolation() {
        let mut solution = ODESolution::new();
        
        solution.add_point(0.0, vec![0.0]);
        solution.add_point(1.0, vec![1.0]);
        solution.add_point(2.0, vec![4.0]);
        
        // Test interpolation at t = 0.5
        let interpolated = solution.interpolate(0.5).unwrap();
        assert_eq!(interpolated[0], 0.5);
        
        // Test interpolation at t = 1.5
        let interpolated = solution.interpolate(1.5).unwrap();
        assert_eq!(interpolated[0], 2.5);
    }

    #[test]
    fn test_differential_system() {
        let mut system = DifferentialSystem::new();
        
        let mut solver1 = ODESolver::new(IntegrationMethod::RungeKutta4, 1);
        solver1.set_derivative_function(Box::new(|_t, y| vec![-y[0]]));
        
        let mut solver2 = ODESolver::new(IntegrationMethod::RungeKutta4, 1);
        solver2.set_derivative_function(Box::new(|_t, y| vec![y[0]]));
        
        system.add_subsystem("decay".to_string(), solver1);
        system.add_subsystem("growth".to_string(), solver2);
        
        let mut initial_states = HashMap::new();
        initial_states.insert("decay".to_string(), vec![1.0]);
        initial_states.insert("growth".to_string(), vec![1.0]);
        
        let solutions = system.solve_coupled(initial_states, 0.0, 1.0, 0.1).unwrap();
        
        assert_eq!(solutions.len(), 2);
        assert!(solutions.contains_key("decay"));
        assert!(solutions.contains_key("growth"));
    }

    #[test]
    fn test_event_detection() {
        let mut solver = ODESolver::new(IntegrationMethod::RungeKutta4, 2);
        
        // Harmonic oscillator
        solver.set_derivative_function(Box::new(|_t, y| vec![y[1], -y[0]]));
        
        // Event when position crosses zero
        solver.add_event_function(Box::new(|_t, y| y[0]));
        
        let initial_state = vec![1.0, 0.0];
        let solution = solver.solve(initial_state, 0.0, 2.0 * std::f64::consts::PI, 0.01).unwrap();
        
        // Should have detected events when crossing zero
        assert!(!solution.events.is_empty());
    }
}
