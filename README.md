# Matrix Language

**A modern JIT-compiled physics simulation language with Unity-style 3D engine**

Matrix Language is a specialized programming language designed for physics simulations, quantum computing, and mathematical modeling, featuring real-time 3D visualization and a powerful JIT compiler.

## 🚀 Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd matrix-lang

# Build the project
cargo build --release

# Run a physics simulation
cargo run --bin matrix-lang examples/physics_basic.matrix

# Launch the Unity-style GUI
cargo run --bin physics-gui
```

## ✨ Features

- **JIT Compilation**: Lightning-fast execution with LLVM backend
- **Physics Engine**: Real-time 3D physics simulation with interactive GUI
- **Quantum Computing**: Built-in quantum circuit simulation
- **Mathematical Toolkit**: Comprehensive math functions and constants
- **Unity-Style Interface**: Professional 3D editor with dockable panels
- **Cross-Platform**: Runs on Linux, macOS, and Windows

## 📁 Project Structure

```
matrix-lang/
├── matrix-lang/          # Core Matrix Language compiler
├── engine/              # Unity-style 3D physics engine
├── examples/            # Example programs and demos
├── tests/               # Comprehensive test suite
└── .github/             # CI/CD workflows
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run Matrix Language test suite
./tests/run_all_tests.sh

# Run specific test category
cargo test --test integration
```

## 📚 Language Examples

### Physics Simulation
```matrix
@sim {
    let ball = {
        position: [0.0, 5.0, 0.0],
        mass: 1.0,
        shape: "sphere"
    }

    physics_step(ball, 0.016)
}
```

### Quantum Computing
```matrix
let circuit = quantum_circuit(2)
h(circuit, 0)
cnot(circuit, 0, 1)
let result = measure_all(circuit)
```

### Mathematical Computing
```matrix
let pi_value = pi
let result = sin(pi_value / 4)
println("sin(π/4) = ", result)
```

## 🔧 Development

### Prerequisites
- Rust 1.70+
- LLVM 17 (for JIT compilation)
- System dependencies for GUI (see CI configuration)

### Building
```bash
cargo build --all --all-features
```

### Running Tests
```bash
# All tests
cargo test --all

# Specific test suite
cargo test --test matrix_language_integration_tests
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the test suite
5. Submit a pull request

See our CI/CD pipeline for automated testing requirements.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
