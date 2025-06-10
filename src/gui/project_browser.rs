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
                self.show_ui_content(ui);
            });
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        self.show_ui_content(ui);
    }

    fn show_ui_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("Project");

        // Toolbar
        self.show_toolbar(ui);
        ui.separator();

        // Favorites section
        self.show_favorites(ui);
        ui.separator();

        // File tree
        self.show_file_tree(ui);
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

            if ui.button("⭐ Add Current").clicked() && !self.favorites.contains(&self.current_directory) {
                self.favorites.push(self.current_directory.clone());
            }
        });
    }

    fn show_file_tree(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Clone the file tree to avoid borrow checker issues
            let mut file_tree = self.file_tree.clone();
            let mut selected_file_changed = None;
            let mut directory_changed = None;
            let mut file_to_open = None;

            for file_node in &mut file_tree {
                let (selected, dir_change, open_file) = self.show_file_node_helper(ui, file_node, 0);
                if let Some(path) = selected {
                    selected_file_changed = Some(path);
                }
                if let Some(dir) = dir_change {
                    directory_changed = Some(dir);
                }
                if let Some(file) = open_file {
                    file_to_open = Some(file);
                }
            }

            // Apply changes after UI rendering
            if let Some(selected) = selected_file_changed {
                self.selected_file = Some(selected);
            }
            if let Some(dir) = directory_changed {
                self.current_directory = dir;
                self.refresh_file_tree();
            } else {
                // Update the file tree with any expansion changes
                self.file_tree = file_tree;
            }
            if let Some(file) = file_to_open {
                self.open_file(&file);
            }
        });
    }

    fn show_file_node_helper(&self, ui: &mut egui::Ui, node: &mut FileNode, depth: usize) -> (Option<String>, Option<String>, Option<String>) {
        // Apply filter
        if !self.file_filter.is_empty()
            && !node
                .name
                .to_lowercase()
                .contains(&self.file_filter.to_lowercase())
            && !node.children.iter().any(|child| self.matches_filter(child))
        {
            return (None, None, None);
        }

        // Skip hidden files if not showing them
        if !self.show_hidden_files && node.name.starts_with('.') {
            return (None, None, None);
        }

        let mut selected_file = None;
        let mut directory_change = None;
        let mut file_to_open = None;

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

                    // File name (selectable)
                    let is_selected = self.selected_file.as_ref() == Some(&node.path);
                    let response = ui.selectable_label(is_selected, &node.name);

                    // Show file size for files
                    if !node.is_directory {
                        if let Some(size) = node.size {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format_file_size(size));
                            });
                        }
                    }

                    if response.clicked() {
                        selected_file = Some(node.path.clone());
                        if node.is_directory {
                            directory_change = Some(node.path.clone());
                        }
                    }

                    // Double-click to open files
                    if response.double_clicked() && !node.is_directory {
                        file_to_open = Some(node.path.clone());
                    }

                    // Context menu
                    response.context_menu(|ui| {
                        if ui.button("Open").clicked() {
                            file_to_open = Some(node.path.clone());
                            ui.close_menu();
                        }

                        if ui.button("Add to Favorites").clicked() {
                            // This will be handled by the parent
                            ui.close_menu();
                        }

                        if ui.button("Show in Explorer").clicked() {
                            // TODO: Implement show in explorer
                            ui.close_menu();
                        }
                    });
                },
            );
        });

        // Show children if expanded
        if node.is_directory && node.expanded {
            for child in &mut node.children {
                let (child_selected, child_dir_change, child_open) = self.show_file_node_helper(ui, child, depth + 1);
                if child_selected.is_some() {
                    selected_file = child_selected;
                }
                if child_dir_change.is_some() {
                    directory_change = child_dir_change;
                }
                if child_open.is_some() {
                    file_to_open = child_open;
                }
            }
        }

        (selected_file, directory_change, file_to_open)
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

        // Determine file type and action
        if let Some(extension) = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            match extension.to_lowercase().as_str() {
                "matrix" | "mtx" => {
                    println!("Matrix script file detected: {}", path);
                    // TODO: Signal to open in script editor
                }
                "scene" => {
                    println!("Scene file detected: {}", path);
                    // TODO: Signal to load scene
                }
                "json" | "toml" | "yaml" | "yml" => {
                    println!("Configuration file detected: {}", path);
                    // TODO: Open in text editor
                }
                "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                    println!("Image file detected: {}", path);
                    // TODO: Open in image viewer
                }
                "txt" | "md" => {
                    println!("Text file detected: {}", path);
                    // TODO: Open in text editor
                }
                _ => {
                    println!("Unknown file type: {}", path);
                    // Try to open as text file
                }
            }
        }

        // Try to read file content for preview (for small files)
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() < 10240 { // Less than 10KB
                if let Ok(content) = std::fs::read_to_string(path) {
                    println!("File content preview:\n{}", &content[..content.len().min(200)]);
                }
            }
        }
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
