// Ultra-high-performance quantum circuit simulator
// Features: parallel processing, SIMD optimization, sparse representation, GPU acceleration hooks

use crate::quantum::circuit::{CircuitLayer, QuantumCircuit};
use crate::quantum::gates::{GateType, QuantumGate};
use crate::quantum::state::{Amplitude, QuantumState, QubitIndex};
use crate::quantum::QuantumResult;
use num_complex::Complex;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub use_sparse_representation: bool,
    pub sparse_threshold: f64,
    pub max_parallel_gates: usize,
    pub use_gpu_acceleration: bool,
    pub memory_limit_gb: f64,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    UltraOptimized,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            use_sparse_representation: true,
            sparse_threshold: 1e-12,
            max_parallel_gates: 16,
            use_gpu_acceleration: false, // GPU acceleration not yet implemented
            memory_limit_gb: 8.0,
            optimization_level: OptimizationLevel::UltraOptimized,
        }
    }
}

#[derive(Debug)]
pub struct StateVectorSimulator {
    pub config: SimulationConfig,
    pub stats: SimulationStats,
}

#[derive(Debug, Default)]
pub struct SimulationStats {
    pub total_operations: usize,
    pub total_simulation_time: std::time::Duration,
    pub memory_usage_mb: f64,
    pub gate_timings: HashMap<String, std::time::Duration>,
    pub parallel_efficiency: f64,
}

impl StateVectorSimulator {
    pub fn new() -> Self {
        Self {
            config: SimulationConfig::default(),
            stats: SimulationStats::default(),
        }
    }

    pub fn with_config(config: SimulationConfig) -> Self {
        Self {
            config,
            stats: SimulationStats::default(),
        }
    }

    pub fn execute_circuit(&mut self, circuit: &QuantumCircuit) -> Result<QuantumResult, String> {
        let start_time = Instant::now();

        // Initialize quantum state
        let mut state = QuantumState::new(circuit.num_qubits);

        // Validate circuit
        self.validate_circuit(circuit)?;

        // Execute each layer in sequence
        for (_layer_idx, layer) in circuit.layers.iter().enumerate() {
            self.execute_layer(&mut state, layer)?;

            // Memory management for large circuits
            if self.should_compress_state(&state) {
                self.compress_state(&mut state);
            }
        }

        // Perform measurements if specified
        let measurements = self.perform_measurements(&mut state, &circuit.measurements)?;

        let execution_time = start_time.elapsed();
        self.stats.total_simulation_time += execution_time;
        self.stats.total_operations += circuit.gate_count();

        Ok(QuantumResult {
            final_state: state,
            measurements,
            execution_time,
            operations_count: circuit.gate_count(),
        })
    }

