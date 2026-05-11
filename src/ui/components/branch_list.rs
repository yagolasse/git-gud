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
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let w = (ui.available_width() - 6.0).max(0.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter…")
                    .desired_width(w),
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

        let selected_branch = state.ui_state.selected_branch.clone();
        let mut branch_to_select: Option<String> = None;
        let mut branch_to_checkout: Option<String> = None;
        let mut create_branch = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if Self::show_section_header(ui, p, "BRANCHES", &mut self.branches_open, true) {
                    create_branch = true;
                }
                if self.branches_open {
                    for branch in &local_branches {
                        let (sel, chk) = Self::show_branch_row(ui, p, &selected_branch, branch, 18.0);
                        if sel { branch_to_select = Some(branch.name.clone()); }
                        if chk { branch_to_checkout = Some(branch.name.clone()); }
                    }
                    if local_branches.is_empty() {
                        Self::show_empty_hint(ui, p, "No local branches", 22.0);
                    }
                }

                Self::show_section_header(ui, p, "REMOTES", &mut self.remotes_open, false);
                if self.remotes_open {
                    for branch in &remote_branches {
                        let (sel, chk) = Self::show_branch_row(ui, p, &selected_branch, branch, 28.0);
                        if sel { branch_to_select = Some(branch.name.clone()); }
                        if chk { branch_to_checkout = Some(branch.name.clone()); }
                    }
                    if remote_branches.is_empty() {
                        Self::show_empty_hint(ui, p, "No remotes", 22.0);
                    }
                }

                Self::show_section_header(ui, p, "TAGS", &mut self.tags_open, false);
                Self::show_section_header(ui, p, "STASHES", &mut self.stashes_open, false);
                Self::show_section_header(ui, p, "SUBMODULES", &mut self.submodules_open, false);
            });

        if create_branch {
            state.set_info("Create branch not yet implemented".to_string());
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
    ) -> (bool, bool) {
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
        response.context_menu(|ui| {
            if !branch.is_current && ui.button("Checkout").clicked() {
                checkout_from_menu.set(true);
                ui.close_menu();
            }
            if ui.button("Copy name").clicked() {
                ui.output_mut(|o| o.copied_text = branch.name.clone());
                ui.close_menu();
            }
        });

        let checkout = (response.double_clicked() || checkout_from_menu.get()) && !branch.is_current;
        (response.clicked(), checkout)
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
