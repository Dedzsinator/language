# Physics Simulation GUI Engine

A Unity-style physics simulation interface that provides a graphical environment for physics experiments and simulations. This GUI engine integrates with the Matrix Language for scripting physics scenarios.

## Features

- **Unity-style Interface**: Familiar GUI layout for game developers
- **3D Viewport**: Interactive 3D scene visualization
- **Physics Debugging**: Real-time physics state inspection
- **Object Hierarchy**: Scene object management
- **Animation Timeline**: Animation and simulation controls
- **Matrix Language Integration**: Script physics scenarios using Matrix Language

## Quick Start

```bash
# Build the physics GUI
cargo build --release

# Launch the GUI
cargo run

# Launch viewport-only mode
cargo run -- --viewport-only

# Launch in fullscreen
cargo run -- --fullscreen
```

## Dependencies

- **Matrix Language**: The core language runtime (as dependency)
- **egui**: Immediate mode GUI framework
- **nalgebra**: Linear algebra operations
- **tokio**: Async runtime for UI responsiveness

## Project Structure

```
src/
├── main.rs              # Entry point
├── gui.rs               # Main GUI interface
├── viewport.rs          # 3D viewport
├── unity_layout.rs      # Unity-style layout
├── scene_view.rs        # Scene management
├── game_view.rs         # Game view mode
├── animation_view.rs    # Animation controls
├── inspector.rs         # Object inspector
├── object_hierarchy.rs  # Object hierarchy
├── project_browser.rs   # Project browser
├── scene_manager.rs     # Scene management
├── console.rs           # Debug console
├── scripting_panel.rs   # Script editor
└── physics_debugger.rs  # Physics debugging
```

## Integration with Matrix Language

The GUI engine can load and execute Matrix Language scripts for physics simulation. Example:

```matrix
# Create physics world
let world = create_physics_world()
set_gravity(world, [0.0, -9.81, 0.0])

# Add objects
let sphere = add_rigid_body(world, "sphere", 1.0, [0.0, 10.0, 0.0])
let ground = add_rigid_body(world, "box", 0.0, [0.0, 0.0, 0.0])

# Run simulation
for i in 1..600 {
    physics_step(world)
}
```

## License

MIT OR Apache-2.0
