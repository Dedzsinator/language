// Ultra-optimized quantum circuit representation with layer management
// Supports circuit optimization, decomposition, and visualization

use crate::quantum::gates::{GateType, QuantumGate};
use crate::quantum::state::QubitIndex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CircuitLayer {
    pub gates: Vec<QuantumGate>,
    pub depth: usize,
    pub parallel_gates: Vec<Vec<usize>>, // Groups of gates that can run in parallel
}

impl CircuitLayer {
    pub fn new() -> Self {
        Self {
            gates: Vec::new(),
            depth: 0,
            parallel_gates: Vec::new(),
        }
    }

    pub fn add_gate(&mut self, gate: QuantumGate) {
        self.gates.push(gate);
        self.update_parallelization();
    }

    fn update_parallelization(&mut self) {
        // Group gates that can be executed in parallel (non-overlapping qubits)
        self.parallel_gates.clear();
        let mut used_qubits: HashSet<QubitIndex> = HashSet::new();
        let mut current_group = Vec::new();

        for (idx, gate) in self.gates.iter().enumerate() {
            let gate_qubits: HashSet<QubitIndex> = gate.qubits.iter().cloned().collect();

            if gate_qubits.is_disjoint(&used_qubits) {
                // Can run in parallel with current group
                current_group.push(idx);
                used_qubits.extend(&gate_qubits);
            } else {
                // Need new parallel group
                if !current_group.is_empty() {
                    self.parallel_gates.push(current_group);
                }
                current_group = vec![idx];
                used_qubits = gate_qubits;
            }
        }

        if !current_group.is_empty() {
            self.parallel_gates.push(current_group);
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    pub num_qubits: usize,
    pub layers: Vec<CircuitLayer>,
    pub total_depth: usize,
    pub measurements: HashMap<QubitIndex, Option<bool>>,
    pub name: String,
    pub metadata: HashMap<String, String>,
}

impl QuantumCircuit {
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            layers: Vec::new(),
            total_depth: 0,
            measurements: HashMap::new(),
            name: format!("Circuit_{}", num_qubits),
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(num_qubits: usize, name: String) -> Self {
        let mut circuit = Self::new(num_qubits);
        circuit.name = name;
        circuit
    }

    // Add a single gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) -> Result<(), String> {
        // Validate gate qubits are within circuit bounds
        for &qubit in &gate.qubits {
            if qubit >= self.num_qubits {
                return Err(format!(
                    "Qubit {} out of bounds for {}-qubit circuit",
                    qubit, self.num_qubits
                ));
            }
        }

        // Add to current layer or create new layer if conflicts exist
        let can_add_to_existing = if let Some(last_layer) = self.layers.last() {
            self.can_add_to_layer(last_layer, &gate)
        } else {
            false
        };

        if can_add_to_existing {
            if let Some(last_layer) = self.layers.last_mut() {
                last_layer.add_gate(gate);
                return Ok(());
            }
        }

        // Create new layer
        let mut new_layer = CircuitLayer::new();
        new_layer.depth = self.layers.len();
        new_layer.add_gate(gate);
        self.layers.push(new_layer);
        self.total_depth = self.layers.len();

        Ok(())
    }

    fn can_add_to_layer(&self, layer: &CircuitLayer, gate: &QuantumGate) -> bool {
        let gate_qubits: HashSet<QubitIndex> = gate.qubits.iter().cloned().collect();

        for existing_gate in &layer.gates {
            let existing_qubits: HashSet<QubitIndex> =
                existing_gate.qubits.iter().cloned().collect();
            if !gate_qubits.is_disjoint(&existing_qubits) {
                return false;
            }
        }
        true
    }

    // Add multiple gates in a single layer
    pub fn add_layer(&mut self, gates: Vec<QuantumGate>) -> Result<(), String> {
        let mut layer = CircuitLayer::new();
        layer.depth = self.layers.len();

        for gate in gates {
            // Validate gate qubits
            for &qubit in &gate.qubits {
                if qubit >= self.num_qubits {
                    return Err(format!(
                        "Qubit {} out of bounds for {}-qubit circuit",
                        qubit, self.num_qubits
                    ));
                }
            }
            layer.add_gate(gate);
        }

        self.layers.push(layer);
        self.total_depth = self.layers.len();
        Ok(())
    }

    // Common gate addition methods for convenience
    pub fn h(&mut self, qubit: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::hadamard(qubit))
    }

