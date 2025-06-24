// Physics Simulation GUI Engine Library
// Provides Unity-style physics simulation interface as a standalone library

// GUI Components
pub mod animation_view;
pub mod console;
pub mod game_view;
pub mod gui;
pub mod inspector;
pub mod object_hierarchy;
pub mod project_browser;
pub mod scene_manager;
pub mod scene_view;
pub mod scripting_panel;
pub mod unity_layout;
pub mod viewport;

// Physics Components (keeping only GUI-related physics)
pub mod physics_debugger;

// Re-exports for easy access
pub use gui::*;
pub use unity_layout::*;
pub use viewport::*;

/// Launch the Unity-style physics simulation GUI
pub fn launch_physics_gui() -> Result<(), Box<dyn std::error::Error>> {
    gui::launch_unity_simulation()
}

/// Get the version of the physics GUI engine
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
