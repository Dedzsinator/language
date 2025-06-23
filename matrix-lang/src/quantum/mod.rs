// Quantum Computing Module for Matrix Language
// Ultra-optimized quantum circuit simulation with GUI support

pub mod algorithms;
pub mod circuit;
pub mod gates;
pub mod gui;
pub mod simulator;
pub mod state;
pub mod visualization;

pub use algorithms::*;
pub use circuit::*;
pub use gates::*;
pub use gui::*;
pub use simulator::*;
pub use state::*;
pub use visualization::*;

// Re-export core types for easy access
pub type QubitIndex = usize;
pub type Amplitude = num_complex::Complex<f64>;
pub type StateVector = Vec<Amplitude>;

#[derive(Debug, Clone)]
pub struct QuantumResult {
    pub final_state: QuantumState,
    pub measurements: Vec<(QubitIndex, bool)>,
    pub execution_time: std::time::Duration,
    pub operations_count: usize,
}

// Main quantum computing interface for Matrix Language
pub struct QuantumEngine {
    pub simulator: StateVectorSimulator,
    pub circuits: Vec<QuantumCircuit>,
    pub active_circuit: Option<usize>,
}

impl QuantumEngine {
    pub fn new() -> Self {
        Self {
            simulator: StateVectorSimulator::new(),
            circuits: Vec::new(),
            active_circuit: None,
        }
    }

    pub fn create_circuit(&mut self, num_qubits: usize) -> usize {
        let circuit = QuantumCircuit::new(num_qubits);
        self.circuits.push(circuit);
        let circuit_id = self.circuits.len() - 1;
        self.active_circuit = Some(circuit_id);
        circuit_id
    }

    pub fn run_circuit(&mut self, circuit_id: usize) -> Result<QuantumResult, String> {
        if let Some(circuit) = self.circuits.get(circuit_id) {
            self.simulator.execute_circuit(circuit)
        } else {
            Err(format!("Circuit {} not found", circuit_id))
        }
    }
}