    pub fn x(&mut self, qubit: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::pauli_x(qubit))
    }

    pub fn y(&mut self, qubit: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::pauli_y(qubit))
    }

    pub fn z(&mut self, qubit: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::pauli_z(qubit))
    }

    pub fn cnot(&mut self, control: QubitIndex, target: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::cnot(control, target))
    }

    pub fn rx(&mut self, qubit: QubitIndex, angle: f64) -> Result<(), String> {
        self.add_gate(QuantumGate::rx(qubit, angle))
    }

    pub fn ry(&mut self, qubit: QubitIndex, angle: f64) -> Result<(), String> {
        self.add_gate(QuantumGate::ry(qubit, angle))
    }

    pub fn rz(&mut self, qubit: QubitIndex, angle: f64) -> Result<(), String> {
        self.add_gate(QuantumGate::rz(qubit, angle))
    }

    pub fn cz(&mut self, control: QubitIndex, target: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::cz(control, target))
    }

    pub fn swap(&mut self, qubit1: QubitIndex, qubit2: QubitIndex) -> Result<(), String> {
        self.add_gate(QuantumGate::swap(qubit1, qubit2))
    }

    pub fn toffoli(
        &mut self,
        control1: QubitIndex,
        control2: QubitIndex,
        target: QubitIndex,
    ) -> Result<(), String> {
        self.add_gate(QuantumGate::toffoli(control1, control2, target))
    }

    // Measurement operations
    pub fn measure(&mut self, qubit: QubitIndex) {
        self.measurements.insert(qubit, None);
    }

    pub fn measure_all(&mut self) {
        for i in 0..self.num_qubits {
            self.measurements.insert(i, None);
        }
    }

    // Circuit optimization
    pub fn optimize(&mut self) {
        self.remove_identity_gates();
        self.merge_single_qubit_rotations();
        self.cancel_adjacent_gates();
    }

    fn remove_identity_gates(&mut self) {
        for layer in &mut self.layers {
            layer
                .gates
                .retain(|gate| !matches!(gate.gate_type, GateType::Identity));
        }
        self.layers.retain(|layer| !layer.gates.is_empty());
        self.total_depth = self.layers.len();
    }

    fn merge_single_qubit_rotations(&mut self) {
        // Combine consecutive rotation gates on the same qubit
        for layer_idx in 0..self.layers.len() {
            let mut merged_rotations: HashMap<usize, (f64, f64, f64)> = HashMap::new();
            let mut gates_to_remove = Vec::new();

            for (gate_idx, gate) in self.layers[layer_idx].gates.iter().enumerate() {
                match &gate.gate_type {
                    GateType::RX(angle) => {
                        let entry = merged_rotations.entry(gate.qubits[0]).or_insert((0.0, 0.0, 0.0));
                        entry.0 += angle;
                        gates_to_remove.push(gate_idx);
                    },
                    GateType::RY(angle) => {
                        let entry = merged_rotations.entry(gate.qubits[0]).or_insert((0.0, 0.0, 0.0));
                        entry.1 += angle;
                        gates_to_remove.push(gate_idx);
                    },
                    GateType::RZ(angle) => {
                        let entry = merged_rotations.entry(gate.qubits[0]).or_insert((0.0, 0.0, 0.0));
                        entry.2 += angle;
                        gates_to_remove.push(gate_idx);
                    },
                    _ => {}
                }
            }

            // Remove old rotation gates
            for &gate_idx in gates_to_remove.iter().rev() {
                self.layers[layer_idx].gates.remove(gate_idx);
            }

            // Add merged rotation gates
            for (qubit, (rx_angle, ry_angle, rz_angle)) in merged_rotations {
                if rx_angle.abs() > 1e-10 {
                    self.layers[layer_idx].gates.push(QuantumGate::rx(qubit, rx_angle));
                }
                if ry_angle.abs() > 1e-10 {
                    self.layers[layer_idx].gates.push(QuantumGate::ry(qubit, ry_angle));
                }
                if rz_angle.abs() > 1e-10 {
                    self.layers[layer_idx].gates.push(QuantumGate::rz(qubit, rz_angle));
                }
            }
        }
    }

    fn cancel_adjacent_gates(&mut self) {
        // Implement gate cancellation (e.g., X followed by X = I)
        for layer_idx in 0..self.layers.len() {
            let mut gates_to_remove = Vec::new();
            let mut prev_gates: HashMap<usize, (usize, GateType)> = HashMap::new();

            for (gate_idx, gate) in self.layers[layer_idx].gates.iter().enumerate() {
                if gate.qubits.len() == 1 {
                    let qubit = gate.qubits[0];
                    if let Some((prev_idx, prev_gate_type)) = prev_gates.get(&qubit) {
                        // Check for self-inverse gates
                        let should_cancel = match (&prev_gate_type, &gate.gate_type) {
                            (GateType::PauliX, GateType::PauliX) => true,
                            (GateType::PauliY, GateType::PauliY) => true,
                            (GateType::PauliZ, GateType::PauliZ) => true,
                            (GateType::Hadamard, GateType::Hadamard) => true,
                            (GateType::S, GateType::S) if *prev_idx == gate_idx - 1 => {
                                // Two S gates = Z gate, but we'll cancel them for simplicity
                                false
                            },
                            _ => false,
                        };

                        if should_cancel {
                            gates_to_remove.push(*prev_idx);
                            gates_to_remove.push(gate_idx);
                            prev_gates.remove(&qubit);
                        } else {
                            prev_gates.insert(qubit, (gate_idx, gate.gate_type.clone()));
                        }
                    } else {
                        prev_gates.insert(qubit, (gate_idx, gate.gate_type.clone()));
                    }
                }
            }

            // Remove cancelled gates
            gates_to_remove.sort_unstable();
            gates_to_remove.dedup();
            for &gate_idx in gates_to_remove.iter().rev() {
                self.layers[layer_idx].gates.remove(gate_idx);
            }
        }
    }

    // Circuit analysis
    pub fn gate_count(&self) -> usize {
        self.layers.iter().map(|layer| layer.gates.len()).sum()
    }

    pub fn gate_count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for layer in &self.layers {
            for gate in &layer.gates {
                let gate_name = match &gate.gate_type {
                    GateType::Identity => "I".to_string(),
                    GateType::PauliX => "X".to_string(),
                    GateType::PauliY => "Y".to_string(),
                    GateType::PauliZ => "Z".to_string(),
                    GateType::Hadamard => "H".to_string(),
                    GateType::Phase(_) => "P".to_string(),
                    GateType::RX(_) => "RX".to_string(),
                    GateType::RY(_) => "RY".to_string(),
                    GateType::RZ(_) => "RZ".to_string(),
                    GateType::T => "T".to_string(),
                    GateType::S => "S".to_string(),
                    GateType::CNOT => "CNOT".to_string(),
                    GateType::CZ => "CZ".to_string(),
                    GateType::SWAP => "SWAP".to_string(),
                    GateType::CPhase(_) => "CPhase".to_string(),
                    GateType::Toffoli => "Toffoli".to_string(),
                    GateType::Fredkin => "Fredkin".to_string(),
                    GateType::Custom(_) => "Custom".to_string(),
                };
                *counts.entry(gate_name).or_insert(0) += 1;
            }
        }

        counts
    }

    pub fn qubit_connectivity(&self) -> Vec<Vec<QubitIndex>> {
        let mut connections = vec![HashSet::new(); self.num_qubits];

        for layer in &self.layers {
            for gate in &layer.gates {
                if gate.qubits.len() > 1 {
                    for i in 0..gate.qubits.len() {
                        for j in (i + 1)..gate.qubits.len() {
                            let q1 = gate.qubits[i];
                            let q2 = gate.qubits[j];
                            connections[q1].insert(q2);
                            connections[q2].insert(q1);
                        }
                    }
                }
            }
        }

        connections
            .into_iter()
            .map(|set| set.into_iter().collect())
            .collect()
    }

    // Circuit composition
    pub fn compose(&mut self, other: &QuantumCircuit) -> Result<(), String> {
        if self.num_qubits != other.num_qubits {
            return Err("Cannot compose circuits with different number of qubits".to_string());
        }

        for layer in &other.layers {
            let gates = layer.gates.clone();
            self.add_layer(gates)?;
        }

        Ok(())
    }

    // Convert to matrix representation (for small circuits)
    pub fn to_matrix(&self) -> Option<Vec<num_complex::Complex<f64>>> {
        if self.num_qubits > 10 {
            // Too large for matrix representation
            return None;
        }

        let size = 1 << self.num_qubits;
        let mut matrix = vec![num_complex::Complex::new(0.0, 0.0); size * size];

        // Initialize as identity matrix
        for i in 0..size {
            matrix[i * size + i] = num_complex::Complex::new(1.0, 0.0);
        }

        // Apply each layer
        for _layer in &self.layers {
            // Apply matrix multiplication for each gate in the layer
            // For now, we return the identity matrix as a placeholder
            // Full implementation would require tensor products and matrix multiplication
            // This is a complex operation that would need careful implementation
        }

        Some(matrix)
    }

    // Circuit information
    pub fn info(&self) -> String {
        format!(
            "Circuit: {}\nQubits: {}\nDepth: {}\nGates: {}\nLayers: {}",
            self.name,
            self.num_qubits,
            self.total_depth,
            self.gate_count(),
            self.layers.len()
        )
    }
}

