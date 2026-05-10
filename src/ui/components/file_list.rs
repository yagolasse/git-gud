use crate::models::{FileChange, FileStatus};
use crate::state::AppState;
use eframe::egui;
use std::cell::Cell;
use std::path::PathBuf;

const BG_SECTION: egui::Color32 = egui::Color32::from_rgb(35, 35, 40);
const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(48, 48, 55);
const TEXT_SECTION: egui::Color32 = egui::Color32::from_rgb(150, 150, 155);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(95, 95, 100);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(212, 212, 212);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(9, 71, 113);

const STATUS_MODIFIED: egui::Color32 = egui::Color32::from_rgb(226, 167, 75);
const STATUS_ADDED: egui::Color32 = egui::Color32::from_rgb(115, 201, 145);
const STATUS_DELETED: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);

pub struct FileList {
    staged_open: bool,
    changes_open: bool,
}

impl FileList {
    pub fn new() -> Self {
        Self {
            staged_open: true,
            changes_open: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if !state.has_repository() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No repository loaded").color(TEXT_DIM).small());
            });
            return;
        }

        let staged_files = state.repository_state().staged_files.clone();
        let unstaged_files = state.repository_state().unstaged_files.clone();
        let selected_file = state.ui_state.selected_file.clone();

        let mut file_to_select: Option<PathBuf> = None;
        let mut file_to_stage: Option<PathBuf> = None;
        let mut file_to_unstage: Option<PathBuf> = None;
        let mut stage_all = false;
        let mut unstage_all = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // STAGED CHANGES — only rendered when staged files exist
                if !staged_files.is_empty() {
                    let unstage_all_clicked = Self::show_section_header(
                        ui,
                        "STAGED CHANGES",
                        staged_files.len(),
                        &mut self.staged_open,
                        Some("Unstage all"),
                    );
                    if unstage_all_clicked {
                        unstage_all = true;
                    }
                    if self.staged_open {
                        for file in &staged_files {
                            let (sel, action) =
                                Self::show_file_row(ui, &selected_file, file, true);
                            if sel {
                                file_to_select = Some(file.path.clone());
                            }
                            if action {
                                file_to_unstage = Some(file.path.clone());
                            }
                        }
                    }
                }

                // CHANGES — always visible
                let action_label = if unstaged_files.is_empty() {
                    None
                } else {
                    Some("Stage all")
                };
                let stage_all_clicked = Self::show_section_header(
                    ui,
                    "CHANGES",
                    unstaged_files.len(),
                    &mut self.changes_open,
                    action_label,
                );
                if stage_all_clicked {
                    stage_all = true;
                }
                if self.changes_open {
                    for file in &unstaged_files {
                        let (sel, action) =
                            Self::show_file_row(ui, &selected_file, file, false);
                        if sel {
                            file_to_select = Some(file.path.clone());
                        }
                        if action {
                            file_to_stage = Some(file.path.clone());
                        }
                    }
                    if unstaged_files.is_empty() {
                        Self::show_empty_hint(ui, "No changes", 16.0);
                    }
                }
            });

        if let Some(path) = file_to_select {
            state.ui_state.select_file(path);
        }
        if stage_all {
            let paths: Vec<_> = state
                .repository_state()
                .unstaged_files
                .iter()
                .map(|f| f.path.clone())
                .collect();
            state.ui_state.pending_action = Some(crate::state::PendingAction::StageAll(paths));
        }
        if unstage_all {
            let paths: Vec<_> = state
                .repository_state()
                .staged_files
                .iter()
                .map(|f| f.path.clone())
                .collect();
            state.ui_state.pending_action =
                Some(crate::state::PendingAction::UnstageAll(paths));
        }
        if let Some(path) = file_to_stage {
            if let Err(e) = state.repository_state_mut().stage_files(&[path.clone()]) {
                state.set_error(format!("Failed to stage: {}", e));
            } else {
                state.set_info(format!("Staged: {}", path.display()));
                state.ui_state.mark_files_staged_or_unstaged();
            }
        }
        if let Some(path) = file_to_unstage {
            if let Err(e) = state.repository_state_mut().unstage_files(&[path.clone()]) {
                state.set_error(format!("Failed to unstage: {}", e));
            } else {
                state.set_info(format!("Unstaged: {}", path.display()));
                state.ui_state.mark_files_staged_or_unstaged();
            }
        }
    }

    fn show_section_header(
        ui: &mut egui::Ui,
        title: &str,
        count: usize,
        open: &mut bool,
        action_label: Option<&str>,
    ) -> bool {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if response.hovered() { BG_HOVER } else { BG_SECTION };
            ui.painter().rect_filled(rect, 0.0, bg);

            let font = egui::FontId::proportional(11.0);
            let y = rect.center().y;

            ui.painter().text(
                egui::pos2(rect.min.x + 6.0, y),
                egui::Align2::LEFT_CENTER,
                if *open { "▾" } else { "▸" },
                font.clone(),
                TEXT_DIM,
            );
            ui.painter().text(
                egui::pos2(rect.min.x + 18.0, y),
                egui::Align2::LEFT_CENTER,
                format!("{} ({})", title, count),
                font,
                TEXT_SECTION,
            );
        }

        if response.clicked() {
            *open = !*open;
        }

        let mut action_clicked = false;
        if let Some(label) = action_label {
            if response.hovered() {
                let font = egui::FontId::proportional(10.0);
                let galley = ui.fonts(|f| {
                    f.layout_no_wrap(label.to_string(), font.clone(), TEXT_SECTION)
                });
                let btn_w = galley.size().x + 10.0;
                let btn_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - btn_w - 4.0, rect.min.y + 3.0),
                    egui::vec2(btn_w, 18.0),
                );
                let btn_id = ui.id().with(title).with("action");
                let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    let bg = if btn.hovered() {
                        egui::Color32::from_rgb(60, 60, 70)
                    } else {
                        egui::Color32::from_rgb(50, 50, 60)
                    };
                    ui.painter().rect_filled(btn_rect, 3.0, bg);
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        font,
                        TEXT_SECTION,
                    );
                }
                action_clicked = btn.clicked();
            }
        }

        action_clicked
    }

    fn show_file_row(
        ui: &mut egui::Ui,
        selected_file: &Option<PathBuf>,
        file: &FileChange,
        is_staged: bool,
    ) -> (bool, bool) {
        let is_selected = selected_file.as_ref() == Some(&file.path);
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if is_selected {
                ACCENT_SEL_BG
            } else if response.hovered() {
                BG_HOVER
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, 0.0, bg);

            let y = rect.center().y;
            let (badge_letter, badge_color) = status_badge(&file.status);

            // Extension-colored dot
            let dot_x = rect.min.x + 22.0;
            ui.painter()
                .circle_filled(egui::pos2(dot_x, y), 3.5, extension_color(&file.path));

            let text_color = if is_selected { egui::Color32::WHITE } else { TEXT_PRIMARY };

            // Filename
            let file_name = file
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.to_string_lossy().to_string());

            let name_rect = ui.painter().text(
                egui::pos2(dot_x + 9.0, y),
                egui::Align2::LEFT_CENTER,
                &file_name,
                egui::FontId::proportional(12.0),
                text_color,
            );

            // Dimmed parent directory (if space allows)
            let badge_x = rect.max.x - 20.0;
            let path_x = name_rect.max.x + 6.0;
            if path_x < badge_x - 20.0 {
                if let Some(parent) = file.path.parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() && parent_str != "." {
                        ui.painter().text(
                            egui::pos2(path_x, y),
                            egui::Align2::LEFT_CENTER,
                            parent_str.as_ref(),
                            egui::FontId::proportional(11.0),
                            TEXT_DIM,
                        );
                    }
                }
            }

            // Status badge
            ui.painter().text(
                egui::pos2(badge_x, y),
                egui::Align2::CENTER_CENTER,
                badge_letter,
                egui::FontId::monospace(11.0),
                badge_color,
            );
        }

        // Context menu
        let action_from_menu = Cell::new(false);
        response.context_menu(|ui| {
            if ui
                .button(if is_staged { "Unstage" } else { "Stage" })
                .clicked()
            {
                action_from_menu.set(true);
                ui.close_menu();
            }
            if ui.button("Copy path").clicked() {
                ui.output_mut(|o| o.copied_text = file.path.to_string_lossy().to_string());
                ui.close_menu();
            }
        });

        // Hover action button (+/−)
        let mut action_btn_clicked = false;
        if response.hovered() {
            let action_char = if is_staged { "−" } else { "+" };
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 38.0, rect.min.y + 3.0),
                egui::vec2(18.0, 18.0),
            );
            let btn_id = ui.id().with(&file.path).with("action");
            let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

            if ui.is_rect_visible(btn_rect) {
                if btn.hovered() {
                    ui.painter()
                        .rect_filled(btn_rect, 3.0, egui::Color32::from_rgb(60, 60, 70));
                }
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    action_char,
                    egui::FontId::proportional(14.0),
                    TEXT_SECTION,
                );
            }
            action_btn_clicked = btn.clicked();
        }

        let action = action_btn_clicked || action_from_menu.get();
        // Suppress row selection if the action button was clicked
        (response.clicked() && !action_btn_clicked, action)
    }

    fn show_empty_hint(ui: &mut egui::Ui, text: &str, indent: f32) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(egui::RichText::new(text).color(TEXT_DIM).small());
        });
    }
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}

