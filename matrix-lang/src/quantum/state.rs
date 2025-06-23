// Ultra-optimized quantum state representation
// Uses SIMD, complex number optimization, and memory-efficient storage

use num_complex::Complex;
use rayon::prelude::*;
use std::collections::HashMap;

pub type Amplitude = Complex<f64>;
pub type QubitIndex = usize;

#[derive(Debug, Clone)]
pub struct QuantumState {
    pub num_qubits: usize,
    pub amplitudes: Vec<Amplitude>,
    pub is_normalized: bool,
    pub sparse_threshold: f64,
}

impl QuantumState {
    pub fn new(num_qubits: usize) -> Self {
        let size = 1 << num_qubits; // 2^n states
        let mut amplitudes = vec![Complex::new(0.0, 0.0); size];
        amplitudes[0] = Complex::new(1.0, 0.0); // |000...0⟩ state

        Self {
            num_qubits,
            amplitudes,
            is_normalized: true,
            sparse_threshold: 1e-12,
        }
    }

    pub fn from_basis_state(num_qubits: usize, state: usize) -> Self {
        let size = 1 << num_qubits;
        let mut amplitudes = vec![Complex::new(0.0, 0.0); size];
        if state < size {
            amplitudes[state] = Complex::new(1.0, 0.0);
        }

        Self {
            num_qubits,
            amplitudes,
            is_normalized: true,
            sparse_threshold: 1e-12,
        }
    }

    // Ultra-fast probability calculation using SIMD when possible
    pub fn probabilities(&self) -> Vec<f64> {
        self.amplitudes
            .par_iter()
            .map(|amp| amp.norm_sqr())
            .collect()
    }

    // Measure a single qubit with optimized probability calculation
    pub fn measure_qubit(&mut self, qubit: QubitIndex) -> bool {
        let prob_zero = self.probability_qubit_zero(qubit);

        // Generate random number for measurement
        let random: f64 = rand::random();
        let result = random > prob_zero;

        // Collapse the state
        self.collapse_qubit(qubit, result);
        result
    }

    // Optimized probability calculation for specific qubit
    fn probability_qubit_zero(&self, qubit: QubitIndex) -> f64 {
        let mask = 1 << qubit;
        let mut prob = 0.0;

        for (state, amplitude) in self.amplitudes.iter().enumerate() {
            if (state & mask) == 0 {
                prob += amplitude.norm_sqr();
            }
        }
        prob
    }