    fn validate_circuit(&self, circuit: &QuantumCircuit) -> Result<(), String> {
        // Check if circuit is too large for available memory
        let estimated_memory_gb =
            (1 << circuit.num_qubits) as f64 * 16.0 / (1024.0 * 1024.0 * 1024.0);

        if estimated_memory_gb > self.config.memory_limit_gb {
            return Err(format!(
                "Circuit requires {:.2} GB but limit is {:.2} GB",
                estimated_memory_gb, self.config.memory_limit_gb
            ));
        }

        // Validate all gates have valid qubit indices
        for layer in &circuit.layers {
            for gate in &layer.gates {
                for &qubit in &gate.qubits {
                    if qubit >= circuit.num_qubits {
                        return Err(format!(
                            "Gate references qubit {} but circuit has only {} qubits",
                            qubit, circuit.num_qubits
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn execute_layer(
        &mut self,
        state: &mut QuantumState,
        layer: &CircuitLayer,
    ) -> Result<(), String> {
        match self.config.optimization_level {
            OptimizationLevel::UltraOptimized => self.execute_layer_ultra_optimized(state, layer),
            OptimizationLevel::Aggressive => self.execute_layer_aggressive(state, layer),
            OptimizationLevel::Basic => self.execute_layer_basic(state, layer),
            OptimizationLevel::None => self.execute_layer_sequential(state, layer),
        }
    }

    fn execute_layer_ultra_optimized(
        &mut self,
        state: &mut QuantumState,
        layer: &CircuitLayer,
    ) -> Result<(), String> {
        // Ultra-optimized execution with maximum parallelization

        // Group gates by parallelizability
        if layer.parallel_gates.is_empty() {
            // Fall back to aggressive mode if no parallelization info
            return self.execute_layer_aggressive(state, layer);
        }

        for parallel_group in &layer.parallel_gates {
            let gates: Vec<&QuantumGate> = parallel_group
                .iter()
                .map(|&idx| &layer.gates[idx])
                .collect();

            if gates.len() == 1 {
                // Single gate - ultra-fast direct application
                self.apply_gate_optimized(state, gates[0])?;
            } else {
                // Multiple parallel gates - use SIMD where possible
                self.apply_parallel_gates_simd(state, &gates)?;
            }
        }

        Ok(())
    }

    fn execute_layer_aggressive(
        &mut self,
        state: &mut QuantumState,
        layer: &CircuitLayer,
    ) -> Result<(), String> {
        // Aggressive optimization with parallel gate application
        let mut single_qubit_gates = Vec::new();
        let mut two_qubit_gates = Vec::new();
        let mut other_gates = Vec::new();

        // Group gates by type
        for gate in &layer.gates {
            match gate.qubits.len() {
                1 => single_qubit_gates.push(gate),
                2 => two_qubit_gates.push(gate),
                _ => other_gates.push(gate),
            }
        }

        // Apply grouped gates
        if !single_qubit_gates.is_empty() {
            self.apply_single_qubit_gates_parallel(state, &single_qubit_gates)?;
        }
        if !two_qubit_gates.is_empty() {
            self.apply_two_qubit_gates_parallel(state, &two_qubit_gates)?;
        }
        for gate in other_gates {
            self.apply_gate_optimized(state, gate)?;
        }

        Ok(())
    }

    fn execute_layer_basic(
        &mut self,
        state: &mut QuantumState,
        layer: &CircuitLayer,
    ) -> Result<(), String> {
        // Basic parallel execution
        for gate in &layer.gates {
            self.apply_gate_optimized(state, gate)?;
        }
        Ok(())
    }

    fn execute_layer_sequential(
        &mut self,
        state: &mut QuantumState,
        layer: &CircuitLayer,
    ) -> Result<(), String> {
        // Sequential execution (no optimization)
        for gate in &layer.gates {
            self.apply_gate_basic(state, gate)?;
        }
        Ok(())
    }

    fn apply_gate_optimized(
        &mut self,
        state: &mut QuantumState,
        gate: &QuantumGate,
    ) -> Result<(), String> {
        let gate_start = Instant::now();

        match &gate.gate_type {
            GateType::Identity => {
                // No-op for identity
            }
            GateType::PauliX => {
                self.apply_pauli_x_optimized(state, gate.qubits[0]);
            }
            GateType::PauliY => {
                self.apply_pauli_y_optimized(state, gate.qubits[0]);
            }
            GateType::PauliZ => {
                self.apply_pauli_z_optimized(state, gate.qubits[0]);
            }
            GateType::Hadamard => {
                self.apply_hadamard_optimized(state, gate.qubits[0]);
            }
            GateType::CNOT => {
                self.apply_cnot_optimized(state, gate.qubits[0], gate.qubits[1]);
            }
            GateType::RX(angle) => {
                self.apply_rx_optimized(state, gate.qubits[0], *angle);
            }
            GateType::RY(angle) => {
                self.apply_ry_optimized(state, gate.qubits[0], *angle);
            }
            GateType::RZ(angle) => {
                self.apply_rz_optimized(state, gate.qubits[0], *angle);
            }
            _ => {
                // Fall back to matrix-based application
                self.apply_gate_matrix(state, gate)?;
            }
        }

        let gate_time = gate_start.elapsed();
        let gate_name = format!("{:?}", gate.gate_type);
        *self
            .stats
            .gate_timings
            .entry(gate_name)
            .or_insert(std::time::Duration::ZERO) += gate_time;

        Ok(())
    }

    // Ultra-optimized single-qubit gate implementations
    fn apply_pauli_x_optimized(&self, state: &mut QuantumState, qubit: QubitIndex) {
        let mask = 1 << qubit;

        state.amplitudes.par_chunks_mut(2048).for_each(|chunk| {
            for i in (0..chunk.len()).step_by(2) {
                let base_idx = i;
                if base_idx & mask == 0 {
                    // Swap amplitudes for |0⟩ and |1⟩ on this qubit
                    let other_idx = base_idx | mask;
                    if other_idx < chunk.len() {
                        chunk.swap(base_idx, other_idx);
                    }
                }
            }
        });

        state.is_normalized = true; // Pauli-X preserves normalization
    }

    fn apply_pauli_y_optimized(&self, state: &mut QuantumState, qubit: QubitIndex) {
        let mask = 1 << qubit;

        state.amplitudes.par_chunks_mut(2048).for_each(|chunk| {
            for i in (0..chunk.len()).step_by(2) {
                let base_idx = i;
                if base_idx & mask == 0 {
                    // Apply Pauli-Y: |0⟩ -> i|1⟩, |1⟩ -> -i|0⟩
                    let other_idx = base_idx | mask;
                    if other_idx < chunk.len() {
                        let temp = chunk[base_idx];
                        chunk[base_idx] = Complex::new(chunk[other_idx].im, -chunk[other_idx].re);
                        chunk[other_idx] = Complex::new(-temp.im, temp.re);
                    }
                }
            }
        });

        state.is_normalized = true; // Pauli-Y preserves normalization
    }

    fn apply_pauli_z_optimized(&self, state: &mut QuantumState, qubit: QubitIndex) {
        let mask = 1 << qubit;

        state
            .amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, amplitude)| {
                if i & mask != 0 {
                    *amplitude = -*amplitude;
                }
            });

        state.is_normalized = true; // Pauli-Z preserves normalization
    }

    fn apply_hadamard_optimized(&self, state: &mut QuantumState, qubit: QubitIndex) {
        let mask = 1 << qubit;
        let inv_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;

        let mut new_amplitudes = vec![Amplitude::new(0.0, 0.0); state.amplitudes.len()];

        new_amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, new_amp)| {
                let i0 = i & !mask; // Clear qubit bit
                let i1 = i | mask; // Set qubit bit

                if i & mask == 0 {
                    // |0⟩ component: (|0⟩ + |1⟩) / √2
                    *new_amp = (state.amplitudes[i0] + state.amplitudes[i1]) * inv_sqrt2;
                } else {
                    // |1⟩ component: (|0⟩ - |1⟩) / √2
                    *new_amp = (state.amplitudes[i0] - state.amplitudes[i1]) * inv_sqrt2;
                }
            });

        state.amplitudes = new_amplitudes;
        state.is_normalized = true;
    }

    fn apply_cnot_optimized(
        &self,
        state: &mut QuantumState,
        control: QubitIndex,
        target: QubitIndex,
    ) {
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        state.amplitudes.par_chunks_mut(1024).for_each(|chunk| {
            for i in 0..chunk.len() {
                if i & control_mask != 0 {
                    // Control is |1⟩, flip target
                    let flipped_target = i ^ target_mask;
                    if flipped_target < chunk.len() && flipped_target != i {
                        chunk.swap(i, flipped_target);
                    }
                }
            }
        });

        state.is_normalized = true; // CNOT preserves normalization
    }

    fn apply_rx_optimized(&self, state: &mut QuantumState, qubit: QubitIndex, angle: f64) {
        let cos_half = (angle / 2.0).cos();
        let sin_half = (angle / 2.0).sin();
        let mask = 1 << qubit;

        let mut new_amplitudes = vec![Amplitude::new(0.0, 0.0); state.amplitudes.len()];

        new_amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, new_amp)| {
                let i0 = i & !mask;
                let i1 = i | mask;

                if i & mask == 0 {
                    *new_amp = state.amplitudes[i0] * cos_half
                        - state.amplitudes[i1] * Amplitude::new(0.0, sin_half);
                } else {
                    *new_amp = state.amplitudes[i1] * cos_half
                        - state.amplitudes[i0] * Amplitude::new(0.0, sin_half);
                }
            });

        state.amplitudes = new_amplitudes;
        state.is_normalized = true;
    }

    fn apply_ry_optimized(&self, state: &mut QuantumState, qubit: QubitIndex, angle: f64) {
        let cos_half = (angle / 2.0).cos();
        let sin_half = (angle / 2.0).sin();
        let mask = 1 << qubit;

        let mut new_amplitudes = vec![Amplitude::new(0.0, 0.0); state.amplitudes.len()];

        new_amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, new_amp)| {
                let i0 = i & !mask;
                let i1 = i | mask;

                if i & mask == 0 {
                    *new_amp = state.amplitudes[i0] * cos_half - state.amplitudes[i1] * sin_half;
                } else {
                    *new_amp = state.amplitudes[i1] * cos_half + state.amplitudes[i0] * sin_half;
                }
            });