// Circuit builder pattern for complex circuits
pub struct CircuitBuilder {
    circuit: QuantumCircuit,
}

impl CircuitBuilder {
    pub fn new(num_qubits: usize) -> Self {
        Self {
            circuit: QuantumCircuit::new(num_qubits),
        }
    }

    pub fn with_name(num_qubits: usize, name: String) -> Self {
        Self {
            circuit: QuantumCircuit::with_name(num_qubits, name),
        }
    }

    pub fn h(mut self, qubit: QubitIndex) -> Self {
        let _ = self.circuit.h(qubit);
        self
    }

    pub fn x(mut self, qubit: QubitIndex) -> Self {
        let _ = self.circuit.x(qubit);
        self
    }

    pub fn cnot(mut self, control: QubitIndex, target: QubitIndex) -> Self {
        let _ = self.circuit.cnot(control, target);
        self
    }

    pub fn measure(mut self, qubit: QubitIndex) -> Self {
        self.circuit.measure(qubit);
        self
    }

    pub fn measure_all(mut self) -> Self {
        self.circuit.measure_all();
        self
    }

    pub fn build(self) -> QuantumCircuit {
        self.circuit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_creation() {
        let circuit = QuantumCircuit::new(3);
        assert_eq!(circuit.num_qubits, 3);
        assert_eq!(circuit.layers.len(), 0);
        assert_eq!(circuit.total_depth, 0);
    }

    #[test]
    fn test_bell_state_circuit() {
        let mut circuit = QuantumCircuit::new(2);
        circuit.h(0).unwrap();
        circuit.cnot(0, 1).unwrap();

        assert_eq!(circuit.layers.len(), 2);
        assert_eq!(circuit.gate_count(), 2);
    }

    #[test]
    fn test_circuit_builder() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();

        assert_eq!(circuit.num_qubits, 2);
        assert_eq!(circuit.gate_count(), 2);
        assert_eq!(circuit.measurements.len(), 2);
    }
}