fn status_badge(status: &FileStatus) -> (&'static str, egui::Color32) {
    match status {
        FileStatus::Modified => ("M", STATUS_MODIFIED),
        FileStatus::Added => ("A", STATUS_ADDED),
        FileStatus::Deleted => ("D", STATUS_DELETED),
        FileStatus::Untracked => ("U", STATUS_ADDED),
        FileStatus::Renamed => ("R", STATUS_MODIFIED),
        FileStatus::Copied => ("C", STATUS_ADDED),
        FileStatus::Ignored => ("I", TEXT_DIM),
        FileStatus::Unmodified => ("·", TEXT_DIM),
    }
}

fn extension_color(path: &std::path::Path) -> egui::Color32 {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts") | Some("tsx") => egui::Color32::from_rgb(49, 120, 198),
        Some("js") | Some("jsx") => egui::Color32::from_rgb(70, 150, 220),
        Some("md") => egui::Color32::from_rgb(94, 158, 110),
        Some("lock") => egui::Color32::from_rgb(232, 168, 74),
        Some("rs") => egui::Color32::from_rgb(222, 165, 132),
        Some("py") => egui::Color32::from_rgb(70, 130, 180),
        Some("go") => egui::Color32::from_rgb(0, 173, 216),
        Some("toml") | Some("yaml") | Some("yml") => egui::Color32::from_rgb(207, 134, 76),
        _ => egui::Color32::from_rgb(150, 150, 155),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_list_new() {
        let list = FileList::new();
        assert!(list.staged_open);
        assert!(list.changes_open);
    }
}
