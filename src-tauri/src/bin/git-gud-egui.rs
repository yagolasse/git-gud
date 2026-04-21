use eframe::egui;
use git2::{Repository, StatusOptions};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

// ========== Data Models ==========

#[derive(Clone, Debug)]
struct FileStatus {
    path: String,
    status: String,
    staged: bool,
    git_status: git2::Status,
}

#[derive(Clone)]
struct RepositoryState {
    path: Option<String>,
    repo: Option<Arc<Mutex<Repository>>>,
    file_statuses: Vec<FileStatus>,
    error_message: Option<String>,
}

impl RepositoryState {
    fn new() -> Self {
        Self {
            path: None,
            repo: None,
            file_statuses: Vec::new(),
            error_message: None,
        }
    }
    
    fn load_repository_status(&mut self, path: String) {
        self.path = Some(path.clone());
        self.error_message = None;
        self.file_statuses.clear();
        
        match Repository::open(&path) {
            Ok(repo) => {
                match get_repo_status(&repo) {
                    Ok(statuses) => {
                        self.file_statuses = statuses;
                        self.repo = Some(Arc::new(Mutex::new(repo)));
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Failed to get repository status: {}", err));
                        self.repo = None;
                    }
                }
            }
            Err(err) => {
                self.error_message = Some(format!("Failed to open repository: {}", err));
                self.repo = None;
            }
        }
    }
    
    fn refresh_status(&mut self) {
        if let Some(path) = &self.path {
            self.load_repository_status(path.clone());
        }
    }
    
    fn stage_files(&mut self, file_paths: Vec<String>) -> Result<(), String> {
        if let Some(repo) = &self.repo {
            let repo = repo.lock().unwrap();
            let result = stage_files(&repo, file_paths);
            drop(repo); // Drop guard before calling refresh_status
            if result.is_ok() {
                self.refresh_status();
            }
            result
        } else {
            Err("No repository open".to_string())
        }
    }
    
    fn unstage_files(&mut self, file_paths: Vec<String>) -> Result<(), String> {
        if let Some(repo) = &self.repo {
            let repo = repo.lock().unwrap();
            let result = unstage_files(&repo, file_paths);
            drop(repo); // Drop guard before calling refresh_status
            if result.is_ok() {
                self.refresh_status();
            }
            result
        } else {
            Err("No repository open".to_string())
        }
    }
    
    fn stage_all(&mut self) -> Result<(), String> {
        let unstaged_files: Vec<String> = self.file_statuses
            .iter()
            .filter(|f| !f.staged)
            .map(|f| f.path.clone())
            .collect();
        
        if unstaged_files.is_empty() {
            return Ok(());
        }
        
        self.stage_files(unstaged_files)
    }
    
    fn unstage_all(&mut self) -> Result<(), String> {
        let staged_files: Vec<String> = self.file_statuses
            .iter()
            .filter(|f| f.staged)
            .map(|f| f.path.clone())
            .collect();
        
        if staged_files.is_empty() {
            return Ok(());
        }
        
        self.unstage_files(staged_files)
    }
    
    fn toggle_file_stage(&mut self, index: usize) -> Result<(), String> {
        if index >= self.file_statuses.len() {
            return Err("Invalid file index".to_string());
        }
        
        let file = &self.file_statuses[index];
        let file_path = file.path.clone();
        
        if file.staged {
            self.unstage_files(vec![file_path])
        } else {
            self.stage_files(vec![file_path])
        }
    }
}

// ========== UI Components ==========

struct RepositoryPanel {
    repo_state: Arc<Mutex<RepositoryState>>,
    selected_file: Option<usize>,
    last_click_time: Option<std::time::Instant>,
}

impl RepositoryPanel {
    fn new(repo_state: Arc<Mutex<RepositoryState>>) -> Self {
        Self {
            repo_state,
            selected_file: None,
            last_click_time: None,
        }
    }
    
    fn show(&mut self, ui: &mut egui::Ui) {
        // Get repository state
        let repo_state = self.repo_state.lock().unwrap();
        
        // Store data we need for UI
        let repo_path = repo_state.path.clone();
        let error_message = repo_state.error_message.clone();
        let file_statuses = repo_state.file_statuses.clone();
        let staged_indices: Vec<usize> = file_statuses
            .iter()
            .enumerate()
            .filter(|(_, f)| f.staged)
            .map(|(i, _)| i)
            .collect();
        let unstaged_indices: Vec<usize> = file_statuses
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.staged)
            .map(|(i, _)| i)
            .collect();
        
        // Drop the lock before UI rendering to avoid borrowing issues
        drop(repo_state);
        
