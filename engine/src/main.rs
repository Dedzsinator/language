// Main entry point for the Physics Simulation GUI
// Launches the Unity-style egui-based physics editor

use physics_simulation_gui::launch_physics_editor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch the Unity-style physics editor GUI
    launch_physics_editor()
}
