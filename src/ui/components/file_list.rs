use crate::models::{FileChange, FileStatus};
use crate::state::AppState;
use eframe::egui;
use std::cell::Cell;
use std::path::PathBuf;

const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 244);
const BG_TERTIARY: egui::Color32 = egui::Color32::from_rgb(235, 235, 234);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(26, 26, 24);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(95, 94, 90);
const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(136, 135, 128);
const BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 38);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(230, 241, 251);
const ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(24, 95, 165);

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
                ui.label(egui::RichText::new("No repository loaded").color(TEXT_TERTIARY).small());
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
            let bg = if response.hovered() { BG_TERTIARY } else { BG_SECONDARY };
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y - 0.5,
                egui::Stroke::new(0.5, BORDER),
            );

            let y = rect.center().y;

            ui.painter().text(
                egui::pos2(rect.min.x + 6.0, y),
                egui::Align2::LEFT_CENTER,
                if *open { "\u{25BE}" } else { "\u{25B8}" },
                egui::FontId::proportional(10.0),
                TEXT_TERTIARY,
            );
            ui.painter().text(
                egui::pos2(rect.min.x + 18.0, y),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(10.0),
                TEXT_SECONDARY,
            );

            // Count badge — rounded pill
            let badge_font = egui::FontId::proportional(10.0);
            let badge_text = count.to_string();
            let badge_galley =
                ui.fonts(|f| f.layout_no_wrap(badge_text.clone(), badge_font.clone(), TEXT_SECONDARY));
            let badge_w = badge_galley.size().x + 10.0;
            let badge_rect = egui::Rect::from_center_size(
                egui::pos2(rect.min.x + 18.0 + badge_galley.size().x + badge_w / 2.0 + 6.0, y),
                egui::vec2(badge_w, 14.0),
            );
            ui.painter().rect_filled(badge_rect, 7.0, BG_TERTIARY);
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                badge_text,
                badge_font,
                TEXT_SECONDARY,
            );
        }

        if response.clicked() {
            *open = !*open;
        }

        let mut action_clicked = false;
        if let Some(label) = action_label {
            if response.hovered() {
                // Use a simple icon character for the action button
                let action_char = if label.contains("Stage") || label.contains("+") {
                    "+"
                } else {
                    "\u{2212}"
                };
                let btn_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - 22.0, rect.min.y + 3.0),
                    egui::vec2(18.0, 18.0),
                );
                let btn_id = ui.id().with(title).with("action");
                let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    if btn.hovered() {
                        ui.painter().rect_filled(btn_rect, 3.0, BG_TERTIARY);
                    }
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        action_char,
                        egui::FontId::proportional(13.0),
                        TEXT_SECONDARY,
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
                BG_SECONDARY
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

            let text_color = if is_selected { ACCENT_TEXT } else { TEXT_PRIMARY };

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
                egui::FontId::proportional(11.0),
                text_color,
            );

            // Dimmed parent directory (if space allows)
            let badge_x = rect.max.x - 18.0;
            let path_x = name_rect.max.x + 5.0;
            if path_x < badge_x - 24.0 {
                if let Some(parent) = file.path.parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() && parent_str != "." {
                        ui.painter().text(
                            egui::pos2(path_x, y),
                            egui::Align2::LEFT_CENTER,
                            parent_str.as_ref(),
                            egui::FontId::proportional(10.0),
                            TEXT_TERTIARY,
                        );
                    }
                }
            }

            // Status badge — colored filled square with letter (only when not hovered)
            if !response.hovered() && !is_selected {
                let sq_rect = egui::Rect::from_center_size(
                    egui::pos2(badge_x, y),
                    egui::vec2(14.0, 14.0),
                );
                ui.painter().rect_filled(sq_rect, 3.0, badge_color);
                ui.painter().text(
                    sq_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_letter,
                    egui::FontId::monospace(9.0),
                    egui::Color32::WHITE,
                );
            }
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
            let action_char = if is_staged { "\u{2212}" } else { "+" };
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 24.0, rect.min.y + 3.0),
                egui::vec2(18.0, 18.0),
            );
            let btn_id = ui.id().with(&file.path).with("action");
            let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

            if ui.is_rect_visible(btn_rect) {
                ui.painter().rect_filled(
                    btn_rect,
                    3.0,
                    if btn.hovered() { BG_TERTIARY } else { egui::Color32::from_rgba_premultiplied(0, 0, 0, 13) },
                );
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    action_char,
                    egui::FontId::proportional(13.0),
                    TEXT_SECONDARY,
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
            ui.label(egui::RichText::new(text).color(TEXT_TERTIARY).small());
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
        FileStatus::Ignored => ("I", TEXT_TERTIARY),
        FileStatus::Unmodified => ("·", TEXT_TERTIARY),
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
