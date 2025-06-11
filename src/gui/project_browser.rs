use super::*;

/// Project browser panel for managing assets and files
pub struct ProjectBrowser {
    current_directory: String,
    file_tree: Vec<FileNode>,
    selected_file: Option<String>,
    show_hidden_files: bool,
    file_filter: String,
    favorites: Vec<String>,
    project_path: Option<String>,
    show_create_dialog: bool,
    create_dialog_name: String,
    create_dialog_is_folder: bool,
    // Callbacks for file operations
    pub script_editor_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub scene_loader_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub text_editor_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
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
            project_path: None,
            show_create_dialog: false,
            create_dialog_name: String::new(),
            create_dialog_is_folder: false,
            script_editor_callback: None,
            scene_loader_callback: None,
            text_editor_callback: None,
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

        // Show create dialog if requested
        if self.show_create_dialog {
            self.show_create_dialog_ui(ui);
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("🔄").clicked() {
                self.refresh_file_tree();
            }

            if ui.button("📁").clicked() {
                // Open file dialog to select directory
                if let Some(path) = self.select_directory() {
                    self.project_path = Some(path);
                    self.refresh_files();
                }
            }

            if ui.button("➕").clicked() {
                // Create new file/folder
                self.show_create_dialog = true;
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
                            // Implement show in explorer
                            self.show_in_explorer(&node.path);
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

        // Implement actual file system scanning
        if self.project_path.is_some() {
            self.scan_directory_recursive(path, &mut nodes);
        } else {
            // No project path set, use mock data for demo
            self.add_mock_data(&mut nodes);
        }

        nodes
    }

    /// Scan directory recursively and populate file nodes
    fn scan_directory_recursive(&self, dir_path: &str, nodes: &mut Vec<FileNode>) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Skip hidden files and common build directories
                if file_name.starts_with('.') ||
                   file_name == "target" ||
                   file_name == "node_modules" ||
                   file_name == "__pycache__" {
                    continue;
                }

                let is_directory = path.is_dir();
                let metadata = entry.metadata().ok();
                let size = if !is_directory {
                    metadata.as_ref().map(|m| m.len())
                } else {
                    None
                };
                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok());

                let mut file_node = FileNode {
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    is_directory,
                    children: Vec::new(),
                    expanded: false,
                    size,
                    modified,
                };

                // Recursively scan subdirectories (but don't expand them by default)
                if is_directory {
                    self.scan_directory_recursive(&file_node.path, &mut file_node.children);
                }

