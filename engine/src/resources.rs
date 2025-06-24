// ECS Resources for Physics Engine
use bevy_ecs::prelude::*;
use crate::physics::math::*;
use crate::physics::spatial::SpatialHash;
use std::collections::HashMap;

/// Global physics configuration
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub time_step: f64,
    pub max_velocity: f64,
    pub solver_iterations: usize,
    pub collision_margin: f64,
    pub sleep_threshold: f64,
    pub enable_sleeping: bool,
    pub enable_continuous_detection: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 1.0 / 60.0,
            max_velocity: 100.0,
            solver_iterations: 10,
            collision_margin: 0.01,
            sleep_threshold: 0.01,
            enable_sleeping: true,
            enable_continuous_detection: false,
        }
    }
}

/// Time management resource
#[derive(Resource, Debug, Clone)]
pub struct Time {
    pub elapsed: f64,
    pub delta: f64,
    pub real_time: f64,
    pub time_scale: f64,
    pub paused: bool,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            delta: 1.0 / 60.0,
            real_time: 0.0,
            time_scale: 1.0,
            paused: false,
        }
    }
}

impl Time {
    pub fn update(&mut self, real_delta: f64) {
        self.real_time += real_delta;
        if !self.paused {
            let scaled_delta = real_delta * self.time_scale;
            self.delta = scaled_delta;
            self.elapsed += scaled_delta;
        } else {
            self.delta = 0.0;
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }
}

/// Spatial indexing resource for efficient collision detection
#[derive(Resource, Debug)]
pub struct SpatialIndex {
    pub spatial_hash: SpatialHash,
    pub dirty: bool,
}

impl SpatialIndex {
    pub fn new(cell_size: f64) -> Self {
        Self {
            spatial_hash: SpatialHash::new(cell_size),
            dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.spatial_hash.clear();
        self.dirty = false;
    }
}

/// Statistics and profiling data
#[derive(Resource, Debug, Default)]
pub struct PhysicsStats {
    pub frame_count: u64,
    pub rigid_body_count: usize,
    pub soft_body_count: usize,
    pub fluid_particle_count: usize,
    pub constraint_count: usize,
    pub collision_count: usize,
    pub integration_time_ms: f64,
    pub collision_time_ms: f64,
    pub solver_time_ms: f64,
    pub total_time_ms: f64,
}

impl PhysicsStats {
    pub fn new_frame(&mut self) {
        self.frame_count += 1;
        self.collision_count = 0;
        self.integration_time_ms = 0.0;
        self.collision_time_ms = 0.0;
        self.solver_time_ms = 0.0;
        self.total_time_ms = 0.0;
    }

    pub fn average_fps(&self) -> f64 {
        if self.total_time_ms > 0.0 {
            1000.0 / (self.total_time_ms / self.frame_count as f64)
        } else {
            0.0
        }
    }
}

/// Performance profiler resource
#[derive(Resource, Debug)]
pub struct Profiler {
    pub timers: HashMap<String, f64>,
    pub start_times: HashMap<String, std::time::Instant>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            timers: HashMap::new(),
            start_times: HashMap::new(),
        }
    }
}

impl Profiler {
    pub fn start_timer(&mut self, name: &str) {
        self.start_times.insert(name.to_string(), std::time::Instant::now());
    }

    pub fn end_timer(&mut self, name: &str) -> f64 {
        if let Some(start_time) = self.start_times.remove(name) {
            let elapsed = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
            self.timers.insert(name.to_string(), elapsed);
            elapsed
        } else {
            0.0
        }
    }

    pub fn get_time(&self, name: &str) -> f64 {
        self.timers.get(name).copied().unwrap_or(0.0)
    }

    pub fn clear(&mut self) {
        self.timers.clear();
        self.start_times.clear();
    }
}

/// Physics material library
#[derive(Resource, Debug, Default)]
pub struct MaterialLibrary {
    pub materials: HashMap<String, MaterialProperties>,
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub density: f64,
    pub young_modulus: f64,
    pub poisson_ratio: f64,
    pub restitution: f64,
    pub friction_static: f64,
    pub friction_kinetic: f64,
    pub damping: f64,
    pub thermal_properties: ThermalProperties,
    pub electrical_properties: ElectricalProperties,
}

