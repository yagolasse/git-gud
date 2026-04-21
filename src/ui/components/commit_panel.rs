//! Commit panel component for Git Gud
//!
//! This component provides UI for creating commits.

use crate::state::AppState;
use eframe::egui;

/// Commit panel UI component
pub struct CommitPanel {
    /// Whether to show advanced options
    show_advanced: bool,

    /// Author name override
    author_name: String,

    /// Author email override
    author_email: String,
}

impl CommitPanel {
    /// Create a new commit panel component
    pub fn new() -> Self {
        Self {
            show_advanced: false,
            author_name: String::new(),
            author_email: String::new(),
        }
    }

    /// Show the commit panel component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Commit");

        // Check if repository is loaded
        if !state.has_repository() {
            ui.label("No repository loaded");
            return;
        }

        // Staged files info
        ui.horizontal(|ui| {
            ui.label("Staged files:");
            let staged_count = state.repository_state().staged_files.len();
            ui.label(format!("{}", staged_count));

            if staged_count > 0 {
                ui.colored_label(egui::Color32::GREEN, "✓ Ready to commit");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "No staged changes");
            }
        });

        ui.separator();

        // Commit message input
        ui.label("Commit message:");

        // Summary (first line)
        ui.horizontal(|ui| {
            ui.label("Summary:");
            ui.text_edit_singleline(&mut state.ui_state.commit_summary)
                .on_hover_text("Brief description of changes (required)");
        });

        // Description (body)
        ui.label("Description:");
        ui.add(
            egui::TextEdit::multiline(&mut state.ui_state.commit_description)
                .hint_text("Detailed description of changes (optional)")
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        );

        // Character counter
        let total_chars =
            state.ui_state.commit_summary.len() + state.ui_state.commit_description.len();
        ui.horizontal(|ui| {
            ui.label(format!("Characters: {}", total_chars));

            if state.ui_state.is_commit_message_valid() {
                ui.colored_label(egui::Color32::GREEN, "✓ Valid message");
            } else {
                ui.colored_label(egui::Color32::RED, "✗ Summary required");
            }
        });

        ui.separator();

        // Advanced options
        ui.collapsing("Advanced options", |ui| {
            ui.horizontal(|ui| {
                ui.label("Author name:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.author_name)
                        .hint_text("Override author name"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Author email:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.author_email)
                        .hint_text("Override author email"),
                );
            });

            ui.checkbox(
                &mut state.ui_state.show_only_staged,
                "Show only staged files",
            );
            ui.checkbox(
                &mut state.ui_state.show_only_unstaged,
                "Show only unstaged files",
            );
        });

        ui.separator();

        // Commit button and actions
        ui.horizontal(|ui| {
            // Commit button
            let commit_enabled = state.repository_state().staged_files.len() > 0
                && state.ui_state.is_commit_message_valid();

            if ui
                .add_enabled(commit_enabled, egui::Button::new("💾 Commit"))
                .on_hover_text("Create commit with staged changes")
                .clicked()
            {
                self.create_commit(state);
            }

            // Clear button
            if ui.button("🗑️ Clear").clicked() {
                state.ui_state.clear_commit_message();
                state.set_info("Commit message cleared".to_string());
            }

            // Refresh button
            if ui.button("🔄 Refresh").clicked() {
                if let Err(e) = state.refresh_repository() {
                    state.set_error(format!("Failed to refresh repository: {}", e));
                }
            }
        });

        // Commit history link
        ui.horizontal(|ui| {
            ui.label("View commit history:");
            if ui.button("📜 History").clicked() {
                // TODO: Implement commit history view
                state.set_info("Commit history feature not yet implemented".to_string());
            }
        });
    }

    /// Create a commit with the current message
    fn create_commit(&mut self, state: &mut AppState) {
        let message = state.ui_state.commit_message();

        log::info!("Creating commit: {}", message);

        match state.repository_state_mut().create_commit(&message) {
            Ok(()) => {
                state.set_info(format!(
                    "Commit created: {}",
                    message.lines().next().unwrap_or("")
                ));
                state.ui_state.clear_commit_message();

                // Clear file selection after commit
                state.ui_state.clear_file_selection();
            }
            Err(e) => {
                state.set_error(format!("Failed to create commit: {}", e));
            }
        }
    }

    /// Clear author overrides
    pub fn clear_author_overrides(&mut self) {
        self.author_name.clear();
        self.author_email.clear();
    }
}

impl Default for CommitPanel {
    fn default() -> Self {
        Self::new()
    }
}