                nodes.push(file_node);
            }
        }

        // Sort nodes: directories first, then files, both alphabetically
        nodes.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    /// Add mock data for demonstration when no project is loaded
    fn add_mock_data(&self, nodes: &mut Vec<FileNode>) {
        nodes.push(FileNode {
            name: "Scenes".to_string(),
            path: "Scenes".to_string(),
            is_directory: true,
            children: vec![
                FileNode {
                    name: "MainScene.scene".to_string(),
                    path: "Scenes/MainScene.scene".to_string(),
                    is_directory: false,
                    children: vec![],
                    expanded: false,
                    size: Some(1024),
                    modified: Some(std::time::SystemTime::now()),
                },
                FileNode {
                    name: "PhysicsDemo.scene".to_string(),
                    path: "Scenes/PhysicsDemo.scene".to_string(),
                    is_directory: false,
                    children: vec![],
                    expanded: false,
                    size: Some(2048),
                    modified: Some(std::time::SystemTime::now()),
                },
            ],
            expanded: false,
            size: None,
            modified: Some(std::time::SystemTime::now()),
        });

        nodes.push(FileNode {
            name: "Scripts".to_string(),
            path: "Scripts".to_string(),
            is_directory: true,
            children: vec![
                FileNode {
                    name: "player_controller.matrix".to_string(),
                    path: "Scripts/player_controller.matrix".to_string(),
                    is_directory: false,
                    children: vec![],
                    expanded: false,
                    size: Some(4096),
                    modified: Some(std::time::SystemTime::now()),
                },
                FileNode {
                    name: "physics_demo.matrix".to_string(),
                    path: "Scripts/physics_demo.matrix".to_string(),
                    is_directory: false,
                    children: vec![],
                    expanded: false,
                    size: Some(3072),
                    modified: Some(std::time::SystemTime::now()),
                },
            ],
            expanded: false,
            size: None,
            modified: Some(std::time::SystemTime::now()),
        });

        nodes.push(FileNode {
            name: "Assets".to_string(),
            path: "Assets".to_string(),
            is_directory: true,
            children: vec![
                FileNode {
                    name: "Textures".to_string(),
                    path: "Assets/Textures".to_string(),
                    is_directory: true,
                    children: vec![],
                    expanded: false,
                    size: None,
                    modified: Some(std::time::SystemTime::now()),
                },
                FileNode {
                    name: "Models".to_string(),
                    path: "Assets/Models".to_string(),
                    is_directory: true,
                    children: vec![],
                    expanded: false,
                    size: None,
                    modified: Some(std::time::SystemTime::now()),
                },
                FileNode {
                    name: "Audio".to_string(),
                    path: "Assets/Audio".to_string(),
                    is_directory: true,
                    children: vec![],
                    expanded: false,
                    size: None,
                    modified: Some(std::time::SystemTime::now()),
                },
            ],
            expanded: false,
            size: None,
            modified: Some(std::time::SystemTime::now()),
        });
    }

    /// Simple directory selection (fallback implementation)
    fn select_directory(&self) -> Option<String> {
        // For now, return current working directory as a fallback
        // In a real implementation, this would open a native file dialog
        std::env::current_dir()
            .ok()
            .and_then(|path| path.to_str().map(String::from))
    }

    /// Refresh the file tree
    fn refresh_files(&mut self) {
        self.refresh_file_tree();
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
                    // Signal to open in script editor
                    if let Some(ref callback) = self.script_editor_callback {
                        callback(path);
                    }
                }
                "scene" => {
                    println!("Scene file detected: {}", path);
                    // Signal to load scene
                    if let Some(ref callback) = self.scene_loader_callback {
                        callback(path);
                    }
                }
                "json" | "toml" | "yaml" | "yml" | "txt" | "md" | "rs" | "py" | "js" | "ts" => {
                    println!("Text file detected: {}", path);
                    // Open in text editor
                    if let Some(ref callback) = self.text_editor_callback {
                        callback(path);
                    }
                }
                "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                    println!("Image file detected: {}", path);
                    // Open in system default image viewer
                    self.open_with_system_default(path);
                }
                _ => {
                    println!("Unknown file type: {}", path);
                    // Try to open as text file
                    if let Some(ref callback) = self.text_editor_callback {
                        callback(path);
                    }
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

    /// Show file/folder in system explorer
    fn show_in_explorer(&self, path: &str) {
        let path = std::path::Path::new(path);

        // Platform-specific commands to open file explorer
        #[cfg(target_os = "windows")]
        {
            if path.is_dir() {
                let _ = std::process::Command::new("explorer")
                    .arg(path)
                    .spawn();
            } else if let Some(parent) = path.parent() {
                let _ = std::process::Command::new("explorer")
                    .arg("/select,")
                    .arg(path)
                    .spawn();
            }
        }

        #[cfg(target_os = "macos")]
        {
            if path.is_dir() {
                let _ = std::process::Command::new("open")
                    .arg(path)
                    .spawn();
            } else {
                let _ = std::process::Command::new("open")
                    .arg("-R")
                    .arg(path)
                    .spawn();
            }
        }

        #[cfg(target_os = "linux")]
        {
            let folder_path = if path.is_dir() {
                path
            } else {
                path.parent().unwrap_or(path)
            };

            let _ = std::process::Command::new("xdg-open")
                .arg(folder_path)
                .spawn();
        }
    }

    /// Open file with system default application
    fn open_with_system_default(&self, path: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", path])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(path)
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn();
        }
    }

    /// Set callback for opening script files
    pub fn set_script_editor_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.script_editor_callback = Some(Box::new(callback));
    }

    /// Set callback for loading scene files
    pub fn set_scene_loader_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.scene_loader_callback = Some(Box::new(callback));
    }

    /// Set callback for opening text files
    pub fn set_text_editor_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.text_editor_callback = Some(Box::new(callback));
    }

    /// Show create file/folder dialog
    fn show_create_dialog_ui(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Create New")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.create_dialog_is_folder, false, "File");
                    ui.radio_value(&mut self.create_dialog_is_folder, true, "Folder");
                });

                ui.separator();

                ui.label(format!("{}:", if self.create_dialog_is_folder { "Folder name" } else { "File name" }));
                ui.text_edit_singleline(&mut self.create_dialog_name);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() && !self.create_dialog_name.is_empty() {
                        // Create the file or folder
                        self.create_file_or_folder();
                        self.show_create_dialog = false;
                        self.create_dialog_name.clear();
                        self.create_dialog_is_folder = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_create_dialog = false;
                        self.create_dialog_name.clear();
                        self.create_dialog_is_folder = false;
                    }
                });
            });
    }

    /// Create a file or folder based on dialog settings
    fn create_file_or_folder(&mut self) {
        let path = format!("{}/{}", self.current_directory, self.create_dialog_name);

        if self.create_dialog_is_folder {
            // Create folder
            if let Err(e) = std::fs::create_dir_all(&path) {
                eprintln!("Failed to create folder {}: {}", path, e);
            } else {
                println!("Created folder: {}", path);
            }
        } else {
            // Create file
            if let Err(e) = std::fs::write(&path, "") {
                eprintln!("Failed to create file {}: {}", path, e);
            } else {
                println!("Created file: {}", path);
            }
        }

        // Refresh file tree to show new item
        self.refresh_file_tree();
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
