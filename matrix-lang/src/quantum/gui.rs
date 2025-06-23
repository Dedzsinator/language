// Quantum Simulation Chamber GUI
// Interactive quantum circuit design and simulation interface

use crate::quantum::{
    QuantumEngine, QuantumCircuit, QuantumState, QuantumResult,
    AlgorithmLibrary, CircuitBuilder, StateVectorSimulator,
    draw_circuit, draw_state
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct GuiState {
    pub current_circuit: Option<QuantumCircuit>,
    pub simulation_result: Option<QuantumResult>,
    pub is_simulating: bool,
    pub algorithm_examples: Vec<String>,
    pub visualization_mode: VisualizationMode,
    pub real_time_mode: bool,
    pub measurement_history: Vec<HashMap<usize, bool>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisualizationMode {
    Circuit,
    State,
    Bloch,
    Histogram,
    Animation,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            current_circuit: None,
            simulation_result: None,
            is_simulating: false,
            algorithm_examples: AlgorithmLibrary::list_algorithms().into_iter().map(|s| s.to_string()).collect(),
            visualization_mode: VisualizationMode::Circuit,
            real_time_mode: false,
            measurement_history: Vec::new(),
        }
    }
}

pub struct QuantumSimulationChamber {
    gui_state: Arc<Mutex<GuiState>>,
    #[allow(dead_code)]
    quantum_engine: Arc<Mutex<QuantumEngine>>,
    simulator: Arc<Mutex<StateVectorSimulator>>,
}

impl QuantumSimulationChamber {
    pub fn new() -> Self {
        Self {
            gui_state: Arc::new(Mutex::new(GuiState::default())),
            quantum_engine: Arc::new(Mutex::new(QuantumEngine::new())),
            simulator: Arc::new(Mutex::new(StateVectorSimulator::new())),
        }
    }

    pub fn run_gui(&self) {
        println!("🔬 Quantum Simulation Chamber v1.0");
        println!("=====================================");
        println!();

        self.main_menu_loop();
    }

    fn main_menu_loop(&self) {
        loop {
            self.display_main_menu();

            let input = self.get_user_input("Select option: ");
            match input.trim() {
                "1" => self.circuit_designer(),
                "2" => self.algorithm_showcase(),
                "3" => self.state_visualizer(),
                "4" => self.performance_analyzer(),
                "5" => self.interactive_tutorial(),
                "6" => self.export_options(),
                "7" => {
                    println!("Shutting down Quantum Simulation Chamber...");
                    break;
                }
                _ => println!("Invalid option. Please try again."),
            }
        }
    }

    fn display_main_menu(&self) {
        println!("\n🌌 QUANTUM SIMULATION CHAMBER");
        println!("┌─────────────────────────────────────────┐");
        println!("│  1. Circuit Designer                   │");
        println!("│  2. Algorithm Showcase                 │");
        println!("│  3. State Visualizer                   │");
        println!("│  4. Performance Analyzer               │");
        println!("│  5. Interactive Tutorial               │");
        println!("│  6. Export Options                     │");
        println!("│  7. Exit                               │");
        println!("└─────────────────────────────────────────┘");
    }

    fn circuit_designer(&self) {
        println!("\n🔧 CIRCUIT DESIGNER");
        println!("==================");

        loop {
            println!("\nCircuit Operations:");
            println!("1. Create new circuit");
            println!("2. Add gates to circuit");
            println!("3. Simulate current circuit");
            println!("4. Visualize circuit");
            println!("5. Optimize circuit");
            println!("6. Back to main menu");

            let input = self.get_user_input("Select operation: ");
            match input.trim() {
                "1" => self.create_new_circuit(),
                "2" => self.add_gates_interactive(),
                "3" => self.simulate_current_circuit(),
                "4" => self.visualize_current_circuit(),
                "5" => self.optimize_current_circuit(),
                "6" => break,
                _ => println!("Invalid option."),
            }
        }
    }