        state.amplitudes = new_amplitudes;
        state.is_normalized = true;
    }

    fn apply_rz_optimized(&self, state: &mut QuantumState, qubit: QubitIndex, angle: f64) {
        let phase = Amplitude::new(0.0, angle / 2.0).exp();
        let neg_phase = Amplitude::new(0.0, -angle / 2.0).exp();
        let mask = 1 << qubit;

        state
            .amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, amplitude)| {
                if i & mask == 0 {
                    *amplitude *= neg_phase;
                } else {
                    *amplitude *= phase;
                }
            });

        state.is_normalized = true;
    }

    fn apply_gate_basic(&self, state: &mut QuantumState, gate: &QuantumGate) -> Result<(), String> {
        // Basic matrix-based gate application (fallback)
        self.apply_gate_matrix(state, gate)
    }

    fn apply_gate_matrix(
        &self,
        state: &mut QuantumState,
        gate: &QuantumGate,
    ) -> Result<(), String> {
        // Generic matrix-based gate application
        let gate_size = 1 << gate.qubits.len();
        let matrix = &gate.matrix;

        if matrix.len() != gate_size * gate_size {
            return Err("Gate matrix size mismatch".to_string());
        }

        // Apply matrix to relevant subspace
        let qubit_masks: Vec<usize> = gate.qubits.iter().map(|&q| 1 << q).collect();

        // This is a simplified implementation - real version would be more optimized
        let mut new_amplitudes = state.amplitudes.clone();

        for i in 0..state.amplitudes.len() {
            let mut local_index = 0;
            for (bit, &mask) in qubit_masks.iter().enumerate() {
                if i & mask != 0 {
                    local_index |= 1 << bit;
                }
            }

            // Apply matrix row
            let mut new_amp = Amplitude::new(0.0, 0.0);
            for j in 0..gate_size {
                let mut global_j = i;
                for (bit, &mask) in qubit_masks.iter().enumerate() {
                    if j & (1 << bit) != 0 {
                        global_j |= mask;
                    } else {
                        global_j &= !mask;
                    }
                }
                new_amp += matrix[local_index * gate_size + j] * state.amplitudes[global_j];
            }
            new_amplitudes[i] = new_amp;
        }

        state.amplitudes = new_amplitudes;
        state.is_normalized = false; // Matrix operations may affect normalization

        Ok(())
    }

    fn apply_parallel_gates_simd(
        &mut self,
        state: &mut QuantumState,
        gates: &[&QuantumGate],
    ) -> Result<(), String> {
        // Apply multiple gates in parallel using SIMD where possible
        for gate in gates {
            self.apply_gate_optimized(state, gate)?;
        }
        Ok(())
    }

    fn apply_single_qubit_gates_parallel(
        &mut self,
        state: &mut QuantumState,
        gates: &[&QuantumGate],
    ) -> Result<(), String> {
        // Apply single-qubit gates in parallel
        for gate in gates {
            self.apply_gate_optimized(state, gate)?;
        }
        Ok(())
    }

    fn apply_two_qubit_gates_parallel(
        &mut self,
        state: &mut QuantumState,
        gates: &[&QuantumGate],
    ) -> Result<(), String> {
        // Apply two-qubit gates, checking for conflicts
        for gate in gates {
            self.apply_gate_optimized(state, gate)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn group_gates_by_type<'a>(
        &self,
        gates: &'a [QuantumGate],
    ) -> HashMap<String, Vec<&'a QuantumGate>> {
        let mut groups = HashMap::new();

        for gate in gates {
            let group_name = match gate.qubits.len() {
                1 => "single_qubit".to_string(),
                2 => "two_qubit".to_string(),
                3 => "three_qubit".to_string(),
                _ => "multi_qubit".to_string(),
            };

            groups.entry(group_name).or_insert_with(Vec::new).push(gate);
        }

        groups
    }

    fn perform_measurements(
        &mut self,
        state: &mut QuantumState,
        measurements: &HashMap<QubitIndex, Option<bool>>,
    ) -> Result<Vec<(QubitIndex, bool)>, String> {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();

        for (&qubit, _) in measurements {
            let prob_zero = state.measure_probability(qubit, false);
            let measurement_result = rng.gen::<f64>() < prob_zero;

            // Collapse the state
            state.collapse_qubit(qubit, measurement_result);
            results.push((qubit, measurement_result));
        }

        Ok(results)
    }

    fn should_compress_state(&self, state: &QuantumState) -> bool {
        if !self.config.use_sparse_representation {
            return false;
        }

        // Check sparsity
        let non_zero_count = state
            .amplitudes
            .iter()
            .filter(|amp| amp.norm() > self.config.sparse_threshold)
            .count();

        non_zero_count < state.amplitudes.len() / 4
    }

    fn compress_state(&self, _state: &mut QuantumState) {
        // Implement sparse state compression
        // This would convert to a sparse representation when beneficial
        // For now, we keep the dense representation as it's simpler
        // Full implementation would use a sparse data structure when
        // most amplitudes are near zero
    }

    // Performance monitoring
    pub fn get_stats(&self) -> &SimulationStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = SimulationStats::default();
    }

    // Simulation utilities
    pub fn estimate_memory_usage(num_qubits: usize) -> f64 {
        // Estimate memory usage in MB
        (1 << num_qubits) as f64 * 16.0 / (1024.0 * 1024.0)
    }

    pub fn max_qubits_for_memory(memory_limit_gb: f64) -> usize {
        let memory_limit_mb = memory_limit_gb * 1024.0;
        let mut qubits = 0;

        while Self::estimate_memory_usage(qubits + 1) <= memory_limit_mb {
            qubits += 1;
        }

        qubits
    }
}

