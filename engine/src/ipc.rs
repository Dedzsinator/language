// Inter-Process Communication module for Matrix Language integration
// Handles data exchange between Matrix Language @sim directives and the GUI

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

/// Physics simulation data that can be sent from Matrix Language to the GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSimulationData {
    pub time_points: Vec<f64>,
    pub objects: Vec<SimulationObject>,
    pub metadata: SimulationMetadata,
}

/// Individual object in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationObject {
    pub id: u32,
    pub name: String,
    pub positions: Vec<(f64, f64, f64)>,  // Position over time
    pub velocities: Vec<(f64, f64, f64)>, // Velocity over time
    pub mass: f64,
    pub shape: ObjectShape,
}

/// Object shape information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectShape {
    Sphere { radius: f64 },
    Box { width: f64, height: f64, depth: f64 },
    Plane { width: f64, height: f64 },
}

/// Simulation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    pub total_time: f64,
    pub time_step: f64,
    pub gravity: (f64, f64, f64),
    pub simulation_id: String,
    pub created_at: String,
}

/// IPC Manager for handling data exchange
pub struct IpcManager {
    data_file_path: String,
    shared_data: Arc<Mutex<Option<PhysicsSimulationData>>>,
}

impl IpcManager {
    pub fn new() -> Self {
        let data_file_path = if cfg!(target_os = "windows") {
            "C:/tmp/matrix_lang_simulation_data.json".to_string()
        } else {
            "/tmp/matrix_lang_simulation_data.json".to_string()
        };

        Self {
            data_file_path,
            shared_data: Arc::new(Mutex::new(None)),
        }
    }

    /// Check for new simulation data from Matrix Language
    pub fn check_for_simulation_data(&self) -> Option<PhysicsSimulationData> {
        if Path::new(&self.data_file_path).exists() {
            match fs::read_to_string(&self.data_file_path) {
                Ok(content) => {
                    match serde_json::from_str::<PhysicsSimulationData>(&content) {
                        Ok(data) => {
                            // Update shared data
                            if let Ok(mut shared) = self.shared_data.lock() {
                                *shared = Some(data.clone());
                            }

                            // Clean up the file after reading
                            let _ = fs::remove_file(&self.data_file_path);

                            Some(data)
                        }
                        Err(e) => {
                            eprintln!("Failed to parse simulation data: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read simulation data file: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Get the latest simulation data
    pub fn get_latest_data(&self) -> Option<PhysicsSimulationData> {
        if let Ok(shared) = self.shared_data.lock() {
            shared.clone()
        } else {
            None
        }
    }

    /// Clear the current simulation data
    pub fn clear_data(&self) {
        if let Ok(mut shared) = self.shared_data.lock() {
            *shared = None;
        }
        // Also remove the file if it exists
        let _ = fs::remove_file(&self.data_file_path);
    }

    /// Write simulation data (used by Matrix Language)
    pub fn write_simulation_data(&self, data: &PhysicsSimulationData) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = serde_json::to_string_pretty(data)?;

        // Ensure the directory exists
        if let Some(parent) = Path::new(&self.data_file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.data_file_path, json_content)?;
        Ok(())
    }
}

/// Convert engine animation data to IPC format
impl From<crate::gui::PhysicsAnimationData> for PhysicsSimulationData {
    fn from(animation_data: crate::gui::PhysicsAnimationData) -> Self {
        let mut objects = Vec::new();

        for (obj_idx, positions) in animation_data.position_data.iter().enumerate() {
            let velocities = animation_data.velocity_data.get(obj_idx)
                .cloned()
                .unwrap_or_else(|| vec![(0.0, 0.0, 0.0); positions.len()]);

            objects.push(SimulationObject {
                id: obj_idx as u32,
                name: format!("Object {}", obj_idx + 1),
                positions: positions.clone(),
                velocities,
                mass: 1.0,
                shape: ObjectShape::Sphere { radius: 0.5 },
            });
        }

        PhysicsSimulationData {
            time_points: animation_data.time_points.clone(),
            objects,
            metadata: SimulationMetadata {
                total_time: animation_data.time_points.last().copied().unwrap_or(0.0),
                time_step: 1.0 / 60.0,
                gravity: (0.0, -9.81, 0.0),
                simulation_id: format!("engine_generated_{}", chrono::Utc::now().timestamp()),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}

/// Convert IPC format to engine animation data
impl From<PhysicsSimulationData> for crate::gui::PhysicsAnimationData {
    fn from(sim_data: PhysicsSimulationData) -> Self {
        let mut position_data = Vec::new();
        let mut velocity_data = Vec::new();

        for object in sim_data.objects {
            position_data.push(object.positions);
            velocity_data.push(object.velocities);
        }

        crate::gui::PhysicsAnimationData {
            time_points: sim_data.time_points,
            position_data,
            velocity_data,
            current_time: 0.0,
            is_playing: false,
            playback_speed: 1.0,
        }
    }
}