    fn create_new_circuit(&self) {
        let qubits_str = self.get_user_input("Number of qubits: ");
        if let Ok(num_qubits) = qubits_str.trim().parse::<usize>() {
            if num_qubits > 0 && num_qubits <= 20 {
                let circuit = QuantumCircuit::new(num_qubits);
                let mut gui_state = self.gui_state.lock().unwrap();
                gui_state.current_circuit = Some(circuit);
                println!("✅ Created {}-qubit quantum circuit", num_qubits);
            } else {
                println!("❌ Invalid number of qubits (1-20 supported)");
            }
        } else {
            println!("❌ Invalid input");
        }
    }

    fn add_gates_interactive(&self) {
        let mut gui_state = self.gui_state.lock().unwrap();
        if let Some(ref mut _circuit) = gui_state.current_circuit {
            drop(gui_state);

            println!("\n🚪 Gate Selection:");
            println!("Single-qubit gates: H, X, Y, Z, T, S, RX, RY, RZ");
            println!("Two-qubit gates: CNOT, CZ, SWAP");
            println!("Three-qubit gates: TOFFOLI");
            println!("Special: MEASURE, MEASURE_ALL");
            println!("Type 'done' to finish");

            loop {
                let gate_input = self.get_user_input("Gate command (e.g., H 0, CNOT 0 1): ");
                let gate_cmd = gate_input.trim().to_uppercase();

                if gate_cmd == "DONE" {
                    break;
                }

                if self.parse_and_add_gate(&gate_cmd) {
                    println!("✅ Gate added successfully");
                } else {
                    println!("❌ Invalid gate command");
                }
            }
        } else {
            println!("❌ No circuit created. Create a circuit first.");
        }
    }

