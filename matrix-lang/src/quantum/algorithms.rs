// Pre-built quantum algorithms for the Matrix Language
// Includes Shor's, Grover's, Bernstein-Vazirani, and more

use crate::quantum::circuit::QuantumCircuit;
use std::f64::consts::PI;

pub trait QuantumAlgorithm {
    fn build_circuit(&self) -> QuantumCircuit;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_qubits(&self) -> usize;
}

// Bernstein-Vazirani Algorithm
#[derive(Debug, Clone)]
pub struct BernsteinVazirani {
    pub secret_string: Vec<bool>,
}

impl BernsteinVazirani {
    pub fn new(secret_string: Vec<bool>) -> Self {
        Self { secret_string }
    }
}

impl QuantumAlgorithm for BernsteinVazirani {
    fn build_circuit(&self) -> QuantumCircuit {
        let n = self.secret_string.len();
        let mut circuit = QuantumCircuit::new(n + 1); // n qubits + 1 ancilla

        // Initialize ancilla in |1⟩ state
        circuit.x(n).unwrap();

        // Apply Hadamard gates to all qubits
        for i in 0..=n {
            circuit.h(i).unwrap();
        }

        // Oracle: apply CNOT for each bit in secret string
        for (i, &bit) in self.secret_string.iter().enumerate() {
            if bit {
                circuit.cnot(i, n).unwrap();
            }
        }

        // Apply Hadamard gates to first n qubits
        for i in 0..n {
            circuit.h(i).unwrap();
        }

        // Measure all qubits except ancilla
        for i in 0..n {
            circuit.measure(i);
        }

        circuit.name = format!("Bernstein-Vazirani_{}",
            self.secret_string.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>());

        circuit
    }

    fn name(&self) -> &str { "Bernstein-Vazirani" }
    fn description(&self) -> &str { "Determines a secret bit string using quantum oracle" }
    fn required_qubits(&self) -> usize { self.secret_string.len() + 1 }
}

// Grover's Search Algorithm
#[derive(Debug, Clone)]
pub struct GroverSearch {
    pub num_qubits: usize,
    pub marked_items: Vec<usize>,
}

impl GroverSearch {
    pub fn new(num_qubits: usize, marked_items: Vec<usize>) -> Self {
        Self { num_qubits, marked_items }
    }

    fn optimal_iterations(&self) -> usize {
        let n = self.num_qubits;
        let m = self.marked_items.len();
        if m == 0 { return 0; }

        let total_items = 1 << n;
        let theta = (m as f64 / total_items as f64).sqrt().asin();
        ((PI / 4.0) / theta - 0.5).round() as usize
    }
}

impl QuantumAlgorithm for GroverSearch {
    fn build_circuit(&self) -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new(self.num_qubits);

        // Initialize superposition
        for i in 0..self.num_qubits {
            circuit.h(i).unwrap();
        }

        let iterations = self.optimal_iterations();

        for _ in 0..iterations {
            // Oracle: mark target items
            self.apply_oracle(&mut circuit);

            // Diffusion operator (amplitude amplification)
            self.apply_diffusion(&mut circuit);
        }

        // Measure all qubits
        circuit.measure_all();

        circuit.name = format!("Grover_{}_qubits", self.num_qubits);
        circuit
    }

    fn name(&self) -> &str { "Grover Search" }
    fn description(&self) -> &str { "Searches unsorted database quadratically faster than classical" }
    fn required_qubits(&self) -> usize { self.num_qubits }
}

impl GroverSearch {
    fn apply_oracle(&self, circuit: &mut QuantumCircuit) {
        // Mark each target item by flipping its phase
        for &item in &self.marked_items {
            self.mark_item(circuit, item);
        }
    }

    fn mark_item(&self, circuit: &mut QuantumCircuit, item: usize) {
        // Convert item index to binary and apply multi-controlled Z gate
        let binary_rep = format!("{:0width$b}", item, width = self.num_qubits);

        // Apply X gates for 0 bits (to create all-1 control)
        for (i, bit_char) in binary_rep.chars().enumerate() {
            if bit_char == '0' {
                circuit.x(i).unwrap();
            }
        }

        // Apply multi-controlled Z gate
        self.apply_multi_controlled_z(circuit);

        // Undo X gates
        for (i, bit_char) in binary_rep.chars().enumerate() {
            if bit_char == '0' {
                circuit.x(i).unwrap();
            }
        }
    }

    fn apply_multi_controlled_z(&self, circuit: &mut QuantumCircuit) {
        // Implement multi-controlled Z using Toffoli decomposition
        if self.num_qubits == 1 {
            circuit.z(0).unwrap();
        } else if self.num_qubits == 2 {
            circuit.cz(0, 1).unwrap();
        } else {
            // For more qubits, use auxiliary qubits or decomposition
            // This is a simplified version - real implementation would be more efficient
            for i in 0..self.num_qubits {
                circuit.z(i).unwrap();
            }
        }
    }

