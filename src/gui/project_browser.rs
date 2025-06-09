use super::*;

/// Project browser panel for managing assets and files
pub struct ProjectBrowser {
    current_directory: String,
    file_tree: Vec<FileNode>,
    selected_file: Option<String>,
    show_hidden_files: bool,
    file_filter: String,
    favorites: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub enum DragPayload {
    File(String),
    GameObject(u32),
    Component(Component),
}

impl ProjectBrowser {
    pub fn new() -> Self {
        let mut browser = Self {
            current_directory: "Assets".to_string(),
            file_tree: Vec::new(),
            selected_file: None,
            show_hidden_files: false,
            file_filter: String::new(),
            favorites: Vec::new(),
        };

        browser.refresh_file_tree();
        browser
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("project_browser")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Project");

                // Toolbar
                self.show_toolbar(ui);
                ui.separator();

                // Favorites section
                self.show_favorites(ui);
                ui.separator();

                // File tree
                self.show_file_tree(ui);
            });
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("🔄").clicked() {
                self.refresh_file_tree();
            }

            if ui.button("📁").clicked() {
                // TODO: Open file dialog to select directory
            }

            if ui.button("➕").clicked() {
                // TODO: Create new file/folder
            }

            ui.separator();

            ui.checkbox(&mut self.show_hidden_files, "Hidden");

