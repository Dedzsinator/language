// Probability Sampling - Advanced Monte Carlo and Statistical Methods
use super::math::*;
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::distributions::{Distribution, Uniform};
use rand_pcg::Pcg64;

/// Random number generator with high-quality PCG algorithm
pub struct AdvancedRng {
    rng: Pcg64,
}

impl AdvancedRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64::seed_from_u64(seed),
        }
    }

    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        Uniform::new(min, max).sample(&mut self.rng)
    }

    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        // Box-Muller transform
        static mut NEXT_GAUSSIAN: Option<f64> = None;
        static mut HAS_NEXT_GAUSSIAN: bool = false;
        
        unsafe {
            if HAS_NEXT_GAUSSIAN {
                HAS_NEXT_GAUSSIAN = false;
                return mean + std_dev * NEXT_GAUSSIAN.unwrap();
            }
        }
        
        let u1 = self.rng.gen::<f64>();
        let u2 = self.rng.gen::<f64>();
        
        let mag = std_dev * (-2.0 * u1.ln()).sqrt();
        let z0 = mag * (2.0 * std::f64::consts::PI * u2).cos();
        let z1 = mag * (2.0 * std::f64::consts::PI * u2).sin();
        
        unsafe {
            NEXT_GAUSSIAN = Some(z1);
            HAS_NEXT_GAUSSIAN = true;
        }
        
        mean + z0
    }

    pub fn exponential(&mut self, lambda: f64) -> f64 {
        -(-self.rng.gen::<f64>()).ln() / lambda
    }

    pub fn gamma(&mut self, shape: f64, scale: f64) -> f64 {
        // Marsaglia and Tsang method for gamma distribution
        if shape < 1.0 {
            return self.gamma(shape + 1.0, scale) * self.uniform(0.0, 1.0).powf(1.0 / shape);
        }

        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();

        loop {
            let x = self.normal(0.0, 1.0);
            let v = (1.0 + c * x).powi(3);
            
            if v > 0.0 {
                let u = self.uniform(0.0, 1.0);
                if u < 1.0 - 0.0331 * x.powi(4) {
                    return d * v * scale;
                }
                if u.ln() < 0.5 * x.powi(2) + d * (1.0 - v + v.ln()) {
                    return d * v * scale;
                }
            }
        }
    }

    pub fn beta(&mut self, alpha: f64, beta: f64) -> f64 {
        let x = self.gamma(alpha, 1.0);
        let y = self.gamma(beta, 1.0);
        x / (x + y)
    }

    pub fn poisson(&mut self, lambda: f64) -> usize {
        // Knuth's algorithm for small lambda
        if lambda < 30.0 {
            let l = (-lambda).exp();
            let mut k = 0;
            let mut p = 1.0;
            
            loop {
                k += 1;
                p *= self.uniform(0.0, 1.0);
                if p <= l {
                    return k - 1;
                }
            }
        } else {
            // For large lambda, use normal approximation
            let x = self.normal(lambda, lambda.sqrt());
            (x.round() as usize).max(0)
        }
    }

    pub fn multivariate_normal(&mut self, mean: &[f64], covariance: &[Vec<f64>]) -> Vec<f64> {
        let n = mean.len();
        let mut z: Vec<f64> = (0..n).map(|_| self.normal(0.0, 1.0)).collect();
        
        // Cholesky decomposition of covariance matrix
        let chol = cholesky_decomposition(covariance);
        
        // Transform z using Cholesky factor
        let mut result = vec![0.0; n];
        for i in 0..n {
            for j in 0..=i {
                result[i] += chol[i][j] * z[j];
            }
            result[i] += mean[i];
        }
        
        result
    }

    pub fn uniform_sphere(&mut self, radius: f64) -> Vec3 {
        // Marsaglia method for uniform distribution on sphere
        loop {
            let x = self.uniform(-1.0, 1.0);
            let y = self.uniform(-1.0, 1.0);
            let r2 = x * x + y * y;
            
            if r2 < 1.0 {
                let factor = 2.0 * (1.0 - r2).sqrt();
                return Vec3::new(
                    x * factor,
                    y * factor,
                    1.0 - 2.0 * r2
                ) * radius;
            }
        }
    }

    pub fn uniform_disk(&mut self, radius: f64) -> Vec3 {
        let angle = self.uniform(0.0, 2.0 * std::f64::consts::PI);
        let r = radius * self.uniform(0.0, 1.0).sqrt();
        Vec3::new(r * angle.cos(), r * angle.sin(), 0.0)
    }
}

