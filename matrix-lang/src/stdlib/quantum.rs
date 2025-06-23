// Quantum Computing Standard Library for Matrix Language
// Provides quantum circuit construction, simulation, and algorithm functions

use crate::eval::{RuntimeError, RuntimeResult, Value};
use crate::quantum::{
    draw_circuit, draw_state, AlgorithmLibrary, CircuitBuilder, QuantumEngine, QuantumGate,
    QuantumSimulationChamber,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global quantum engine instance
lazy_static! {
    static ref QUANTUM_ENGINE: Arc<Mutex<QuantumEngine>> =
        Arc::new(Mutex::new(QuantumEngine::new()));
}

// Register all quantum computing functions
pub fn register_quantum_functions(interpreter: &mut crate::eval::Interpreter) {
    // Circuit creation and manipulation
    register_circuit_functions(interpreter);

    // Gate operations
    register_gate_functions(interpreter);

    // Simulation functions
    register_simulation_functions(interpreter);

    // Visualization functions
    register_visualization_functions(interpreter);

    // Algorithm functions
    register_algorithm_functions(interpreter);

    // GUI functions
    register_gui_functions(interpreter);
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

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                let circuit_id = engine.create_circuit(num_qubits);
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

                let engine = QUANTUM_ENGINE.lock().unwrap();
                if let Some(circuit) = engine.circuits.get(circuit_id) {
                    Ok(Value::String(circuit.info()))
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
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
                let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                engine.circuits.push(circuit);
                let circuit_id = engine.circuits.len() - 1;
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );
}

fn register_gate_functions(interpreter: &mut crate::eval::Interpreter) {
    // Single-qubit gates
    register_single_qubit_gate(interpreter, "h", "Hadamard");
    register_single_qubit_gate(interpreter, "hadamard", "Hadamard"); // Add full name
    register_single_qubit_gate(interpreter, "x", "PauliX");
    register_single_qubit_gate(interpreter, "y", "PauliY");
    register_single_qubit_gate(interpreter, "z", "PauliZ");
    register_single_qubit_gate(interpreter, "t", "T");
    register_single_qubit_gate(interpreter, "s", "S");

    // Parametric single-qubit gates
    register_parametric_gate(interpreter, "rx", "RX");
    register_parametric_gate(interpreter, "ry", "RY");
    register_parametric_gate(interpreter, "rz", "RZ");

    // Two-qubit gates
    register_two_qubit_gate(interpreter, "cnot", "CNOT");
    register_two_qubit_gate(interpreter, "cz", "CZ");
    register_two_qubit_gate(interpreter, "swap", "SWAP");

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

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
                    circuit
                        .toffoli(control1, control2, target)
                        .map_err(|e| RuntimeError::Generic { message: e })?;
                    Ok(Value::Unit)
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // Measurement
    interpreter.environment.define(
        "measure".to_string(),
        Value::BuiltinFunction {
            name: "measure".to_string(),
            arity: 2, // circuit_id, qubit
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;
                let qubit = get_qubit_index(&args[1])?;

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
                    circuit.measure(qubit);
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

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
                    circuit.measure_all();
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

fn register_single_qubit_gate(
    interpreter: &mut crate::eval::Interpreter,
    name: &str,
    gate_type: &str,
) {
    let gate_name = name.to_string();

    let func = match gate_type {
        "Hadamard" => apply_hadamard_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "PauliX" => apply_pauli_x_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "PauliY" => apply_pauli_y_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "PauliZ" => apply_pauli_z_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "T" => apply_t_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "S" => apply_s_gate as fn(&[Value]) -> RuntimeResult<Value>,
        _ => return,
    };

    interpreter.environment.define(
        gate_name.clone(),
        Value::BuiltinFunction {
            name: gate_name,
            arity: 2, // circuit_id, qubit
            func,
        },
    );
}

fn register_parametric_gate(
    interpreter: &mut crate::eval::Interpreter,
    name: &str,
    gate_type: &str,
) {
    let gate_name = name.to_string();

    let func = match gate_type {
        "RX" => apply_rx_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "RY" => apply_ry_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "RZ" => apply_rz_gate as fn(&[Value]) -> RuntimeResult<Value>,
        _ => return,
    };

    interpreter.environment.define(
        gate_name.clone(),
        Value::BuiltinFunction {
            name: gate_name,
            arity: 3, // circuit_id, qubit, angle
            func,
        },
    );
}

fn register_two_qubit_gate(
    interpreter: &mut crate::eval::Interpreter,
    name: &str,
    gate_type: &str,
) {
    let gate_name = name.to_string();

    let func = match gate_type {
        "CNOT" => apply_cnot_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "CZ" => apply_cz_gate as fn(&[Value]) -> RuntimeResult<Value>,
        "SWAP" => apply_swap_gate as fn(&[Value]) -> RuntimeResult<Value>,
        _ => return,
    };

    interpreter.environment.define(
        gate_name.clone(),
        Value::BuiltinFunction {
            name: gate_name,
            arity: 3, // circuit_id, qubit1, qubit2
            func,
        },
    );
}

fn register_simulation_functions(interpreter: &mut crate::eval::Interpreter) {
    // simulate(circuit_id) -> result
    interpreter.environment.define(
        "simulate".to_string(),
        Value::BuiltinFunction {
            name: "simulate".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                match engine.run_circuit(circuit_id) {
                    Ok(result) => {
                        // Convert result to Matrix Language value
                        let mut result_map = HashMap::new();
                        result_map.insert(
                            "execution_time".to_string(),
                            Value::Float(result.execution_time.as_secs_f64()),
                        );
                        result_map.insert(
                            "operations_count".to_string(),
                            Value::Int(result.operations_count as i64),
                        );

                        // Convert measurements
                        let measurements: Vec<Value> = result
                            .measurements
                            .iter()
                            .map(|(qubit, value)| {
                                let mut measurement = HashMap::new();
                                measurement.insert("qubit".to_string(), Value::Int(*qubit as i64));
                                measurement.insert("result".to_string(), Value::Bool(*value));
                                Value::Struct {
                                    name: "Measurement".to_string(),
                                    fields: measurement,
                                }
                            })
                            .collect();
                        result_map.insert("measurements".to_string(), Value::Array(measurements));

                        Ok(Value::Struct {
                            name: "SimulationResult".to_string(),
                            fields: result_map,
                        })
                    }
                    Err(error) => Err(RuntimeError::Generic { message: error }),
                }
            },
        },
    );

    // simulate_circuit(circuit_id) -> result (alias for simulate)
    interpreter.environment.define(
        "simulate_circuit".to_string(),
        Value::BuiltinFunction {
            name: "simulate_circuit".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                match engine.run_circuit(circuit_id) {
                    Ok(_result) => {
                        // Return the result ID as an integer for simplicity
                        Ok(Value::Int(circuit_id as i64))
                    }
                    Err(error) => Err(RuntimeError::Generic { message: error }),
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

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                match engine.run_circuit(circuit_id) {
                    Ok(result) => {
                        let probabilities = result.final_state.probabilities();
                        let prob_values: Vec<Value> =
                            probabilities.iter().map(|&p| Value::Float(p)).collect();
                        Ok(Value::Array(prob_values))
                    }
                    Err(error) => Err(RuntimeError::Generic { message: error }),
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

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                match engine.run_circuit(circuit_id) {
                    Ok(result) => {
                        let mut info = HashMap::new();
                        info.insert(
                            "num_qubits".to_string(),
                            Value::Int(result.final_state.num_qubits as i64),
                        );
                        info.insert(
                            "is_normalized".to_string(),
                            Value::Bool(result.final_state.is_normalized),
                        );

                        let probabilities = result.final_state.probabilities();
                        let entropy = -probabilities
                            .iter()
                            .filter(|&&p| p > 1e-12)
                            .map(|&p| p * p.log2())
                            .sum::<f64>();
                        info.insert("entropy".to_string(), Value::Float(entropy));

                        Ok(Value::Struct {
                            name: "StateInfo".to_string(),
                            fields: info,
                        })
                    }
                    Err(error) => Err(RuntimeError::Generic { message: error }),
                }
            },
        },
    );
}

fn register_visualization_functions(interpreter: &mut crate::eval::Interpreter) {
    // draw_circuit(circuit_id) -> circuit diagram
    interpreter.environment.define(
        "draw_circuit".to_string(),
        Value::BuiltinFunction {
            name: "draw_circuit".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let engine = QUANTUM_ENGINE.lock().unwrap();
                if let Some(circuit) = engine.circuits.get(circuit_id) {
                    Ok(Value::String(draw_circuit(circuit)))
                } else {
                    Err(RuntimeError::Generic {
                        message: format!("Circuit {} not found", circuit_id),
                    })
                }
            },
        },
    );

    // show_state(circuit_id) -> state visualization
    interpreter.environment.define(
        "show_state".to_string(),
        Value::BuiltinFunction {
            name: "show_state".to_string(),
            arity: 1,
            func: |args| {
                let circuit_id = get_circuit_id(&args[0])?;

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                match engine.run_circuit(circuit_id) {
                    Ok(result) => Ok(Value::String(draw_state(&result.final_state))),
                    Err(error) => Err(RuntimeError::Generic { message: error }),
                }
            },
        },
    );
}

fn register_algorithm_functions(interpreter: &mut crate::eval::Interpreter) {
    // bernstein_vazirani(secret_string) -> circuit_id
    interpreter.environment.define(
        "bernstein_vazirani".to_string(),
        Value::BuiltinFunction {
            name: "bernstein_vazirani".to_string(),
            arity: 1,
            func: |args| {
                let secret = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "bernstein_vazirani expects string secret".to_string(),
                        })
                    }
                };

                let algorithm = AlgorithmLibrary::bernstein_vazirani(&secret);
                let circuit = algorithm.build_circuit();

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                engine.circuits.push(circuit);
                let circuit_id = engine.circuits.len() - 1;
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );

    // grover_search(num_qubits, targets) -> circuit_id
    interpreter.environment.define(
        "grover_search".to_string(),
        Value::BuiltinFunction {
            name: "grover_search".to_string(),
            arity: 2,
            func: |args| {
                let num_qubits = match &args[0] {
                    Value::Int(n) => *n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "grover_search expects integer number of qubits".to_string(),
                        })
                    }
                };

                let targets = match &args[1] {
                    Value::Array(arr) => {
                        let mut target_vec = Vec::new();
                        for val in arr {
                            match val {
                                Value::Int(n) => target_vec.push(*n as usize),
                                _ => {
                                    return Err(RuntimeError::TypeError {
                                        message: "Target array must contain integers".to_string(),
                                    })
                                }
                            }
                        }
                        target_vec
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "grover_search expects array of target integers".to_string(),
                        })
                    }
                };

                let algorithm = AlgorithmLibrary::grover_search(num_qubits, targets);
                let circuit = algorithm.build_circuit();

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                engine.circuits.push(circuit);
                let circuit_id = engine.circuits.len() - 1;
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );

    // qft(num_qubits, inverse) -> circuit_id
    interpreter.environment.define(
        "qft".to_string(),
        Value::BuiltinFunction {
            name: "qft".to_string(),
            arity: 2,
            func: |args| {
                let num_qubits = match &args[0] {
                    Value::Int(n) => *n as usize,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "qft expects integer number of qubits".to_string(),
                        })
                    }
                };

                let inverse = match &args[1] {
                    Value::Bool(b) => *b,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "qft expects boolean for inverse parameter".to_string(),
                        })
                    }
                };

                let algorithm = AlgorithmLibrary::qft(num_qubits, inverse);
                let circuit = algorithm.build_circuit();

                let mut engine = QUANTUM_ENGINE.lock().unwrap();
                engine.circuits.push(circuit);
                let circuit_id = engine.circuits.len() - 1;
                Ok(Value::Int(circuit_id as i64))
            },
        },
    );
}

