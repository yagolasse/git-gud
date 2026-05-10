use crate::state::AppState;
use eframe::egui;

const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(95, 95, 100);
const STATUS_MODIFIED: egui::Color32 = egui::Color32::from_rgb(226, 167, 75);
const STATUS_DELETED: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(9, 71, 113);
const ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(100, 170, 240);

pub struct CommitPanel {
    amend: bool,
}

impl CommitPanel {
    pub fn new() -> Self {
        Self { amend: false }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if !state.has_repository() {
            return;
        }

        ui.separator();

        let staged_count = state.repository_state().staged_files.len();

        // Subject line
        ui.add(
            egui::TextEdit::singleline(&mut state.ui_state.commit_summary)
                .hint_text("Summary (required)")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(2.0);

        // Body textarea
        ui.add(
            egui::TextEdit::multiline(&mut state.ui_state.commit_description)
                .hint_text("Description (optional)")
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );

        // Character hint — right-aligned, only shown when summary is non-empty
        let n = state.ui_state.commit_summary.len();
        if n > 0 {
            let (hint_text, hint_color) = if n <= 50 {
                (format!("{}/50", n), TEXT_DIM)
            } else if n <= 72 {
                (format!("{}/72 — getting long", n), STATUS_MODIFIED)
            } else {
                (format!("{} — over recommended limit", n), STATUS_DELETED)
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(hint_text).color(hint_color).small());
            });
        }

        ui.add_space(2.0);

        // Footer: Amend checkbox + Commit button
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.amend, "Amend");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let commit_enabled = if self.amend {
                    state.ui_state.is_commit_message_valid()
                } else {
                    staged_count > 0 && state.ui_state.is_commit_message_valid()
                };

                let btn_label = if self.amend {
                    "Amend commit".to_string()
                } else if staged_count > 0 {
                    format!(
                        "Commit {} file{}",
                        staged_count,
                        if staged_count == 1 { "" } else { "s" }
                    )
                } else {
                    "Commit".to_string()
                };

                let btn = egui::Button::new(
                    egui::RichText::new(&btn_label).color(ACCENT_TEXT),
                )
                .fill(if commit_enabled {
                    ACCENT_SEL_BG
                } else {
                    egui::Color32::from_rgb(30, 30, 35)
                });

                if ui.add_enabled(commit_enabled, btn).clicked() {
                    self.create_commit(state);
                }
            });
        });
    }

    fn create_commit(&mut self, state: &mut AppState) {
        if self.amend {
            state.set_info("Amend not yet implemented".to_string());
            return;
        }

        let message = state.ui_state.commit_message();
        match state.repository_state_mut().create_commit(&message) {
            Ok(()) => {
                state.set_info(format!(
                    "Commit created: {}",
                    message.lines().next().unwrap_or("")
                ));
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
    fn default() -> Self {
        Self::new()
    }
}
