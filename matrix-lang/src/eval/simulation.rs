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

    // Write simulation data to IPC file for GUI communication
    println!("📝 Writing simulation data to IPC...");
    if let Err(e) = write_simulation_data_to_ipc(&context) {
        println!("⚠️ Failed to write simulation data for GUI: {}", e);
    } else {
        println!("✅ Successfully wrote simulation data to IPC");
    }

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
    cmd.args([
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

/// Write simulation data to IPC file for GUI communication
fn write_simulation_data_to_ipc(
    context: &SimulationContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde::{Deserialize, Serialize};
    use std::fs;

    // Convert Matrix Language simulation context to IPC format
    #[derive(Serialize, Deserialize)]
    struct IpcSimulationData {
        time_points: Vec<f64>,
        objects: Vec<IpcSimulationObject>,
        metadata: IpcSimulationMetadata,
    }

    #[derive(Serialize, Deserialize)]
    struct IpcSimulationObject {
        id: u32,
        name: String,
        positions: Vec<(f64, f64, f64)>,
        velocities: Vec<(f64, f64, f64)>,
        mass: f64,
        shape: IpcObjectShape,
    }

    #[derive(Serialize, Deserialize)]
    enum IpcObjectShape {
        Sphere { radius: f64 },
        Box { width: f64, height: f64, depth: f64 },
        Plane { width: f64, height: f64 },
    }

    #[derive(Serialize, Deserialize)]
    struct IpcSimulationMetadata {
        total_time: f64,
        time_step: f64,
        gravity: (f64, f64, f64),
        simulation_id: String,
        created_at: String,
    }

    // Generate time points (simulate physics over time)
    let time_step = 1.0 / 60.0; // 60 FPS
    let total_time = 10.0; // 10 seconds
    let mut time_points = Vec::new();
    for i in 0..=(total_time / time_step) as i32 {
        time_points.push(i as f64 * time_step);
    }

    // Generate simulation objects
    let mut ipc_objects = Vec::new();
    for (obj_idx, obj) in context.world.objects.iter().enumerate() {
        let mut positions = Vec::new();
        let mut velocities = Vec::new();

        // Simulate physics over time for this object
        for &t in &time_points {
            // Simple falling object physics simulation
            let gravity = -9.81;
            let initial_height = obj.position.y;
            let initial_velocity = obj.velocity.y;

            // Physics equations: y = y0 + v0*t + 0.5*a*t^2
            let y = initial_height + initial_velocity * t + 0.5 * gravity * t * t;
            let vy = initial_velocity + gravity * t;

            // Add some horizontal motion for visual interest
            let x = obj.position.x + (t * 0.5).sin() * 2.0;
            let z = obj.position.z + (t * 0.3).cos() * 1.5;
            let vx = (t * 0.5).cos() * 0.5 * 2.0;
            let vz = -(t * 0.3).sin() * 0.3 * 1.5;

            positions.push((x, y.max(0.0), z)); // Don't go below ground
            velocities.push((vx, vy, vz));
        }

        ipc_objects.push(IpcSimulationObject {
            id: obj_idx as u32,
            name: format!("Object_{}", obj_idx),
            positions,
            velocities,
            mass: 1.0,
            shape: IpcObjectShape::Sphere { radius: 0.5 },
        });
    }

    // Create metadata
    let metadata = IpcSimulationMetadata {
        total_time,
        time_step,
        gravity: (0.0, -9.81, 0.0),
        simulation_id: format!("matrix_sim_{}", chrono::Utc::now().timestamp()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Create final data structure
    let sim_data = IpcSimulationData {
        time_points,
        objects: ipc_objects,
        metadata,
    };

    // Write to IPC file
    let data_file_path = if cfg!(target_os = "windows") {
        "C:/tmp/matrix_lang_simulation_data.json"
    } else {
        "/tmp/matrix_lang_simulation_data.json"
    };

    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(data_file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let json_content = serde_json::to_string_pretty(&sim_data)?;
    fs::write(data_file_path, json_content)?;

    println!("📊 Wrote simulation data to {}", data_file_path);

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