fn register_gui_functions(interpreter: &mut crate::eval::Interpreter) {
    // quantum_gui() -> starts the quantum simulation chamber
    interpreter.environment.define(
        "quantum_gui".to_string(),
        Value::BuiltinFunction {
            name: "quantum_gui".to_string(),
            arity: 0,
            func: |_args| {
                let chamber = QuantumSimulationChamber::new();
                std::thread::spawn(move || {
                    chamber.run_gui();
                });
                Ok(Value::String(
                    "Quantum Simulation Chamber started".to_string(),
                ))
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
            if *n >= 0 {
                Ok(*n as usize)
            } else {
                Err(RuntimeError::Generic {
                    message: "Qubit index cannot be negative".to_string(),
                })
            }
        }
        _ => Err(RuntimeError::TypeError {
            message: "Expected qubit index (integer)".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Interpreter;

    #[test]
    fn test_quantum_circuit_creation() {
        let mut interpreter = Interpreter::new();
        register_quantum_functions(&mut interpreter);

        // Test quantum_circuit function
        let result = interpreter.environment.get("quantum_circuit").unwrap();
        match result {
            Value::BuiltinFunction { .. } => assert!(true),
            _ => panic!("quantum_circuit not registered as function"),
        }
    }

    #[test]
    fn test_bell_state_function() {
        let mut interpreter = Interpreter::new();
        register_quantum_functions(&mut interpreter);

        let bell_fn = interpreter.environment.get("bell_state").unwrap();
        match bell_fn {
            Value::BuiltinFunction { func, .. } => {
                let result = func(&[]);
                assert!(result.is_ok());
            }
            _ => panic!("bell_state not registered correctly"),
        }
    }
}

// Individual gate functions to avoid closure capture issues
fn apply_hadamard_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .h(qubit)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_pauli_x_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .x(qubit)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_pauli_y_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .y(qubit)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_pauli_z_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .z(qubit)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_t_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .add_gate(QuantumGate::t_gate(qubit))
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_s_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit = get_qubit_index(&args[1])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .add_gate(QuantumGate::s_gate(qubit))
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

// Parametric gate functions
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

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .rx(qubit, angle)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
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

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .ry(qubit, angle)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
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

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .rz(qubit, angle)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

// Two-qubit gate functions
fn apply_cnot_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit1 = get_qubit_index(&args[1])?;
    let qubit2 = get_qubit_index(&args[2])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .cnot(qubit1, qubit2)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_cz_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit1 = get_qubit_index(&args[1])?;
    let qubit2 = get_qubit_index(&args[2])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .cz(qubit1, qubit2)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}

fn apply_swap_gate(args: &[Value]) -> RuntimeResult<Value> {
    let circuit_id = get_circuit_id(&args[0])?;
    let qubit1 = get_qubit_index(&args[1])?;
    let qubit2 = get_qubit_index(&args[2])?;

    let mut engine = QUANTUM_ENGINE.lock().unwrap();
    if let Some(circuit) = engine.circuits.get_mut(circuit_id) {
        circuit
            .swap(qubit1, qubit2)
            .map_err(|e| RuntimeError::Generic { message: e })?;
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::Generic {
            message: format!("Circuit {} not found", circuit_id),
        })
    }
}