/// Cholesky decomposition for covariance matrices
fn cholesky_decomposition(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut l = vec![vec![0.0; n]; n];
    
    for i in 0..n {
        for j in 0..=i {
            if i == j {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[j][k] * l[j][k];
                }
                l[j][j] = (matrix[j][j] - sum).sqrt();
            } else {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[i][k] * l[j][k];
                }
                l[i][j] = (matrix[i][j] - sum) / l[j][j];
            }
        }
    }
    
    l
}

/// Monte Carlo integration methods
#[derive(Debug, Clone)]
pub struct MonteCarloIntegrator {
    pub samples: usize,
    pub confidence_level: f64,
    pub variance_reduction: VarianceReduction,
}

#[derive(Debug, Clone, Copy)]
pub enum VarianceReduction {
    None,
    AntitheticVariates,
    ControlVariates,
    ImportanceSampling,
    StratifiedSampling,
}

impl MonteCarloIntegrator {
    pub fn new(samples: usize) -> Self {
        Self {
            samples,
            confidence_level: 0.95,
            variance_reduction: VarianceReduction::None,
        }
    }

    /// Integrate function over [0,1]^n using Monte Carlo
    pub fn integrate<F>(&self, func: F, dimension: usize, rng: &mut AdvancedRng) -> MonteCarloResult
    where
        F: Fn(&[f64]) -> f64,
    {
        match self.variance_reduction {
            VarianceReduction::None => self.simple_monte_carlo(func, dimension, rng),
            VarianceReduction::AntitheticVariates => self.antithetic_variates(func, dimension, rng),
            VarianceReduction::StratifiedSampling => self.stratified_sampling(func, dimension, rng),
            _ => self.simple_monte_carlo(func, dimension, rng), // Fallback
        }
    }

    fn simple_monte_carlo<F>(&self, func: F, dimension: usize, rng: &mut AdvancedRng) -> MonteCarloResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let mut sum = 0.0;
        let mut sum_squared = 0.0;
        let mut values = Vec::new();
        
        for _ in 0..self.samples {
            let point: Vec<f64> = (0..dimension).map(|_| rng.uniform(0.0, 1.0)).collect();
            let value = func(&point);
            values.push(value);
            sum += value;
            sum_squared += value * value;
        }
        
        let mean = sum / self.samples as f64;
        let variance = (sum_squared / self.samples as f64) - mean * mean;
        let std_error = (variance / self.samples as f64).sqrt();
        
        // Confidence interval
        let t_value = self.get_t_value();
        let margin_of_error = t_value * std_error;
        