        // Repository selection
        ui.horizontal(|ui| {
            if ui.button("Open Repository").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let repo_path = path.display().to_string();
                    let mut repo_state = self.repo_state.lock().unwrap();
                    repo_state.load_repository_status(repo_path.clone());
                    
                    // Save the repository path for next time
                    if let Err(err) = save_last_repository(&repo_path) {
                        repo_state.error_message = Some(format!("Failed to save repository path: {}", err));
                    }
                }
            }
            
            if let Some(path) = &repo_path {
                ui.label(format!("Repository: {}", path));
            } else {
                ui.label("No repository selected");
            }
            
            if ui.button("Refresh").clicked() {
                let mut repo_state = self.repo_state.lock().unwrap();
                repo_state.refresh_status();
            }
        });
        
        ui.separator();
        
        // Error display
        if let Some(error) = &error_message {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.separator();
        }
        
        // File status display - Vertical layout with unstaged above staged
        if file_statuses.is_empty() {
            ui.label("No files to display");
        } else {
            // Unstaged files section (top)
            self.show_file_section(ui, "Unstaged Files", &unstaged_indices, &file_statuses, false);
            
            ui.separator();
            
            // Staged files section (bottom)
            self.show_file_section(ui, "Staged Files", &staged_indices, &file_statuses, true);
        }
        
        // Stats
        ui.separator();
        ui.horizontal(|ui| {
            let staged_count = file_statuses.iter().filter(|f| f.staged).count();
            let unstaged_count = file_statuses.iter().filter(|f| !f.staged).count();
            ui.label(format!("Total: {} files ({} staged, {} unstaged)", 
                file_statuses.len(), staged_count, unstaged_count));
        });
    }
    
    fn show_file_section(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        indices: &[usize],
        file_statuses: &[FileStatus],
        is_staged: bool
    ) {
        // Section header with bulk action button
        ui.horizontal(|ui| {
            ui.heading(title);
            if !indices.is_empty() {
                let button_text = if is_staged { "Unstage All" } else { "Stage All" };
                if ui.button(button_text).clicked() {
                    let mut repo_state = self.repo_state.lock().unwrap();
                    let result = if is_staged {
                        repo_state.unstage_all()
                    } else {
                        repo_state.stage_all()
                    };
                    if let Err(err) = result {
                        repo_state.error_message = Some(err);
                    }
                }
            }
        });
        
        if indices.is_empty() {
            ui.label(if is_staged { "No staged files" } else { "No unstaged files" });
        } else {
            // Create a scrollable area for the file list
            egui::ScrollArea::vertical()
                .max_height(200.0) // Limit height to prevent taking too much space
                .show(ui, |ui| {
                    for &index in indices {
                        self.show_file_item(ui, index, file_statuses, is_staged);
                    }
                });
        }
    }
    
    fn show_file_item(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        file_statuses: &[FileStatus],
        is_staged: bool
    ) {
        let file = &file_statuses[index];
        let is_selected = self.selected_file == Some(index);
        
        // Determine icon based on Git status
        let (icon, color, tooltip) = if file.git_status.is_index_new() || file.git_status.is_wt_new() {
            ("+", egui::Color32::GREEN, "New file")
        } else if file.git_status.is_index_modified() || file.git_status.is_wt_modified() {
            ("M", egui::Color32::YELLOW, "Modified")
        } else if file.git_status.is_index_deleted() || file.git_status.is_wt_deleted() {
            ("D", egui::Color32::RED, "Deleted")
        } else if file.git_status.is_index_renamed() || file.git_status.is_wt_renamed() {
            ("R", egui::Color32::BLUE, "Renamed")
        } else if file.git_status.is_index_typechange() || file.git_status.is_wt_typechange() {
            ("T", egui::Color32::LIGHT_BLUE, "Type changed")
        } else if is_staged {
            ("✓", egui::Color32::GREEN, "Staged")
        } else {
            ("●", egui::Color32::GRAY, "Unknown status")
        };
        
        let response = ui.horizontal(|ui| {
            // Show icon to the left of filename
            let icon_label = ui.colored_label(color, icon);
            icon_label.on_hover_text(tooltip);
            
            // Show filename as a button
            ui.add(
                egui::Button::new(&file.path)
                    .fill(if is_selected {
                        egui::Color32::from_rgba_unmultiplied(50, 50, 50, 100)
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .frame(false)
            )
        }).inner;
        
        // Handle clicks
        if response.clicked() {
            let now = std::time::Instant::now();
            let mut toggle_stage = false;
            let mut select_file = false;
            
            if let Some(last_click) = self.last_click_time {
                if now.duration_since(last_click).as_millis() < 500 {
                    // Double click detected - toggle stage
                    toggle_stage = true;
                    self.last_click_time = None;
                } else {
                    // Single click - select file
                    select_file = true;
                    self.last_click_time = Some(now);
                }
            } else {
                // First click
                select_file = true;
                self.last_click_time = Some(now);
            }
            
            // Apply actions
            if toggle_stage {
                let mut repo_state = self.repo_state.lock().unwrap();
                if let Err(err) = repo_state.toggle_file_stage(index) {
                    repo_state.error_message = Some(err);
                }
            }
            if select_file {
                self.selected_file = Some(index);
            }
        }
        
        // Show file status on hover
        let action = if is_staged { "unstage" } else { "stage" };
        response.on_hover_text(format!("{} - {} (click to select, double-click to {})", 
            file.status, tooltip, action));
        

    }
}

// ========== Persistence Functions ==========

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("git-gud")
        .join("last_repo.txt")
}

