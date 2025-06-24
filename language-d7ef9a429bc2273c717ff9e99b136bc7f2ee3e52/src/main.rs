// Physics Simulation GUI - Main Entry Point
// Unity-style physics simulation interface

use clap::{Arg, Command};
use physics_simulation_gui;

fn main() {
    let matches = Command::new("physics-gui")
        .about("Unity-style Physics Simulation GUI")
        .version("0.1.0")
        .arg(
            Arg::new("fullscreen")
                .long("fullscreen")
                .short('f')
                .help("Launch in fullscreen mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("viewport-only")
                .long("viewport-only")
                .help("Launch only the viewport without full GUI")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    println!("🎮 Physics Simulation GUI Engine v{}", physics_simulation_gui::version());
    println!("======================================");

    let result = if matches.get_flag("viewport-only") {
        launch_viewport_only()
    } else {
        physics_simulation_gui::launch_physics_gui()
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn launch_viewport_only() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️ Launching Viewport-Only Mode...");

    // This would launch just the 3D viewport without the full GUI
    // For now, we'll use the main GUI interface
    physics_simulation_gui::launch_physics_gui()
}
