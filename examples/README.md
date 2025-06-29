# Matrix Language Examples

This directory contains example Matrix Language programs organized by category.

## Directory Structure

### `/tests/` - Basic Language Tests
Contains fundamental language feature tests including:
- Basic math operations (`test_math.matrix`, `test_math_float.matrix`)
- String handling (`str_test.matrix`, `string_concat_test.matrix`)
- Constants and variables (`test_constants.matrix`, `int_var_print.matrix`)
- Quantum computing features (`test_quantum.matrix`, `test_quantum_complete.matrix`)
- Debugging and development files (`debug_*.matrix`, `minimal_*.matrix`)

### `/physics_tests/` - Physics Simulation Tests
Contains physics simulation and directive tests:
- **Directive Tests**: `test_sim_directive.matrix`, `test_plot_directive.matrix`
- **Physics Animation**: `physics_animation_demo.matrix`
- **Integration Tests**: `test_directive_integration.matrix`
- **Physics Functions**: `test_physics_functions.matrix`

### `/optimization_tests/` - Performance Tests
Contains optimization and performance-related tests:
- `optimization_comprehensive_test.matrix`
- `optimization_test_comprehensive.matrix`
- `simple_optimization_test.matrix`

### `/comprehensive_tests/` - Full Feature Tests
Contains comprehensive test suites that test multiple language features:
- Final integration tests (`final_*.matrix`)
- Comprehensive test suites (`comprehensive_*.matrix`)
- Working test examples (`working_*.matrix`)

### `/demos/` - Demonstration Programs
Contains demonstration programs showcasing Matrix Language features:
- `directive_demo.matrix` - Shows @sim and @plot directive usage

### Physics Examples (Root Level)
- `physics_basic.matrix` - Basic physics simulation
- `physics_multi_object.matrix` - Multiple object physics
- `physics_pendulum.matrix` - Pendulum simulation
- `physics_tower.matrix` - Tower building simulation
- `physics_test.matrix` - General physics tests

## Running Examples

To run any example:

```bash
# From the project root
cargo run -- examples/path/to/file.matrix

# For physics examples that use @sim directive
cargo run -- examples/physics_tests/test_sim_directive.matrix

# For plot examples that use @plot directive
cargo run -- examples/physics_tests/test_plot_directive.matrix
```

## Matrix Language Directives

### @sim Directive
Creates 3D physics simulations:
```matrix
let simulation = @sim {
    let gravity = 9.81
    let mass = 2.5
    let height = 10.0
    let energy = mass * gravity * height
    energy
}
```

### @plot Directive
Creates animated plots and visualizations:
```matrix
let visualization = @plot {
    let time_points = [0.0, 1.0, 2.0, 3.0]
    let data = [1.0, 0.8, 0.6, 0.4]
    data
}
```

## Contributing

When adding new examples:
1. Place them in the appropriate subdirectory
2. Use descriptive filenames
3. Add comments explaining the example's purpose
4. Update this README if adding new categories
