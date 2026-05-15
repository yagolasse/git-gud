use crate::models::{FileChange, FileStatus};
use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::cell::Cell;
use std::path::PathBuf;

/// Which multi-selection action was triggered from the context menu
#[derive(Clone, Copy, PartialEq, Eq)]
enum MultiAction {
    None,
    StageOrUnstage,
    Discard,
}

pub struct FileList {
    staged_open: bool,
    changes_open: bool,
    discard_confirm: Option<PathBuf>,
}

impl FileList {
    pub fn new() -> Self {
        Self { staged_open: true, changes_open: true, discard_confirm: None }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let p = crate::ui::colors::get(state.dark_mode);

        if !state.has_repository() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No repository loaded").color(p.text_tertiary).small());
            });
            return;
        }

        let staged_files = state.repository_state().staged_files.clone();
        let unstaged_files = state.repository_state().unstaged_files.clone();
        let selected_file = state.ui_state.selected_file.clone();

        let mut file_to_select: Option<PathBuf> = None;
        let mut file_to_stage: Option<PathBuf> = None;
        let mut file_to_unstage: Option<PathBuf> = None;
        let mut file_history_for: Option<PathBuf> = None;
        let mut file_discard_want_confirm: Option<PathBuf> = None;
        let mut file_to_discard: Option<PathBuf> = None;
        let mut stage_all = false;
        let mut unstage_all = false;
        let mut multi_stage: Option<Vec<PathBuf>> = None;
        let mut multi_unstage: Option<Vec<PathBuf>> = None;
        let mut multi_discard: Option<Vec<PathBuf>> = None;

        let discard_confirm_path = self.discard_confirm.clone();

        let sel_count = state.ui_state.selection.len();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                if !staged_files.is_empty() {
                    let unstage_all_clicked = Self::show_section_header(
                        ui, p, "STAGED CHANGES", staged_files.len(), &mut self.staged_open, Some("Unstage all"),
                    );
                    if unstage_all_clicked { unstage_all = true; }
                    if self.staged_open {
                        for file in &staged_files {
                            let in_selection = state.ui_state.is_selected(&file.path);
                            let multi_sel = if in_selection { sel_count } else { 0 };
                            let (sel, action, hist, _, _, ctrl_click, multi) =
                                Self::show_file_row(ui, p, &selected_file, file, true, false, multi_sel);
                            if ctrl_click {
                                state.ui_state.toggle_selection(file.path.clone());
                            } else if sel {
                                state.ui_state.clear_selection();
                                file_to_select = Some(file.path.clone());
                            }
                            if action { file_to_unstage = Some(file.path.clone()); }
                            if hist { file_history_for = Some(file.path.clone()); }
                            if multi == MultiAction::StageOrUnstage {
                                // Staged section: multi-action is always unstage
                                multi_unstage = Some(state.ui_state.selection.iter().cloned().collect());
                            }
                        }
                    }
                }

                let action_label = if unstaged_files.is_empty() { None } else { Some("Stage all") };
                let stage_all_clicked = Self::show_section_header(
                    ui, p, "CHANGES", unstaged_files.len(), &mut self.changes_open, action_label,
                );
                if stage_all_clicked { stage_all = true; }
                if self.changes_open {
                    for file in &unstaged_files {
                        let in_confirm = discard_confirm_path.as_ref() == Some(&file.path);
                        let in_selection = state.ui_state.is_selected(&file.path);
                        let multi_sel = if in_selection { sel_count } else { 0 };
                        let (sel, action, hist, want_confirm, confirmed, ctrl_click, multi) =
                            Self::show_file_row(ui, p, &selected_file, file, false, in_confirm, multi_sel);
                        if ctrl_click {
                            state.ui_state.toggle_selection(file.path.clone());
                        } else if sel {
                            state.ui_state.clear_selection();
                            file_to_select = Some(file.path.clone());
                        }
                        if action { file_to_stage = Some(file.path.clone()); }
                        if hist { file_history_for = Some(file.path.clone()); }
                        if want_confirm { file_discard_want_confirm = Some(file.path.clone()); }
                        if confirmed { file_to_discard = Some(file.path.clone()); }
                        if multi == MultiAction::StageOrUnstage {
                            multi_stage = Some(state.ui_state.selection.iter().cloned().collect());
                        }
                        if multi == MultiAction::Discard {
                            multi_discard = Some(state.ui_state.selection.iter().cloned().collect());
                        }
                    }
                    if unstaged_files.is_empty() {
                        Self::show_empty_hint(ui, p, "No changes", 16.0);
                    }
                }
            });

        if let Some(path) = file_to_select {
            state.ui_state.select_file(path);
        }
        if let Some(path) = file_history_for {
            state.ui_state.file_history_path = Some(path);
            state.ui_state.show_file_history = true;
        }
        if let Some(path) = file_to_discard {
            self.discard_confirm = None;
            state.ui_state.pending_action = Some(crate::state::PendingAction::DiscardChanges(path));
        } else if let Some(path) = file_discard_want_confirm {
            self.discard_confirm = Some(path);
        }
        if stage_all {
            let paths: Vec<_> = state.repository_state().unstaged_files.iter().map(|f| f.path.clone()).collect();
            state.ui_state.pending_action = Some(crate::state::PendingAction::StageAll(paths));
        }
        if unstage_all {
            let paths: Vec<_> = state.repository_state().staged_files.iter().map(|f| f.path.clone()).collect();
            state.ui_state.pending_action = Some(crate::state::PendingAction::UnstageAll(paths));
        }
        if let Some(paths) = multi_stage {
            state.ui_state.pending_action = Some(crate::state::PendingAction::StageSelected(paths));
            state.ui_state.clear_selection();
        }
        if let Some(paths) = multi_unstage {
            state.ui_state.pending_action = Some(crate::state::PendingAction::UnstageSelected(paths));
            state.ui_state.clear_selection();
        }
        if let Some(paths) = multi_discard {
            let has_repo = state.repository_state.is_some();
            if has_repo {
                // Discard each path individually; ignore per-file errors (best effort)
                for path in &paths {
                    if let Some(repo_state) = &state.repository_state {
                        let _ = crate::services::GitService::discard_changes(&repo_state.repository, path);
                    }
                }
                let _ = state.refresh_repository();
            }
            state.ui_state.clear_selection();
        }
        if let Some(path) = file_to_stage {
            if let Err(e) = state.repository_state_mut().stage_files(std::slice::from_ref(&path)) {
                state.set_error(format!("Failed to stage: {}", e));
            } else {
                state.set_info(format!("Staged: {}", path.display()));
                state.ui_state.mark_files_staged_or_unstaged();
            }
        }
        if let Some(path) = file_to_unstage {
            if let Err(e) = state.repository_state_mut().unstage_files(std::slice::from_ref(&path)) {
                state.set_error(format!("Failed to unstage: {}", e));
            } else {
                state.set_info(format!("Unstaged: {}", path.display()));
                state.ui_state.mark_files_staged_or_unstaged();
            }
        }
    }

    fn show_section_header(
        ui: &mut egui::Ui,
        p: &Palette,
        title: &str,
        count: usize,
        open: &mut bool,
        action_label: Option<&str>,
    ) -> bool {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if response.hovered() { p.bg_tertiary } else { p.bg_secondary };
            ui.painter().rect_filled(rect, 0.0, bg);
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y - 0.5,
                egui::Stroke::new(0.5, p.border),
            );

            let y = rect.center().y;
            paint_chevron(ui.painter(), egui::pos2(rect.min.x + 11.0, y), *open, p.text_tertiary);
            ui.painter().text(
                egui::pos2(rect.min.x + 18.0, y),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(10.0),
                p.text_secondary,
            );

            let title_font = egui::FontId::proportional(10.0);
            let title_w = ui.fonts(|f| {
                f.layout_no_wrap(title.to_string(), title_font.clone(), p.text_secondary).size().x
            });

            let badge_font = egui::FontId::proportional(10.0);
            let badge_text = count.to_string();
            let badge_text_w =
                ui.fonts(|f| f.layout_no_wrap(badge_text.clone(), badge_font.clone(), p.text_secondary).size().x);
            let badge_w = badge_text_w + 10.0;
            let badge_rect = egui::Rect::from_center_size(
                egui::pos2(rect.min.x + 22.0 + title_w + badge_w / 2.0, y),
                egui::vec2(badge_w, 14.0),
            );
            ui.painter().rect_filled(badge_rect, 7.0, p.bg_tertiary);
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                badge_text,
                badge_font,
                p.text_secondary,
            );
        }

        if response.clicked() {
            *open = !*open;
        }

        let mut action_clicked = false;
        if let Some(label) = action_label
            && response.hovered() {
                let text_w = ui.fonts(|f| {
                    f.layout_no_wrap(
                        label.to_string(),
                        egui::FontId::proportional(10.0),
                        p.text_secondary,
                    )
                    .size()
                    .x
                });
                let btn_w = text_w + 10.0;
                let btn_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - btn_w - 4.0, rect.min.y + 3.0),
                    egui::vec2(btn_w, 18.0),
                );
                let btn_id = ui.id().with(title).with("action");
                let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

                if ui.is_rect_visible(btn_rect) {
                    if btn.hovered() {
                        ui.painter().rect_filled(btn_rect, 3.0, p.bg_tertiary);
                    }
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0),
                        p.text_secondary,
                    );
                }
                action_clicked = btn.clicked();
            }

        action_clicked
    }

    /// Returns: (row_selected, stage_or_unstage, show_history, want_discard_confirm, discard_confirmed, ctrl_click, multi_action)
    ///
    /// `multi_sel_count`: 0 = file not in multi-selection, >1 = in multi-selection with this many files.
    fn show_file_row(
        ui: &mut egui::Ui,
        p: &Palette,
        selected_file: &Option<PathBuf>,
        file: &FileChange,
        is_staged: bool,
        discard_confirm: bool,
        multi_sel_count: usize,
    ) -> (bool, bool, bool, bool, bool, bool, MultiAction) {
        let is_selected = selected_file.as_ref() == Some(&file.path);
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
        let ctrl_click = response.clicked() && ctrl_held;

        let y = rect.center().y;
        // Use a fixed icon slot on the right edge — same rect for both badge and action button
        let icon_center = egui::pos2(rect.max.x - 15.0, y);
        let btn_rect = egui::Rect::from_center_size(icon_center, egui::vec2(18.0, 18.0));
        let btn_id = ui.id().with(&file.path).with(is_staged).with("action");
        let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

        let in_multi_selection = multi_sel_count > 0;

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() || btn.hovered();
            let bg = if is_selected || in_multi_selection {
                p.accent_sel_bg
            } else if hovered {
                p.bg_secondary
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, 0.0, bg);

            let (badge_letter, badge_color) = status_badge(file, p);

            let dot_x = rect.min.x + 22.0;
            ui.painter().circle_filled(egui::pos2(dot_x, y), 3.5, extension_color(&file.path, p));

            let text_color = if is_selected { p.accent_text } else { p.text_primary };
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

            let icon_left = icon_center.x - 9.0;
            let path_x = name_rect.max.x + 5.0;
            if path_x < icon_left - 4.0
                && let Some(parent) = file.path.parent() {
                    let parent_str = parent.to_string_lossy();
                    if !parent_str.is_empty() && parent_str != "." {
                        ui.painter().text(
                            egui::pos2(path_x, y),
                            egui::Align2::LEFT_CENTER,
                            parent_str.as_ref(),
                            egui::FontId::proportional(10.0),
                            p.text_tertiary,
                        );
                    }
                }

            let can_stage = file.status != FileStatus::Conflicted;
            if hovered && can_stage {
                let action_char = if is_staged { "-" } else { "+" };
                ui.painter().rect_filled(
                    btn_rect,
                    3.0,
                    if btn.hovered() {
                        p.bg_tertiary
                    } else {
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 13)
                    },
                );
                ui.painter().text(
                    icon_center,
                    egui::Align2::CENTER_CENTER,
                    action_char,
                    egui::FontId::proportional(13.0),
                    p.text_secondary,
                );
            } else if !is_selected {
                let sq_rect = egui::Rect::from_center_size(icon_center, egui::vec2(14.0, 14.0));
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

        let action_from_menu = Cell::new(false);
        let history_from_menu = Cell::new(false);
        let discard_want_confirm = Cell::new(false);
        let discard_confirmed = Cell::new(false);
        let multi_action: Cell<MultiAction> = Cell::new(MultiAction::None);
        let can_stage = file.status != FileStatus::Conflicted;
        let can_discard = !is_staged;
        response.context_menu(|ui| {
            if can_stage
                && ui.button(if is_staged { "Unstage" } else { "Stage" }).clicked() {
                    action_from_menu.set(true);
                    ui.close_menu();
                }
            if can_discard {
                let label = if file.status == FileStatus::Untracked { "Delete file" } else { "Discard Changes" };
                let confirm_label = if file.status == FileStatus::Untracked { "Confirm delete?" } else { "Confirm discard?" };
                if discard_confirm {
                    if ui.add(egui::Button::new(confirm_label)
                        .fill(egui::Color32::from_rgb(160, 30, 30))).clicked() {
                        discard_confirmed.set(true);
                        ui.close_menu();
                    }
                } else if ui.button(label).clicked() {
                    // Don't close menu — show confirm button next frame
                    discard_want_confirm.set(true);
                }
            }
            if ui.button("Copy path").clicked() {
                ui.output_mut(|o| o.copied_text = file.path.to_string_lossy().to_string());
                ui.close_menu();
            }
            if ui.button("File History").clicked() {
                history_from_menu.set(true);
                ui.close_menu();
            }
            // Multi-selection actions — only when this file is part of a multi-selection (>1)
            if multi_sel_count > 1 {
                ui.separator();
                if is_staged {
                    if ui.button(format!("Unstage selected ({})", multi_sel_count)).clicked() {
                        multi_action.set(MultiAction::StageOrUnstage);
                        ui.close_menu();
                    }
                } else {
                    if ui.button(format!("Stage selected ({})", multi_sel_count)).clicked() {
                        multi_action.set(MultiAction::StageOrUnstage);
                        ui.close_menu();
                    }
                    if can_discard && ui.button(format!("Discard selected ({})", multi_sel_count)).clicked() {
                        multi_action.set(MultiAction::Discard);
                        ui.close_menu();
                    }
                }
            }
        });

        let action = (btn.clicked() || action_from_menu.get()) && can_stage;
        // (row_selected, stage/unstage, show_history, want_discard_confirm, discard_confirmed, ctrl_click, multi_action)
        (
            response.clicked() && !btn.clicked() && !ctrl_held,
            action,
            history_from_menu.get(),
            discard_want_confirm.get(),
            discard_confirmed.get(),
            ctrl_click,
            multi_action.get(),
        )
    }

    fn show_empty_hint(ui: &mut egui::Ui, p: &Palette, text: &str, indent: f32) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(egui::RichText::new(text).color(p.text_tertiary).small());
        });
    }
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}

