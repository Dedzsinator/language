// Quantum Computing Standard Library for Matrix Language
// Simplified version with placeholder functions for testing

use crate::eval::{RuntimeError, RuntimeResult, Value};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// Simplified quantum state for testing
#[derive(Debug, Clone)]
struct SimpleQuantumCircuit {
    id: usize,
    num_qubits: usize,
    gates: Vec<String>,
}

static NEXT_CIRCUIT_ID: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(0));
static CIRCUITS: LazyLock<Mutex<HashMap<usize, SimpleQuantumCircuit>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Register all quantum computing functions
pub fn register_quantum_functions(interpreter: &mut crate::eval::Interpreter) {
    register_circuit_functions(interpreter);
    register_gate_functions(interpreter);
    register_simulation_functions(interpreter);
}

fn register_circuit_functions(interpreter: &mut crate::eval::Interpreter) {
    // quantum_circuit(num_qubits) -> circuit_id
    interpreter.environment.define(
        "quantum_circuit".to_string(),
        Value::BuiltinFunction {
            name: "quantum_circuit".to_string(),
            arity: 1,
            func: |args| {
                let num_qubits = match &args[0] {
                    Value::Int(n) => *n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "quantum_circuit expects integer number of qubits".to_string(),
                        })
                    }
                };

                if num_qubits == 0 || num_qubits > 20 {
                    return Err(RuntimeError::Generic {
                        message: "Number of qubits must be between 1 and 20".to_string(),
                    });
                }

                let mut next_id = NEXT_CIRCUIT_ID.lock().unwrap();
                let circuit_id = *next_id;
                *next_id += 1;

                let circuit = SimpleQuantumCircuit {
                    id: circuit_id,
                    num_qubits,
                    gates: Vec::new(),
                };

                CIRCUITS.lock().unwrap().insert(circuit_id, circuit);
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );

    // bell_state() -> circuit_id (convenience function)
    interpreter.environment.define(
        "bell_state".to_string(),
        Value::BuiltinFunction {
            name: "bell_state".to_string(),
            arity: 0,
            func: |_args| {
                let mut next_id = NEXT_CIRCUIT_ID.lock().unwrap();
                let circuit_id = *next_id;
                *next_id += 1;

                let circuit = SimpleQuantumCircuit {
                    id: circuit_id,
                    num_qubits: 2,
                    gates: vec!["H(0)".to_string(), "CNOT(0,1)".to_string()],
                };

                CIRCUITS.lock().unwrap().insert(circuit_id, circuit);
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );

    // circuit_info(circuit_id) -> info_string
    interpreter.environment.define(
        "circuit_info".to_string(),
        Value::BuiltinFunction {
            name: "circuit_info".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = match &args[0] {
                    Value::Int(n) => *n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "circuit_info expects circuit ID".to_string(),
                        })
                    }
                };

                let circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get(&circuit_id) {
                    let info = format!(
                        "Circuit {} with {} qubits and {} gates",
                        circuit.id,
                        circuit.num_qubits,
                        circuit.gates.len()
                    );
                    Ok(Value::String(info))
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );
}

