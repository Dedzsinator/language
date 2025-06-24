// Main entry point for the Physics Simulation GUI
// Launches the Unity-style egui-based physics editor

mod gui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch the Unity-style physics editor GUI
    gui::launch_physics_editor()
}
