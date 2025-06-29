// Main entry point for the Physics Simulation GUI
// Launches the Unity-style egui-based physics editor with Matrix Language directive support

use clap::{Arg, Command};
use physics_simulation_gui::gui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("physics-engine-gui")
        .version("0.1.0")
        .about("Unity-style Physics Simulation Engine with GUI")
        .arg(
            Arg::new("mode")
                .long("mode")
                .help("Specify the launch mode (3d_sim, plot_anim)")
                .value_name("MODE")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    if let Some(mode) = matches.get_one::<String>("mode") {
        launch_with_mode(mode)
    } else {
        // Default to standard GUI mode
        gui::launch_physics_editor()
    }
}

/// Launch with specific mode for Matrix Language directives
fn launch_with_mode(mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "3d_sim" => {
            println!("🎬 Launching 3D Physics Simulation mode...");
            // Launch GUI in simulation mode
            gui::launch_physics_editor()
        }
        "plot_anim" => {
            println!("📊 Launching Plot Animation mode...");
            // Launch GUI in plotting mode
            gui::launch_physics_editor()
        }
        _ => {
            eprintln!("Unknown mode: {}. Available modes: 3d_sim, plot_anim", mode);
            std::process::exit(1);
        }
    }
}