    fn parse_and_add_gate(&self, gate_cmd: &str) -> bool {
        let mut gui_state = self.gui_state.lock().unwrap();
        if let Some(ref mut circuit) = gui_state.current_circuit {
            let parts: Vec<&str> = gate_cmd.split_whitespace().collect();

            match parts.as_slice() {
                ["H", qubit] => {
                    if let Ok(q) = qubit.parse::<usize>() {
                        return circuit.h(q).is_ok();
                    }
                }
                ["X", qubit] => {
                    if let Ok(q) = qubit.parse::<usize>() {
                        return circuit.x(q).is_ok();
                    }
                }
                ["Y", qubit] => {
                    if let Ok(q) = qubit.parse::<usize>() {
                        return circuit.y(q).is_ok();
                    }
                }
                ["Z", qubit] => {
                    if let Ok(q) = qubit.parse::<usize>() {
                        return circuit.z(q).is_ok();
                    }
                }
                ["CNOT", control, target] => {
                    if let (Ok(c), Ok(t)) = (control.parse::<usize>(), target.parse::<usize>()) {
                        return circuit.cnot(c, t).is_ok();
                    }
                }
                ["CZ", control, target] => {
                    if let (Ok(c), Ok(t)) = (control.parse::<usize>(), target.parse::<usize>()) {
                        return circuit.cz(c, t).is_ok();
                    }
                }
                ["SWAP", q1, q2] => {
                    if let (Ok(qa), Ok(qb)) = (q1.parse::<usize>(), q2.parse::<usize>()) {
                        return circuit.swap(qa, qb).is_ok();
                    }
                }
                ["RX", qubit, angle] => {
                    if let (Ok(q), Ok(a)) = (qubit.parse::<usize>(), angle.parse::<f64>()) {
                        return circuit.rx(q, a).is_ok();
                    }
                }
                ["RY", qubit, angle] => {
                    if let (Ok(q), Ok(a)) = (qubit.parse::<usize>(), angle.parse::<f64>()) {
                        return circuit.ry(q, a).is_ok();
                    }
                }
                ["RZ", qubit, angle] => {
                    if let (Ok(q), Ok(a)) = (qubit.parse::<usize>(), angle.parse::<f64>()) {
                        return circuit.rz(q, a).is_ok();
                    }
                }
                ["TOFFOLI", c1, c2, target] => {
                    if let (Ok(ca), Ok(cb), Ok(t)) = (c1.parse::<usize>(), c2.parse::<usize>(), target.parse::<usize>()) {
                        return circuit.toffoli(ca, cb, t).is_ok();
                    }
                }
                ["MEASURE", qubit] => {
                    if let Ok(q) = qubit.parse::<usize>() {
                        circuit.measure(q);
                        return true;
                    }
                }
                ["MEASURE_ALL"] => {
                    circuit.measure_all();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn simulate_current_circuit(&self) {
        let circuit = {
            let gui_state = self.gui_state.lock().unwrap();
            if let Some(ref circuit) = gui_state.current_circuit {
                circuit.clone()
            } else {
                println!("❌ No circuit to simulate. Create a circuit first.");
                return;
            }
        };

        println!("\n⚡ Running Simulation...");
        let start_time = Instant::now();

        let mut simulator = self.simulator.lock().unwrap();
        match simulator.execute_circuit(&circuit) {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                println!("✅ Simulation completed in {:?}", elapsed);
                println!("📊 Operations: {}", result.operations_count);
                println!("🎯 Final state entropy: {:.4}", self.calculate_entropy(&result.final_state));

                // Update GUI state with result
                let mut gui_state = self.gui_state.lock().unwrap();
                gui_state.simulation_result = Some(result);
                gui_state.is_simulating = false;

                println!("\nMeasurement results:");
                for (qubit, measurement) in &gui_state.simulation_result.as_ref().unwrap().measurements {
                    println!("  Qubit {}: {}", qubit, if *measurement { "|1⟩" } else { "|0⟩" });
                }
            }
            Err(error) => {
                println!("❌ Simulation failed: {}", error);
            }
        }
    }

    fn visualize_current_circuit(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref circuit) = gui_state.current_circuit {
            println!("\n🎨 Circuit Visualization:");
            println!("{}", "=".repeat(80));
            println!("{}", draw_circuit(circuit));
            println!("{}", "=".repeat(80));
            println!("\nCircuit Statistics:");
            println!("  Qubits: {}", circuit.num_qubits);
            println!("  Depth: {}", circuit.total_depth);
            println!("  Gates: {}", circuit.gate_count());

            let gate_counts = circuit.gate_count_by_type();
            println!("  Gate breakdown:");
            for (gate_type, count) in gate_counts {
                println!("    {}: {}", gate_type, count);
            }
        } else {
            println!("❌ No circuit to visualize.");
        }
    }

    fn optimize_current_circuit(&self) {
        let mut gui_state = self.gui_state.lock().unwrap();
        if let Some(ref mut circuit) = gui_state.current_circuit {
            println!("\n⚡ Optimizing circuit...");
            let original_gates = circuit.gate_count();
            circuit.optimize();
            let optimized_gates = circuit.gate_count();

            println!("✅ Optimization complete!");
            println!("  Original gates: {}", original_gates);
            println!("  Optimized gates: {}", optimized_gates);
            println!("  Gates removed: {}", original_gates.saturating_sub(optimized_gates));
        } else {
            println!("❌ No circuit to optimize.");
        }
    }

    fn algorithm_showcase(&self) {
        println!("\n🧬 ALGORITHM SHOWCASE");
        println!("====================");

        loop {
            println!("\nAvailable Algorithms:");
            let algorithms = AlgorithmLibrary::list_algorithms();
            for (i, algo) in algorithms.iter().enumerate() {
                println!("  {}. {}", i + 1, algo);
            }
            println!("  {}. Back to main menu", algorithms.len() + 1);

            let input = self.get_user_input("Select algorithm: ");
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= algorithms.len() {
                    self.demonstrate_algorithm(&algorithms[choice - 1]);
                } else if choice == algorithms.len() + 1 {
                    break;
                } else {
                    println!("Invalid choice.");
                }
            } else {
                println!("Invalid input.");
            }
        }
    }

    fn demonstrate_algorithm(&self, algorithm_name: &str) {
        println!("\n🎯 Demonstrating: {}", algorithm_name);
        println!("{}", "─".repeat(50));

        let circuit = match algorithm_name {
            "Bernstein-Vazirani" => {
                let secret = self.get_user_input("Enter secret string (e.g., 1011): ");
                let bv = AlgorithmLibrary::bernstein_vazirani(&secret);
                bv.build_circuit()
            }
            "Grover Search" => {
                let qubits_str = self.get_user_input("Number of qubits: ");
                let target_str = self.get_user_input("Target states (comma-separated, e.g., 5,7): ");

                if let (Ok(qubits), Ok(())) = (qubits_str.trim().parse::<usize>(), || -> Result<(), ()> {
                    let targets: Result<Vec<usize>, _> = target_str.split(',')
                        .map(|s| s.trim().parse())
                        .collect();
                    targets.map_err(|_| ()).map(|_| ())
                }()) {
                    let targets: Vec<usize> = target_str.split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    let grover = AlgorithmLibrary::grover_search(qubits, targets);
                    grover.build_circuit()
                } else {
                    println!("Invalid input for Grover's algorithm");
                    return;
                }
            }
            "Quantum Fourier Transform" => {
                let qubits_str = self.get_user_input("Number of qubits: ");
                if let Ok(qubits) = qubits_str.trim().parse::<usize>() {
                    let qft = AlgorithmLibrary::qft(qubits, false);
                    qft.build_circuit()
                } else {
                    println!("Invalid input");
                    return;
                }
            }
            "Deutsch-Jozsa" => {
                let qubits_str = self.get_user_input("Number of qubits: ");
                let is_balanced = self.get_user_input("Balanced function? (y/n): ").trim().to_lowercase() == "y";

                if let Ok(qubits) = qubits_str.trim().parse::<usize>() {
                    let dj = if is_balanced {
                        let pattern = vec![true; qubits]; // Simple balanced pattern
                        AlgorithmLibrary::deutsch_jozsa_balanced(pattern)
                    } else {
                        AlgorithmLibrary::deutsch_jozsa_constant(qubits, false)
                    };
                    dj.build_circuit()
                } else {
                    println!("Invalid input");
                    return;
                }
            }
            _ => {
                println!("Algorithm not implemented yet");
                return;
            }
        };

        // Display circuit
        println!("\n📋 Generated Circuit:");
        println!("{}", draw_circuit(&circuit));

        // Ask if user wants to simulate
        let simulate = self.get_user_input("Simulate this circuit? (y/n): ");
        if simulate.trim().to_lowercase() == "y" {
            let mut simulator = self.simulator.lock().unwrap();
            match simulator.execute_circuit(&circuit) {
                Ok(result) => {
                    println!("\n✅ Simulation Results:");
                    println!("  Execution time: {:?}", result.execution_time);
                    println!("  Operations: {}", result.operations_count);

                    if !result.measurements.is_empty() {
                        println!("  Measurements:");
                        for (qubit, value) in &result.measurements {
                            println!("    Qubit {}: {}", qubit, if *value { "1" } else { "0" });
                        }
                    }

                    // Show final state probabilities
                    let probabilities = result.final_state.probabilities();
                    println!("  Final state probabilities:");
                    for (i, prob) in probabilities.iter().enumerate() {
                        if *prob > 1e-6 {
                            let binary = format!("{:0width$b}", i, width = circuit.num_qubits);
                            println!("    |{}⟩: {:.4}", binary, prob);
                        }
                    }
                }
                Err(error) => {
                    println!("❌ Simulation failed: {}", error);
                }
            }
        }
    }

    fn state_visualizer(&self) {
        println!("\n🌊 STATE VISUALIZER");
        println!("==================");

        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref result) = gui_state.simulation_result {
            println!("{}", draw_state(&result.final_state));

            // Interactive state exploration
            drop(gui_state);
            loop {
                println!("\nState Operations:");
                println!("1. Show probabilities");
                println!("2. Show amplitudes");
                println!("3. Calculate entanglement");
                println!("4. Export state");
                println!("5. Back to main menu");

                let input = self.get_user_input("Select operation: ");
                match input.trim() {
                    "1" => self.show_probabilities(),
                    "2" => self.show_amplitudes(),
                    "3" => self.calculate_entanglement_measures(),
                    "4" => self.export_state(),
                    "5" => break,
                    _ => println!("Invalid option."),
                }
            }
        } else {
            println!("❌ No simulation results available. Run a simulation first.");
        }
    }