    fn apply_diffusion(&self, circuit: &mut QuantumCircuit) {
        // Diffusion operator: 2|s⟩⟨s| - I where |s⟩ is uniform superposition

        // Apply H gates
        for i in 0..self.num_qubits {
            circuit.h(i).unwrap();
        }

        // Apply X gates
        for i in 0..self.num_qubits {
            circuit.x(i).unwrap();
        }

        // Apply multi-controlled Z
        self.apply_multi_controlled_z(circuit);

        // Undo X gates
        for i in 0..self.num_qubits {
            circuit.x(i).unwrap();
        }

        // Undo H gates
        for i in 0..self.num_qubits {
            circuit.h(i).unwrap();
        }
    }
}

// Quantum Fourier Transform
#[derive(Debug, Clone)]
pub struct QuantumFourierTransform {
    pub num_qubits: usize,
    pub inverse: bool,
}

impl QuantumFourierTransform {
    pub fn new(num_qubits: usize) -> Self {
        Self { num_qubits, inverse: false }
    }

    pub fn inverse(num_qubits: usize) -> Self {
        Self { num_qubits, inverse: true }
    }
}

impl QuantumAlgorithm for QuantumFourierTransform {
    fn build_circuit(&self) -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new(self.num_qubits);

        if self.inverse {
            self.build_inverse_qft(&mut circuit);
        } else {
            self.build_qft(&mut circuit);
        }

        circuit.name = format!("QFT_{}{}",
            if self.inverse { "inv_" } else { "" },
            self.num_qubits);
        circuit
    }

    fn name(&self) -> &str {
        if self.inverse { "Inverse QFT" } else { "QFT" }
    }

    fn description(&self) -> &str {
        "Quantum Fourier Transform - quantum analog of classical DFT"
    }

    fn required_qubits(&self) -> usize { self.num_qubits }
}

impl QuantumFourierTransform {
    fn build_qft(&self, circuit: &mut QuantumCircuit) {
        for i in 0..self.num_qubits {
            circuit.h(i).unwrap();

            for j in (i + 1)..self.num_qubits {
                let angle = PI / (1 << (j - i)) as f64;
                self.controlled_phase(circuit, j, i, angle);
            }
        }

        // Swap qubits to reverse order
        for i in 0..(self.num_qubits / 2) {
            circuit.swap(i, self.num_qubits - 1 - i).unwrap();
        }
    }

    fn build_inverse_qft(&self, circuit: &mut QuantumCircuit) {
        // Swap qubits first (reverse of QFT)
        for i in 0..(self.num_qubits / 2) {
            circuit.swap(i, self.num_qubits - 1 - i).unwrap();
        }

        for i in (0..self.num_qubits).rev() {
            for j in ((i + 1)..self.num_qubits).rev() {
                let angle = -PI / (1 << (j - i)) as f64;
                self.controlled_phase(circuit, j, i, angle);
            }

            circuit.h(i).unwrap();
        }
    }

    fn controlled_phase(&self, circuit: &mut QuantumCircuit, control: usize, target: usize, angle: f64) {
        // Implement controlled phase gate using RZ and CNOT decomposition
        circuit.rz(target, angle / 2.0).unwrap();
        circuit.cnot(control, target).unwrap();
        circuit.rz(target, -angle / 2.0).unwrap();
        circuit.cnot(control, target).unwrap();
        circuit.rz(control, angle / 2.0).unwrap();
    }
}

// Quantum Phase Estimation
#[derive(Debug, Clone)]
pub struct QuantumPhaseEstimation {
    pub precision_qubits: usize,
    pub eigenstate_qubits: usize,
}

impl QuantumPhaseEstimation {
    pub fn new(precision_qubits: usize, eigenstate_qubits: usize) -> Self {
        Self { precision_qubits, eigenstate_qubits }
    }
}

impl QuantumAlgorithm for QuantumPhaseEstimation {
    fn build_circuit(&self) -> QuantumCircuit {
        let total_qubits = self.precision_qubits + self.eigenstate_qubits;
        let mut circuit = QuantumCircuit::new(total_qubits);

        // Initialize precision qubits in superposition
        for i in 0..self.precision_qubits {
            circuit.h(i).unwrap();
        }

        // Apply controlled unitary operations
        for i in 0..self.precision_qubits {
            let power = 1 << i;
            for _ in 0..power {
                // This would apply controlled-U operations
                // For demonstration, we'll use controlled-Z
                for j in self.precision_qubits..total_qubits {
                    circuit.cz(i, j).unwrap();
                }
            }
        }

        // Apply inverse QFT to precision qubits
        let qft = QuantumFourierTransform::inverse(self.precision_qubits);
        let qft_circuit = qft.build_circuit();
        circuit.compose(&qft_circuit).unwrap();

        // Measure precision qubits
        for i in 0..self.precision_qubits {
            circuit.measure(i);
        }

        circuit.name = format!("QPE_{}_{}", self.precision_qubits, self.eigenstate_qubits);
        circuit
    }

