use crate::state::AppState;
use eframe::egui;
use std::cell::Cell;

// Dark-mode color palette (from design spec, adapted for dark theme)
const BG_SECTION: egui::Color32 = egui::Color32::from_rgb(35, 35, 40);
const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(48, 48, 55);
const TEXT_SECTION: egui::Color32 = egui::Color32::from_rgb(150, 150, 155);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(95, 95, 100);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(9, 71, 113);
const ACCENT_SUCCESS: egui::Color32 = egui::Color32::from_rgb(99, 153, 34);

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
        // Always-visible filter input at the top.
        // desired_width fills remaining space; no trailing add_space to avoid overflow.
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
                ui.label(egui::RichText::new("No repository loaded").color(TEXT_DIM).small());
            });
            return;
        }

        let filter_lower = self.filter.to_lowercase();

        let local_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| !b.is_remote)
                .filter(|b| {
                    filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect()
        };

        let remote_branches: Vec<crate::models::Branch> = {
            let branches = &state.repository_state().branches;
            branches
                .iter()
                .filter(|b| b.is_remote)
                .filter(|b| {
                    filter_lower.is_empty() || b.name.to_lowercase().contains(&filter_lower)
                })
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
                // BRANCHES
                if Self::show_section_header(ui, "BRANCHES", &mut self.branches_open, true) {
                    create_branch = true;
                }
                if self.branches_open {
                    for branch in &local_branches {
                        let (sel, chk) =
                            Self::show_branch_row(ui, &selected_branch, branch, 18.0);
                        if sel {
                            branch_to_select = Some(branch.name.clone());
                        }
                        if chk {
                            branch_to_checkout = Some(branch.name.clone());
                        }
                    }
                    if local_branches.is_empty() {
                        Self::show_empty_hint(ui, "No local branches", 22.0);
                    }
                }

                // REMOTES
                Self::show_section_header(ui, "REMOTES", &mut self.remotes_open, false);
                if self.remotes_open {
                    for branch in &remote_branches {
                        let (sel, chk) =
                            Self::show_branch_row(ui, &selected_branch, branch, 28.0);
                        if sel {
                            branch_to_select = Some(branch.name.clone());
                        }
                        if chk {
                            branch_to_checkout = Some(branch.name.clone());
                        }
                    }
                    if remote_branches.is_empty() {
                        Self::show_empty_hint(ui, "No remotes", 22.0);
                    }
                }

                // Empty sections — placeholder for future data
                Self::show_section_header(ui, "TAGS", &mut self.tags_open, false);
                Self::show_section_header(ui, "STASHES", &mut self.stashes_open, false);
                Self::show_section_header(ui, "SUBMODULES", &mut self.submodules_open, false);
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

    /// Draws a section header row using the painter directly (no child UI allocation).
    /// Returns true if the "+" add button was clicked.
    fn show_section_header(
        ui: &mut egui::Ui,
        title: &str,
        open: &mut bool,
        show_add: bool,
    ) -> bool {
        let available_width = ui.available_width();
        // Allocate exactly the row height — one allocation only, no child UI.
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
                title,
                font,
                TEXT_SECTION,
            );
        }

        if response.clicked() {
            *open = !*open;
        }

        // "+" button: uses ui.interact() on a sub-rect so no extra space is allocated.
        let mut add_clicked = false;
        if show_add && response.hovered() {
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - 22.0, rect.min.y + 2.0),
                egui::vec2(18.0, 20.0),
            );
            let btn_id = ui.id().with(title).with("add");
            let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

            if ui.is_rect_visible(btn_rect) {
                if btn.hovered() {
                    ui.painter()
                        .rect_filled(btn_rect, 3.0, egui::Color32::from_rgb(60, 60, 70));
                }
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "+",
                    egui::FontId::proportional(14.0),
                    TEXT_SECTION,
                );
            }

            add_clicked = btn.clicked();
        }

        add_clicked
    }

    /// Draws a single branch row using the painter directly. Returns `(selected, checkout)`.
    fn show_branch_row(
        ui: &mut egui::Ui,
        selected_branch: &Option<String>,
        branch: &crate::models::Branch,
        indent: f32,
    ) -> (bool, bool) {
        let is_selected = selected_branch.as_ref() == Some(&branch.name);
        let available_width = ui.available_width();
        // One allocation for the whole row.
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

            let font = egui::FontId::proportional(12.0);
            let y = rect.center().y;
            let mut x = rect.min.x + indent;

            if branch.is_current {
                let check_rect = ui.painter().text(
                    egui::pos2(x, y),
                    egui::Align2::LEFT_CENTER,
                    "✓",
                    font.clone(),
                    ACCENT_SUCCESS,
                );
                x = check_rect.max.x + 4.0;
            }

            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(212, 212, 212)
            };

            let name = if branch.is_current {
                // Bold via font weight isn't directly available in egui's
                // proportional fonts here, so we just paint with the same font.
                branch.name.clone()
            } else {
                branch.name.clone()
            };

            ui.painter()
                .text(egui::pos2(x, y), egui::Align2::LEFT_CENTER, name, font, text_color);
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

        let checkout =
            (response.double_clicked() || checkout_from_menu.get()) && !branch.is_current;
        (response.clicked(), checkout)
    }

    fn show_empty_hint(ui: &mut egui::Ui, text: &str, indent: f32) {
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.label(egui::RichText::new(text).color(TEXT_DIM).small());
        });
    }
}

impl Default for BranchList {
    fn default() -> Self {
        Self::new()
    }
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
