// Physics Simulation GUI Engine Library
// Provides Unity-style physics simulation interface as a standalone library

// Core GUI Module
pub mod gui;

// Re-exports for easy access
pub use gui::*;

/// Launch the Unity-style physics simulation GUI
pub fn launch_physics_gui() -> Result<(), Box<dyn std::error::Error>> {
    gui::launch_physics_editor()
}

/// Get the version of the physics GUI engine
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