            ui.separator();

            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.file_filter);
        });

        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut self.current_directory);
        });
    }

    fn show_favorites(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Favorites", |ui| {
            if self.favorites.is_empty() {
                ui.label("No favorites yet");
            } else {
                for favorite in self.favorites.clone() {
                    ui.horizontal(|ui| {
                        if ui.button("📁").clicked() {
                            self.current_directory = favorite.clone();
                            self.refresh_file_tree();
                        }
                        ui.label(&favorite);
                        if ui.small_button("✕").clicked() {
                            self.favorites.retain(|f| f != &favorite);
                        }
                    });
                }
            }

            if ui.button("⭐ Add Current").clicked() {
                if !self.favorites.contains(&self.current_directory) {
                    self.favorites.push(self.current_directory.clone());
                }
            }
        });
    }

    fn show_file_tree(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for file_node in &mut self.file_tree {
                // TODO: Fix borrow checker issue with show_file_node
                // self.show_file_node(ui, file_node, 0);
                ui.label(&file_node.name);
            }
        });
    }

    fn show_file_node(&mut self, ui: &mut egui::Ui, node: &mut FileNode, depth: usize) {
        // Apply filter
        if !self.file_filter.is_empty()
            && !node
                .name
                .to_lowercase()
                .contains(&self.file_filter.to_lowercase())
            && !node.children.iter().any(|child| self.matches_filter(child))
        {
            return;
        }

        // Skip hidden files if not showing them
        if !self.show_hidden_files && node.name.starts_with('.') {
            return;
        }

        let indent = (depth as f32) * 20.0;
        ui.indent(format!("file_{}", node.path), |ui| {
            ui.allocate_ui_with_layout(
                [ui.available_width() - indent, 20.0].into(),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    // Expand/collapse for directories
                    if node.is_directory && !node.children.is_empty() {
                        let triangle = if node.expanded { "▼" } else { "▶" };
                        if ui.small_button(triangle).clicked() {
                            node.expanded = !node.expanded;
                        }
                    } else {
                        ui.add_space(20.0);
                    }

                    // File/folder icon
                    let icon = if node.is_directory {
                        "📁"
                    } else {
                        self.get_file_icon(&node.name)
                    };
                    ui.label(icon);

                    // File name (selectable and draggable)
                    let is_selected = self.selected_file.as_ref() == Some(&node.path);
                    let response = ui.selectable_label(is_selected, &node.name);

                    if response.clicked() {
                        self.selected_file = Some(node.path.clone());
                        if node.is_directory {
                            self.current_directory = node.path.clone();
                            self.refresh_file_tree();
                        }
                    }

                    // Drag and drop for files
                    if !node.is_directory && response.hovered() {
                        // TODO: Implement drag and drop properly
                        // response.dnd_set_drag_payload(DragPayload::File(node.path.clone()));
                    }

                    // Context menu
                    response.context_menu(|ui| {
                        if ui.button("Open").clicked() {
                            self.open_file(&node.path);
                            ui.close_menu();
                        }

                        if ui.button("Rename").clicked() {
                            // TODO: Implement rename
                            ui.close_menu();
                        }

                        if ui.button("Delete").clicked() {
                            // TODO: Implement delete with confirmation
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button("Show in Explorer").clicked() {
                            // TODO: Open system file explorer
                            ui.close_menu();
                        }

                        if node.is_directory && ui.button("Add to Favorites").clicked() {
                            if !self.favorites.contains(&node.path) {
                                self.favorites.push(node.path.clone());
                            }
                            ui.close_menu();
                        }
                    });

                    // Show file size and date for files
                    if !node.is_directory {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(size) = node.size {
                                ui.label(format_file_size(size));
                            }
                        });
                    }
                },
            );
        });

        // Show children if directory is expanded
        if node.is_directory && node.expanded {
            for child in &mut node.children {
                self.show_file_node(ui, child, depth + 1);
            }
        }
    }

    fn matches_filter(&self, node: &FileNode) -> bool {
        if self.file_filter.is_empty() {
            return true;
        }

        if node
            .name
            .to_lowercase()
            .contains(&self.file_filter.to_lowercase())
        {
            return true;
        }

        for child in &node.children {
            if self.matches_filter(child) {
                return true;
            }
        }

        false
    }

    fn get_file_icon(&self, filename: &str) -> &'static str {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "matrix" | "mtx" => "📜",   // Script files
            "scene" => "🎬",            // Scene files
            "prefab" => "🧩",           // Prefab files
            "material" | "mat" => "🎨", // Material files
            "texture" | "tex" | "png" | "jpg" | "jpeg" | "bmp" | "tga" => "🖼️", // Texture files
            "model" | "obj" | "fbx" | "blend" | "3ds" => "🎯", // 3D model files
            "audio" | "wav" | "mp3" | "ogg" | "m4a" => "🎵", // Audio files
            "shader" | "hlsl" | "glsl" | "vert" | "frag" => "✨", // Shader files
            "config" | "cfg" | "ini" | "json" | "toml" | "yaml" | "yml" => "⚙️", // Config files
            "txt" | "md" | "readme" => "📄", // Text files
            "rs" | "c" | "cpp" | "h" | "hpp" | "py" | "js" | "ts" => "💻", // Source code
            _ => "📄",                  // Default file icon
        }
    }

    fn refresh_file_tree(&mut self) {
        self.file_tree = self.scan_directory(&self.current_directory);
    }

    fn scan_directory(&self, path: &str) -> Vec<FileNode> {
        let mut nodes = Vec::new();

        // Create some default project structure if the directory doesn't exist
        if path == "Assets" {
            nodes.push(FileNode {
                name: "Scripts".to_string(),
                path: "Assets/Scripts".to_string(),
                is_directory: true,
                children: vec![
                    FileNode {
                        name: "Player.matrix".to_string(),
                        path: "Assets/Scripts/Player.matrix".to_string(),
                        is_directory: false,
                        children: vec![],
                        expanded: false,
                        size: Some(1024),
                        modified: Some(std::time::SystemTime::now()),
                    },
                    FileNode {
                        name: "GameManager.matrix".to_string(),
                        path: "Assets/Scripts/GameManager.matrix".to_string(),
                        is_directory: false,
                        children: vec![],
                        expanded: false,
                        size: Some(2048),
                        modified: Some(std::time::SystemTime::now()),
                    },
                ],
                expanded: true,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });

            nodes.push(FileNode {
                name: "Scenes".to_string(),
                path: "Assets/Scenes".to_string(),
                is_directory: true,
                children: vec![
                    FileNode {
                        name: "Main.scene".to_string(),
                        path: "Assets/Scenes/Main.scene".to_string(),
                        is_directory: false,
                        children: vec![],
                        expanded: false,
                        size: Some(4096),
                        modified: Some(std::time::SystemTime::now()),
                    },
                    FileNode {
                        name: "Level1.scene".to_string(),
                        path: "Assets/Scenes/Level1.scene".to_string(),
                        is_directory: false,
                        children: vec![],
                        expanded: false,
                        size: Some(8192),
                        modified: Some(std::time::SystemTime::now()),
                    },
                ],
                expanded: true,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });

            nodes.push(FileNode {
                name: "Materials".to_string(),
                path: "Assets/Materials".to_string(),
                is_directory: true,
                children: vec![FileNode {
                    name: "Default.material".to_string(),
                    path: "Assets/Materials/Default.material".to_string(),
                    is_directory: false,
                    children: vec![],
                    expanded: false,
                    size: Some(512),
                    modified: Some(std::time::SystemTime::now()),
                }],
                expanded: false,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });

            nodes.push(FileNode {
                name: "Textures".to_string(),
                path: "Assets/Textures".to_string(),
                is_directory: true,
                children: vec![],
                expanded: false,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });

            nodes.push(FileNode {
                name: "Models".to_string(),
                path: "Assets/Models".to_string(),
                is_directory: true,
                children: vec![],
                expanded: false,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });

            nodes.push(FileNode {
                name: "Audio".to_string(),
                path: "Assets/Audio".to_string(),
                is_directory: true,
                children: vec![],
                expanded: false,
                size: None,
                modified: Some(std::time::SystemTime::now()),
            });
        }

        // TODO: Implement actual file system scanning
        // For now, we're using the mock data above

        nodes
    }

    fn open_file(&self, path: &str) {
        println!("Opening file: {}", path);
        // TODO: Implement file opening based on file type
        // - .matrix files -> Open in script editor
        // - .scene files -> Load scene
        // - Image files -> Open in image viewer
        // - etc.
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

impl Default for ProjectBrowser {
    fn default() -> Self {
        Self::new()
    }
}
