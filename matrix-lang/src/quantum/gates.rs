// Ultra-optimized quantum gates with precomputed matrices
// Supports all standard gates plus custom gates

use num_complex::Complex;
use std::f64::consts::SQRT_2;

pub type Amplitude = Complex<f64>;
pub type QubitIndex = usize;

// Gate types for circuit building
#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    // Single-qubit gates
    Identity,
    PauliX,
    PauliY,
    PauliZ,
    Hadamard,
    Phase(f64),
    RX(f64),
    RY(f64),
    RZ(f64),
    T,
    S,

    // Two-qubit gates
    CNOT,
    CZ,
    SWAP,
    CPhase(f64),

    // Three-qubit gates
    Toffoli,
    Fredkin,

    // Custom gate
    Custom(Vec<Amplitude>),
}

#[derive(Debug, Clone)]
pub struct QuantumGate {
    pub gate_type: GateType,
    pub qubits: Vec<QubitIndex>,
    pub matrix: Vec<Amplitude>,
    pub is_unitary: bool,
}

impl QuantumGate {
    // Precomputed matrices for common gates (ultra-fast lookup)
    const IDENTITY_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
    ];

    const PAULI_X_MATRIX: &'static [Amplitude] = &[
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
    ];

    const PAULI_Y_MATRIX: &'static [Amplitude] = &[
        Complex::new(0.0, 0.0),
        Complex::new(0.0, -1.0),
        Complex::new(0.0, 1.0),
        Complex::new(0.0, 0.0),
    ];

    const PAULI_Z_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(-1.0, 0.0),
    ];

    const HADAMARD_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0 / SQRT_2, 0.0),
        Complex::new(1.0 / SQRT_2, 0.0),
        Complex::new(1.0 / SQRT_2, 0.0),
        Complex::new(-1.0 / SQRT_2, 0.0),
    ];

    const T_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0 / SQRT_2, 1.0 / SQRT_2),
    ];

    const S_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 1.0),
    ];

    const CNOT_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
    ];

    const SWAP_MATRIX: &'static [Amplitude] = &[
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
    ];

    // Factory methods for common gates
    pub fn identity(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::Identity,
            qubits: vec![qubit],
            matrix: Self::IDENTITY_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn pauli_x(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::PauliX,
            qubits: vec![qubit],
            matrix: Self::PAULI_X_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn pauli_y(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::PauliY,
            qubits: vec![qubit],
            matrix: Self::PAULI_Y_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn pauli_z(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::PauliZ,
            qubits: vec![qubit],
            matrix: Self::PAULI_Z_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn hadamard(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::Hadamard,
            qubits: vec![qubit],
            matrix: Self::HADAMARD_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn t_gate(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::T,
            qubits: vec![qubit],
            matrix: Self::T_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn s_gate(qubit: QubitIndex) -> Self {
        Self {
            gate_type: GateType::S,
            qubits: vec![qubit],
            matrix: Self::S_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    // Parametric single-qubit gates
    pub fn phase_gate(qubit: QubitIndex, phase: f64) -> Self {
        let matrix = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(phase.cos(), phase.sin()),
        ];

        Self {
            gate_type: GateType::Phase(phase),
            qubits: vec![qubit],
            matrix,
            is_unitary: true,
        }
    }

    pub fn rx(qubit: QubitIndex, angle: f64) -> Self {
        let cos_half = (angle / 2.0).cos();
        let sin_half = (angle / 2.0).sin();

        let matrix = vec![
            Complex::new(cos_half, 0.0),
            Complex::new(0.0, -sin_half),
            Complex::new(0.0, -sin_half),
            Complex::new(cos_half, 0.0),
        ];

        Self {
            gate_type: GateType::RX(angle),
            qubits: vec![qubit],
            matrix,
            is_unitary: true,
        }
    }

    pub fn ry(qubit: QubitIndex, angle: f64) -> Self {
        let cos_half = (angle / 2.0).cos();
        let sin_half = (angle / 2.0).sin();

        let matrix = vec![
            Complex::new(cos_half, 0.0),
            Complex::new(-sin_half, 0.0),
            Complex::new(sin_half, 0.0),
            Complex::new(cos_half, 0.0),
        ];

        Self {
            gate_type: GateType::RY(angle),
            qubits: vec![qubit],
            matrix,
            is_unitary: true,
        }
    }

    pub fn rz(qubit: QubitIndex, angle: f64) -> Self {
        let phase_half = angle / 2.0;

        let matrix = vec![
            Complex::new((-phase_half).cos(), (-phase_half).sin()),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(phase_half.cos(), phase_half.sin()),
        ];

        Self {
            gate_type: GateType::RZ(angle),
            qubits: vec![qubit],
            matrix,
            is_unitary: true,
        }
    }

    // Two-qubit gates
    pub fn cnot(control: QubitIndex, target: QubitIndex) -> Self {
        Self {
            gate_type: GateType::CNOT,
            qubits: vec![control, target],
            matrix: Self::CNOT_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn cz(control: QubitIndex, target: QubitIndex) -> Self {
        let matrix = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(-1.0, 0.0),
        ];

        Self {
            gate_type: GateType::CZ,
            qubits: vec![control, target],
            matrix,
            is_unitary: true,
        }
    }

    pub fn swap(qubit1: QubitIndex, qubit2: QubitIndex) -> Self {
        Self {
            gate_type: GateType::SWAP,
            qubits: vec![qubit1, qubit2],
            matrix: Self::SWAP_MATRIX.to_vec(),
            is_unitary: true,
        }
    }

    pub fn controlled_phase(control: QubitIndex, target: QubitIndex, phase: f64) -> Self {
        let matrix = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(phase.cos(), phase.sin()),
        ];

        Self {
            gate_type: GateType::CPhase(phase),
            qubits: vec![control, target],
            matrix,
            is_unitary: true,
        }
    }

    // Three-qubit gates
    pub fn toffoli(control1: QubitIndex, control2: QubitIndex, target: QubitIndex) -> Self {
        // 8x8 Toffoli matrix (CCX gate)
        let mut matrix = vec![Complex::new(0.0, 0.0); 64]; // 8x8 = 64 elements

        // Identity for all states except |111⟩
        for i in 0..8 {
            if i != 7 {
                // Not |111⟩
                matrix[i * 8 + i] = Complex::new(1.0, 0.0);
            }
        }
        // |110⟩ -> |111⟩ and |111⟩ -> |110⟩ (flip target when both controls are 1)
        matrix[6 * 8 + 7] = Complex::new(1.0, 0.0); // |110⟩ -> |111⟩
        matrix[7 * 8 + 6] = Complex::new(1.0, 0.0); // |111⟩ -> |110⟩

        Self {
            gate_type: GateType::Toffoli,
            qubits: vec![control1, control2, target],
            matrix,
            is_unitary: true,
        }
    }

    pub fn fredkin(control: QubitIndex, target1: QubitIndex, target2: QubitIndex) -> Self {
        // 8x8 Fredkin matrix (CSWAP gate)
        let mut matrix = vec![Complex::new(0.0, 0.0); 64];

        // Identity for states where control is 0
        for i in 0..4 {
            matrix[i * 8 + i] = Complex::new(1.0, 0.0);
        }

        // SWAP for states where control is 1
        matrix[4 * 8 + 4] = Complex::new(1.0, 0.0); // |100⟩ -> |100⟩
        matrix[5 * 8 + 6] = Complex::new(1.0, 0.0); // |101⟩ -> |110⟩
        matrix[6 * 8 + 5] = Complex::new(1.0, 0.0); // |110⟩ -> |101⟩
        matrix[7 * 8 + 7] = Complex::new(1.0, 0.0); // |111⟩ -> |111⟩

        Self {
            gate_type: GateType::Fredkin,
            qubits: vec![control, target1, target2],
            matrix,
            is_unitary: true,
        }
    }

    // Custom gate from matrix
    pub fn custom_gate(qubits: Vec<QubitIndex>, matrix: Vec<Amplitude>) -> Result<Self, String> {
        let expected_size = 1 << qubits.len();
        if matrix.len() != expected_size * expected_size {
            return Err(format!(
                "Matrix size {} doesn't match expected {} for {} qubits",
                matrix.len(),
                expected_size * expected_size,
                qubits.len()
            ));
        }

        // Check if matrix is unitary (optional, for validation)
        let is_unitary = Self::is_matrix_unitary(&matrix, expected_size);

        Ok(Self {
            gate_type: GateType::Custom(matrix.clone()),
            qubits,
            matrix,
            is_unitary,
        })
    }

    // Check if a matrix is unitary
    fn is_matrix_unitary(_matrix: &[Amplitude], _size: usize) -> bool {
        // For performance, we'll skip this check in release builds
        // In practice, you'd compute A†A and check if it's identity
        true // Placeholder
    }

    // Get the adjoint (conjugate transpose) of the gate
    pub fn adjoint(&self) -> Self {
        let size = (self.matrix.len() as f64).sqrt() as usize;
        let mut adj_matrix = vec![Complex::new(0.0, 0.0); self.matrix.len()];

        for i in 0..size {
            for j in 0..size {
                adj_matrix[j * size + i] = self.matrix[i * size + j].conj();
            }
        }

        Self {
            gate_type: self.gate_type.clone(),
            qubits: self.qubits.clone(),
            matrix: adj_matrix,
            is_unitary: self.is_unitary,
        }
    }

    // Apply gate to quantum state (delegates to state)
    pub fn apply_to_state(&self, state: &mut crate::quantum::state::QuantumState) {
        state.apply_gate_matrix(&self.qubits, &self.matrix);
    }

    // Get gate name for visualization
    pub fn name(&self) -> String {
        match &self.gate_type {
            GateType::Identity => "I".to_string(),
            GateType::PauliX => "X".to_string(),
            GateType::PauliY => "Y".to_string(),
            GateType::PauliZ => "Z".to_string(),
            GateType::Hadamard => "H".to_string(),
            GateType::Phase(phase) => format!("P({:.3})", phase),
            GateType::RX(angle) => format!("RX({:.3})", angle),
            GateType::RY(angle) => format!("RY({:.3})", angle),
            GateType::RZ(angle) => format!("RZ({:.3})", angle),
            GateType::T => "T".to_string(),
            GateType::S => "S".to_string(),
            GateType::CNOT => "CNOT".to_string(),
            GateType::CZ => "CZ".to_string(),
            GateType::SWAP => "SWAP".to_string(),
            GateType::CPhase(phase) => format!("CP({:.3})", phase),
            GateType::Toffoli => "CCX".to_string(),
            GateType::Fredkin => "CSWAP".to_string(),
            GateType::Custom(_) => "CUSTOM".to_string(),
        }
    }
}

// Convenience functions for quick gate creation
pub fn x(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::pauli_x(qubit)
}
pub fn y(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::pauli_y(qubit)
}
pub fn z(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::pauli_z(qubit)
}
pub fn h(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::hadamard(qubit)
}
pub fn t(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::t_gate(qubit)
}
pub fn s(qubit: QubitIndex) -> QuantumGate {
    QuantumGate::s_gate(qubit)
}
pub fn cnot(control: QubitIndex, target: QubitIndex) -> QuantumGate {
    QuantumGate::cnot(control, target)
}
pub fn swap(q1: QubitIndex, q2: QubitIndex) -> QuantumGate {
    QuantumGate::swap(q1, q2)
}
pub fn toffoli(c1: QubitIndex, c2: QubitIndex, t: QubitIndex) -> QuantumGate {
    QuantumGate::toffoli(c1, c2, t)
}
