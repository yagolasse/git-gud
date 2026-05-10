//! Main window for Git Gud application
//!
//! This module contains the main application window UI.

use crate::state::{AppState, SharedAppState};
use crate::ui::{ErrorDialog, FileDialog, RecentRepos};
use eframe::egui;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(PartialEq, Clone, Copy)]
enum ActiveTab {
    Changes,
    History,
}

/// Main application window
pub struct MainWindow {
    /// Shared application state
    state: SharedAppState,

    /// Branch list component
    branch_list: crate::ui::BranchList,

    /// Unified file list (staged + unstaged sections)
    file_list: crate::ui::FileList,

    /// Enhanced diff viewer component
    diff_viewer: crate::ui::EnhancedDiffViewer,

    /// Commit panel component
    commit_panel: crate::ui::CommitPanel,

    /// Command log floating window
    command_log: crate::ui::CommandLog,

    /// Error dialog component
    error_dialog: ErrorDialog,

    /// Recent repositories
    recent_repos: RecentRepos,

    /// File watcher service for auto-refresh
    file_watcher: crate::services::file_watcher_service::SharedFileWatcher,

    /// Whether to show the repository open dialog
    show_open_dialog: bool,

    /// Repository path for open dialog
    open_repo_path: String,

    /// Active tab (Changes or History)
    active_tab: ActiveTab,
}