        MonteCarloResult {
            estimate: mean,
            standard_error: std_error,
            confidence_interval: (mean - margin_of_error, mean + margin_of_error),
            samples_used: self.samples,
            variance,
            values,
        }
    }

    fn antithetic_variates<F>(&self, func: F, dimension: usize, rng: &mut AdvancedRng) -> MonteCarloResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let pairs = self.samples / 2;
        let mut sum = 0.0;
        let mut sum_squared = 0.0;
        let mut values = Vec::new();
        
        for _ in 0..pairs {
            let point: Vec<f64> = (0..dimension).map(|_| rng.uniform(0.0, 1.0)).collect();
            let antithetic_point: Vec<f64> = point.iter().map(|x| 1.0 - x).collect();
            
            let value1 = func(&point);
            let value2 = func(&antithetic_point);
            let avg_value = (value1 + value2) / 2.0;
            
            values.push(value1);
            values.push(value2);
            sum += avg_value;
            sum_squared += avg_value * avg_value;
        }
        
        let mean = sum / pairs as f64;
        let variance = (sum_squared / pairs as f64) - mean * mean;
        let std_error = (variance / pairs as f64).sqrt();
        
        let t_value = self.get_t_value();
        let margin_of_error = t_value * std_error;
        
        MonteCarloResult {
            estimate: mean,
            standard_error: std_error,
            confidence_interval: (mean - margin_of_error, mean + margin_of_error),
            samples_used: self.samples,
            variance,
            values,
        }
    }

    fn stratified_sampling<F>(&self, func: F, dimension: usize, rng: &mut AdvancedRng) -> MonteCarloResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let strata_per_dim = (self.samples as f64).powf(1.0 / dimension as f64).ceil() as usize;
        let total_strata = strata_per_dim.pow(dimension as u32);
        let samples_per_stratum = (self.samples / total_strata).max(1);
        
        let mut sum = 0.0;
        let mut sum_squared = 0.0;
        let mut values = Vec::new();
        let mut total_samples = 0;
        
        // Generate all stratum indices
        for stratum_idx in 0..total_strata {
            let mut stratum_coords = vec![0; dimension];
            let mut temp_idx = stratum_idx;
            
            for d in 0..dimension {
                stratum_coords[d] = temp_idx % strata_per_dim;
                temp_idx /= strata_per_dim;
            }
            
            // Sample within this stratum
            for _ in 0..samples_per_stratum {
                let point: Vec<f64> = stratum_coords.iter().enumerate()
                    .map(|(d, &coord)| {
                        let stratum_min = coord as f64 / strata_per_dim as f64;
                        let stratum_max = (coord + 1) as f64 / strata_per_dim as f64;
                        rng.uniform(stratum_min, stratum_max)
                    })
                    .collect();
                
                let value = func(&point);
                values.push(value);
                sum += value;
                sum_squared += value * value;
                total_samples += 1;
            }
        }
        
        let mean = sum / total_samples as f64;
        let variance = (sum_squared / total_samples as f64) - mean * mean;
        let std_error = (variance / total_samples as f64).sqrt();
        
        let t_value = self.get_t_value();
        let margin_of_error = t_value * std_error;
        
        MonteCarloResult {
            estimate: mean,
            standard_error: std_error,
            confidence_interval: (mean - margin_of_error, mean + margin_of_error),
            samples_used: total_samples,
            variance,
            values,
        }
    }

    fn get_t_value(&self) -> f64 {
        // Simplified t-value for common confidence levels
        match self.confidence_level {
            level if level >= 0.99 => 2.576,
            level if level >= 0.95 => 1.96,
            level if level >= 0.90 => 1.645,
            _ => 1.0,
        }
    }
}

/// Result of Monte Carlo integration
#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    pub estimate: f64,
    pub standard_error: f64,
    pub confidence_interval: (f64, f64),
    pub samples_used: usize,
    pub variance: f64,
    pub values: Vec<f64>,
}

/// Markov Chain Monte Carlo (MCMC) sampler
#[derive(Debug)]
pub struct MCMCSampler {
    pub chain_length: usize,
    pub burn_in: usize,
    pub thinning: usize,
    pub algorithm: MCMCAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum MCMCAlgorithm {
    Metropolis,
    MetropolisHastings,
    Gibbs,
    HamiltonianMonteCarlo,
}

impl MCMCSampler {
    pub fn new(algorithm: MCMCAlgorithm, chain_length: usize) -> Self {
        Self {
            chain_length,
            burn_in: chain_length / 10,
            thinning: 1,
            algorithm,
        }
    }

