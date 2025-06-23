// Quantum circuit and state visualization module
// Supports ASCII art, SVG export, and interactive displays

use crate::quantum::circuit::{QuantumCircuit, CircuitLayer};
use crate::quantum::gates::{QuantumGate, GateType};
use crate::quantum::state::{QuantumState, QubitIndex};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub style: RenderStyle,
    pub width: usize,
    pub show_measurements: bool,
    pub show_probabilities: bool,
    pub compact_mode: bool,
    pub color_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderStyle {
    ASCII,
    Unicode,
    SVG,
    HTML,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            style: RenderStyle::Unicode,
            width: 120,
            show_measurements: true,
            show_probabilities: true,
            compact_mode: false,
            color_enabled: true,
        }
    }
}

// Circuit visualization
pub struct CircuitRenderer {
    config: VisualizationConfig,
}

impl CircuitRenderer {
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    pub fn render_circuit(&self, circuit: &QuantumCircuit) -> String {
        match self.config.style {
            RenderStyle::ASCII => self.render_ascii(circuit),
            RenderStyle::Unicode => self.render_unicode(circuit),
            RenderStyle::SVG => self.render_svg(circuit),
            RenderStyle::HTML => self.render_html(circuit),
        }
    }

    fn render_ascii(&self, circuit: &QuantumCircuit) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "Circuit: {}", circuit.name).unwrap();
        writeln!(output, "Qubits: {}, Depth: {}, Gates: {}",
                circuit.num_qubits, circuit.total_depth, circuit.gate_count()).unwrap();
        writeln!(output).unwrap();

        // Calculate column widths
        let mut gate_positions = Vec::new();
        let mut max_width = 0;

        for layer in &circuit.layers {
            let mut layer_gates = vec![None; circuit.num_qubits];
            for gate in &layer.gates {
                if gate.qubits.len() == 1 {
                    layer_gates[gate.qubits[0]] = Some(gate);
                }
            }
            gate_positions.push(layer_gates);
            max_width = std::cmp::max(max_width, layer.gates.len() * 8);
        }

        // Draw qubit lines and gates
        for qubit in 0..circuit.num_qubits {
            write!(output, "q{:<2}: ", qubit).unwrap();

            for (layer_idx, layer) in circuit.layers.iter().enumerate() {
                let gate_str = self.get_gate_string_for_qubit(layer, qubit);
                write!(output, "{:<8}", gate_str).unwrap();

                if layer_idx < circuit.layers.len() - 1 {
                    write!(output, "─").unwrap();
                }
            }

            // Show measurement
            if circuit.measurements.contains_key(&qubit) && self.config.show_measurements {
                write!(output, " ►[M]").unwrap();
            }

            writeln!(output).unwrap();
        }

        // Draw multi-qubit gates connections
        for (layer_idx, layer) in circuit.layers.iter().enumerate() {
            for gate in &layer.gates {
                if gate.qubits.len() > 1 {
                    output.push_str(&self.render_multi_qubit_gate_ascii(gate, layer_idx, circuit.num_qubits));
                }
            }
        }

        output
    }

    fn render_unicode(&self, circuit: &QuantumCircuit) -> String {
        let mut output = String::new();

        // Header with Unicode box drawing
        writeln!(output, "┌─ Circuit: {} ─┐", circuit.name).unwrap();
        writeln!(output, "│ Qubits: {:<3} Depth: {:<3} Gates: {:<3} │",
                circuit.num_qubits, circuit.total_depth, circuit.gate_count()).unwrap();
        writeln!(output, "└{}┘", "─".repeat(circuit.name.len() + 12)).unwrap();
        writeln!(output).unwrap();

        // Create a grid representation
        let mut grid = vec![vec![' '; circuit.layers.len() * 10 + 20]; circuit.num_qubits * 2];

        // Draw qubit lines
        for qubit in 0..circuit.num_qubits {
            let row = qubit * 2;
            write!(output, "q{:<2} ", qubit).unwrap();

            // Draw horizontal line
            for col in 5..grid[row].len() {
                if col % 10 < 8 {
                    grid[row][col] = '─';
                }
            }
        }

        // Place gates on grid
        for (layer_idx, layer) in circuit.layers.iter().enumerate() {
            let col_start = 5 + layer_idx * 10;

            for gate in &layer.gates {
                self.place_gate_on_grid(&mut grid, gate, col_start);
            }
        }

        // Convert grid to string
        for row in 0..circuit.num_qubits {
            let grid_row = row * 2;
            for &ch in &grid[grid_row] {
                output.push(ch);
            }

            // Add measurement indicator
            if circuit.measurements.contains_key(&row) && self.config.show_measurements {
                output.push_str(" ►╔═╗");
            }

            output.push('\n');

            // Connection lines for multi-qubit gates
            if row < circuit.num_qubits - 1 {
                for &ch in &grid[grid_row + 1] {
                    output.push(ch);
                }
                output.push('\n');
            }
        }

        output
    }

    fn render_svg(&self, circuit: &QuantumCircuit) -> String {
        let mut svg = String::new();
        let width = circuit.layers.len() * 80 + 100;
        let height = circuit.num_qubits * 60 + 100;

        // SVG header
        writeln!(svg, r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#, width, height).unwrap();
        writeln!(svg, r#"<style>
            .qubit-line {{ stroke: #333; stroke-width: 2; }}
            .gate-box {{ fill: #e8f4fd; stroke: #1976d2; stroke-width: 2; rx: 4; }}
            .gate-text {{ font-family: monospace; font-size: 12px; text-anchor: middle; }}
            .control-dot {{ fill: #333; }}
            .measurement {{ fill: #ff9800; stroke: #f57c00; stroke-width: 2; }}
        </style>"#).unwrap();

        // Draw qubit lines
        for qubit in 0..circuit.num_qubits {
            let y = 50 + qubit * 60;
            writeln!(svg, r#"<line x1="30" y1="{}" x2="{}" y2="{}" class="qubit-line"/>"#,
                    y, width - 30, y).unwrap();
            writeln!(svg, r#"<text x="15" y="{}" class="gate-text">q{}</text>"#, y + 5, qubit).unwrap();
        }

        // Draw gates
        for (layer_idx, layer) in circuit.layers.iter().enumerate() {
            let x = 60 + layer_idx * 80;

            for gate in &layer.gates {
                self.draw_gate_svg(&mut svg, gate, x);
            }
        }

        // Draw measurements
        for (&qubit, _) in &circuit.measurements {
            let x = width - 50;
            let y = 40 + qubit * 60;
            writeln!(svg, r#"<rect x="{}" y="{}" width="30" height="20" class="measurement"/>"#, x, y).unwrap();
            writeln!(svg, r#"<text x="{}" y="{}" class="gate-text">M</text>"#, x + 15, y + 15).unwrap();
        }

        writeln!(svg, "</svg>").unwrap();
        svg
    }

    fn render_html(&self, circuit: &QuantumCircuit) -> String {
        let mut html = String::new();

        writeln!(html, r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: 'Courier New', monospace; margin: 20px; }}
        .circuit-container {{ background: #f5f5f5; padding: 20px; border-radius: 8px; }}
        .circuit-header {{ font-weight: bold; margin-bottom: 10px; }}
        .qubit-line {{ margin: 5px 0; }}
        .gate {{ display: inline-block; width: 60px; text-align: center;
                 background: #e3f2fd; border: 1px solid #1976d2;
                 border-radius: 4px; margin: 0 2px; }}
        .control {{ background: #333; color: white; border-radius: 50%;
                   width: 20px; height: 20px; display: inline-block; }}
        .measurement {{ background: #ff9800; color: white; }}
    </style>
</head>
<body>
    <div class="circuit-container">
        <div class="circuit-header">Circuit: {}</div>
        <div>Qubits: {} | Depth: {} | Gates: {}</div>
        <br>"#, circuit.name, circuit.name,
               circuit.num_qubits, circuit.total_depth, circuit.gate_count()).unwrap();

        for qubit in 0..circuit.num_qubits {
            write!(html, r#"<div class="qubit-line">q{}: "#, qubit).unwrap();

            for layer in &circuit.layers {
                let gate_html = self.get_gate_html_for_qubit(layer, qubit);
                write!(html, "{}", gate_html).unwrap();
            }

            if circuit.measurements.contains_key(&qubit) {
                write!(html, r#"<span class="gate measurement">M</span>"#).unwrap();
            }

            writeln!(html, "</div>").unwrap();
        }

        writeln!(html, r#"    </div>
</body>
</html>"#).unwrap();

        html
    }

    fn get_gate_string_for_qubit(&self, layer: &CircuitLayer, qubit: QubitIndex) -> String {
        for gate in &layer.gates {
            if gate.qubits.contains(&qubit) {
                return match &gate.gate_type {
                    GateType::Identity => "I".to_string(),
                    GateType::PauliX => "X".to_string(),
                    GateType::PauliY => "Y".to_string(),
                    GateType::PauliZ => "Z".to_string(),
                    GateType::Hadamard => "H".to_string(),
                    GateType::Phase(angle) => format!("P({:.2})", angle),
                    GateType::RX(angle) => format!("RX({:.2})", angle),
                    GateType::RY(angle) => format!("RY({:.2})", angle),
                    GateType::RZ(angle) => format!("RZ({:.2})", angle),
                    GateType::T => "T".to_string(),
                    GateType::S => "S".to_string(),
                    GateType::CNOT => {
                        if gate.qubits[0] == qubit { "●".to_string() }
                        else { "⊕".to_string() }
                    },
                    GateType::CZ => {
                        if gate.qubits[0] == qubit { "●".to_string() }
                        else { "Z".to_string() }
                    },
                    GateType::SWAP => "✕".to_string(),
                    GateType::Toffoli => {
                        if gate.qubits[2] == qubit { "⊕".to_string() }
                        else { "●".to_string() }
                    },
                    _ => "G".to_string(),
                };
            }
        }
        "─".to_string()
    }

    fn get_gate_html_for_qubit(&self, layer: &CircuitLayer, qubit: QubitIndex) -> String {
        for gate in &layer.gates {
            if gate.qubits.contains(&qubit) {
                let gate_text = match &gate.gate_type {
                    GateType::Identity => "I",
                    GateType::PauliX => "X",
                    GateType::PauliY => "Y",
                    GateType::PauliZ => "Z",
                    GateType::Hadamard => "H",
                    GateType::T => "T",
                    GateType::S => "S",
                    GateType::CNOT => if gate.qubits[0] == qubit { "●" } else { "⊕" },
                    GateType::CZ => if gate.qubits[0] == qubit { "●" } else { "Z" },
                    GateType::SWAP => "✕",
                    _ => "G",
                };
                return format!(r#"<span class="gate">{}</span>"#, gate_text);
            }
        }
        r#"<span style="width: 60px; display: inline-block;">───</span>"#.to_string()
    }

    fn render_multi_qubit_gate_ascii(&self, gate: &QuantumGate, layer_idx: usize, _num_qubits: usize) -> String {
        let mut output = String::new();

        if gate.qubits.len() == 2 {
            let (min_qubit, max_qubit) = (
                *gate.qubits.iter().min().unwrap(),
                *gate.qubits.iter().max().unwrap()
            );

            // Draw vertical connection
            for _q in (min_qubit + 1)..max_qubit {
                for _ in 0..layer_idx {
                    output.push_str("        ");
                }
                output.push_str("   │    ");
                output.push('\n');
            }
        }

        output
    }

    fn place_gate_on_grid(&self, grid: &mut Vec<Vec<char>>, gate: &QuantumGate, col_start: usize) {
        for &qubit in &gate.qubits {
            let row = qubit * 2;
            let gate_char = match &gate.gate_type {
                GateType::Hadamard => 'H',
                GateType::PauliX => 'X',
                GateType::PauliY => 'Y',
                GateType::PauliZ => 'Z',
                GateType::CNOT => if gate.qubits[0] == qubit { '●' } else { '⊕' },
                _ => 'G',
            };

            if col_start < grid[row].len() {
                grid[row][col_start] = gate_char;
            }
        }

        // Draw connections for multi-qubit gates
        if gate.qubits.len() > 1 {
            let min_qubit = *gate.qubits.iter().min().unwrap();
            let max_qubit = *gate.qubits.iter().max().unwrap();

            for q in (min_qubit + 1)..max_qubit {
                let row = q * 2 - 1;
                if col_start < grid[row].len() {
                    grid[row][col_start] = '│';
                }
            }
        }
    }

    fn draw_gate_svg(&self, svg: &mut String, gate: &QuantumGate, x: usize) {
        match &gate.gate_type {
            GateType::CNOT => {
                let control_y = 50 + gate.qubits[0] * 60;
                let target_y = 50 + gate.qubits[1] * 60;

                // Control dot
                writeln!(svg, r#"<circle cx="{}" cy="{}" r="4" class="control-dot"/>"#, x, control_y).unwrap();

                // Target
                writeln!(svg, "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"none\" stroke=\"#333\" stroke-width=\"2\"/>", x, target_y).unwrap();
                writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>",
                        x - 7, target_y, x + 7, target_y).unwrap();
                writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>",
                        x, target_y - 7, x, target_y + 7).unwrap();

                // Connection line
                let min_y = std::cmp::min(control_y, target_y);
                let max_y = std::cmp::max(control_y, target_y);
                writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>",
                        x, min_y, x, max_y).unwrap();
            }
            _ => {
                // Single qubit gate
                if !gate.qubits.is_empty() {
                    let y = 40 + gate.qubits[0] * 60;
                    let gate_text = match &gate.gate_type {
                        GateType::Hadamard => "H",
                        GateType::PauliX => "X",
                        GateType::PauliY => "Y",
                        GateType::PauliZ => "Z",
                        GateType::T => "T",
                        GateType::S => "S",
                        _ => "G",
                    };

                    writeln!(svg, r#"<rect x="{}" y="{}" width="30" height="20" class="gate-box"/>"#, x - 15, y).unwrap();
                    writeln!(svg, r#"<text x="{}" y="{}" class="gate-text">{}</text>"#, x, y + 15, gate_text).unwrap();
                }
            }
        }
    }
}

// State visualization
pub struct StateRenderer {
    config: VisualizationConfig,
}

impl StateRenderer {
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    pub fn render_state(&self, state: &QuantumState) -> String {
        match self.config.style {
            RenderStyle::ASCII | RenderStyle::Unicode => self.render_state_text(state),
            RenderStyle::HTML => self.render_state_html(state),
            RenderStyle::SVG => self.render_state_svg(state),
        }
    }

    fn render_state_text(&self, state: &QuantumState) -> String {
        let mut output = String::new();

        writeln!(output, "Quantum State ({} qubits):", state.num_qubits).unwrap();
        writeln!(output, "{}",  "─".repeat(50)).unwrap();

        let probabilities = state.probabilities();

        for (i, (amplitude, probability)) in state.amplitudes.iter().zip(probabilities.iter()).enumerate() {
            if amplitude.norm() > state.sparse_threshold {
                let binary = format!("{:0width$b}", i, width = state.num_qubits);
                writeln!(output, "|{}⟩: {:.6} + {:.6}i (p = {:.4})",
                        binary, amplitude.re, amplitude.im, probability).unwrap();
            }
        }

        if !state.is_normalized {
            writeln!(output, "⚠ State is not normalized!").unwrap();
        }

        output
    }

    fn render_state_html(&self, state: &QuantumState) -> String {
        let mut html = String::new();

        writeln!(html, r#"<!DOCTYPE html>
<html>
<head>
    <title>Quantum State Visualization</title>
    <style>
        body {{ font-family: monospace; margin: 20px; }}
        .state-container {{ background: #f8f9fa; padding: 20px; border-radius: 8px; }}
        .amplitude {{ margin: 2px 0; padding: 5px; background: white; border-radius: 4px; }}
        .probability-bar {{ height: 20px; background: linear-gradient(90deg, #4caf50, #8bc34a);
                           border-radius: 4px; display: inline-block; }}
        .basis-state {{ font-weight: bold; color: #1976d2; }}
    </style>
</head>
<body>
    <div class="state-container">
        <h2>Quantum State ({} qubits)</h2>"#, state.num_qubits).unwrap();

        let probabilities = state.probabilities();

        for (i, (amplitude, probability)) in state.amplitudes.iter().zip(probabilities.iter()).enumerate() {
            if amplitude.norm() > state.sparse_threshold {
                let binary = format!("{:0width$b}", i, width = state.num_qubits);
                let bar_width = (probability * 200.0) as usize;

                writeln!(html, r#"        <div class="amplitude">
            <span class="basis-state">|{}⟩</span>:
            {:.6} + {:.6}i
            <div class="probability-bar" style="width: {}px;"></div>
            <span>(p = {:.4})</span>
        </div>"#, binary, amplitude.re, amplitude.im, bar_width, probability).unwrap();
            }
        }

        writeln!(html, r#"    </div>
</body>
</html>"#).unwrap();

        html
    }

    fn render_state_svg(&self, state: &QuantumState) -> String {
        let mut svg = String::new();
        let width = 800;
        let height = std::cmp::max(400, state.amplitudes.len() * 30 + 100);

        writeln!(svg, r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#, width, height).unwrap();

        // Draw probability bars
        let probabilities = state.probabilities();
        let mut y = 50;

        for (i, (amplitude, probability)) in state.amplitudes.iter().zip(probabilities.iter()).enumerate() {
            if amplitude.norm() > state.sparse_threshold {
                let binary = format!("{:0width$b}", i, width = state.num_qubits);
                let bar_width = (probability * 300.0) as usize;

                writeln!(svg, "<text x=\"10\" y=\"{}\" font-family=\"monospace\" font-size=\"12\">|{}⟩</text>", y, binary).unwrap();
                writeln!(svg, "<rect x=\"100\" y=\"{}\" width=\"{}\" height=\"15\" fill=\"#4caf50\" opacity=\"0.8\"/>", y - 12, bar_width).unwrap();
                writeln!(svg, "<text x=\"420\" y=\"{}\" font-family=\"monospace\" font-size=\"10\">{:.6} + {:.6}i (p = {:.4})</text>",
                        y, amplitude.re, amplitude.im, probability).unwrap();

                y += 25;
            }
        }

        writeln!(svg, "</svg>").unwrap();
        svg
    }

    pub fn render_histogram(&self, measurements: &[(usize, usize)]) -> String {
        let mut output = String::new();

        writeln!(output, "Measurement Histogram:").unwrap();
        writeln!(output, "{}", "─".repeat(50)).unwrap();

        let total_shots: usize = measurements.iter().map(|(_, count)| count).sum();

        for &(state, count) in measurements {
            let probability = count as f64 / total_shots as f64;
            let bar_length = (probability * 40.0) as usize;
            let bar = "█".repeat(bar_length);

            writeln!(output, "{:3}: {:>6} │{:<40}│ {:.3}%",
                    state, count, bar, probability * 100.0).unwrap();
        }

        output
    }
}

// Utility functions
pub fn draw_circuit(circuit: &QuantumCircuit) -> String {
    let renderer = CircuitRenderer::new(VisualizationConfig::default());
    renderer.render_circuit(circuit)
}

pub fn draw_circuit_ascii(circuit: &QuantumCircuit) -> String {
    let mut config = VisualizationConfig::default();
    config.style = RenderStyle::ASCII;
    let renderer = CircuitRenderer::new(config);
    renderer.render_circuit(circuit)
}

pub fn draw_state(state: &QuantumState) -> String {
    let renderer = StateRenderer::new(VisualizationConfig::default());
    renderer.render_state(state)
}

pub fn export_circuit_svg(circuit: &QuantumCircuit, filename: &str) -> std::io::Result<()> {
    let mut config = VisualizationConfig::default();
    config.style = RenderStyle::SVG;
    let renderer = CircuitRenderer::new(config);
    let svg_content = renderer.render_circuit(circuit);
    std::fs::write(filename, svg_content)
}

pub fn export_state_html(state: &QuantumState, filename: &str) -> std::io::Result<()> {
    let mut config = VisualizationConfig::default();
    config.style = RenderStyle::HTML;
    let renderer = StateRenderer::new(config);
    let html_content = renderer.render_state(state);
    std::fs::write(filename, html_content)
}

// Interactive visualization (placeholder for GUI integration)
pub struct InteractiveVisualizer {
    circuit: Option<QuantumCircuit>,
    state: Option<QuantumState>,
}

impl InteractiveVisualizer {
    pub fn new() -> Self {
        Self {
            circuit: None,
            state: None,
        }
    }

    pub fn load_circuit(&mut self, circuit: QuantumCircuit) {
        self.circuit = Some(circuit);
    }

    pub fn load_state(&mut self, state: QuantumState) {
        self.state = Some(state);
    }

    pub fn animate_execution(&self) -> Vec<String> {
        // Returns frames for animation
        let mut frames = Vec::new();

        if let Some(ref circuit) = self.circuit {
            let renderer = CircuitRenderer::new(VisualizationConfig::default());

            // Create frames for each layer execution
            for i in 0..=circuit.layers.len() {
                let mut partial_circuit = QuantumCircuit::new(circuit.num_qubits);
                partial_circuit.name = format!("{} (Step {})", circuit.name, i);

                for j in 0..i {
                    if j < circuit.layers.len() {
                        for gate in &circuit.layers[j].gates {
                            partial_circuit.layers.push(crate::quantum::circuit::CircuitLayer {
                                gates: vec![gate.clone()],
                                depth: j,
                                parallel_gates: Vec::new(),
                            });
                        }
                    }
                }

                frames.push(renderer.render_circuit(&partial_circuit));
            }
        }

        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum::circuit::CircuitBuilder;

    #[test]
    fn test_circuit_visualization() {
        let circuit = CircuitBuilder::new(2)
            .h(0)
            .cnot(0, 1)
            .measure_all()
            .build();

        let ascii_output = draw_circuit_ascii(&circuit);
        assert!(ascii_output.contains("H"));
        assert!(ascii_output.contains("CNOT") || ascii_output.contains("●"));
    }

    #[test]
    fn test_state_visualization() {
        let state = QuantumState::new(2);
        let output = draw_state(&state);
        assert!(output.contains("|00⟩"));
        assert!(output.contains("1.000000"));
    }

    #[test]
    fn test_export_functions() {
        let circuit = CircuitBuilder::new(1).h(0).build();

        // Test that export functions don't panic (actual file I/O tested separately)
        let svg_content = {
            let mut config = VisualizationConfig::default();
            config.style = RenderStyle::SVG;
            let renderer = CircuitRenderer::new(config);
            renderer.render_circuit(&circuit)
        };

        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("</svg>"));
    }
}
