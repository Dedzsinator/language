// Physics Simulation GUI - Main Entry Point
// Unity-style physics simulation interface

use clap::{Arg, Command};

mod gui;

fn main() {
    let _matches = Command::new("physics-gui")
        .about("Unity-style Physics Simulation GUI")
        .version("0.1.0")
        .arg(
            Arg::new("fullscreen")
                .long("fullscreen")
                .short('f')
                .help("Launch in fullscreen mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    println!("🎮 Physics Simulation GUI Engine v0.1.0");
    println!("========================================");

    let result = gui::launch_unity_simulation();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