fn register_gate_functions(interpreter: &mut crate::eval::Interpreter) {
    // Single-qubit gates
    register_single_qubit_gate(interpreter, "h");
    register_single_qubit_gate(interpreter, "hadamard");
    register_single_qubit_gate(interpreter, "x");
    register_single_qubit_gate(interpreter, "y");
    register_single_qubit_gate(interpreter, "z");
    register_single_qubit_gate(interpreter, "t");
    register_single_qubit_gate(interpreter, "s");

    // Parametric single-qubit gates
    register_parametric_gate(interpreter, "rx");
    register_parametric_gate(interpreter, "ry");
    register_parametric_gate(interpreter, "rz");

    // Two-qubit gates
    register_two_qubit_gate(interpreter, "cnot");
    register_two_qubit_gate(interpreter, "cz");
    register_two_qubit_gate(interpreter, "swap");

    // Three-qubit gates
    interpreter.environment.define(
        "toffoli".to_string(),
        Value::BuiltinFunction {
            name: "toffoli".to_string(),
            arity: 4, // circuit_id, control1, control2, target
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;
                let control1 = get_qubit_index(&args[1])?;
                let control2 = get_qubit_index(&args[2])?;
                let target = get_qubit_index(&args[3])?;

                let mut circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get_mut(&circuit_id) {
                    let gate_str = format!("TOFFOLI({},{},{})", control1, control2, target);
                    circuit.gates.push(gate_str);
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // Measurement functions
    interpreter.environment.define(
        "measure".to_string(),
        Value::BuiltinFunction {
            name: "measure".to_string(),
            arity: 2, // circuit_id, qubit
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;
                let qubit = get_qubit_index(&args[1])?;

                let mut circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get_mut(&circuit_id) {
                    let gate_str = format!("MEASURE({})", qubit);
                    circuit.gates.push(gate_str);
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    interpreter.environment.define(
        "measure_all".to_string(),
        Value::BuiltinFunction {
            name: "measure_all".to_string(),
            arity: 1, // circuit_id
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let mut circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get_mut(&circuit_id) {
                    circuit.gates.push("MEASURE_ALL".to_string());
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );
}

fn register_single_qubit_gate(interpreter: &mut crate::eval::Interpreter, name: &str) {
    match name {
        "h" | "hadamard" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_h_gate,
                },
            );
        }
        "x" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_x_gate,
                },
            );
        }
        "y" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_y_gate,
                },
            );
        }
        "z" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_z_gate,
                },
            );
        }
        "t" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_t_gate,
                },
            );
        }
        "s" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 2,
                    func: apply_s_gate,
                },
            );
        }
        _ => {}
    }
}

fn register_parametric_gate(interpreter: &mut crate::eval::Interpreter, name: &str) {
    match name {
        "rx" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_rx_gate,
                },
            );
        }
        "ry" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_ry_gate,
                },
            );
        }
        "rz" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_rz_gate,
                },
            );
        }
        _ => {}
    }
}

fn register_two_qubit_gate(interpreter: &mut crate::eval::Interpreter, name: &str) {
    match name {
        "cnot" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_cnot_gate,
                },
            );
        }
        "cz" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_cz_gate,
                },
            );
        }
        "swap" => {
            interpreter.environment.define(
                name.to_string(),
                Value::BuiltinFunction {
                    name: name.to_string(),
                    arity: 3,
                    func: apply_swap_gate,
                },
            );
        }
        _ => {}
    }
}

