//! Diff viewer component for Git Gud
//!
//! This component displays diffs for selected files.

use crate::services::GitService;
use crate::state::AppState;
use eframe::egui;

/// Diff viewer UI component
pub struct DiffViewer {
    /// Current diff text
    diff_text: String,

    /// Whether to show line numbers
    show_line_numbers: bool,

    /// Whether to wrap lines
    wrap_lines: bool,

    /// Last selected file path (to detect changes)
    last_selected_file: Option<std::path::PathBuf>,
}

impl DiffViewer {
    /// Create a new diff viewer component
    pub fn new() -> Self {
        Self {
            diff_text: String::new(),
            show_line_numbers: true,
            wrap_lines: false,
            last_selected_file: None,
        }
    }

    /// Show the diff viewer component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Diff View");

        // Check if we need to refresh the diff
        let current_selected_file = state.ui_state.selected_file_path().cloned();
        let needs_refresh = current_selected_file != self.last_selected_file;

        // Also check if files have been staged/unstaged
        let files_changed = state.ui_state.check_and_reset_staged_unstaged();

        if needs_refresh || files_changed {
            self.refresh_diff(state);
            self.last_selected_file = current_selected_file;
        }

        // Options toolbar
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_line_numbers, "Line numbers");
            ui.checkbox(&mut self.wrap_lines, "Wrap lines");

            if ui.button("Copy").clicked() && !self.diff_text.is_empty() {
                ui.output_mut(|o| o.copied_text = self.diff_text.clone());
                state.set_info("Diff copied to clipboard".to_string());
            }

            if ui.button("Refresh").clicked() {
                self.refresh_diff(state);
            }
        });

        ui.separator();

        // Simple version for now
        if !state.has_repository() {
            ui.label("No repository loaded");
            return;
        }

        if !state.ui_state.has_file_selection() {
            ui.label("Select a file to view diff");
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap();
        ui.label(format!("Selected: {}", selected_file.display()));

        // Show actual diff text
        self.show_diff_text(ui);
    }

    /// Refresh the diff for the currently selected file
    fn refresh_diff(&mut self, state: &mut AppState) {
        if !state.has_repository() || !state.ui_state.has_file_selection() {
            self.diff_text.clear();
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap();

        // Get actual diff from Git service
        if let Some(repo_state) = &state.repository_state {
            match GitService::get_file_diff(&repo_state.repository, selected_file) {
                Ok(diff) => {
                    self.diff_text = diff;
                    log::debug!(
                        "Loaded diff for file: {:?} ({} bytes)",
                        selected_file,
                        self.diff_text.len()
                    );
                }
                Err(e) => {
                    self.diff_text = format!("Error loading diff: {}", e);
                    log::error!("Failed to load diff for file {:?}: {}", selected_file, e);
                    state.set_error(format!("Failed to load diff: {}", e));
                }
            }
        } else {
            self.diff_text = "No repository loaded".to_string();
        }
    }

    /// Display diff text with syntax highlighting
    fn show_diff_text(&mut self, ui: &mut egui::Ui) {
        let mut job = egui::text::LayoutJob::default();

        for (line_num, line) in self.diff_text.lines().enumerate() {
            if self.show_line_numbers {
                let line_num_text = format!("{:4}: ", line_num + 1);
                job.append(
                    &line_num_text,
                    0.0,
                    egui::TextFormat::simple(egui::FontId::monospace(12.0), egui::Color32::GRAY),
                );
            }

            // Color code diff lines
            let (color, text) = if line.starts_with('+') {
                (egui::Color32::DARK_GREEN, line)
            } else if line.starts_with('-') {
                (egui::Color32::DARK_RED, line)
            } else if line.starts_with('@') {
                (egui::Color32::BLUE, line)
            } else {
                (egui::Color32::WHITE, line)
            };

            job.append(
                text,
                0.0,
                egui::TextFormat::simple(egui::FontId::monospace(12.0), color),
            );

            job.append("\n", 0.0, egui::TextFormat::default());
        }

        ui.add(
            egui::TextEdit::multiline(&mut self.diff_text)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .frame(false)
                .layouter(&mut |ui, _text, wrap_width| {
                    let mut layout_job = job.clone();
                    layout_job.wrap.max_width = wrap_width;
                    if !self.wrap_lines {
                        layout_job.wrap.max_width = f32::INFINITY;
                    }
                    ui.fonts(|f| f.layout_job(layout_job))
                }),
        );
    }

    /// Clear the current diff
    pub fn clear(&mut self) {
        self.diff_text.clear();
        self.last_selected_file = None;
    }

    /// Force refresh of the diff (e.g., when file is staged/unstaged)
    pub fn force_refresh(&mut self) {
        self.last_selected_file = None;
    }
}

impl Default for DiffViewer {
    fn default() -> Self {
        Self::new()
    }
}
