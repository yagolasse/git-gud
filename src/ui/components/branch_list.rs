use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::cell::Cell;

pub struct BranchList {
    filter: String,
    branches_open: bool,
    remotes_open: bool,
    tags_open: bool,
    stashes_open: bool,
    submodules_open: bool,
}

impl BranchList {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            branches_open: true,
            remotes_open: true,
            tags_open: false,
            stashes_open: false,
            submodules_open: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let p = crate::ui::colors::get(state.dark_mode);

        ui.add_space(6.0);
        egui::Frame::none()
            .inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 0.0, bottom: 0.0 })
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("Filter…")
                        .desired_width(f32::INFINITY),
                );
            });
        ui.add_space(4.0);

        if !state.has_repository() {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No repository loaded").color(p.text_tertiary).small());
            });
            return;
        }

        let filter_lower = self.filter.to_lowercase();

        let local_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| !b.is_remote)
                .filter(|b| filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        };

        let remote_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| b.is_remote)
                .filter(|b| filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        };

        let stashes: Vec<crate::models::StashEntry> = state.repository_state().stashes.clone();
        let selected_branch = state.ui_state.selected_branch.clone();
        let mut branch_to_select: Option<String> = None;
        let mut branch_to_checkout: Option<String> = None;
        let mut branch_to_delete: Option<String> = None;
        let mut create_branch = false;
        let mut stash_to_pop: Option<usize> = None;
        let mut stash_to_drop: Option<usize> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if Self::show_section_header(ui, p, "BRANCHES", &mut self.branches_open, true) {
                    create_branch = true;
                }
                if self.branches_open {
                    for branch in &local_branches {
                        let (sel, chk, del) = Self::show_branch_row(ui, p, &selected_branch, branch, 18.0);
                        if sel { branch_to_select = Some(branch.name.clone()); }
                        if chk { branch_to_checkout = Some(branch.name.clone()); }
                        if del { branch_to_delete = Some(branch.name.clone()); }
                    }
                    if local_branches.is_empty() {
                        Self::show_empty_hint(ui, p, "No local branches", 22.0);
                    }
                }

                Self::show_section_header(ui, p, "REMOTES", &mut self.remotes_open, false);
                if self.remotes_open {
                    for branch in &remote_branches {
                        let (sel, chk, _del) = Self::show_branch_row(ui, p, &selected_branch, branch, 28.0);
                        if sel { branch_to_select = Some(branch.name.clone()); }
                        if chk { branch_to_checkout = Some(branch.name.clone()); }
                    }
                    if remote_branches.is_empty() {
                        Self::show_empty_hint(ui, p, "No remotes", 22.0);
                    }
                }

                Self::show_section_header(ui, p, "TAGS", &mut self.tags_open, false);

                Self::show_section_header(ui, p, "STASHES", &mut self.stashes_open, false);
                if self.stashes_open {
                    if stashes.is_empty() {
                        Self::show_empty_hint(ui, p, "No stashes", 22.0);
                    } else {
                        for stash in &stashes {
                            if let Some((pop, drop)) = Self::show_stash_row(ui, p, stash) {
                                if pop { stash_to_pop = Some(stash.index); }
                                if drop { stash_to_drop = Some(stash.index); }
                            }
                        }
                    }
                }

                Self::show_section_header(ui, p, "SUBMODULES", &mut self.submodules_open, false);
            });

        if create_branch {
            state.ui_state.show_create_branch_dialog = true;
        }
        if let Some(name) = branch_to_select {
            state.ui_state.select_branch(name);
        }
        if let Some(name) = branch_to_checkout {
            if let Err(e) = state.repository_state_mut().checkout_branch(&name) {
                state.set_error(format!("Failed to checkout {}: {}", name, e));
            } else {
                state.set_info(format!("Checked out: {}", name));
            }
        }
        if let Some(name) = branch_to_delete {
            match state.repository_state_mut().delete_branch(&name) {
                Ok(()) => state.set_info(format!("Branch '{}' deleted", name)),
                Err(e) => state.set_error(format!("Failed to delete '{}': {}", name, e)),
            }
        }
        if let Some(index) = stash_to_pop {
            match state.repository_state_mut().stash_pop(index) {
                Ok(()) => state.set_info("Stash applied and removed".to_string()),
                Err(e) => state.set_error(format!("Failed to pop stash: {}", e)),
            }
        }
        if let Some(index) = stash_to_drop {
            match state.repository_state_mut().stash_drop(index) {
                Ok(()) => state.set_info("Stash dropped".to_string()),
                Err(e) => state.set_error(format!("Failed to drop stash: {}", e)),
            }
        }

        // Create branch dialog
        if state.ui_state.show_create_branch_dialog {
            let ctx = ui.ctx().clone();
            let mut do_create = false;
            let mut do_cancel = false;
            egui::Window::new("Create Branch")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label("Branch name:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.ui_state.new_branch_name)
                            .desired_width(240.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_create = true;
                    }
                    ui.checkbox(&mut state.ui_state.new_branch_checkout, "Checkout after create");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let name_ok = !state.ui_state.new_branch_name.trim().is_empty();
                        if ui.add_enabled(name_ok, egui::Button::new("Create")).clicked() {
                            do_create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            if do_create && !state.ui_state.new_branch_name.trim().is_empty() {
                let name = state.ui_state.new_branch_name.trim().to_string();
                let checkout = state.ui_state.new_branch_checkout;
                state.ui_state.show_create_branch_dialog = false;
                state.ui_state.new_branch_name.clear();
                match state.repository_state_mut().create_branch(&name, checkout) {
                    Ok(()) => state.set_info(format!("Branch '{}' created", name)),
                    Err(e) => state.set_error(format!("Failed to create branch: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_create_branch_dialog = false;
                state.ui_state.new_branch_name.clear();
            }
        }
    }

    fn show_section_header(
        ui: &mut egui::Ui,
        p: &Palette,
        title: &str,
        open: &mut bool,
        show_add: bool,
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
        }

        if response.clicked() {
            *open = !*open;
        }

        let mut add_clicked = false;
        if show_add && response.hovered() {
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 22.0, rect.min.y + 3.0),
                egui::vec2(18.0, 18.0),
            );
            let btn_id = ui.id().with(title).with("add");
            let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

            if ui.is_rect_visible(btn_rect) {
                if btn.hovered() {
                    ui.painter().rect_filled(btn_rect, 3.0, p.bg_tertiary);
                }
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(13.0),
                    p.text_secondary,
                );
            }
            add_clicked = btn.clicked();
        }

        add_clicked
    }

    fn show_branch_row(
        ui: &mut egui::Ui,
        p: &Palette,
        selected_branch: &Option<String>,
        branch: &crate::models::Branch,
        indent: f32,
    ) -> (bool, bool, bool) {
        let is_selected = selected_branch.as_ref() == Some(&branch.name);
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let bg = if is_selected {
                p.accent_sel_bg
            } else if response.hovered() {
                p.bg_tertiary
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, 0.0, bg);

            let font = egui::FontId::proportional(11.0);
            let y = rect.center().y;
            let mut x = rect.min.x + indent;

            if branch.is_current {
                paint_check(ui.painter(), egui::pos2(x + 5.0, y), p.accent_success);
                x += 14.0;
            }

            let text_color = if is_selected { p.accent_text } else { p.text_secondary };
            ui.painter().text(
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                &branch.name,
                font,
                text_color,
            );
        }

        let checkout_from_menu = Cell::new(false);
        let delete_from_menu = Cell::new(false);
        response.context_menu(|ui| {
            if !branch.is_current && ui.button("Checkout").clicked() {
                checkout_from_menu.set(true);
                ui.close_menu();
            }
            if ui.button("Copy name").clicked() {
                ui.output_mut(|o| o.copied_text = branch.name.clone());
                ui.close_menu();
            }
            ui.separator();
            if branch.is_current {
                ui.add_enabled(false, egui::Button::new("Delete branch"));
            } else if ui.button("Delete branch").clicked() {
                delete_from_menu.set(true);
                ui.close_menu();
            }
        });

        let checkout = (response.double_clicked() || checkout_from_menu.get()) && !branch.is_current;
        (response.clicked(), checkout, delete_from_menu.get())
    }

    /// Returns `Some((pop_clicked, drop_clicked))` for the given stash row
    fn show_stash_row(
        ui: &mut egui::Ui,
        p: &Palette,
        stash: &crate::models::StashEntry,
    ) -> Option<(bool, bool)> {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, 24.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let bg = if response.hovered() { p.bg_tertiary } else { egui::Color32::TRANSPARENT };
        if bg != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 0.0, bg);
        }

        let y = rect.center().y;
        let font = egui::FontId::proportional(11.0);

        // Truncate message to fit
        let btn_area_w = 70.0;
        let max_text_w = available_width - 22.0 - btn_area_w;
        ui.painter().text(
            egui::pos2(rect.min.x + 22.0, y),
            egui::Align2::LEFT_CENTER,
            &stash.message,
            font.clone(),
            p.text_secondary,
        );
        // Clip the text visually (painter doesn't clip; we rely on the panel clip rect)
        let _ = max_text_w;

        // Pop / Drop buttons (only visible on hover)
        let mut pop_clicked = false;
        let mut drop_clicked = false;
        if response.hovered() {
            let pop_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 66.0, rect.min.y + 3.0),
                egui::vec2(28.0, 18.0),
            );
            let drop_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 34.0, rect.min.y + 3.0),
                egui::vec2(30.0, 18.0),
            );
            let pop_id = ui.id().with("stash_pop").with(stash.index);
            let drop_id = ui.id().with("stash_drop").with(stash.index);
            let pop_resp = ui.interact(pop_rect, pop_id, egui::Sense::click());
            let drop_resp = ui.interact(drop_rect, drop_id, egui::Sense::click());

            for (r, label) in [(&pop_resp, "Pop"), (&drop_resp, "Drop")] {
                let btn_rect = if label == "Pop" { pop_rect } else { drop_rect };
                let bg = if r.hovered() { p.bg_secondary } else { egui::Color32::TRANSPARENT };
                if bg != egui::Color32::TRANSPARENT {
                    ui.painter().rect_filled(btn_rect, 3.0, bg);
                }
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(10.0),
                    p.text_secondary,
                );
            }

            pop_clicked = pop_resp.clicked();
            drop_clicked = drop_resp.clicked();
        }

        Some((pop_clicked, drop_clicked))
    }

    fn show_empty_hint(ui: &mut egui::Ui, p: &Palette, text: &str, indent: f32) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(egui::RichText::new(text).color(p.text_tertiary).small());
        });
    }
}

impl Default for BranchList {
    fn default() -> Self {
        Self::new()
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

fn paint_check(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    painter.line_segment(
        [egui::pos2(center.x - 4.0, center.y), egui::pos2(center.x - 1.0, center.y + 3.0)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(center.x - 1.0, center.y + 3.0), egui::pos2(center.x + 4.0, center.y - 3.0)],
        stroke,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_list_new() {
        let list = BranchList::new();
        assert!(list.filter.is_empty());
        assert!(list.branches_open);
        assert!(list.remotes_open);
        assert!(!list.tags_open);
        assert!(!list.stashes_open);
        assert!(!list.submodules_open);
    }
}