fn register_simulation_functions(interpreter: &mut crate::eval::Interpreter) {
    // simulate_circuit(circuit_id) -> result_id
    interpreter.environment.define(
        "simulate_circuit".to_string(),
        Value::BuiltinFunction {
            name: "simulate_circuit".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let circuits = CIRCUITS.lock().unwrap();
                if circuits.contains_key(&circuit_id) {
                    // Return the circuit_id as the result_id for simplicity
                    Ok(Value::Int(circuit_id as i64))
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // simulate(circuit_id) -> result (alias for simulate_circuit)
    interpreter.environment.define(
        "simulate".to_string(),
        Value::BuiltinFunction {
            name: "simulate".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get(&circuit_id) {
                    // Return a simple result struct
                    let mut result = HashMap::new();
                    result.insert("circuit_id".to_string(), Value::Int(circuit_id as i64));
                    result.insert(
                        "num_qubits".to_string(),
                        Value::Int(circuit.num_qubits as i64),
                    );
                    result.insert(
                        "num_gates".to_string(),
                        Value::Int(circuit.gates.len() as i64),
                    );
                    Ok(Value::Struct {
                        name: "SimulationResult".to_string(),
                        fields: result,
                    })
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // get_probabilities(circuit_id) -> array of probabilities
    interpreter.environment.define(
        "get_probabilities".to_string(),
        Value::BuiltinFunction {
            name: "get_probabilities".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get(&circuit_id) {
                    // Return dummy probabilities
                    let num_states = 1 << circuit.num_qubits; // 2^n states
                    let mut probs = Vec::new();
                    for i in 0..num_states {
                        let prob = if i == 0 { 1.0 } else { 0.0 }; // All probability on |0...0>
                        probs.push(Value::Float(prob));
                    }
                    Ok(Value::Array(probs))
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // quantum_state_info(circuit_id) -> state information
    interpreter.environment.define(
        "quantum_state_info".to_string(),
        Value::BuiltinFunction {
            name: "quantum_state_info".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get(&circuit_id) {
                    let mut info = HashMap::new();
                    info.insert(
                        "num_qubits".to_string(),
                        Value::Int(circuit.num_qubits as i64),
                    );
                    info.insert("is_normalized".to_string(), Value::Bool(true));
                    info.insert("entropy".to_string(), Value::Float(0.0));
                    info.insert("purity".to_string(), Value::Float(1.0));
                    Ok(Value::Struct {
                        name: "QuantumStateInfo".to_string(),
                        fields: info,
                    })
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // print_state(result_id) -> Unit
    interpreter.environment.define(
        "print_state".to_string(),
        Value::BuiltinFunction {
            name: "print_state".to_string(),
            arity: 1,
            func: |args| {
                let result_id = get_circuit_id(&args[0])?;

                let circuits = CIRCUITS.lock().unwrap();
                if let Some(circuit) = circuits.get(&result_id) {
                    println!("Quantum Circuit {} State:", circuit.id);
                    println!("  Qubits: {}", circuit.num_qubits);
                    println!("  Gates: {:?}", circuit.gates);
                    println!("  State: |0...0> (simplified)");
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", result_id),
                    })
                }
            },
        },
    );
}

// Helper functions
fn get_circuit_id(value: &Value) -> RuntimeResult<usize> {
    match value {
        Value::Int(n) => Ok(*n as usize),
        _ => Err(RuntimeError::TypeError {
            message: "Expected circuit ID (integer)".to_string(),
        }),
    }
}

fn get_qubit_index(value: &Value) -> RuntimeResult<usize> {
    match value {
        Value::Int(n) => {
            if *n < 0 {
                Err(RuntimeError::Generic {
                    message: "Qubit index cannot be negative".to_string(),
                })
            } else {
                Ok(*n as usize)
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected qubit index (integer)".to_string(),
        }),
    }
}

// Gate implementation functions
fn apply_h_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("H({})", qubit))
}

fn apply_x_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("X({})", qubit))
}

fn apply_y_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("Y({})", qubit))
}

fn apply_z_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("Z({})", qubit))
}

fn apply_t_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("T({})", qubit))
}

fn apply_s_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    add_gate_to_circuit(circuit_id, format!("S({})", qubit))
}

fn apply_rx_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    let angle = match &args[2] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Angle must be a number".to_string(),
            })
        }
    };
    add_gate_to_circuit(circuit_id, format!("RX({},{})", qubit, angle))
}

fn apply_ry_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    let angle = match &args[2] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Angle must be a number".to_string(),
            })
        }
    };
    add_gate_to_circuit(circuit_id, format!("RY({},{})", qubit, angle))
}

fn apply_rz_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;
    let angle = match &args[2] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        _ => {
            return Err(RuntimeError::TypeError {
                message: "Angle must be a number".to_string(),
            })
        }
    };
    add_gate_to_circuit(circuit_id, format!("RZ({},{})", qubit, angle))
}

fn apply_cnot_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let control = get_qubit_index(&args[1])?;
    let target = get_qubit_index(&args[2])?;
    add_gate_to_circuit(circuit_id, format!("CNOT({},{})", control, target))
}

fn apply_cz_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit1 = get_qubit_index(&args[1])?;
    let qubit2 = get_qubit_index(&args[2])?;
    add_gate_to_circuit(circuit_id, format!("CZ({},{})", qubit1, qubit2))
}

fn apply_swap_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit1 = get_qubit_index(&args[1])?;
    let qubit2 = get_qubit_index(&args[2])?;
    add_gate_to_circuit(circuit_id, format!("SWAP({},{})", qubit1, qubit2))
}

fn add_gate_to_circuit(circuit_id: usize, gate_str: String) -> RuntimeResult<Value> {
    let mut circuits = CIRCUITS.lock().unwrap();
    if let Some(circuit) = circuits.get_mut(&circuit_id) {
        circuit.gates.push(gate_str);
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}