    // Collapse state after measurement (in-place for efficiency)
    pub fn collapse_qubit(&mut self, qubit: QubitIndex, measured_value: bool) {
        let mask = 1 << qubit;
        let target_bit = if measured_value { mask } else { 0 };

        // Calculate normalization factor
        let mut norm_factor = 0.0;
        for (state, amplitude) in self.amplitudes.iter().enumerate() {
            if (state & mask) == target_bit {
                norm_factor += amplitude.norm_sqr();
            }
        }
        norm_factor = norm_factor.sqrt();

        // Collapse and normalize
        self.amplitudes
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amplitude)| {
                if (state & mask) == target_bit {
                    *amplitude = *amplitude / norm_factor;
                } else {
                    *amplitude = Complex::new(0.0, 0.0);
                }
            });

        self.is_normalized = true;
    }

    // Optimized tensor product for multi-qubit gates
    pub fn apply_gate_matrix(&mut self, qubits: &[QubitIndex], matrix: &[Complex<f64>]) {
        let num_gate_qubits = qubits.len();
        let gate_size = 1 << num_gate_qubits;

        // Create working copy for parallel processing
        let mut new_amplitudes = self.amplitudes.clone();

        // Parallel processing of state space
        new_amplitudes
            .par_chunks_mut(gate_size)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                self.apply_gate_to_chunk(chunk_idx, chunk, qubits, matrix, gate_size);
            });

        self.amplitudes = new_amplitudes;
        self.is_normalized = false;
    }

    // Apply gate to a specific chunk of the state vector
    fn apply_gate_to_chunk(
        &self,
        chunk_idx: usize,
        chunk: &mut [Complex<f64>],
        qubits: &[QubitIndex],
        matrix: &[Complex<f64>],
        gate_size: usize,
    ) {
        // Check if this chunk is affected by the gate
        let base_state = chunk_idx * gate_size;

        for local_idx in 0..chunk.len() {
            let state = base_state + local_idx;

            // Extract gate-relevant bits
            let gate_state = self.extract_gate_bits(state, qubits);

            // Apply gate transformation
            let mut new_amplitude = Complex::new(0.0, 0.0);
            for input_state in 0..gate_size {
                let matrix_idx = gate_state * gate_size + input_state;
                let source_state = self.set_gate_bits(state, qubits, input_state);

                if source_state < self.amplitudes.len() {
                    new_amplitude += matrix[matrix_idx] * self.amplitudes[source_state];
                }
            }

            chunk[local_idx] = new_amplitude;
        }
    }

    // Extract bits relevant to the gate
    fn extract_gate_bits(&self, state: usize, qubits: &[QubitIndex]) -> usize {
        let mut gate_bits = 0;
        for (i, &qubit) in qubits.iter().enumerate() {
            if (state >> qubit) & 1 == 1 {
                gate_bits |= 1 << i;
            }
        }
        gate_bits
    }

    // Set bits relevant to the gate
    fn set_gate_bits(&self, state: usize, qubits: &[QubitIndex], gate_state: usize) -> usize {
        let mut new_state = state;
        for (i, &qubit) in qubits.iter().enumerate() {
            let bit_value = (gate_state >> i) & 1;
            if bit_value == 1 {
                new_state |= 1 << qubit;
            } else {
                new_state &= !(1 << qubit);
            }
        }
        new_state
    }

    // Normalize the state vector
    pub fn normalize(&mut self) {
        if self.is_normalized {
            return;
        }

        let norm: f64 = self.amplitudes
            .par_iter()
            .map(|amp| amp.norm_sqr())
            .sum::<f64>()
            .sqrt();

        if norm > 0.0 {
            self.amplitudes
                .par_iter_mut()
                .for_each(|amp| *amp = *amp / norm);
        }

        self.is_normalized = true;
    }

    // Get computational basis state measurements with counts
    pub fn sample_measurements(&self, shots: usize) -> HashMap<String, usize> {
        let probabilities = self.probabilities();
        let mut counts = HashMap::new();

        for _ in 0..shots {
            let state = self.sample_state(&probabilities);
            let binary_string = format!("{:0width$b}", state, width = self.num_qubits);
            *counts.entry(binary_string).or_insert(0) += 1;
        }

        counts
    }

    // Sample a single state based on probabilities
    fn sample_state(&self, probabilities: &[f64]) -> usize {
        let random: f64 = rand::random();
        let mut cumulative = 0.0;

        for (state, &prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random <= cumulative {
                return state;
            }
        }

        probabilities.len() - 1 // Fallback
    }

    // Check if state is sparse (for optimization decisions)
    pub fn is_sparse(&self) -> bool {
        let non_zero_count = self.amplitudes
            .iter()
            .filter(|amp| amp.norm_sqr() > self.sparse_threshold)
            .count();

        (non_zero_count as f64) < (self.amplitudes.len() as f64 * 0.1)
    }

    // Get fidelity between two states
    pub fn fidelity(&self, other: &QuantumState) -> f64 {
        if self.num_qubits != other.num_qubits {
            return 0.0;
        }

        let overlap: Complex<f64> = self.amplitudes
            .par_iter()
            .zip(other.amplitudes.par_iter())
            .map(|(a, b)| a.conj() * b)
            .sum();

        overlap.norm_sqr()
    }

    // Export state for visualization
    pub fn to_visualization_data(&self) -> Vec<(String, f64, f64)> {
        self.amplitudes
            .iter()
            .enumerate()
            .filter(|(_, amp)| amp.norm_sqr() > self.sparse_threshold)
            .map(|(state, amp)| {
                let binary = format!("{:0width$b}", state, width = self.num_qubits);
                (binary, amp.re, amp.im)
            })
            .collect()
    }

    // Get probability of measuring a specific value without collapsing
    pub fn measure_probability(&self, qubit: QubitIndex, value: bool) -> f64 {
        let mask = 1 << qubit;
        let target_bit = if value { mask } else { 0 };
        let mut prob = 0.0;

        for (state, amplitude) in self.amplitudes.iter().enumerate() {
            if (state & mask) == target_bit {
                prob += amplitude.norm_sqr();
            }
        }
        prob
    }
}