    fn name(&self) -> &str { "Quantum Phase Estimation" }
    fn description(&self) -> &str { "Estimates eigenvalue phases of unitary operators" }
    fn required_qubits(&self) -> usize { self.precision_qubits + self.eigenstate_qubits }
}

// Shor's Algorithm (simplified factoring)
#[derive(Debug, Clone)]
pub struct ShorAlgorithm {
    pub number_to_factor: u64,
    pub register_size: usize,
}

impl ShorAlgorithm {
    pub fn new(number_to_factor: u64) -> Self {
        let register_size = (number_to_factor as f64).log2().ceil() as usize + 1;
        Self { number_to_factor, register_size }
    }
}

impl QuantumAlgorithm for ShorAlgorithm {
    fn build_circuit(&self) -> QuantumCircuit {
        let counting_qubits = 2 * self.register_size;
        let total_qubits = counting_qubits + self.register_size;
        let mut circuit = QuantumCircuit::new(total_qubits);

        // Initialize counting register in superposition
        for i in 0..counting_qubits {
            circuit.h(i).unwrap();
        }

        // Initialize target register to |1⟩
        circuit.x(counting_qubits).unwrap();

        // Apply controlled modular exponentiation
        // This is greatly simplified - real Shor's would implement modular arithmetic
        for i in 0..counting_qubits {
            for j in counting_qubits..total_qubits {
                circuit.cnot(i, j).unwrap();
            }
        }

        // Apply inverse QFT to counting register
        let qft = QuantumFourierTransform::inverse(counting_qubits);
        let qft_circuit = qft.build_circuit();
        circuit.compose(&qft_circuit).unwrap();

        // Measure counting register
        for i in 0..counting_qubits {
            circuit.measure(i);
        }

        circuit.name = format!("Shor_{}", self.number_to_factor);
        circuit
    }

    fn name(&self) -> &str { "Shor's Algorithm" }
    fn description(&self) -> &str { "Factorizes integers exponentially faster than classical algorithms" }
    fn required_qubits(&self) -> usize { 3 * self.register_size }
}

// Deutsch-Jozsa Algorithm
#[derive(Debug, Clone)]
pub struct DeutschJozsa {
    pub num_qubits: usize,
    pub oracle_type: OracleType,
}

#[derive(Debug, Clone)]
pub enum OracleType {
    Constant(bool),
    Balanced(Vec<bool>),
}

impl DeutschJozsa {
    pub fn constant(num_qubits: usize, value: bool) -> Self {
        Self {
            num_qubits,
            oracle_type: OracleType::Constant(value),
        }
    }

    pub fn balanced(oracle_pattern: Vec<bool>) -> Self {
        Self {
            num_qubits: oracle_pattern.len(),
            oracle_type: OracleType::Balanced(oracle_pattern),
        }
    }
}

impl QuantumAlgorithm for DeutschJozsa {
    fn build_circuit(&self) -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new(self.num_qubits + 1);

        // Initialize ancilla in |1⟩
        circuit.x(self.num_qubits).unwrap();

        // Apply Hadamard to all qubits
        for i in 0..=self.num_qubits {
            circuit.h(i).unwrap();
        }

        // Apply oracle
        match &self.oracle_type {
            OracleType::Constant(value) => {
                if *value {
                    circuit.x(self.num_qubits).unwrap();
                }
            }
            OracleType::Balanced(pattern) => {
                for (i, &bit) in pattern.iter().enumerate() {
                    if bit {
                        circuit.cnot(i, self.num_qubits).unwrap();
                    }
                }
            }
        }

        // Apply Hadamard to input qubits
        for i in 0..self.num_qubits {
            circuit.h(i).unwrap();
        }

        // Measure input qubits
        for i in 0..self.num_qubits {
            circuit.measure(i);
        }

        circuit.name = "Deutsch-Jozsa".to_string();
        circuit
    }

    fn name(&self) -> &str { "Deutsch-Jozsa" }
    fn description(&self) -> &str { "Determines if function is constant or balanced" }
    fn required_qubits(&self) -> usize { self.num_qubits + 1 }
}

// Algorithm factory and utilities
pub struct AlgorithmLibrary;

impl AlgorithmLibrary {
    pub fn bernstein_vazirani(secret: &str) -> Box<dyn QuantumAlgorithm> {
        let secret_bits: Vec<bool> = secret.chars()
            .map(|c| c == '1')
            .collect();
        Box::new(BernsteinVazirani::new(secret_bits))
    }