#[derive(Debug, Clone)]
pub struct ThermalProperties {
    pub thermal_conductivity: f64,
    pub specific_heat: f64,
    pub thermal_expansion: f64,
}

#[derive(Debug, Clone)]
pub struct ElectricalProperties {
    pub resistivity: f64,
    pub permittivity: f64,
    pub permeability: f64,
}

impl MaterialLibrary {
    pub fn new() -> Self {
        let mut library = Self::default();
        library.register_common_materials();
        library
    }

    fn register_common_materials(&mut self) {
        // Steel
        self.materials.insert("steel".to_string(), MaterialProperties {
            density: 7850.0,
            young_modulus: 200e9,
            poisson_ratio: 0.3,
            restitution: 0.4,
            friction_static: 0.7,
            friction_kinetic: 0.6,
            damping: 0.99,
            thermal_properties: ThermalProperties {
                thermal_conductivity: 50.0,
                specific_heat: 490.0,
                thermal_expansion: 12e-6,
            },
            electrical_properties: ElectricalProperties {
                resistivity: 1.7e-7,
                permittivity: 8.854e-12,
                permeability: 1.26e-6,
            },
        });

        // Rubber
        self.materials.insert("rubber".to_string(), MaterialProperties {
            density: 1200.0,
            young_modulus: 1e6,
            poisson_ratio: 0.49,
            restitution: 0.9,
            friction_static: 1.2,
            friction_kinetic: 1.0,
            damping: 0.95,
            thermal_properties: ThermalProperties {
                thermal_conductivity: 0.16,
                specific_heat: 1500.0,
                thermal_expansion: 200e-6,
            },
            electrical_properties: ElectricalProperties {
                resistivity: 1e13,
                permittivity: 8.854e-12 * 3.0,
                permeability: 1.26e-6,
            },
        });

        // Water
        self.materials.insert("water".to_string(), MaterialProperties {
            density: 1000.0,
            young_modulus: 2.2e9,
            poisson_ratio: 0.5,
            restitution: 0.1,
            friction_static: 0.0,
            friction_kinetic: 0.0,
            damping: 0.98,
            thermal_properties: ThermalProperties {
                thermal_conductivity: 0.6,
                specific_heat: 4186.0,
                thermal_expansion: 214e-6,
            },
            electrical_properties: ElectricalProperties {
                resistivity: 1.8e5,
                permittivity: 8.854e-12 * 81.0,
                permeability: 1.26e-6,
            },
        });
    }

    pub fn get_material(&self, name: &str) -> Option<&MaterialProperties> {
        self.materials.get(name)
    }

    pub fn add_material(&mut self, name: String, properties: MaterialProperties) {
        self.materials.insert(name, properties);
    }
}

/// Event system for physics callbacks
#[derive(Debug, Clone)]
pub enum PhysicsEvent {
    Collision {
        entity_a: bevy_ecs::entity::Entity,
        entity_b: bevy_ecs::entity::Entity,
        contact_point: Vec3,
        normal: Vec3,
        impulse: f64,
    },
    ConstraintBroken {
        entity: bevy_ecs::entity::Entity,
        constraint_type: String,
    },
    TriggerEntered {
        trigger: bevy_ecs::entity::Entity,
        entity: bevy_ecs::entity::Entity,
    },
    TriggerExited {
        trigger: bevy_ecs::entity::Entity,
        entity: bevy_ecs::entity::Entity,
    },
    SleepStateChanged {
        entity: bevy_ecs::entity::Entity,
        sleeping: bool,
    },
}

/// Events resource for this frame
#[derive(Resource, Debug, Default)]
pub struct PhysicsEvents {
    pub events: Vec<PhysicsEvent>,
}

impl PhysicsEvents {
    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn push(&mut self, event: PhysicsEvent) {
        self.events.push(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &PhysicsEvent> {
        self.events.iter()
    }
}