impl MainWindow {
    /// Create a new main window
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_path(cc, None)
    }

    /// Create a new main window with an optional initial repository path
    pub fn new_with_path(
        cc: &eframe::CreationContext<'_>,
        initial_path: Option<&std::path::Path>,
    ) -> Self {
        // Set dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut window = Self {
            state: Arc::new(Mutex::new(AppState::new())),
            branch_list: crate::ui::BranchList::new(),
            file_list: crate::ui::FileList::new(),
            diff_viewer: crate::ui::EnhancedDiffViewer::new(),
            commit_panel: crate::ui::CommitPanel::new(),
            command_log: crate::ui::CommandLog::new(),
            error_dialog: ErrorDialog::new(),
            recent_repos: RecentRepos::load_default(),
            file_watcher: crate::services::file_watcher_service::SharedFileWatcher::new(),
            show_open_dialog: true, // Show dialog on startup
            open_repo_path: ".".to_string(),
            active_tab: ActiveTab::Changes,
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
        // Check for file changes and trigger auto-refresh
        self.check_file_changes();

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

        // Command log window (floating, toggled from View menu)
        {
            let entries = { self.state.lock().command_log.clone() };
            if self.command_log.show(ctx, &entries) {
                self.state.lock().clear_command_log();
            }
        }

        // Status bar — declared before menu bar and central panel so egui
        // allocates it from the bottom of the window first.
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(30, 30, 35),
                inner_margin: egui::Margin::symmetric(8.0, 4.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let state = self.state.lock();
                if let Some(msg) = &state.error_message {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 80, 80),
                        format!("✗  {msg}"),
                    );
                } else if let Some(msg) = &state.info_message {
                    ui.colored_label(
                        egui::Color32::from_rgb(150, 210, 150),
                        format!("✓  {msg}"),
                    );
                } else {
                    ui.label(" ");
                }
            });

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

                            // Stop file watcher
                            self.file_watcher.stop_watching();
                            log::info!("File watcher stopped");

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

                // Repository menu
                ui.menu_button("Repository", |ui| {
                    // Refresh repository (only enabled if repository is loaded)
                    if state.has_repository() {
                        if ui.button("Refresh").clicked() {
                            if let Err(e) = state.refresh_repository() {
                                state.set_error(format!("Failed to refresh repository: {}", e));
                            } else {
                                state.set_info("Repository refreshed".to_string());
                            }
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Refresh"));
                    }

                    ui.separator();

                    // Show in file explorer (only enabled if repository is loaded)
                    if state.has_repository() {
                        if ui.button("Show in File Explorer").clicked() {
                            // TODO: Implement show in file explorer
                            state.set_info(
                                "Show in file explorer feature not yet implemented".to_string(),
                            );
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Show in File Explorer"));
                    }
                });

                // View menu
                ui.menu_button("View", |ui| {
                    let label = if self.command_log.is_visible() {
                        "Hide Command Log"
                    } else {
                        "Show Command Log"
                    };
                    if ui.button(label).clicked() {
                        self.command_log.toggle();
                        ui.close_menu();
                    }
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

    /// Save recent repositories to disk
    fn save_recent_repos(&self) {
        if let Err(e) = self.recent_repos.save_default() {
            log::error!("Failed to save recent repositories: {}", e);
        }
    }

    /// Check for file changes and trigger refresh if needed
    fn check_file_changes(&mut self) {
        if self.file_watcher.is_watching() && self.file_watcher.should_refresh() {
            log::debug!("File changes detected, triggering refresh");

            let mut state = self.state.lock();
            if let Err(e) = state.refresh_repository() {
                log::error!("Failed to refresh repository after file change: {}", e);
                state.set_error(format!("Auto-refresh failed: {}", e));
            } else {
                log::debug!("Repository refreshed successfully");
            }
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
                                // Add to recent repos and save
                                self.recent_repos.add(&path);
                                self.save_recent_repos();

                                // Start file watcher for auto-refresh
                                if let Err(e) = self.file_watcher.start_watching(&path) {
                                    log::error!("Failed to start file watcher: {}", e);
                                    state.set_error(format!("Auto-refresh disabled: {}", e));
                                } else {
                                    log::info!("File watcher started for repository");
                                }
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

    /// Show main UI: sidebar + tab bar spanning center+right + tabbed content
    fn show_main_ui(&mut self, ctx: &egui::Context) {
        let panel_frame = egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            ..Default::default()
        };

        const SIDEBAR_W: f32 = 186.0;

        // Sidebar — fixed, declared first for z-order
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(SIDEBAR_W)
            .width_range(SIDEBAR_W..=350.0)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(30, 30, 35),
                ..panel_frame
            })
            .show(ctx, |ui| {
                let mut state = self.state.lock();
                self.branch_list.show(ui, &mut state);
            });

        // Everything else: tab bar + tab content
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(42, 42, 47),
                ..panel_frame
            })
            .show(ctx, |ui| {
                self.show_tab_bar(ui);
                match self.active_tab {
                    ActiveTab::Changes => self.show_changes_tab(ui, panel_frame),
                    ActiveTab::History => self.show_history_tab(ui),
                }
            });
    }

    /// Painter-based tab bar spanning the full center+right width
    fn show_tab_bar(&mut self, ui: &mut egui::Ui) {
        const TAB_H: f32 = 32.0;
        const TAB_BG: egui::Color32 = egui::Color32::from_rgb(25, 25, 30);
        const TEXT_INACTIVE: egui::Color32 = egui::Color32::from_rgb(150, 150, 155);
        const TEXT_HOVER: egui::Color32 = egui::Color32::from_rgb(210, 210, 215);
        const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgb(50, 50, 58);

        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, TAB_H), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, TAB_BG);
            // Bottom separator
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y - 1.0,
                egui::Stroke::new(1.0, BORDER_COLOR),
            );
        }

        let font = egui::FontId::proportional(12.0);
        let mut x = rect.min.x + 8.0;

        for (label, tab) in [
            ("Changes", ActiveTab::Changes),
            ("History", ActiveTab::History),
        ] {
            let galley =
                ui.fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), TEXT_INACTIVE));
            let tab_w = galley.size().x + 28.0;
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(x, rect.min.y),
                egui::vec2(tab_w, TAB_H),
            );
            let tab_id = ui.id().with("tab_bar").with(label);
            let tab_resp = ui.interact(tab_rect, tab_id, egui::Sense::click());

            let active = self.active_tab == tab;
            let text_color = if active {
                egui::Color32::WHITE
            } else if tab_resp.hovered() {
                TEXT_HOVER
            } else {
                TEXT_INACTIVE
            };

            if ui.is_rect_visible(tab_rect) {
                ui.painter().text(
                    tab_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    font.clone(),
                    text_color,
                );

                if active {
                    // 2px bottom border
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(tab_rect.min.x + 4.0, tab_rect.max.y - 2.0),
                            egui::vec2(tab_w - 8.0, 2.0),
                        ),
                        0.0,
                        egui::Color32::WHITE,
                    );
                }
            }

            if tab_resp.clicked() {
                self.active_tab = tab;
            }

            x += tab_w;
        }
    }

    /// Changes tab: resizable file list panel on the left, diff viewer on the right
    fn show_changes_tab(&mut self, ui: &mut egui::Ui, panel_frame: egui::Frame) {
        // File list + commit panel on the left
        egui::SidePanel::left("file_list_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(140.0..=400.0)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(42, 42, 47),
                ..panel_frame
            })
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::bottom("commit_panel_bottom")
                    .resizable(false)
                    .frame(egui::Frame {
                        fill: egui::Color32::from_rgb(42, 42, 47),
                        inner_margin: egui::Margin::symmetric(8.0, 6.0),
                        ..Default::default()
                    })
                    .show_inside(ui, |ui| {
                        let mut state = self.state.lock();
                        self.commit_panel.show(ui, &mut state);
                    });
                let mut state = self.state.lock();
                self.file_list.show(ui, &mut state);
            });

        // Diff viewer fills the rest
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(30, 30, 35),
                ..panel_frame
            })
            .show_inside(ui, |ui| {
                let mut state = self.state.lock();
                self.diff_viewer.show(ui, &mut state);
            });
    }

    /// History tab: placeholder for the commit graph
    fn show_history_tab(&mut self, ui: &mut egui::Ui) {
        let center = ui.available_rect_before_wrap().center();
        ui.allocate_space(ui.available_size());
        if ui.is_rect_visible(ui.max_rect()) {
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                "Commit history coming soon",
                egui::FontId::proportional(13.0),
                egui::Color32::from_rgb(95, 95, 100),
            );
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        // Save recent repositories when the window is closed
        self.save_recent_repos();
    }
}
