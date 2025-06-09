use super::*;
use std::path::Path;

/// Manages multiple scenes and scene operations
pub struct SceneManager {
    pub scenes: Vec<Scene>,
    pub current_scene: usize,
    pub scene_files: Vec<String>,
}

impl SceneManager {
    pub fn new() -> Self {
        let mut manager = Self {
            scenes: Vec::new(),
            current_scene: 0,
            scene_files: Vec::new(),
        };

        // Create default scene
        manager.create_new_scene("Main Scene".to_string(), false);
        manager
    }

    pub fn create_new_scene(&mut self, name: String, is_2d: bool) -> usize {
        let scene = Scene::new(name, is_2d);
        self.scenes.push(scene);
        self.scene_files.push(String::new()); // No file path yet
        self.scenes.len() - 1
    }

    pub fn current_scene(&self) -> Option<&Scene> {
        self.scenes.get(self.current_scene)
    }

    pub fn current_scene_mut(&mut self) -> Option<&mut Scene> {
        self.scenes.get_mut(self.current_scene)
    }

    pub fn switch_scene(&mut self, index: usize) {
        if index < self.scenes.len() {
            self.current_scene = index;
        }
    }

    pub fn save_scene(&mut self, index: usize, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(scene) = self.scenes.get(index) {
            let serialized = serde_json::to_string_pretty(scene)?;
            std::fs::write(path, serialized)?;
            if index < self.scene_files.len() {
                self.scene_files[index] = path.to_string();
            }
        }
        Ok(())
    }

    pub fn load_scene(&mut self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let scene: Scene = serde_json::from_str(&content)?;

        self.scenes.push(scene);
        self.scene_files.push(path.to_string());
        Ok(self.scenes.len() - 1)
    }

    pub fn duplicate_scene(&mut self, index: usize) -> Option<usize> {
        if let Some(scene) = self.scenes.get(index).cloned() {
            let mut new_scene = scene;
            new_scene.name = format!("{} Copy", new_scene.name);
            self.scenes.push(new_scene);
            self.scene_files.push(String::new());
            Some(self.scenes.len() - 1)
        } else {
            None
        }
    }

    pub fn delete_scene(&mut self, index: usize) -> bool {
        if self.scenes.len() > 1 && index < self.scenes.len() {
            self.scenes.remove(index);
            self.scene_files.remove(index);

            // Adjust current scene index if necessary
            if self.current_scene >= index && self.current_scene > 0 {
                self.current_scene -= 1;
            }
            true
        } else {
            false
        }
    }

    pub fn show_scene_manager_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Scene Manager")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New 2D Scene").clicked() {
                        self.create_new_scene("New 2D Scene".to_string(), true);
                    }
                    if ui.button("New 3D Scene").clicked() {
                        self.create_new_scene("New 3D Scene".to_string(), false);
                    }
                });

                ui.separator();

                ui.label("Scenes:");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let current_scene = self.current_scene;
                    let mut scene_to_switch = None;

                    for (index, scene) in self.scenes.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_current = index == current_scene;
                            let button_text = if is_current {
                                format!("● {}", scene.name)
                            } else {
                                scene.name.clone()
                            };

                            if ui.selectable_label(is_current, &button_text).clicked() {
                                scene_to_switch = Some(index);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🗑").clicked() {
                                    self.delete_scene(index);
                                }
                                if ui.small_button("📋").clicked() {
                                    self.duplicate_scene(index);
                                }
                                if ui.small_button("💾").clicked() {
                                    // TODO: Open file dialog
                                    let _ = self.save_scene(index, &format!("{}.scene", scene.name));
                                }
                            });
                        });
                    }

                    // Handle scene switch outside the iterator
                    if let Some(index) = scene_to_switch {
                        self.switch_scene(index);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Load Scene").clicked() {
                        // TODO: Open file dialog
                    }
                    if ui.button("Save All").clicked() {
                        let indices_to_save: Vec<_> = self.scenes.iter().enumerate()
                            .filter_map(|(index, _)| {
                                if !self.scene_files[index].is_empty() {
                                    Some((index, self.scene_files[index].clone()))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for (index, path) in indices_to_save {
                            let _ = self.save_scene(index, &path);
                        }
                    }
                });
            });
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}