    pub fn grover_search(num_qubits: usize, targets: Vec<usize>) -> Box<dyn QuantumAlgorithm> {
        Box::new(GroverSearch::new(num_qubits, targets))
    }

    pub fn qft(num_qubits: usize, inverse: bool) -> Box<dyn QuantumAlgorithm> {
        if inverse {
            Box::new(QuantumFourierTransform::inverse(num_qubits))
        } else {
            Box::new(QuantumFourierTransform::new(num_qubits))
        }
    }

    pub fn phase_estimation(precision: usize, eigenstate: usize) -> Box<dyn QuantumAlgorithm> {
        Box::new(QuantumPhaseEstimation::new(precision, eigenstate))
    }

    pub fn shor_factoring(number: u64) -> Box<dyn QuantumAlgorithm> {
        Box::new(ShorAlgorithm::new(number))
    }

    pub fn deutsch_jozsa_constant(num_qubits: usize, value: bool) -> Box<dyn QuantumAlgorithm> {
        Box::new(DeutschJozsa::constant(num_qubits, value))
    }

    pub fn deutsch_jozsa_balanced(pattern: Vec<bool>) -> Box<dyn QuantumAlgorithm> {
        Box::new(DeutschJozsa::balanced(pattern))
    }

    pub fn list_algorithms() -> Vec<&'static str> {
        vec![
            "Bernstein-Vazirani",
            "Grover Search",
            "Quantum Fourier Transform",
            "Quantum Phase Estimation",
            "Shor's Algorithm",
            "Deutsch-Jozsa",
        ]
    }
}

// Quantum supremacy and demonstration circuits
pub struct QuantumSupremacyCircuits;

impl QuantumSupremacyCircuits {
    pub fn random_circuit(num_qubits: usize, depth: usize, seed: u64) -> QuantumCircuit {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed);
        let mut circuit = QuantumCircuit::new(num_qubits);

        for _ in 0..depth {
            match rng.gen_range(0..3) {
                0 => {
                    // Single qubit gate
                    let qubit = rng.gen_range(0..num_qubits);
                    match rng.gen_range(0..3) {
                        0 => circuit.h(qubit).unwrap(),
                        1 => circuit.x(qubit).unwrap(),
                        _ => circuit.z(qubit).unwrap(),
                    }
                }
                1 => {
                    // Two qubit gate
                    let q1 = rng.gen_range(0..num_qubits);
                    let q2 = rng.gen_range(0..num_qubits);
                    if q1 != q2 {
                        circuit.cnot(q1, q2).unwrap();
                    }
                }
                _ => {
                    // Rotation gate
                    let qubit = rng.gen_range(0..num_qubits);
                    let angle = rng.gen::<f64>() * 2.0 * PI;
                    circuit.ry(qubit, angle).unwrap();
                }
            }
        }

        circuit.name = format!("Random_{}q_{}d", num_qubits, depth);
        circuit
    }

    pub fn google_supremacy_inspired(num_qubits: usize) -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new(num_qubits);

        // Pattern inspired by Google's supremacy experiment
        for layer in 0..20 {
            // Layer of single-qubit gates
            for i in 0..num_qubits {
                if layer % 2 == 0 {
                    circuit.h(i).unwrap();
                } else {
                    circuit.ry(i, PI / 3.0).unwrap();
                }
            }

            // Layer of two-qubit gates
            for i in (0..num_qubits - 1).step_by(2) {
                circuit.cnot(i, i + 1).unwrap();
            }
        }

        circuit.measure_all();
        circuit.name = format!("Google_Supremacy_{}q", num_qubits);
        circuit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bernstein_vazirani() {
        let secret = vec![true, false, true, true];
        let bv = BernsteinVazirani::new(secret);
        let circuit = bv.build_circuit();

        assert_eq!(circuit.num_qubits, 5); // 4 + 1 ancilla
        assert!(circuit.gate_count() > 0);
    }

    #[test]
    fn test_grover_search() {
        let grover = GroverSearch::new(3, vec![5, 7]);
        let circuit = grover.build_circuit();

        assert_eq!(circuit.num_qubits, 3);
        assert!(circuit.gate_count() > 0);
    }

    #[test]
    fn test_qft() {
        let qft = QuantumFourierTransform::new(4);
        let circuit = qft.build_circuit();

        assert_eq!(circuit.num_qubits, 4);
        assert!(circuit.gate_count() > 0);
    }

    #[test]
    fn test_algorithm_library() {
        let algorithms = AlgorithmLibrary::list_algorithms();
        assert_eq!(algorithms.len(), 6);

        let bv = AlgorithmLibrary::bernstein_vazirani("1011");
        assert_eq!(bv.required_qubits(), 5);
    }
}