    /// Sample from a probability distribution using MCMC
    pub fn sample<F>(&self, log_density: F, initial_state: Vec<f64>, rng: &mut AdvancedRng) -> MCMCResult
    where
        F: Fn(&[f64]) -> f64,
    {
        match self.algorithm {
            MCMCAlgorithm::Metropolis => self.metropolis_sampling(log_density, initial_state, rng),
            MCMCAlgorithm::MetropolisHastings => self.metropolis_hastings_sampling(log_density, initial_state, rng),
            _ => self.metropolis_sampling(log_density, initial_state, rng), // Fallback
        }
    }

    fn metropolis_sampling<F>(&self, log_density: F, mut current_state: Vec<f64>, rng: &mut AdvancedRng) -> MCMCResult
    where
        F: Fn(&[f64]) -> f64,
    {
        let mut samples = Vec::new();
        let mut current_log_density = log_density(&current_state);
        let mut accepted = 0;
        let step_size = 0.5; // Could be adaptive
        
        for i in 0..self.chain_length {
            // Propose new state
            let mut proposed_state = current_state.clone();
            for j in 0..proposed_state.len() {
                proposed_state[j] += rng.normal(0.0, step_size);
            }
            
            let proposed_log_density = log_density(&proposed_state);
            let log_acceptance_ratio = proposed_log_density - current_log_density;
            
            // Accept or reject
            if log_acceptance_ratio > 0.0 || rng.uniform(0.0, 1.0).ln() < log_acceptance_ratio {
                current_state = proposed_state;
                current_log_density = proposed_log_density;
                accepted += 1;
            }
            
            // Store sample (after burn-in and thinning)
            if i >= self.burn_in && (i - self.burn_in) % self.thinning == 0 {
                samples.push(current_state.clone());
            }
        }
        
        let effective_sample_size = self.estimate_ess(&samples);
        
        MCMCResult {
            samples,
            acceptance_rate: accepted as f64 / self.chain_length as f64,
            effective_sample_size,
        }
    }

    fn metropolis_hastings_sampling<F>(&self, log_density: F, initial_state: Vec<f64>, rng: &mut AdvancedRng) -> MCMCResult
    where
        F: Fn(&[f64]) -> f64,
    {
        // For simplicity, using same implementation as Metropolis
        // In practice, would include proposal distribution handling
        self.metropolis_sampling(log_density, initial_state, rng)
    }

    fn estimate_ess(&self, samples: &[Vec<f64>]) -> f64 {
        // Simplified effective sample size estimation
        // In practice, would compute autocorrelation
        samples.len() as f64 * 0.5 // Conservative estimate
    }
}

/// Result of MCMC sampling
#[derive(Debug, Clone)]
pub struct MCMCResult {
    pub samples: Vec<Vec<f64>>,
    pub acceptance_rate: f64,
    pub effective_sample_size: f64,
}

/// Importance sampling for rare event simulation
#[derive(Debug)]
pub struct ImportanceSampler {
    pub samples: usize,
}

impl ImportanceSampler {
    pub fn new(samples: usize) -> Self {
        Self { samples }
    }

    /// Estimate probability of rare event using importance sampling
    pub fn estimate_rare_event<F, G>(&self, event_indicator: F, importance_density: G, 
                                    rng: &mut AdvancedRng) -> f64
    where
        F: Fn(&[f64]) -> f64,
        G: Fn(&mut AdvancedRng) -> (Vec<f64>, f64), // Returns (sample, weight)
    {
        let mut total_weight = 0.0;
        let mut weighted_events = 0.0;
        
        for _ in 0..self.samples {
            let (sample, weight) = importance_density(rng);
            let event_value = event_indicator(&sample);
            
            weighted_events += event_value * weight;
            total_weight += weight;
        }
        
        weighted_events / total_weight
    }
}

/// Quasi-Monte Carlo using low-discrepancy sequences
#[derive(Debug)]
pub struct QuasiMonteCarloSampler {
    pub samples: usize,
    pub sequence_type: QuasiSequence,
}

#[derive(Debug, Clone, Copy)]
pub enum QuasiSequence {
    Halton,
    Sobol,
    Faure,
}

impl QuasiMonteCarloSampler {
    pub fn new(sequence_type: QuasiSequence, samples: usize) -> Self {
        Self {
            samples,
            sequence_type,
        }
    }

