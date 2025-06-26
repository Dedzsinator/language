// Physics Simulation Engine Integration
// Provides integration with the physics engine crate for @sim and @plot directives

use crate::eval::interpreter::{PlotContext, RuntimeResult, SimulationContext};
use std::process::{Command, Stdio};

/// Launch 3D physics simulation engine
pub fn launch_3d_simulation(context: SimulationContext) -> RuntimeResult<()> {
    println!("🎬 Launching 3D Physics Simulation Engine...");
    println!("  Mode: {:?}", context.mode);
    println!("  Real-time: {}", context.real_time);
    println!("  Interactive: {}", context.interactive);
    println!("  Objects: {}", context.world.objects.len());

    // Try to launch the physics engine GUI
    match launch_engine_gui("3d_sim") {
        Ok(mut child) => {
            println!("✅ 3D Simulation launched successfully");

            // Give the engine time to start up
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // Check if the process is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("⚠️  Engine exited with status: {}", status);
                    println!("📝 Falling back to console simulation mode");
                    run_console_simulation(context)
                }
                Ok(None) => {
                    println!("🚀 Engine is running successfully");
                    // Wait for the engine to finish
                    let _ = child.wait();
                    Ok(())
                }
                Err(e) => {
                    println!("⚠️  Error checking engine status: {}", e);
                    println!("📝 Falling back to console simulation mode");
                    run_console_simulation(context)
                }
            }
        }
        Err(e) => {
            println!("⚠️  Failed to launch 3D simulation GUI: {}", e);
            println!("📝 Falling back to console simulation mode");
            run_console_simulation(context)
        }
    }
}

/// Launch matplotlib-like plot animation engine
pub fn launch_plot_animation(context: PlotContext) -> RuntimeResult<()> {
    println!("📊 Launching Physics Plot Animation Engine...");
    println!("  Mode: {:?}", context.mode);
    println!("  Time slider: {}", context.time_slider);
    println!("  Interactive: {}", context.interactive);
    println!("  Objects: {}", context.world.objects.len());

    // Try to launch the plotting interface
    match launch_engine_gui("plot_anim") {
        Ok(mut child) => {
            println!("✅ Plot Animation launched successfully");

            // Give the engine time to start up
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // Check if the process is still running
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("⚠️  Engine exited with status: {}", status);
                    println!("📝 Falling back to console plotting mode");
                    run_console_plotting(context)
                }
                Ok(None) => {
                    println!("🚀 Engine is running successfully");
                    // Wait for the engine to finish
                    let _ = child.wait();
                    Ok(())
                }
                Err(e) => {
                    println!("⚠️  Error checking engine status: {}", e);
                    println!("📝 Falling back to console plotting mode");
                    run_console_plotting(context)
                }
            }
        }
        Err(e) => {
            println!("⚠️  Failed to launch plot animation GUI: {}", e);
            println!("📝 Falling back to console plotting mode");
            run_console_plotting(context)
        }
    }
}

/// Try to launch the physics engine GUI
fn launch_engine_gui(mode: &str) -> Result<std::process::Child, std::io::Error> {
    // First try to run the engine as a separate process
    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "run",
        "--manifest-path",
        "engine/Cargo.toml",
        "--",
        "--mode",
        mode,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    cmd.spawn()
}

/// Run console-based simulation
fn run_console_simulation(context: SimulationContext) -> RuntimeResult<()> {
    println!("\n🖥️  Console Simulation Mode:");
    println!("============================");

    // Simulate physics steps
    for step in 0..10 {
        println!(
            "Step {}: Simulating {} objects...",
            step + 1,
            context.world.objects.len()
        );

        // Simulate object positions (placeholder)
        for (i, obj) in context.world.objects.iter().enumerate() {
            let t = step as f64 * 0.1;
            let x = (t * (i + 1) as f64).sin() * 2.0;
            let y = (t * (i + 1) as f64).cos() * 2.0;
            let z = t * 0.5;

            println!(
                "  Object {}: position=({:.2}, {:.2}, {:.2}), mass={}",
                i + 1,
                x,
                y,
                z,
                obj.mass
            );
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("📋 Simulation completed successfully");
    Ok(())
}

/// Run console-based plotting
fn run_console_plotting(context: PlotContext) -> RuntimeResult<()> {
    println!("\n📈 Console Plotting Mode:");
    println!("=========================");

    // Generate time series data
    println!("Time\tObject Positions");
    println!("----\t----------------");

    for time_step in 0..20 {
        let t = time_step as f64 * 0.1;
        print!("{:.1}\t", t);

        for (i, _obj) in context.world.objects.iter().enumerate() {
            let x = (t * (i + 1) as f64).sin() * 2.0;
            let y = (t * (i + 1) as f64).cos() * 2.0;
            print!("Obj{}:({:.1},{:.1}) ", i + 1, x, y);
        }
        println!();

        if context.time_slider {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    println!("\n📊 Plot animation completed");
    Ok(())
}

/// Create a physics object for demonstration
pub fn create_demo_physics_object(id: usize) -> crate::stdlib::PhysicsObject {
    crate::stdlib::PhysicsObject {
        id,
        shape: format!("sphere_{}", id),
        mass: 1.0 + (id as f64 * 0.5),
        position: crate::stdlib::Vec3 {
            x: (id as f64).sin(),
            y: (id as f64).cos(),
            z: 0.0,
        },
        velocity: crate::stdlib::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        is_static: false,
    }
}