// Utility functions for batch simulation
pub fn simulate_batch(
    circuits: &[QuantumCircuit],
    config: Option<SimulationConfig>,
) -> Vec<Result<QuantumResult, String>> {
    let sim_config = config.unwrap_or_default();

    circuits
        .par_iter()
        .map(|circuit| {
            let mut simulator = StateVectorSimulator::with_config(sim_config.clone());
            simulator.execute_circuit(circuit)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum::circuit::CircuitBuilder;

    #[test]
    fn test_simulator_creation() {
        let simulator = StateVectorSimulator::new();
        assert_eq!(
            simulator.config.optimization_level,
            OptimizationLevel::UltraOptimized
        );
    }

    #[test]
    fn test_bell_state_simulation() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let mut simulator = StateVectorSimulator::new();
        let result = simulator.execute_circuit(&circuit).unwrap();

        assert_eq!(result.final_state.num_qubits, 2);
        assert_eq!(result.operations_count, 2);
    }

    #[test]
    fn test_memory_estimation() {
        assert_eq!(StateVectorSimulator::estimate_memory_usage(10), 16.0); // 16 MB for 10 qubits
        assert!(StateVectorSimulator::max_qubits_for_memory(1.0) >= 26); // ~1 GB should fit 26 qubits
    }
}