    /// Generate low-discrepancy sequence
    pub fn generate_sequence(&self, dimension: usize) -> Vec<Vec<f64>> {
        match self.sequence_type {
            QuasiSequence::Halton => self.halton_sequence(dimension),
            QuasiSequence::Sobol => self.sobol_sequence(dimension),
            QuasiSequence::Faure => self.halton_sequence(dimension), // Fallback to Halton
        }
    }

    fn halton_sequence(&self, dimension: usize) -> Vec<Vec<f64>> {
        let primes = self.first_primes(dimension);
        let mut sequence = Vec::new();
        
        for i in 1..=self.samples {
            let mut point = Vec::new();
            for &prime in &primes {
                point.push(self.radical_inverse(i, prime));
            }
            sequence.push(point);
        }
        
        sequence
    }

    fn sobol_sequence(&self, dimension: usize) -> Vec<Vec<f64>> {
        // Simplified Sobol sequence - in practice would use proper direction numbers
        self.halton_sequence(dimension)
    }

    fn radical_inverse(&self, n: usize, base: usize) -> f64 {
        let mut result = 0.0;
        let mut f = 1.0 / base as f64;
        let mut i = n;
        
        while i > 0 {
            result += f * (i % base) as f64;
            i /= base;
            f /= base as f64;
        }
        
        result
    }

    fn first_primes(&self, count: usize) -> Vec<usize> {
        let mut primes = Vec::new();
        let mut candidate = 2;
        
        while primes.len() < count {
            if self.is_prime(candidate) {
                primes.push(candidate);
            }
            candidate += 1;
        }
        
        primes
    }

