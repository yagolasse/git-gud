use crate::state::{AppPrefs, AppState, SharedAppState};
use crate::ui::{ErrorDialog, FileDialog, RecentRepos};
use eframe::egui;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(PartialEq, Clone, Copy)]
enum ActiveTab {
    Changes,
    History,
}

pub struct MainWindow {
    state: SharedAppState,
    branch_list: crate::ui::BranchList,
    file_list: crate::ui::FileList,
    diff_viewer: crate::ui::EnhancedDiffViewer,
    commit_panel: crate::ui::CommitPanel,
    commit_graph: crate::ui::CommitGraph,
    command_log: crate::ui::CommandLog,
    error_dialog: ErrorDialog,
    recent_repos: RecentRepos,
    file_watcher: crate::services::file_watcher_service::SharedFileWatcher,
    show_open_dialog: bool,
    open_repo_path: String,
    active_tab: ActiveTab,
    toolbar: crate::ui::Toolbar,
    passphrase_dialog: crate::ui::PassphraseDialog,
    file_history: crate::ui::FileHistoryPanel,
    dark_mode: bool,
    prefs: AppPrefs,
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_path(cc, None)
    }

    pub fn new_with_path(
        cc: &eframe::CreationContext<'_>,
        initial_path: Option<&std::path::Path>,
    ) -> Self {
        let prefs = AppPrefs::load_default();

        let dark_mode = prefs.dark_mode;
        if dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        let state = Arc::new(Mutex::new(AppState::new()));
        state.lock().dark_mode = dark_mode;

        let mut window = Self {
            state,
            branch_list: crate::ui::BranchList::new(),
            file_list: crate::ui::FileList::new(),
            diff_viewer: crate::ui::EnhancedDiffViewer::new(),
            commit_panel: crate::ui::CommitPanel::new(),
            commit_graph: crate::ui::CommitGraph::new(),
            command_log: crate::ui::CommandLog::new(),
            error_dialog: ErrorDialog::new(),
            recent_repos: RecentRepos::load_default(),
            file_watcher: crate::services::file_watcher_service::SharedFileWatcher::new(),
            show_open_dialog: true,
            open_repo_path: ".".to_string(),
            active_tab: ActiveTab::Changes,
            toolbar: crate::ui::Toolbar::new(),
            passphrase_dialog: crate::ui::PassphraseDialog::new(),
            file_history: crate::ui::FileHistoryPanel::new(),
            dark_mode,
            prefs,
        };

        if let Some(path) = initial_path {
            let path_buf = path.to_path_buf();
            let mut state = window.state.lock();
            if state.load_repository(path_buf.clone()).is_ok() {
                window.show_open_dialog = false;
                window.open_repo_path = path.to_string_lossy().to_string();
                window.recent_repos.add(&path_buf);
            }
        } else if let Some(last_repo) = window.prefs.last_repo.clone()
            && last_repo.exists() {
                let loaded = {
                    let mut state = window.state.lock();
                    state.load_repository(last_repo.clone()).is_ok()
                };
                if loaded {
                    window.show_open_dialog = false;
                    window.open_repo_path = last_repo.to_string_lossy().to_string();
                    window.recent_repos.add(&last_repo);
                    if let Err(e) = window.file_watcher.start_watching(&last_repo) {
                        log::error!("Failed to start file watcher on startup: {}", e);
                    }
                }
            }

        window
    }

    pub fn show(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_file_changes();

        {
            let mut state = self.state.lock();
            state.handle_pending_actions();
            state.poll_network();
        }

        self.handle_global_shortcuts(ctx);

        self.passphrase_dialog.poll_and_show(ctx, &mut self.state.lock().ui_state);

        {
            let mut state = self.state.lock();
            self.file_history.show(ctx, &mut state);
        }

        // Pull dialog
        {
            let show = self.state.lock().ui_state.show_pull_dialog;
            if show {
                let ctx_clone = ctx.clone();
                let mut do_pull = false;
                let mut do_cancel = false;
                {
                    let mut state = self.state.lock();
                    egui::Window::new("Pull")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .show(&ctx_clone, |ui| {
                            ui.label("Pull from:");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.ui_state.pull_from_branch)
                                    .desired_width(220.0)
                                    .hint_text("origin/main"),
                            );
                            ui.add_space(4.0);
                            ui.checkbox(&mut state.ui_state.pull_auto_stash, "Stash changes, pull, reapply");
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                let branch_ok = !state.ui_state.pull_from_branch.trim().is_empty();
                                if ui.add_enabled(branch_ok, egui::Button::new("Pull")).clicked() { do_pull = true; }
                                if ui.button("Cancel").clicked() { do_cancel = true; }
                            });
                        });
                }
                if do_pull {
                    let mut state = self.state.lock();
                    let auto_stash = state.ui_state.pull_auto_stash;
                    let ref_spec = state.ui_state.pull_from_branch.clone();
                    state.ui_state.show_pull_dialog = false;
                    if auto_stash {
                        state.ui_state.pending_action = Some(crate::state::PendingAction::PullWithAutoStash(ref_spec));
                    } else {
                        state.ui_state.pending_action = Some(crate::state::PendingAction::Pull(ref_spec));
                    }
                }
                if do_cancel {
                    self.state.lock().ui_state.show_pull_dialog = false;
                }
            }
        }

        // Sync dark mode and set egui visuals
        self.dark_mode = self.state.lock().dark_mode;
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let p = crate::ui::colors::get(self.dark_mode);

        if self.show_open_dialog {
            self.show_open_dialog(ctx);
        }

        {
            let state = self.state.lock();
            if let Some(error) = &state.error_message
                && !self.error_dialog.is_visible() {
                    self.error_dialog.show_error(error.clone());
                }
        }

        if self.error_dialog.show(ctx) {
            self.state.lock().clear_error();
        }

        {
            let entries = { self.state.lock().command_log.clone() };
            if self.command_log.show(ctx, &entries) {
                self.state.lock().clear_command_log();
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let mut state = self.state.lock();

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Repository...").clicked() {
                        if let Some(path) = crate::ui::FileDialog::open_directory() {
                            match state.load_repository(path.clone()) {
                                Ok(_) => {
                                    self.show_open_dialog = false;
                                    state.clear_error();
                                    self.recent_repos.add(&path);
                                    if let Err(e) = self.recent_repos.save_default() {
                                        log::error!("Failed to save recent repos: {}", e);
                                    }
                                    if let Err(e) = self.file_watcher.start_watching(&path) {
                                        log::error!("Failed to start file watcher: {}", e);
                                        state.set_error(format!("Auto-refresh disabled: {}", e));
                                    } else {
                                        log::info!("File watcher started for repository");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to load repository: {}", e);
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if state.has_repository() {
                        if ui.button("Close Repository").clicked() {
                            state.repository_state = None;
                            state.ui_state.reset();
                            state.clear_error();
                            state.clear_info();
                            self.file_watcher.stop_watching();
                            log::info!("File watcher stopped");
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Close Repository"));
                    }
                    if ui.button("New Repository...").clicked() {
                        if let Some(path) = crate::ui::FileDialog::open_directory() {
                            match crate::services::GitService::init_repository(&path) {
                                Ok(_) => match state.load_repository(path.clone()) {
                                    Ok(_) => {
                                        self.show_open_dialog = false;
                                        state.clear_error();
                                        self.recent_repos.add(&path);
                                        if let Err(e) = self.recent_repos.save_default() {
                                            log::error!("Failed to save recent repos: {}", e);
                                        }
                                        if let Err(e) = self.file_watcher.start_watching(&path) {
                                            log::error!("Failed to start file watcher: {}", e);
                                        }
                                    }
                                    Err(e) => state.set_error(format!("Failed to load new repo: {}", e)),
                                },
                                Err(e) => state.set_error(format!("Init failed: {}", e)),
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    ui.label("Edit features coming soon");
                });

                ui.menu_button("Repository", |ui| {
                    let has_repo = state.has_repository();
                    if ui.add_enabled(has_repo, egui::Button::new("Fetch").shortcut_text("Ctrl+Shift+F")).clicked() {
                        state.ui_state.pending_action = Some(crate::state::PendingAction::Fetch);
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_repo, egui::Button::new("Pull").shortcut_text("Ctrl+Shift+L")).clicked() {
                        let default_branch = state.repository_state.as_ref()
                            .and_then(|rs| rs.model.head.clone())
                            .map(|h| format!("origin/{}", h))
                            .unwrap_or_default();
                        state.ui_state.pull_from_branch = default_branch;
                        state.ui_state.show_pull_dialog = true;
                        ui.close_menu();
                    }
                    if ui.add_enabled(has_repo, egui::Button::new("Push").shortcut_text("Ctrl+Shift+P")).clicked() {
                        state.ui_state.pending_action = Some(crate::state::PendingAction::Push);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add_enabled(has_repo, egui::Button::new("Refresh").shortcut_text("Ctrl+R")).clicked() {
                        if let Err(e) = state.refresh_repository() {
                            state.set_error(format!("Failed to refresh: {}", e));
                        } else {
                            state.set_info("Repository refreshed".to_string());
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Open Repository...").clicked() {
                        self.show_open_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("New Repository...").clicked() {
                        if let Some(path) = crate::ui::FileDialog::open_directory() {
                            match crate::services::GitService::init_repository(&path) {
                                Ok(_) => match state.load_repository(path.clone()) {
                                    Ok(_) => {
                                        self.show_open_dialog = false;
                                        state.clear_error();
                                        self.recent_repos.add(&path);
                                        if let Err(e) = self.recent_repos.save_default() {
                                            log::error!("Failed to save recent repos: {}", e);
                                        }
                                        if let Err(e) = self.file_watcher.start_watching(&path) {
                                            log::error!("Failed to start file watcher: {}", e);
                                        }
                                    }
                                    Err(e) => state.set_error(format!("Failed to load new repo: {}", e)),
                                },
                                Err(e) => state.set_error(format!("Init failed: {}", e)),
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add_enabled(has_repo, egui::Button::new("Show in File Explorer")).clicked() {
                        if let Some(rs) = state.repository_state.as_ref() {
                            let repo_path = rs.path.clone();
                            crate::ui::components::open_in_explorer(&repo_path);
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    let log_label = if self.command_log.is_visible() { "Hide Command Log" } else { "Show Command Log" };
                    if ui.button(log_label).clicked() {
                        self.command_log.toggle();
                        ui.close_menu();
                    }
                    ui.separator();
                    let theme_label = if state.dark_mode { "Switch to Light Mode" } else { "Switch to Dark Mode" };
                    if ui.button(theme_label).clicked() {
                        state.toggle_dark_mode();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Help content coming soon");
                });

                if let Some(repo_state) = &state.repository_state {
                    ui.add_space(10.0);
                    ui.label(format!("Repository: {}", repo_state.path.display()));
                }
            });
        });

        if self.state.lock().has_repository() {
            let recent: Vec<std::path::PathBuf> =
                self.recent_repos.get().iter().map(|p| (*p).clone()).collect();
            let toolbar_resp = egui::TopBottomPanel::top("toolbar")
                .frame(egui::Frame {
                    fill: p.bg_primary,
                    inner_margin: egui::Margin::ZERO,
                    outer_margin: egui::Margin::ZERO,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    let mut state = self.state.lock();
                    self.toolbar.show(ui, &mut state, &recent)
                });
            if let Some(action) = toolbar_resp.inner {
                match action {
                    crate::ui::ToolbarAction::OpenRepo(path) => {
                        let mut state = self.state.lock();
                        if state.load_repository(path.clone()).is_ok() {
                            self.show_open_dialog = false;
                            state.clear_error();
                            self.recent_repos.add(&path);
                            self.save_recent_repos();
                            if let Err(e) = self.file_watcher.start_watching(&path) {
                                log::error!("Failed to start file watcher: {}", e);
                            }
                        }
                    }
                    crate::ui::ToolbarAction::ShowOpenDialog => {
                        self.show_open_dialog = true;
                    }
                }
            }
        }

        let has_repo = self.state.lock().has_repository();
        if has_repo {
            self.show_main_ui(ctx);
        } else if !self.show_open_dialog {
            self.show_empty_state(ctx);
        }
    }

    fn save_recent_repos(&self) {
        if let Err(e) = self.recent_repos.save_default() {
            log::error!("Failed to save recent repositories: {}", e);
        }
    }

    fn save_prefs(&mut self) {
        let state = self.state.lock();
        self.prefs.dark_mode = state.dark_mode;
        self.prefs.last_repo = state.repository_state.as_ref().map(|r| r.path.clone());
        drop(state);
        if let Err(e) = self.prefs.save_default() {
            log::error!("Failed to save preferences: {}", e);
        }
    }

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

    fn show_open_dialog(&mut self, ctx: &egui::Context) {
        use std::path::PathBuf;

        egui::Window::new("Open Repository")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Enter the path to a Git repository:");

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
                    if ui.button("Browse...").clicked()
                        && let Some(path) = FileDialog::open_directory() {
                            self.open_repo_path = path.to_string_lossy().to_string();
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
                                self.recent_repos.add(&path);
                                self.save_recent_repos();
                                if let Err(e) = self.file_watcher.start_watching(&path) {
                                    log::error!("Failed to start file watcher: {}", e);
                                    state.set_error(format!("Auto-refresh disabled: {}", e));
                                } else {
                                    log::info!("File watcher started for repository");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to load repository: {}", e);
                            }
                        }
                    }
                });
            });
    }

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

    fn show_main_ui(&mut self, ctx: &egui::Context) {
        let p = crate::ui::colors::get(self.dark_mode);
        let panel_frame = egui::Frame {
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            ..Default::default()
        };

        const SIDEBAR_W: f32 = 186.0;

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(SIDEBAR_W)
            .width_range(SIDEBAR_W..=350.0)
            .frame(egui::Frame {
                fill: p.bg_secondary,
                stroke: egui::Stroke::new(0.5, p.border),
                ..panel_frame
            })
            .show(ctx, |ui| {
                let mut state = self.state.lock();
                self.branch_list.show(ui, &mut state);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: p.bg_secondary,
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

    fn show_tab_bar(&mut self, ui: &mut egui::Ui) {
        let p = crate::ui::colors::get(self.dark_mode);
        const TAB_H: f32 = 32.0;

        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, TAB_H), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, p.bg_secondary);
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y - 0.5,
                egui::Stroke::new(0.5, p.border),
            );
        }

        let font = egui::FontId::proportional(12.0);
        let mut x = rect.min.x;

        for (label, tab) in [("Changes", ActiveTab::Changes), ("History", ActiveTab::History)] {
            let galley = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), p.text_secondary));
            let tab_w = galley.size().x + 28.0;
            let tab_rect = egui::Rect::from_min_size(egui::pos2(x, rect.min.y), egui::vec2(tab_w, TAB_H));
            let tab_id = ui.id().with("tab_bar").with(label);
            let tab_resp = ui.interact(tab_rect, tab_id, egui::Sense::click());
            let active = self.active_tab == tab;

            if ui.is_rect_visible(tab_rect) {
                let bg = if active {
                    p.bg_primary
                } else if tab_resp.hovered() {
                    p.bg_tertiary
                } else {
                    egui::Color32::TRANSPARENT
                };
                if bg != egui::Color32::TRANSPARENT {
                    ui.painter().rect_filled(tab_rect, 0.0, bg);
                }

                let text_color = if active || tab_resp.hovered() { p.text_primary } else { p.text_secondary };
                ui.painter().text(tab_rect.center(), egui::Align2::CENTER_CENTER, label, font.clone(), text_color);

                if active {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(tab_rect.min.x, tab_rect.max.y - 2.0),
                            egui::vec2(tab_w, 2.0),
                        ),
                        0.0,
                        p.text_primary,
                    );
                }
            }

            if tab_resp.clicked() { self.active_tab = tab; }
            x += tab_w;
        }
    }

    fn show_changes_tab(&mut self, ui: &mut egui::Ui, panel_frame: egui::Frame) {
        let p = crate::ui::colors::get(self.dark_mode);
        let border = egui::Stroke::new(0.5, p.border);

        egui::SidePanel::left("file_list_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(140.0..=400.0)
            .frame(egui::Frame { fill: p.bg_primary, stroke: border, ..panel_frame })
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::bottom("commit_panel_bottom")
                    .resizable(false)
                    .frame(egui::Frame {
                        fill: p.bg_primary,
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

        egui::CentralPanel::default()
            .frame(egui::Frame { fill: p.bg_primary, ..panel_frame })
            .show_inside(ui, |ui| {
                let mut state = self.state.lock();
                self.diff_viewer.show(ui, &mut state);
            });
    }

    fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.wants_keyboard_input();

        ctx.input(|i| {
            // Ctrl+Enter — commit (active even while typing in the message box)
            if i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl {
                let mut state = self.state.lock();
                let msg = state.ui_state.commit_message().trim().to_owned();
                if !msg.is_empty() {
                    state.ui_state.pending_action =
                        Some(crate::state::PendingAction::CreateCommit(msg));
                }
            }

            if typing { return; }

            // Ctrl+Shift+F — Fetch
            if i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.lock().ui_state.pending_action =
                    Some(crate::state::PendingAction::Fetch);
            }
            // Ctrl+Shift+L — Pull (open pull dialog)
            if i.key_pressed(egui::Key::L) && i.modifiers.ctrl && i.modifiers.shift {
                let mut state = self.state.lock();
                if state.has_repository() {
                    let default_branch = state.repository_state.as_ref()
                        .and_then(|rs| rs.model.head.clone())
                        .map(|h| format!("origin/{}", h))
                        .unwrap_or_default();
                    state.ui_state.pull_from_branch = default_branch;
                    state.ui_state.show_pull_dialog = true;
                }
            }
            // Ctrl+Shift+P — Push
            if i.key_pressed(egui::Key::P) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.lock().ui_state.pending_action =
                    Some(crate::state::PendingAction::Push);
            }
            // Ctrl+R — Refresh
            if i.key_pressed(egui::Key::R) && i.modifiers.ctrl {
                let mut state = self.state.lock();
                if state.has_repository()
                    && let Err(e) = state.repository_state_mut().refresh() {
                        state.set_error(format!("Refresh failed: {}", e));
                    }
            }

            // ArrowUp / ArrowDown — navigate file list
            let up = i.key_pressed(egui::Key::ArrowUp);
            let down = i.key_pressed(egui::Key::ArrowDown);
            if up || down {
                let mut state = self.state.lock();
                if let Some(rs) = state.repository_state.as_ref() {
                    let paths: Vec<std::path::PathBuf> = rs.staged_files.iter()
                        .chain(rs.unstaged_files.iter())
                        .map(|f| f.path.clone())
                        .collect();
                    if !paths.is_empty() {
                        let cur = state.ui_state.selected_file.as_ref();
                        let idx = cur
                            .and_then(|p| paths.iter().position(|q| q == p))
                            .unwrap_or(0);
                        let next = if down {
                            (idx + 1).min(paths.len() - 1)
                        } else {
                            idx.saturating_sub(1)
                        };
                        state.ui_state.select_file(paths[next].clone());
                    }
                }
            }

            // Space — stage / unstage selected file
            if i.key_pressed(egui::Key::Space) {
                let mut state = self.state.lock();
                if let Some(path) = state.ui_state.selected_file.clone() {
                    let is_staged = state.repository_state()
                        .staged_files.iter().any(|f| f.path == path);
                    if is_staged {
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::UnstageSelected(vec![path]));
                    } else {
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::StageSelected(vec![path]));
                    }
                }
            }

            // Enter — checkout selected branch
            if i.key_pressed(egui::Key::Enter) && !i.modifiers.ctrl {
                let mut state = self.state.lock();
                if let Some(name) = state.ui_state.selected_branch.clone() {
                    let already_current = state.repository_state.as_ref()
                        .map(|rs| rs.branches.iter().any(|b| b.name == name && b.is_current))
                        .unwrap_or(false);
                    if !already_current {
                        match state.repository_state_mut().checkout_branch(&name) {
                            Ok(()) => state.set_info(format!("Checked out: {}", name)),
                            Err(e) => {
                                let msg = e.to_string();
                                let display = if msg.to_lowercase().contains("conflict") {
                                    format!("Cannot checkout '{}': resolve conflicts first", name)
                                } else {
                                    format!("Failed to checkout {}: {}", name, e)
                                };
                                state.set_error(display);
                            }
                        }
                    }
                }
            }

            // C — cherry-pick selected commit
            if i.key_pressed(egui::Key::C) && !i.modifiers.ctrl && !i.modifiers.shift {
                let mut state = self.state.lock();
                let commit_id = state.repository_state.as_ref()
                    .and_then(|rs| self.commit_graph.selected_commit_id(&rs.commits))
                    .map(|s| s.to_owned());
                if let Some(id) = commit_id {
                    let short = id[..7.min(id.len())].to_owned();
                    match state.repository_state_mut().cherry_pick(&id) {
                        Ok(()) => state.set_info(format!("Cherry-picked {}", short)),
                        Err(e) => {
                            let msg = e.to_string();
                            if msg.contains("allow-empty") || msg.contains("is now empty") {
                                let _ = state.repository_state_mut().cherry_pick_skip();
                                state.set_info(format!("Skipped {}: already on this branch", short));
                            } else {
                                state.set_error(format!("Cherry-pick failed: {}", e));
                            }
                        }
                    }
                }
            }
        });
    }

    fn show_history_tab(&mut self, ui: &mut egui::Ui) {
        let mut state = self.state.lock();
        if let Some(commit_id) = self.commit_graph.show(ui, &mut state) {
            let short = &commit_id[..7.min(commit_id.len())];
            match state.repository_state_mut().cherry_pick(&commit_id) {
                Ok(()) => state.set_info(format!("Cherry-picked {}", short)),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("allow-empty") || msg.contains("is now empty") {
                        let _ = state.repository_state_mut().cherry_pick_skip();
                        state.set_info(format!("Skipped {}: changes already present on this branch", short));
                    } else {
                        state.set_error(format!("Cherry-pick failed: {}", e));
                    }
                }
            }
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        self.save_recent_repos();
        self.save_prefs();
    }
}
