//! Main window for Git Gud application
//!
//! This module contains the main application window UI.

use crate::state::{AppState, SharedAppState};
use crate::ui::{ErrorDialog, FileDialog, RecentRepos};
use eframe::egui;
use std::sync::Arc;
use parking_lot::Mutex;

/// Main application window
pub struct MainWindow {
    /// Shared application state
    state: SharedAppState,
    
    /// Branch list component
    branch_list: crate::ui::BranchList,
    
    /// Unstaged files list component
    unstaged_list: crate::ui::FileList,
    
    /// Staged files list component  
    staged_list: crate::ui::FileList,
    
    /// Diff viewer component
    diff_viewer: crate::ui::DiffViewer,
    
    /// Commit panel component
    commit_panel: crate::ui::CommitPanel,
    
    /// Error dialog component
    error_dialog: ErrorDialog,
    
    /// Recent repositories
    recent_repos: RecentRepos,
    
    /// Whether to show the repository open dialog
    show_open_dialog: bool,
    
    /// Repository path for open dialog
    open_repo_path: String,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_path(cc, None)
    }
    
    /// Create a new main window with an optional initial repository path
    pub fn new_with_path(cc: &eframe::CreationContext<'_>, initial_path: Option<&std::path::Path>) -> Self {
        // Set dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        // Set dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        let mut window = Self {
            state: Arc::new(Mutex::new(AppState::new())),
            branch_list: crate::ui::BranchList::new(),
            unstaged_list: crate::ui::FileList::new("Unstaged Files", false),
            staged_list: crate::ui::FileList::new("Staged Files", true),
            diff_viewer: crate::ui::DiffViewer::new(),
            commit_panel: crate::ui::CommitPanel::new(),
            error_dialog: ErrorDialog::new(),
            recent_repos: RecentRepos::default(),
            show_open_dialog: true, // Show dialog on startup
            open_repo_path: ".".to_string(),
        };
        
        // Try to load initial repository if provided
        if let Some(path) = initial_path {
            let path_buf = path.to_path_buf();
            let mut state = window.state.lock();
            if state.load_repository(path_buf.clone()).is_ok() {
                window.show_open_dialog = false;
                window.open_repo_path = path.to_string_lossy().to_string();
                // Add to recent repos
                window.recent_repos.add(&path_buf);
            }
        }
        
        window
    }
    
    /// Show the main window
    pub fn show(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle pending actions from previous frame
        {
            let mut state = self.state.lock();
            state.handle_pending_actions();
        }
        
        // Show repository open dialog if needed
        if self.show_open_dialog {
            self.show_open_dialog(ctx);
        }
        
        // Update error dialog if there's an error
        {
            let state = self.state.lock();
            if let Some(error) = &state.error_message {
                if !self.error_dialog.is_visible() {
                    self.error_dialog.show_error(error.clone());
                }
            }
        }
        
        // Show error dialog
        self.error_dialog.show(ctx);
        
        // Show menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let mut state = self.state.lock();
            
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    // Open Repository
                    if ui.button("Open Repository...").clicked() {
                        self.show_open_dialog = true;
                        ui.close_menu();
                    }
                    
                    // Close Repository (only enabled if repository is loaded)
                    if state.has_repository() {
                        if ui.button("Close Repository").clicked() {
                            state.repository_state = None;
                            state.ui_state.reset();
                            state.clear_error();
                            state.clear_info();
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Close Repository"));
                    }
                    
                    ui.separator();
                    
                    // Exit
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                
                // Edit menu (placeholder for future)
                ui.menu_button("Edit", |ui| {
                    ui.label("Edit features coming soon");
                });
                
                // View menu (placeholder for future)
                ui.menu_button("View", |ui| {
                    ui.label("View options coming soon");
                });
                
                // Help menu (placeholder for future)
                ui.menu_button("Help", |ui| {
                    ui.label("Help content coming soon");
                });
                
                // Show current repository path if loaded
                if let Some(repo_state) = &state.repository_state {
                    ui.add_space(10.0);
                    ui.label(format!("Repository: {}", repo_state.path.display()));
                }
            });
        });
        
        // Show main UI if repository loaded, otherwise show empty state
        let has_repo = self.state.lock().has_repository();
        if has_repo {
            self.show_main_ui(ctx);
        } else if !self.show_open_dialog {
            // Only show empty state if we're not showing the open dialog
            self.show_empty_state(ctx);
        }
    }
    
    /// Show the repository open dialog
    fn show_open_dialog(&mut self, ctx: &egui::Context) {
        use std::path::PathBuf;
        
        egui::Window::new("Open Repository")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Enter the path to a Git repository:");
                
                // Recent repositories section
                if !self.recent_repos.is_empty() {
                    ui.separator();
                    ui.label("Recent repositories:");
                    
                    let recent_repos = self.recent_repos.get();
                    for repo_path in recent_repos {
                        let path_str = repo_path.to_string_lossy();
                        if ui.button(path_str.as_ref()).clicked() {
                            self.open_repo_path = path_str.to_string();
                        }
                    }
                    
                    ui.separator();
                }
                
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut self.open_repo_path);
                    
                    if ui.button("Browse...").clicked() {
                        // Open native file dialog
                        if let Some(path) = FileDialog::open_directory() {
                            self.open_repo_path = path.to_string_lossy().to_string();
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_open_dialog = false;
                    }
                    
                    if ui.button("Open").clicked() {
                        let path = PathBuf::from(&self.open_repo_path);
                        let mut state = self.state.lock();
                        
                        match state.load_repository(path.clone()) {
                            Ok(_) => {
                                self.show_open_dialog = false;
                                state.clear_error();
                                // Add to recent repos
                                self.recent_repos.add(&path);
                            }
                            Err(e) => {
                                // Error is already set by load_repository
                                log::error!("Failed to load repository: {}", e);
                            }
                        }
                    }
                });
            });
    }
    

    
    /// Show empty state when no repository is loaded
    fn show_empty_state(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Git Gud");
                ui.add_space(20.0);
                ui.label("No repository loaded");
                ui.add_space(10.0);
                
                if ui.button("Open Repository...").clicked() {
                    self.show_open_dialog = true;
                }
            });
        });
    }
    
    /// Show main UI with three-panel layout
    fn show_main_ui(&mut self, ctx: &egui::Context) {
        // Left panel - Branches
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                let mut state = self.state.lock();
                self.branch_list.show(ui, &mut state);
            });
        
        // Central panel - Unstaged and Staged files
        egui::CentralPanel::default().show(ctx, |ui| {
            // Middle panel with two vertical sections
            egui::TopBottomPanel::top("middle_top")
                .resizable(true)
                .default_height(300.0)
                .show_inside(ui, |ui| {
                    let mut state = self.state.lock();
                    self.unstaged_list.show(ui, &mut state);
                });
            
            egui::TopBottomPanel::bottom("middle_bottom")
                .resizable(true)
                .default_height(300.0)
                .show_inside(ui, |ui| {
                    let mut state = self.state.lock();
                    self.staged_list.show(ui, &mut state);
                });
        });
        
        // Right panel - Diff view and Commit
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                // Right panel with two vertical sections
                egui::TopBottomPanel::top("right_top")
                    .resizable(true)
                    .default_height(400.0)
                    .show_inside(ui, |ui| {
                        let mut state = self.state.lock();
                        self.diff_viewer.show(ui, &mut state);
                    });
                
                egui::TopBottomPanel::bottom("right_bottom")
                    .resizable(true)
                    .default_height(200.0)
                    .show_inside(ui, |ui| {
                        let mut state = self.state.lock();
                        self.commit_panel.show(ui, &mut state);
                    });
            });
    }
    

}