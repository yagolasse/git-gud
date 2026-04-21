//! File list component for Git Gud
//!
//! This component displays a list of files (unstaged or staged)
//! and allows the user to stage/unstage files.

use crate::state::AppState;
use eframe::egui;
use std::path::PathBuf;

/// File list UI component
pub struct FileList {
    /// Title to display above the list
    title: String,

    /// Whether this list shows staged files (true) or unstaged files (false)
    is_staged: bool,

    /// Checked files (for staging/unstaging)
    checked_files: std::collections::HashSet<PathBuf>,

    /// Filter text for files
    filter: String,
}

impl FileList {
    /// Create a new file list component
    pub fn new(title: &str, is_staged: bool) -> Self {
        Self {
            title: title.to_string(),
            is_staged,
            checked_files: std::collections::HashSet::new(),
            filter: String::new(),
        }
    }

    /// Show the file list component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading(&self.title);

        if !state.has_repository() {
            ui.label("No repository loaded");
            return;
        }

        // Get file lists before entering closures
        let unstaged_files = state.repository_state().unstaged_files.clone();
        let staged_files = state.repository_state().staged_files.clone();

        let files = if self.is_staged {
            &staged_files
        } else {
            &unstaged_files
        };

        let file_count = files.len();

        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);

            if ui.button("Clear").clicked() {
                self.filter.clear();
            }
        });

        // File count and action buttons
        ui.horizontal(|ui| {
            ui.label(format!("{} files", file_count));

            if file_count > 0 {
                if self.is_staged {
                    if ui.button("Unstage All").clicked() {
                        let all_paths: Vec<_> = files.iter().map(|f| f.path.clone()).collect();
                        // We need to modify state, so we'll handle this after the UI rendering
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::UnstageAll(all_paths));
                    }
                } else {
                    if ui.button("Stage All").clicked() {
                        let all_paths: Vec<_> = files.iter().map(|f| f.path.clone()).collect();
                        // We need to modify state, so we'll handle this after the UI rendering
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::StageAll(all_paths));
                    }
                }
            }
        });

        // File list with scroll area - fixed scrollbar positioning
        let scroll_area = egui::ScrollArea::vertical()
            .max_height(300.0)
            .auto_shrink([false, false]); // Don't shrink, always show scroll area

        scroll_area.show(ui, |ui| {
            // Use a container to ensure content fills width
            ui.vertical(|ui| {
                if files.is_empty() {
                    ui.label("No files");
                    return;
                }

                // Filter files based on filter text
                let filtered_files: Vec<_> = if self.filter.is_empty() {
                    files.iter().collect()
                } else {
                    files
                        .iter()
                        .filter(|file| {
                            file.path
                                .to_string_lossy()
                                .to_lowercase()
                                .contains(&self.filter.to_lowercase())
                        })
                        .collect()
                };

                if filtered_files.is_empty() {
                    ui.label("No files match filter");
                    return;
                }

                // Display each file - ensure they fill width
                for file in filtered_files {
                    ui.horizontal(|ui| {
                        ui.set_min_width(ui.available_width());
                        self.show_file(ui, state, file);
                    });
                }
            });
        });

        // Batch actions for checked files
        if !self.checked_files.is_empty() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("{} files selected", self.checked_files.len()));

                if ui.button("Clear Selection").clicked() {
                    self.clear_checked();
                }

                if self.is_staged {
                    if ui.button("Unstage Selected").clicked() {
                        let selected_paths: Vec<_> = self.checked_files.iter().cloned().collect();
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::UnstageSelected(selected_paths));
                    }
                } else {
                    if ui.button("Stage Selected").clicked() {
                        let selected_paths: Vec<_> = self.checked_files.iter().cloned().collect();
                        state.ui_state.pending_action =
                            Some(crate::state::PendingAction::StageSelected(selected_paths));
                    }
                }
            });
        }
    }

    /// Show a single file item
    fn show_file(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        file: &crate::models::FileChange,
    ) {
        ui.horizontal(|ui| {
            // Checkbox for staging/unstaging
            let mut is_checked = self.checked_files.contains(&file.path);
            if ui.checkbox(&mut is_checked, "").changed() {
                if is_checked {
                    self.checked_files.insert(file.path.clone());
                } else {
                    self.checked_files.remove(&file.path);
                }
            }

            // File icon based on status
            let icon = match file.status {
                crate::models::FileStatus::Modified => "📝",
                crate::models::FileStatus::Added => "➕",
                crate::models::FileStatus::Deleted => "🗑️",
                crate::models::FileStatus::Renamed => "↔️",
                crate::models::FileStatus::Untracked => "❓",
                crate::models::FileStatus::Copied => "📋",
                crate::models::FileStatus::Ignored => "👁️",
                crate::models::FileStatus::Unmodified => "📄",
            };

            // File name with icon
            let file_name = file
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.to_string_lossy().to_string());

            let file_text = format!("{} {}", icon, file_name);

            // Selectable label for diff view
            let response = ui.selectable_label(
                state.ui_state.selected_file.as_ref() == Some(&file.path),
                file_text,
            );

            // Handle selection for diff view
            if response.clicked() {
                state.ui_state.select_file(file.path.clone());
            }

            // Context menu
            response.context_menu(|ui| {
                if self.is_staged {
                    if ui.button("Unstage").clicked() {
                        if let Err(e) = state
                            .repository_state_mut()
                            .unstage_files(&[file.path.clone()])
                        {
                            state.set_error(format!("Failed to unstage file: {}", e));
                        } else {
                            state.set_info(format!("Unstaged: {}", file.path.display()));
                            state.ui_state.mark_files_staged_or_unstaged();
                        }
                        ui.close_menu();
                    }
                } else {
                    if ui.button("Stage").clicked() {
                        if let Err(e) = state
                            .repository_state_mut()
                            .stage_files(&[file.path.clone()])
                        {
                            state.set_error(format!("Failed to stage file: {}", e));
                        } else {
                            state.set_info(format!("Staged: {}", file.path.display()));
                            state.ui_state.mark_files_staged_or_unstaged();
                        }
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("Copy path").clicked() {
                    ui.output_mut(|o| o.copied_text = file.path.to_string_lossy().to_string());
                    ui.close_menu();
                }

                if ui.button("Show in folder").clicked() {
                    // TODO: Implement show in folder
                    state.set_info("Show in folder feature not yet implemented".to_string());
                    ui.close_menu();
                }
            });

            // Tooltip with full path
            response.on_hover_text(file.path.display().to_string());
        });
    }

    /// Clear checked files
    pub fn clear_checked(&mut self) {
        self.checked_files.clear();
    }
}
