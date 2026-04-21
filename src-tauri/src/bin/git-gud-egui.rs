use eframe::egui;
use git2::{Repository, StatusOptions};
use notify::{Watcher, RecommendedWatcher, Event, RecursiveMode};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, Duration};

// ========== Data Models ==========

#[derive(Clone, Debug)]
struct FileStatus {
    path: String,
    status: String,
    staged: bool,
    git_status: git2::Status,
}

struct FileWatcher {
    watcher: RecommendedWatcher,
    last_event_time: Instant,
}

impl FileWatcher {
    fn new(ctx: egui::Context) -> Result<Self, String> {
        let ctx_clone = ctx.clone();
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                // Check if this is a Git metadata change
                let mut should_refresh = false;
                for path in event.paths {
                    if let Some(filename) = path.file_name() {
                        let name = filename.to_string_lossy();
                        if name == "index" || name == "HEAD" || name.starts_with("refs") {
                            should_refresh = true;
                            break;
                        }
                    }
                }
                
                if should_refresh {
                    // Request UI repaint on next frame
                    ctx_clone.request_repaint();
                }
            }
        }).map_err(|e| format!("Failed to create watcher: {}", e))?;
        
        Ok(Self {
            watcher,
            last_event_time: Instant::now(),
        })
    }
    
    fn watch_repo(&mut self, repo_path: &str) -> Result<(), String> {
        let git_dir = PathBuf::from(repo_path).join(".git");
        if !git_dir.exists() {
            return Ok(());
        }
        
        self.watcher.watch(&git_dir, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch repository: {}", e))?;
        
        Ok(())
    }
    
    fn should_refresh(&mut self) -> bool {
        // Debounce: only refresh if at least 500ms since last event
        let now = Instant::now();
        let time_since_last = now.duration_since(self.last_event_time);
        if time_since_last >= Duration::from_millis(500) {
            self.last_event_time = now;
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
struct RepositoryState {
    path: Option<String>,
    repo: Option<Arc<Mutex<Repository>>>,
    file_statuses: Vec<FileStatus>,
    error_message: Option<String>,
    selected_file: Option<(String, bool)>, // (path, staged)
    diff_content: Option<String>,
    diff_error: Option<String>,
    last_refresh: Instant,
    needs_refresh: bool,
}

impl RepositoryState {
    fn new() -> Self {
        Self {
            path: None,
            repo: None,
            file_statuses: Vec::new(),
            error_message: None,
            selected_file: None,
            diff_content: None,
            diff_error: None,
            last_refresh: Instant::now(),
            needs_refresh: false,
        }
    }
    
    fn load_repository_status(&mut self, path: String) {
        self.path = Some(path.clone());
        self.error_message = None;
        self.file_statuses.clear();
        self.clear_selection();
        
        match Repository::open(&path) {
            Ok(repo) => {
                match get_repo_status(&repo) {
                    Ok(statuses) => {
                        self.file_statuses = statuses;
                        self.repo = Some(Arc::new(Mutex::new(repo)));
                        self.last_refresh = Instant::now();
                        self.needs_refresh = false;
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
    
    fn refresh_if_needed(&mut self) {
        if self.needs_refresh {
            self.refresh_status();
        }
    }
    
    fn mark_needs_refresh(&mut self) {
        self.needs_refresh = true;
    }
    
    fn refresh_status(&mut self) {
        if let Some(path) = &self.path {
            // Store current selection to check if it still exists after refresh
            let old_selection = self.selected_file.clone();
            
            // Try to use existing repository if available
            let result = if let Some(repo_arc) = &self.repo {
                let repo = repo_arc.lock().unwrap();
                get_repo_status(&repo)
            } else {
                // No repository open, load fresh
                return self.load_repository_status(path.clone());
            };
            
            match result {
                Ok(statuses) => {
                    self.file_statuses = statuses;
                    self.last_refresh = Instant::now();
                    self.needs_refresh = false;
                    
                    // If we had a file selected, check if it still exists
                    if let Some((ref path, staged)) = old_selection {
                        if let Some(_file) = self.file_statuses.iter().find(|f| f.path == *path && f.staged == staged) {
                            // File still exists, reselect it
                            self.selected_file = Some((path.clone(), staged));
                            self.fetch_diff_for_selected_file();
                        } else {
                            // File no longer exists, clear selection
                            self.clear_selection();
                        }
                    }
                }
                Err(err) => {
                    // Repository might be invalid, try reopening
                    self.load_repository_status(path.clone());
                }
            }
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
    
    fn select_file(&mut self, index: usize) {
        if index >= self.file_statuses.len() {
            return;
        }
        
        let file = &self.file_statuses[index];
        self.selected_file = Some((file.path.clone(), file.staged));
        self.diff_content = None;
        self.diff_error = None;
        
        // Fetch diff in background
        self.fetch_diff_for_selected_file();
    }
    
    fn fetch_diff_for_selected_file(&mut self) {
        if let Some((ref path, staged)) = self.selected_file {
            if let Some(repo) = &self.repo {
                let repo = repo.lock().unwrap();
                match get_file_diff(&repo, path, staged) {
                    Ok(diff) => {
                        self.diff_content = Some(diff);
                        self.diff_error = None;
                    }
                    Err(err) => {
                        self.diff_content = None;
                        self.diff_error = Some(err);
                    }
                }
            }
        }
    }
    
    fn clear_selection(&mut self) {
        self.selected_file = None;
        self.diff_content = None;
        self.diff_error = None;
    }
}

// ========== UI Components ==========

struct RepositoryPanel {
    repo_state: Arc<Mutex<RepositoryState>>,
    file_watcher: Option<FileWatcher>,
    last_click_time: Option<std::time::Instant>,
    open_repo_dialog_requested: bool,
}

impl RepositoryPanel {
    fn new(repo_state: Arc<Mutex<RepositoryState>>) -> Self {
        Self {
            repo_state,
            file_watcher: None,
            last_click_time: None,
            open_repo_dialog_requested: false,
        }
    }
    
    fn show_file_lists(&mut self, ui: &mut egui::Ui) {
        // Get repository state
        let repo_state = self.repo_state.lock().unwrap();
        
        // Store data we need for UI
        let _repo_path = repo_state.path.clone();
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
    
    fn show_diff_view(&mut self, ui: &mut egui::Ui) {
        let repo_state = self.repo_state.lock().unwrap();
        
        // Repository selection at the top
        ui.horizontal(|ui| {
            ui.heading("Git Gud");
            
            if ui.button("Open Repository").clicked() {
                self.open_repo_dialog_requested = true;
            }
            
            if let Some(path) = &repo_state.path {
                ui.label(format!("Repository: {}", path));
            } else {
                ui.label("No repository selected");
            }
        });
        
        ui.separator();
        
        // Show general errors
        if let Some(error) = &repo_state.error_message {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.separator();
        }
        
        // Show diff header
        if let Some((ref path, staged)) = &repo_state.selected_file {
            ui.heading(format!("Diff: {} ({})", path, if *staged { "Staged" } else { "Unstaged" }));
        } else {
            ui.heading("Diff: No file selected");
            ui.label("Select a file from the left panel to view its changes");
            return;
        }
        
        ui.separator();
        
        // Show diff error if any
        if let Some(error) = &repo_state.diff_error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.separator();
        }
        
        // Show diff content
        if let Some(diff) = &repo_state.diff_content {
            if diff.trim().is_empty() {
                ui.label("No changes to show (might be a new untracked file or binary).");
            } else {
                // For now, show plain text diff
                // We'll add syntax highlighting in the next step
                egui::ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        // Use monospaced font for diff
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::monospace(12.0)
                        );
                        
                        // Split diff into lines and render with basic coloring
                        for line in diff.lines() {
                            if line.starts_with('+') {
                                ui.colored_label(egui::Color32::GREEN, line);
                            } else if line.starts_with('-') {
                                ui.colored_label(egui::Color32::RED, line);
                            } else if line.starts_with("@@") {
                                ui.colored_label(egui::Color32::BLUE, line);
                            } else {
                                ui.label(line);
                            }
                        }
                        
                        // Restore default font
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::default()
                        );
                    });
            }
        } else {
            // Diff is still loading
            ui.spinner();
            ui.label("Loading diff...");
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
        let is_selected = {
            let repo_state = self.repo_state.lock().unwrap();
            repo_state.selected_file.as_ref().map_or(false, |(selected_path, selected_staged)| {
                selected_path == &file.path && *selected_staged == is_staged
            })
        };
        
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
                let mut repo_state = self.repo_state.lock().unwrap();
                repo_state.select_file(index);
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
                    repo_state.load_repository_status(last_repo.clone());
                    
                    // Initialize file watcher for this repository
                    if self.repo_panel.file_watcher.is_none() {
                        match FileWatcher::new(ctx.clone()) {
                            Ok(mut watcher) => {
                                if let Err(err) = watcher.watch_repo(&last_repo) {
                                    eprintln!("Failed to start file watcher: {}", err);
                                } else {
                                    self.repo_panel.file_watcher = Some(watcher);
                                }
                            }
                    Err(_err) => {
                        eprintln!("Failed to create file watcher");
                    }
                        }
                    }
                }
            }
        }
        
        // Clear old click timer if too much time has passed
        if let Some(last_click) = self.repo_panel.last_click_time {
            if last_click.elapsed().as_millis() > 1000 {
                self.repo_panel.last_click_time = None;
            }
        }
        
        // Check if file watcher indicates we need to refresh
        if let Some(watcher) = &mut self.repo_panel.file_watcher {
            if watcher.should_refresh() {
                let mut repo_state = self.repo_panel.repo_state.lock().unwrap();
                repo_state.mark_needs_refresh();
            }
        }
        
        // Refresh repository status if needed
        {
            let mut repo_state = self.repo_panel.repo_state.lock().unwrap();
            repo_state.refresh_if_needed();
        }
        
        // Handle open repository dialog if requested
        if self.repo_panel.open_repo_dialog_requested {
            self.repo_panel.open_repo_dialog_requested = false;
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                let repo_path = path.display().to_string();
                let mut repo_state = self.repo_panel.repo_state.lock().unwrap();
                repo_state.load_repository_status(repo_path.clone());
                
                // Save the repository path for next time
                if let Err(err) = save_last_repository(&repo_path) {
                    eprintln!("Failed to save repository path: {}", err);
                }
                
                // Initialize or update file watcher for this repository
                if let Some(_watcher) = &mut self.repo_panel.file_watcher {
                    // Stop watching previous repository and start watching new one
                    // Note: notify watcher doesn't have an easy way to unwatch specific paths
                    // For simplicity, we'll create a new watcher
                    match FileWatcher::new(ctx.clone()) {
                        Ok(mut new_watcher) => {
                            if let Err(err) = new_watcher.watch_repo(&repo_path) {
                                eprintln!("Failed to start file watcher: {}", err);
                            } else {
                                self.repo_panel.file_watcher = Some(new_watcher);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to create file watcher: {}", err);
                        }
                    }
                } else {
                    // Create new file watcher
                    match FileWatcher::new(ctx.clone()) {
                        Ok(mut watcher) => {
                            if let Err(err) = watcher.watch_repo(&repo_path) {
                                eprintln!("Failed to start file watcher: {}", err);
                            } else {
                                self.repo_panel.file_watcher = Some(watcher);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to create file watcher: {}", err);
                        }
                    }
                }
            }
        }
        
        // Left panel for file lists
        egui::SidePanel::left("file_panel")
            .min_width(300.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                self.repo_panel.show_file_lists(ui);
            });
        
        // Central panel for diff view
        egui::CentralPanel::default().show(ctx, |ui| {
            self.repo_panel.show_diff_view(ui);
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
        
        // Check for staged changes
        let has_staged = status.is_index_new() || 
                         status.is_index_modified() || 
                         status.is_index_deleted() || 
                         status.is_index_renamed() || 
                         status.is_index_typechange();
        
        // Check for unstaged changes  
        let has_unstaged = status.is_wt_new() ||
                           status.is_wt_modified() ||
                           status.is_wt_deleted() ||
                           status.is_wt_renamed() ||
                           status.is_wt_typechange();
        
        // If file has both staged and unstaged changes, we need to create two entries
        if has_staged && has_unstaged {
            // Create staged entry
            let staged_status = if status.is_index_new() { "Added" }
                else if status.is_index_modified() { "Modified" }
                else if status.is_index_deleted() { "Deleted" }
                else if status.is_index_renamed() { "Renamed" }
                else if status.is_index_typechange() { "Type changed" }
                else { "Unknown" };
            
            files.push(FileStatus {
                path: path.clone(),
                status: staged_status.to_string(),
                staged: true,
                git_status: status,
            });
            
            // Create unstaged entry
            let unstaged_status = if status.is_wt_new() { "New" }
                else if status.is_wt_modified() { "Modified" }
                else if status.is_wt_deleted() { "Deleted" }
                else if status.is_wt_renamed() { "Renamed" }
                else if status.is_wt_typechange() { "Type changed" }
                else { "Unknown" };
            
            files.push(FileStatus {
                path,
                status: unstaged_status.to_string(),
                staged: false,
                git_status: status,
            });
        } else {
            // File has only staged or only unstaged changes (or neither)
            let staged = has_staged;
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
    // Get HEAD commit to reset to
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit()
        .map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    // Reset the specified files in index to match HEAD
    repo.reset_default(Some(commit.as_object()), &file_paths)
        .map_err(|e| format!("Failed to unstage files: {}", e))?;
    
    Ok(())
}

/// Get file diff (similar to git.rs version but simplified for egui)
fn get_file_diff(repo: &Repository, file_path: &str, staged: bool) -> Result<String, String> {
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    opts.context_lines(3);
    opts.interhunk_lines(1);

    let diff = if staged {
        let head = repo.head().ok();
        let tree = match head {
            Some(h) => Some(h.peel_to_tree().map_err(|e| format!("Failed to peel HEAD to tree: {}", e))?),
            None => None,
        };
        repo.diff_tree_to_index(tree.as_ref(), None, Some(&mut opts))
            .map_err(|e| format!("Failed to get staged diff: {}", e))?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| format!("Failed to get unstaged diff: {}", e))?
    };

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        match origin {
            '+' | '-' | ' ' => {
                diff_text.push(origin);
                diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            }
            'H' => {
                diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            }
            _ => {}
        }
        true
    }).map_err(|e| format!("Failed to format diff: {}", e))?;

    Ok(diff_text)
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