    fn is_prime(&self, n: usize) -> bool {
        if n < 2 {
            return false;
        }
        for i in 2..=(n as f64).sqrt() as usize {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_rng() {
        let mut rng = AdvancedRng::new(12345);
        
        // Test uniform distribution
        let uniform_val = rng.uniform(0.0, 1.0);
        assert!(uniform_val >= 0.0 && uniform_val <= 1.0);
        
        // Test normal distribution
        let normal_val = rng.normal(0.0, 1.0);
        assert!(normal_val.abs() < 10.0); // Very loose bound
        
        // Test exponential distribution
        let exp_val = rng.exponential(1.0);
        assert!(exp_val >= 0.0);
        
        // Test sphere sampling
        let sphere_point = rng.uniform_sphere(1.0);
        assert!((sphere_point.magnitude() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_monte_carlo_integration() {
        let integrator = MonteCarloIntegrator::new(10000);
        let mut rng = AdvancedRng::new(42);
        
        // Integrate f(x) = x over [0,1], exact answer = 0.5
        let result = integrator.integrate(|x| x[0], 1, &mut rng);
        
        assert!((result.estimate - 0.5).abs() < 0.1);
        assert!(result.standard_error > 0.0);
        assert!(result.confidence_interval.0 < result.confidence_interval.1);
    }

    #[test]
    fn test_antithetic_variates() {
        let mut integrator = MonteCarloIntegrator::new(1000);
        integrator.variance_reduction = VarianceReduction::AntitheticVariates;
        
        let mut rng = AdvancedRng::new(42);
        
        // Integrate f(x) = x^2 over [0,1], exact answer = 1/3
        let result = integrator.integrate(|x| x[0] * x[0], 1, &mut rng);
        
        assert!((result.estimate - 1.0/3.0).abs() < 0.1);
    }

    #[test]
    fn test_stratified_sampling() {
        let mut integrator = MonteCarloIntegrator::new(1000);
        integrator.variance_reduction = VarianceReduction::StratifiedSampling;
        
        let mut rng = AdvancedRng::new(42);
        
        // Integrate f(x) = 1 over [0,1], exact answer = 1
        let result = integrator.integrate(|_x| 1.0, 1, &mut rng);
        
        assert!((result.estimate - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_mcmc_sampling() {
        let sampler = MCMCSampler::new(MCMCAlgorithm::Metropolis, 1000);
        let mut rng = AdvancedRng::new(123);
        
        // Sample from standard normal distribution
        let log_density = |x: &[f64]| -0.5 * x[0] * x[0];
        let initial_state = vec![0.0];
        
        let result = sampler.sample(log_density, initial_state, &mut rng);
        
        assert!(!result.samples.is_empty());
        assert!(result.acceptance_rate > 0.0 && result.acceptance_rate <= 1.0);
        assert!(result.effective_sample_size > 0.0);
        
        // Check that samples are roughly normally distributed
        let mean: f64 = result.samples.iter().map(|s| s[0]).sum::<f64>() / result.samples.len() as f64;
        assert!(mean.abs() < 0.5); // Should be close to 0
    }

    #[test]
    fn test_halton_sequence() {
        let sampler = QuasiMonteCarloSampler::new(QuasiSequence::Halton, 100);
        let sequence = sampler.generate_sequence(2);
        
        assert_eq!(sequence.len(), 100);
        assert_eq!(sequence[0].len(), 2);
        
        // Check that all points are in [0,1]^2
        for point in &sequence {
            for &coord in point {
                assert!(coord >= 0.0 && coord <= 1.0);
            }
        }
        
        // First few Halton points for base 2 and 3
        assert!((sequence[0][0] - 0.5).abs() < 1e-10);
        assert!((sequence[0][1] - 1.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_multivariate_normal() {
        let mut rng = AdvancedRng::new(456);
        
        let mean = vec![1.0, 2.0];
        let covariance = vec![
            vec![1.0, 0.5],
            vec![0.5, 2.0],
        ];
        
        let sample = rng.multivariate_normal(&mean, &covariance);
        
        assert_eq!(sample.len(), 2);
        // Just check that sampling doesn't crash - proper testing would require many samples
    }

    #[test]
    fn test_importance_sampling() {
        let sampler = ImportanceSampler::new(1000);
        let mut rng = AdvancedRng::new(789);
        
        // Estimate P(X > 3) where X ~ N(0,1) using importance sampling
        let event_indicator = |x: &[f64]| if x[0] > 3.0 { 1.0 } else { 0.0 };
        let importance_density = |rng: &mut AdvancedRng| {
            let x = rng.normal(4.0, 1.0); // Shift distribution to make event more likely
            let weight = (-0.5 * x * x).exp() / (-0.5 * (x - 4.0) * (x - 4.0)).exp();
            (vec![x], weight)
        };
        
        let prob_estimate = sampler.estimate_rare_event(event_indicator, importance_density, &mut rng);
        
        // The true probability is approximately 0.00135
        assert!(prob_estimate >= 0.0 && prob_estimate <= 1.0);
    }

    #[test]
    fn test_beta_distribution() {
        let mut rng = AdvancedRng::new(999);
        
        let sample = rng.beta(2.0, 5.0);
        assert!(sample >= 0.0 && sample <= 1.0);
        
        // Test multiple samples
        let samples: Vec<f64> = (0..1000).map(|_| rng.beta(2.0, 5.0)).collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        
        // Theoretical mean of Beta(2,5) is 2/(2+5) = 2/7 ≈ 0.286
        assert!((mean - 2.0/7.0).abs() < 0.1);
    }

    #[test]
    fn test_poisson_distribution() {
        let mut rng = AdvancedRng::new(111);
        
        let lambda = 3.0;
        let samples: Vec<usize> = (0..1000).map(|_| rng.poisson(lambda)).collect();
        let mean = samples.iter().sum::<usize>() as f64 / samples.len() as f64;
        
        // Mean of Poisson distribution should be lambda
        assert!((mean - lambda).abs() < 0.5);
    }
}