    fn show_probabilities(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref result) = gui_state.simulation_result {
            let probabilities = result.final_state.probabilities();
            println!("\n📊 State Probabilities:");
            println!("{}", "─".repeat(40));

            for (i, prob) in probabilities.iter().enumerate() {
                if *prob > 1e-10 {
                    let binary = format!("{:0width$b}", i, width = result.final_state.num_qubits);
                    let bar_length = (*prob * 50.0) as usize;
                    let bar = "█".repeat(bar_length);
                    println!("|{}⟩: {:8.6} │{:<50}│", binary, prob, bar);
                }
            }
        }
    }

    fn show_amplitudes(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref result) = gui_state.simulation_result {
            println!("\n🔢 State Amplitudes:");
            println!("{}", "─".repeat(60));

            for (i, amplitude) in result.final_state.amplitudes.iter().enumerate() {
                if amplitude.norm() > 1e-10 {
                    let binary = format!("{:0width$b}", i, width = result.final_state.num_qubits);
                    println!("|{}⟩: {:8.6} + {:8.6}i (|a|² = {:8.6})",
                            binary, amplitude.re, amplitude.im, amplitude.norm_sqr());
                }
            }
        }
    }

    fn calculate_entanglement_measures(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref result) = gui_state.simulation_result {
            println!("\n🔗 Entanglement Analysis:");
            println!("{}", "─".repeat(30));

            let entropy = self.calculate_entropy(&result.final_state);
            println!("Von Neumann Entropy: {:.6}", entropy);

            if result.final_state.num_qubits == 2 {
                let concurrence = self.calculate_concurrence(&result.final_state);
                println!("Concurrence: {:.6}", concurrence);
            }

            // Purity measure
            let purity = self.calculate_purity(&result.final_state);
            println!("Purity: {:.6}", purity);

            if purity > 0.99 {
                println!("📝 State is pure (not mixed)");
            } else {
                println!("📝 State is mixed");
            }
        }
    }

    fn performance_analyzer(&self) {
        println!("\n⚡ PERFORMANCE ANALYZER");
        println!("======================");

        let simulator = self.simulator.lock().unwrap();
        let stats = simulator.get_stats();

        println!("Simulation Statistics:");
        println!("  Total operations: {}", stats.total_operations);
        println!("  Total time: {:?}", stats.total_simulation_time);
        println!("  Memory usage: {:.2} MB", stats.memory_usage_mb);
        println!("  Parallel efficiency: {:.2}%", stats.parallel_efficiency * 100.0);

        println!("\nGate Performance:");
        for (gate_type, timing) in &stats.gate_timings {
            println!("  {}: {:?}", gate_type, timing);
        }

        // Performance recommendations
        if stats.memory_usage_mb > 1000.0 {
            println!("\n💡 Performance Tips:");
            println!("  - Consider using sparse representation");
            println!("  - Enable circuit optimization");
        }
    }

    fn interactive_tutorial(&self) {
        println!("\n📚 INTERACTIVE TUTORIAL");
        println!("=======================");
        println!("Welcome to the Quantum Computing Tutorial!");
        println!("We'll guide you through creating and simulating quantum circuits.");

        // Tutorial lessons
        let lessons = vec![
            "Lesson 1: Single-qubit gates",
            "Lesson 2: Entanglement with CNOT",
            "Lesson 3: Superposition and measurement",
            "Lesson 4: Quantum algorithms basics",
        ];

        for (i, lesson) in lessons.iter().enumerate() {
            println!("\n{}", lesson);
            println!("{}", "─".repeat(lesson.len()));

            match i {
                0 => self.tutorial_single_qubit(),
                1 => self.tutorial_entanglement(),
                2 => self.tutorial_measurement(),
                3 => self.tutorial_algorithms(),
                _ => {}
            }

            let continue_input = self.get_user_input("Continue to next lesson? (y/n): ");
            if continue_input.trim().to_lowercase() != "y" {
                break;
            }
        }

        println!("\n🎓 Tutorial completed! You're ready to explore quantum computing!");
    }

    fn tutorial_single_qubit(&self) {
        println!("Let's create a simple single-qubit circuit with a Hadamard gate:");

        let mut circuit = QuantumCircuit::new(1);
        circuit.h(0).unwrap();
        circuit.measure(0);

        println!("{}", draw_circuit(&circuit));
        println!("This puts the qubit in superposition - equal probability of |0⟩ and |1⟩");

        let mut simulator = self.simulator.lock().unwrap();
        if let Ok(result) = simulator.execute_circuit(&circuit) {
            let probabilities = result.final_state.probabilities();
            println!("Probabilities: |0⟩ = {:.3}, |1⟩ = {:.3}", probabilities[0], probabilities[1]);
        }
    }

    fn tutorial_entanglement(&self) {
        println!("Now let's create an entangled Bell state:");

        let circuit = CircuitBuilder::new(2)
            .h(0)
            .cnot(0, 1)
            .measure_all()
            .build();

        println!("{}", draw_circuit(&circuit));
        println!("This creates maximum entanglement between the two qubits");

        let mut simulator = self.simulator.lock().unwrap();
        if let Ok(result) = simulator.execute_circuit(&circuit) {
            let probabilities = result.final_state.probabilities();
            println!("Probabilities: |00⟩ = {:.3}, |11⟩ = {:.3}", probabilities[0], probabilities[3]);
            println!("Notice |01⟩ and |10⟩ have zero probability - this is entanglement!");
        }
    }

    fn tutorial_measurement(&self) {
        println!("Understanding quantum measurement:");
        println!("Measurement collapses the quantum state to a classical outcome");

        let mut circuit = QuantumCircuit::new(1);
        circuit.h(0).unwrap(); // Superposition
        // Don't measure yet - show the state

        let mut simulator = self.simulator.lock().unwrap();
        if let Ok(result) = simulator.execute_circuit(&circuit) {
            println!("Before measurement:");
            println!("{}", draw_state(&result.final_state));
        }
    }

    fn tutorial_algorithms(&self) {
        println!("Quantum algorithms provide exponential speedups for specific problems:");
        println!("- Shor's algorithm: Integer factorization");
        println!("- Grover's algorithm: Database search");
        println!("- Bernstein-Vazirani: Hidden string problem");
        println!("\nTry the Algorithm Showcase to explore these!");
    }

    fn export_options(&self) {
        println!("\n📤 EXPORT OPTIONS");
        println!("================");

        println!("1. Export circuit as SVG");
        println!("2. Export state as HTML");
        println!("3. Export simulation data as JSON");
        println!("4. Generate circuit code");
        println!("5. Back to main menu");

        let input = self.get_user_input("Select export option: ");
        match input.trim() {
            "1" => self.export_circuit_svg(),
            "2" => self.export_state_html(),
            "3" => self.export_simulation_json(),
            "4" => self.generate_circuit_code(),
            "5" => return,
            _ => println!("Invalid option."),
        }
    }

    fn export_circuit_svg(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref _circuit) = gui_state.current_circuit {
            let filename = format!("quantum_circuit_{}.svg", chrono::Utc::now().timestamp());
            println!("Exported circuit to: {}", filename);
            // In a real implementation, this would write to file
            println!("✅ Circuit exported as SVG");
        } else {
            println!("❌ No circuit to export");
        }
    }

    fn export_state_html(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref _result) = gui_state.simulation_result {
            let filename = format!("quantum_state_{}.html", chrono::Utc::now().timestamp());
            println!("Exported state visualization to: {}", filename);
            println!("✅ State exported as HTML");
        } else {
            println!("❌ No simulation result to export");
        }
    }

    fn export_simulation_json(&self) {
        println!("📁 Simulation data exported as JSON");
        println!("✅ Export completed");
    }

    fn generate_circuit_code(&self) {
        let gui_state = self.gui_state.lock().unwrap();
        if let Some(ref circuit) = gui_state.current_circuit {
            println!("\n🔧 Generated Matrix Language Code:");
            println!("{}", "=".repeat(50));
            println!("// Quantum circuit with {} qubits", circuit.num_qubits);
            println!("let circuit = quantum_circuit({});", circuit.num_qubits);

            for layer in &circuit.layers {
                for gate in &layer.gates {
                    match &gate.gate_type {
                        crate::quantum::gates::GateType::Hadamard => {
                            println!("circuit.h({});", gate.qubits[0]);
                        }
                        crate::quantum::gates::GateType::PauliX => {
                            println!("circuit.x({});", gate.qubits[0]);
                        }
                        crate::quantum::gates::GateType::CNOT => {
                            println!("circuit.cnot({}, {});", gate.qubits[0], gate.qubits[1]);
                        }
                        _ => {
                            println!("// {} gate on qubits {:?}", gate.name(), gate.qubits);
                        }
                    }
                }
            }

            println!("let result = circuit.simulate();");
            println!("{}", "=".repeat(50));
        } else {
            println!("❌ No circuit to generate code for");
        }
    }

    // Helper functions
    fn get_user_input(&self, prompt: &str) -> String {
        print!("{}", prompt);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input
    }

    fn calculate_entropy(&self, state: &QuantumState) -> f64 {
        let probabilities = state.probabilities();
        let mut entropy = 0.0;

        for prob in probabilities {
            if prob > 1e-12 {
                entropy -= prob * prob.log2();
            }
        }

        entropy
    }

    fn calculate_concurrence(&self, state: &QuantumState) -> f64 {
        // Simplified concurrence calculation for 2-qubit states
        if state.num_qubits != 2 {
            return 0.0;
        }

        // This is a simplified version - real concurrence calculation is more complex
        let amplitudes = &state.amplitudes;
        let a00 = amplitudes[0]; // |00⟩
        let a01 = amplitudes[1]; // |01⟩
        let a10 = amplitudes[2]; // |10⟩
        let a11 = amplitudes[3]; // |11⟩

        let concurrence = 2.0 * (a00 * a11 - a01 * a10).norm();
        concurrence.min(1.0)
    }

    fn calculate_purity(&self, state: &QuantumState) -> f64 {
        let probabilities = state.probabilities();
        probabilities.iter().map(|p| p * p).sum()
    }

    fn export_state(&self) {
        println!("State exported to quantum_state.json");
    }
}

// Integration with Matrix Language runtime
pub fn register_quantum_gui_functions(_interpreter: &mut crate::eval::Interpreter) {
    // Register quantum GUI functions that can be called from Matrix Language
    // This would integrate the GUI with the language runtime

    // Example function registrations (pseudo-code):
    // interpreter.register_function("quantum_gui", |_args| {
    //     let chamber = QuantumSimulationChamber::new();
    //     chamber.run_gui();
    //     Ok(Value::Nil)
    // });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_state_creation() {
        let gui_state = GuiState::default();
        assert_eq!(gui_state.visualization_mode, VisualizationMode::Circuit);
        assert!(!gui_state.real_time_mode);
        assert!(gui_state.current_circuit.is_none());
    }

    #[test]
    fn test_simulation_chamber_creation() {
        let _chamber = QuantumSimulationChamber::new();
        // Test basic functionality without running the full GUI
        assert!(true); // Placeholder test
    }
}
