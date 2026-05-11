use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;

const ITEM_H: f32 = 26.0;
const TOOLBAR_H: f32 = 38.0;

pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self { Self }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let p = crate::ui::colors::get(state.dark_mode);

        let available_width = ui.available_width();
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, TOOLBAR_H), egui::Sense::hover());
        if !ui.is_rect_visible(bar_rect) { return; }

        ui.painter().rect_filled(bar_rect, 0.0, p.bg_primary);
        ui.painter().hline(
            bar_rect.min.x..=bar_rect.max.x,
            bar_rect.max.y - 0.5,
            egui::Stroke::new(0.5, p.border),
        );

        let cy = bar_rect.center().y;
        let mut x = bar_rect.min.x + 10.0;

        let repo_name = state.repository_state.as_ref().map(|r| r.model.name.clone()).unwrap_or_default();
        if !repo_name.is_empty() {
            x = self.pill_button(ui, p, x, cy, &repo_name, None, "pill_repo");
            x = self.vsep(ui, p, x, cy);
        }

        let branch_name = state
            .repository_state
            .as_ref()
            .and_then(|r| r.branches.iter().find(|b| b.is_current).map(|b| b.name.clone()))
            .unwrap_or_else(|| "—".to_string());
        x = self.pill_button(ui, p, x, cy, &branch_name, Some(p.lane_0), "pill_branch");

        let (ahead, behind) = state
            .repository_state
            .as_ref()
            .map(|r| (r.ahead, r.behind))
            .unwrap_or((0, 0));

        {
            let font = egui::FontId::proportional(11.0);
            let up_c = egui::pos2(x + 5.0, cy);
            let up_color = if ahead > 0 { p.accent_success } else { p.text_tertiary };
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(up_c.x - 3.0, up_c.y + 2.0),
                    egui::pos2(up_c.x + 3.0, up_c.y + 2.0),
                    egui::pos2(up_c.x, up_c.y - 2.5),
                ],
                up_color,
                egui::Stroke::NONE,
            ));
            ui.painter().text(egui::pos2(x + 10.0, cy), egui::Align2::LEFT_CENTER, &ahead.to_string(), font.clone(), p.text_tertiary);
            let dn_c = egui::pos2(x + 24.0, cy);
            let dn_color = if behind > 0 { p.accent_success } else { p.text_tertiary };
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(dn_c.x - 3.0, dn_c.y - 2.0),
                    egui::pos2(dn_c.x + 3.0, dn_c.y - 2.0),
                    egui::pos2(dn_c.x, dn_c.y + 2.5),
                ],
                dn_color,
                egui::Stroke::NONE,
            ));
            ui.painter().text(egui::pos2(x + 29.0, cy), egui::Align2::LEFT_CENTER, &behind.to_string(), font, p.text_tertiary);
        }
        x += 44.0;

        x = self.vsep(ui, p, x, cy);

        let (fetch_r, fetch_clk) = self.ghost_btn(ui, p, x, cy, "Fetch", false, "btn_fetch");
        x = fetch_r.max.x + 4.0;
        if fetch_clk && state.has_repository() {
            if let Err(e) = state.refresh_repository() {
                state.set_error(format!("Refresh failed: {}", e));
            } else {
                state.set_info("Repository refreshed".to_string());
            }
        }

        let (pull_r, pull_clk) = self.ghost_btn(ui, p, x, cy, "Pull", false, "btn_pull");
        x = pull_r.max.x + 4.0;
        if pull_clk && state.has_repository() {
            state.ui_state.pending_action = Some(crate::state::PendingAction::Pull);
        }

        let (push_r, push_clk) = self.ghost_btn(ui, p, x, cy, "Push", false, "btn_push");
        x = push_r.max.x + 4.0;
        if push_clk && state.has_repository() {
            state.ui_state.pending_action = Some(crate::state::PendingAction::Push);
        }

        if let crate::state::NetworkStatus::Running { operation, .. } = &state.network_status {
            let label = format!("{}...", operation);
            let font = egui::FontId::proportional(11.0);
            let label_w = ui.fonts(|f| f.layout_no_wrap(label.clone(), font.clone(), p.accent_text).size().x);
            let ind_rect = egui::Rect::from_min_size(egui::pos2(x, cy - ITEM_H / 2.0), egui::vec2(label_w + 12.0, ITEM_H));
            let time = ui.ctx().input(|i| i.time) as usize;
            let dots = &["   ", ".  ", ".. ", "..."][time % 4usize];
            let label_with_dots = format!("{}{}", label.trim_end_matches('.'), dots);
            ui.painter().text(ind_rect.center(), egui::Align2::CENTER_CENTER, &label_with_dots, font, p.accent_text);
            x = ind_rect.max.x + 4.0;
        }

        x = self.vsep(ui, p, x, cy);

        let (nb_r, nb_clk) = self.ghost_btn(ui, p, x, cy, "New branch", false, "btn_newbranch");
        x = nb_r.max.x + 4.0;
        if nb_clk && state.has_repository() {
            state.ui_state.show_create_branch_dialog = true;
        }

        let (_, st_clk) = self.ghost_btn(ui, p, x, cy, "Stash", false, "btn_stash");
        if st_clk && state.has_repository() {
            state.ui_state.show_stash_save_dialog = true;
        }

        // Stash save dialog
        if state.ui_state.show_stash_save_dialog {
            let ctx = ui.ctx().clone();
            let mut do_save = false;
            let mut do_cancel = false;
            egui::Window::new("Stash Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&ctx, |ui| {
                    ui.label("Message (optional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.ui_state.stash_message)
                            .desired_width(240.0),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() { do_save = true; }
                        if ui.button("Cancel").clicked() { do_cancel = true; }
                    });
                });
            if do_save {
                let msg = if state.ui_state.stash_message.is_empty() {
                    "WIP".to_string()
                } else {
                    state.ui_state.stash_message.clone()
                };
                state.ui_state.show_stash_save_dialog = false;
                state.ui_state.stash_message.clear();
                match state.repository_state_mut().stash_save(&msg) {
                    Ok(()) => state.set_info("Stash saved".to_string()),
                    Err(e) => state.set_error(format!("Failed to stash: {}", e)),
                }
            }
            if do_cancel {
                state.ui_state.show_stash_save_dialog = false;
                state.ui_state.stash_message.clear();
            }
        }

        // Settings — far right
        let gear_rect = egui::Rect::from_center_size(
            egui::pos2(bar_rect.max.x - 10.0 - 13.0, cy),
            egui::vec2(26.0, ITEM_H),
        );
        let gear_id = ui.id().with("btn_settings");
        let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
        if gear_resp.hovered() {
            ui.painter().rect_filled(gear_rect, 6.0, p.bg_secondary);
            ui.painter().rect_stroke(gear_rect, 6.0, egui::Stroke::new(0.5, p.border));
        }
        paint_gear(ui.painter(), gear_rect.center(), if gear_resp.hovered() { p.text_primary } else { p.text_secondary });
        if gear_resp.clicked() {
            state.set_info("Settings not yet implemented".to_string());
        }
    }

    fn pill_button(
        &self,
        ui: &mut egui::Ui,
        p: &Palette,
        x: f32,
        cy: f32,
        label: &str,
        dot_color: Option<egui::Color32>,
        id_key: &str,
    ) -> f32 {
        let font = egui::FontId::proportional(12.0);
        let label_w = ui
            .fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), p.text_primary).size().x)
            .min(120.0);
        let dot_w = if dot_color.is_some() { 12.0 } else { 0.0 };
        let pill_w = 8.0 + dot_w + label_w + 18.0;
        let pill_rect = egui::Rect::from_min_size(
            egui::pos2(x, cy - ITEM_H / 2.0),
            egui::vec2(pill_w, ITEM_H),
        );
        let id = ui.id().with(id_key);
        let resp = ui.interact(pill_rect, id, egui::Sense::click());

        let (bg, bd) = if resp.hovered() { (p.bg_tertiary, p.border_hover) } else { (p.bg_secondary, p.border) };
        ui.painter().rect_filled(pill_rect, 6.0, bg);
        ui.painter().rect_stroke(pill_rect, 6.0, egui::Stroke::new(0.5, bd));

        let mut px = pill_rect.min.x + 8.0;
        let py = pill_rect.center().y;

        if let Some(color) = dot_color {
            ui.painter().circle_filled(egui::pos2(px + 4.0, py), 3.5, color);
            px += 12.0;
        }

        ui.painter().text(egui::pos2(px, py), egui::Align2::LEFT_CENTER, label, font, p.text_primary);
        paint_pill_chevron(ui.painter(), egui::pos2(pill_rect.max.x - 8.0, py), p.text_tertiary);

        pill_rect.max.x + 4.0
    }

    fn ghost_btn(
        &self,
        ui: &mut egui::Ui,
        p: &Palette,
        x: f32,
        cy: f32,
        label: &str,
        primary: bool,
        id_key: &str,
    ) -> (egui::Rect, bool) {
        let font = egui::FontId::proportional(12.0);
        let label_w = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), p.text_secondary).size().x);
        let btn_w = label_w + 18.0;
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(x, cy - ITEM_H / 2.0),
            egui::vec2(btn_w, ITEM_H),
        );
        let id = ui.id().with(id_key);
        let resp = ui.interact(btn_rect, id, egui::Sense::click());
        let hov = resp.hovered();
        let clk = resp.clicked();

        let (bg, bd, tc) = if primary {
            (p.accent_sel_bg, p.accent_border, p.accent_text)
        } else if hov {
            (p.bg_secondary, p.border, p.text_primary)
        } else {
            (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT, p.text_secondary)
        };

        if bg != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(btn_rect, 6.0, bg);
        }
        if bd != egui::Color32::TRANSPARENT {
            ui.painter().rect_stroke(btn_rect, 6.0, egui::Stroke::new(0.5, bd));
        }
        ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, label, font, tc);

        (btn_rect, clk)
    }

    fn vsep(&self, ui: &mut egui::Ui, p: &Palette, x: f32, cy: f32) -> f32 {
        ui.painter().vline(x + 4.0, (cy - 9.0)..=(cy + 9.0), egui::Stroke::new(0.5, p.border));
        x + 12.0
    }
}

impl Default for Toolbar {
    fn default() -> Self { Self::new() }
}

fn paint_gear(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    let w = 6.0;
    for dy in [-2.5f32, 0.0, 2.5] {
        painter.line_segment(
            [egui::pos2(center.x - w, center.y + dy), egui::pos2(center.x + w, center.y + dy)],
            stroke,
        );
    }
}

fn paint_pill_chevron(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.2, color);
    painter.line_segment(
        [egui::pos2(center.x - 3.0, center.y - 1.5), egui::pos2(center.x, center.y + 1.5)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(center.x, center.y + 1.5), egui::pos2(center.x + 3.0, center.y - 1.5)],
        stroke,
    );
}