fn status_badge(file: &FileChange, p: &Palette) -> (String, egui::Color32) {
    match file.status {
        FileStatus::Modified    => ("M".into(), p.status_modified),
        FileStatus::Added       => ("A".into(), p.status_added),
        FileStatus::Deleted     => ("D".into(), p.status_deleted),
        FileStatus::Untracked   => ("U".into(), p.status_added),
        FileStatus::Renamed     => ("R".into(), p.status_modified),
        FileStatus::Copied      => ("C".into(), p.status_added),
        FileStatus::Ignored     => ("I".into(), p.text_tertiary),
        FileStatus::Unmodified  => ("·".into(), p.text_tertiary),
        FileStatus::Conflicted  => {
            let label = file.conflict_count
                .filter(|&n| n > 0)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "!".into());
            (label, p.status_deleted)
        }
    }
}

fn extension_color(path: &std::path::Path, p: &Palette) -> egui::Color32 {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts") | Some("tsx") => p.file_ts,
        Some("js") | Some("jsx") => egui::Color32::from_rgb(70, 150, 220),
        Some("md")               => p.file_md,
        Some("lock")             => p.file_lock,
        Some("rs")               => egui::Color32::from_rgb(222, 165, 132),
        Some("py")               => egui::Color32::from_rgb(70, 130, 180),
        Some("go")               => egui::Color32::from_rgb(0, 173, 216),
        Some("toml") | Some("yaml") | Some("yml") => egui::Color32::from_rgb(207, 134, 76),
        _                        => p.text_tertiary,
    }
}

fn paint_chevron(painter: &egui::Painter, center: egui::Pos2, open: bool, color: egui::Color32) {
    let points = if open {
        vec![
            egui::pos2(center.x - 4.0, center.y - 2.5),
            egui::pos2(center.x + 4.0, center.y - 2.5),
            egui::pos2(center.x, center.y + 2.5),
        ]
    } else {
        vec![
            egui::pos2(center.x - 2.5, center.y - 4.0),
            egui::pos2(center.x + 2.5, center.y),
            egui::pos2(center.x - 2.5, center.y + 4.0),
        ]
    };
    painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
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