fn save_last_repository(path: &str) -> std::io::Result<()> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, path)
}

fn load_last_repository() -> Option<String> {
    let config_path = get_config_path();
    fs::read_to_string(config_path).ok()
}

// ========== Main Application ==========

struct GitGudApp {
    repo_panel: RepositoryPanel,
    last_repo_loaded: bool,
}

impl Default for GitGudApp {
    fn default() -> Self {
        let repo_state = Arc::new(Mutex::new(RepositoryState::new()));
        Self {
            repo_panel: RepositoryPanel::new(repo_state),
            last_repo_loaded: false,
        }
    }
}

impl eframe::App for GitGudApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load last repository on first run
        if !self.last_repo_loaded {
            self.last_repo_loaded = true;
            if let Some(last_repo) = load_last_repository() {
                // Check if the repository still exists
                if Repository::open(&last_repo).is_ok() {
                    let mut repo_state = self.repo_panel.repo_state.lock().unwrap();
                    repo_state.load_repository_status(last_repo);
                }
            }
        }
        
        // Clear old click timer if too much time has passed
        if let Some(last_click) = self.repo_panel.last_click_time {
            if last_click.elapsed().as_millis() > 1000 {
                self.repo_panel.last_click_time = None;
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Git Gud - egui version");
            self.repo_panel.show(ui);
        });
    }
}

// ========== Git Operations ==========

fn get_repo_status(repo: &Repository) -> Result<Vec<FileStatus>, String> {
    let mut status_options = StatusOptions::new();
    status_options
        .include_untracked(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    
    let statuses = repo.statuses(Some(&mut status_options))
        .map_err(|e| format!("Failed to get statuses: {}", e))?;
    
    let mut files = Vec::new();
    
        for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("").to_string();
        
        // Determine if file is staged
        let staged = status.is_index_new() || 
                     status.is_index_modified() || 
                     status.is_index_deleted() || 
                     status.is_index_renamed() || 
                     status.is_index_typechange();
        
        // Get human-readable status
        let status_text = if status.is_wt_new() { "New" }
            else if status.is_wt_modified() { "Modified" }
            else if status.is_wt_deleted() { "Deleted" }
            else if status.is_wt_renamed() { "Renamed" }
            else if status.is_wt_typechange() { "Type changed" }
            else if status.is_index_new() { "Added" }
            else if status.is_index_modified() { "Modified" }
            else if status.is_index_deleted() { "Deleted" }
            else if status.is_index_renamed() { "Renamed" }
            else if status.is_index_typechange() { "Type changed" }
            else { "Unknown" };
        
        files.push(FileStatus {
            path,
            status: status_text.to_string(),
            staged,
            git_status: status,
        });
    }
    
    Ok(files)
}

fn stage_files(repo: &Repository, file_paths: Vec<String>) -> Result<(), String> {
    let mut index = repo.index()
        .map_err(|e| format!("Failed to open index: {}", e))?;
    
    for path in &file_paths {
        index.add_path(std::path::Path::new(path))
            .map_err(|e| format!("Failed to stage {}: {}", path, e))?;
    }
    
    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;
    
    Ok(())
}

fn unstage_files(repo: &Repository, file_paths: Vec<String>) -> Result<(), String> {
    let mut index = repo.index()
        .map_err(|e| format!("Failed to open index: {}", e))?;
    
    for path in &file_paths {
        index.remove_path(std::path::Path::new(path))
            .map_err(|e| format!("Failed to unstage {}: {}", path, e))?;
    }
    
    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;
    
    Ok(())
}

// ========== Main Function ==========

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Git Gud - egui (modular)"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Git Gud - egui (modular)",
        options,
        Box::new(|_cc| Ok(Box::<GitGudApp>::default())),
    )
}