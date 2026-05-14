use crate::state::AppState;
use eframe::egui;

pub struct CommitPanel {
    amend: bool,
}

impl CommitPanel {
    pub fn new() -> Self {
        Self { amend: false }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if !state.has_repository() { return; }

        let p = crate::ui::colors::get(state.dark_mode);

        ui.separator();

        let staged_count = state.repository_state().staged_files.len();
        let has_conflicts = state.repository_state().unstaged_files.iter()
            .any(|f| f.status == crate::models::FileStatus::Conflicted);

        ui.add(
            egui::TextEdit::singleline(&mut state.ui_state.commit_summary)
                .hint_text("Summary (required)")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(2.0);

        ui.add(
            egui::TextEdit::multiline(&mut state.ui_state.commit_description)
                .hint_text("Description (optional)")
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );

        let n = state.ui_state.commit_summary.len();
        let (hint_text, hint_color) = if n <= 50 {
            (format!("{}/50", n), p.text_tertiary)
        } else if n <= 72 {
            (format!("{}/72 — getting long", n), p.status_modified)
        } else {
            (format!("{} — over recommended limit", n), p.status_deleted)
        };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.label(egui::RichText::new(hint_text).color(hint_color).small());
        });

        ui.add_space(2.0);

        if has_conflicts {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Resolve conflicts before committing")
                    .color(p.status_deleted)
                    .small(),
            );
        }

        ui.horizontal(|ui| {
            let checkbox_resp = ui.checkbox(&mut self.amend, "Amend");
            if checkbox_resp.changed() && self.amend {
                state.prefill_amend_message();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let commit_enabled = if has_conflicts {
                    false
                } else if self.amend {
                    state.ui_state.is_commit_message_valid()
                } else {
                    staged_count > 0 && state.ui_state.is_commit_message_valid()
                };

                let btn_label = if self.amend {
                    "Amend commit".to_string()
                } else if staged_count > 0 {
                    format!("Commit {} file{}", staged_count, if staged_count == 1 { "" } else { "s" })
                } else {
                    "Commit".to_string()
                };

                let btn = egui::Button::new(egui::RichText::new(&btn_label).color(
                    if commit_enabled { p.accent_text } else { p.text_tertiary },
                ))
                .fill(p.accent_sel_bg)
                .stroke(egui::Stroke::new(0.5, p.accent_border));

                if ui.add_enabled(commit_enabled, btn).clicked() {
                    self.create_commit(state);
                }
            });
        });
    }

    fn create_commit(&mut self, state: &mut AppState) {
        if self.amend {
            let summary = state.ui_state.commit_summary.clone();
            let description = state.ui_state.commit_description.clone();
            match state.repository_state_mut().amend_commit(&summary, &description) {
                Ok(()) => {
                    state.set_info(format!("Commit amended: {}", summary));
                    state.ui_state.clear_commit_message();
                    state.ui_state.clear_file_selection();
                    self.amend = false;
                }
                Err(e) => state.set_error(format!("Failed to amend commit: {}", e)),
            }
            return;
        }
        let message = state.ui_state.commit_message();
        match state.repository_state_mut().create_commit(&message) {
            Ok(()) => {
                state.set_info(format!("Commit created: {}", message.lines().next().unwrap_or("")));
                state.ui_state.clear_commit_message();
                state.ui_state.clear_file_selection();
            }
            Err(e) => {
                state.set_error(format!("Failed to create commit: {}", e));
            }
        }
    }
}

impl Default for CommitPanel {
    fn default() -> Self { Self::new() }
}
