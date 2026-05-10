use crate::state::AppState;
use eframe::egui;

const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 244);
const BG_TERTIARY: egui::Color32 = egui::Color32::from_rgb(235, 235, 234);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(26, 26, 24);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(95, 94, 90);
const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(136, 135, 128);
const BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 38);
const BORDER_HOVER: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 77);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(230, 241, 251);
const ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(24, 95, 165);
const ACCENT_BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(7, 29, 50, 77);
const LANE_0: egui::Color32 = egui::Color32::from_rgb(55, 138, 221);

const ITEM_H: f32 = 26.0;
const TOOLBAR_H: f32 = 38.0;

pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let available_width = ui.available_width();
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, TOOLBAR_H), egui::Sense::hover());
        if !ui.is_rect_visible(bar_rect) {
            return;
        }

        ui.painter().rect_filled(bar_rect, 0.0, BG_PRIMARY);
        ui.painter().hline(
            bar_rect.min.x..=bar_rect.max.x,
            bar_rect.max.y - 0.5,
            egui::Stroke::new(0.5, BORDER),
        );

        let cy = bar_rect.center().y;
        let mut x = bar_rect.min.x + 10.0;

        // Repo pill
        let repo_name = state
            .repository_state
            .as_ref()
            .map(|r| r.model.name.clone())
            .unwrap_or_default();
        if !repo_name.is_empty() {
            x = self.pill_button(ui, x, cy, &repo_name, None, "pill_repo");
            x = self.vsep(ui, x, cy);
        }

        // Branch pill
        let branch_name = state
            .repository_state
            .as_ref()
            .and_then(|r| r.branches.iter().find(|b| b.is_current).map(|b| b.name.clone()))
            .unwrap_or_else(|| "—".to_string());
        x = self.pill_button(ui, x, cy, &branch_name, Some(LANE_0), "pill_branch");

        // Sync counter
        ui.painter().text(
            egui::pos2(x + 2.0, cy),
            egui::Align2::LEFT_CENTER,
            "\u{2191}0 \u{2193}0",
            egui::FontId::proportional(11.0),
            TEXT_TERTIARY,
        );
        x += 44.0;

        x = self.vsep(ui, x, cy);

        // Fetch
        let (fetch_r, fetch_clk) = self.ghost_btn(ui, x, cy, "Fetch", false, "btn_fetch");
        x = fetch_r.max.x + 4.0;
        if fetch_clk && state.has_repository() {
            if let Err(e) = state.refresh_repository() {
                state.set_error(format!("Refresh failed: {}", e));
            } else {
                state.set_info("Repository refreshed".to_string());
            }
        }

        // Pull
        let (pull_r, pull_clk) = self.ghost_btn(ui, x, cy, "Pull", false, "btn_pull");
        x = pull_r.max.x + 4.0;
        if pull_clk {
            state.set_info("Pull not yet implemented".to_string());
        }

        // Push
        let (push_r, push_clk) = self.ghost_btn(ui, x, cy, "Push", false, "btn_push");
        x = push_r.max.x + 4.0;
        if push_clk {
            state.set_info("Push not yet implemented".to_string());
        }

        x = self.vsep(ui, x, cy);

        // New branch
        let (nb_r, nb_clk) = self.ghost_btn(ui, x, cy, "New branch", false, "btn_newbranch");
        x = nb_r.max.x + 4.0;
        if nb_clk {
            state.set_info("New branch not yet implemented".to_string());
        }

        // Stash
        let (_, st_clk) = self.ghost_btn(ui, x, cy, "Stash", false, "btn_stash");
        if st_clk {
            state.set_info("Stash not yet implemented".to_string());
        }

        // Settings — far right
        let gear_rect = egui::Rect::from_center_size(
            egui::pos2(bar_rect.max.x - 10.0 - 13.0, cy),
            egui::vec2(26.0, ITEM_H),
        );
        let gear_id = ui.id().with("btn_settings");
        let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
        if gear_resp.hovered() {
            ui.painter().rect_filled(gear_rect, 6.0, BG_SECONDARY);
            ui.painter()
                .rect_stroke(gear_rect, 6.0, egui::Stroke::new(0.5, BORDER));
        }
        ui.painter().text(
            gear_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{2699}",
            egui::FontId::proportional(14.0),
            if gear_resp.hovered() {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            },
        );
        if gear_resp.clicked() {
            state.set_info("Settings not yet implemented".to_string());
        }
    }

    fn pill_button(
        &self,
        ui: &mut egui::Ui,
        x: f32,
        cy: f32,
        label: &str,
        dot_color: Option<egui::Color32>,
        id_key: &str,
    ) -> f32 {
        let font = egui::FontId::proportional(12.0);
        let label_w = ui
            .fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), TEXT_PRIMARY).size().x)
            .min(120.0);
        let dot_w = if dot_color.is_some() { 12.0 } else { 0.0 };
        let pill_w = 8.0 + dot_w + label_w + 18.0;
        let pill_rect = egui::Rect::from_min_size(
            egui::pos2(x, cy - ITEM_H / 2.0),
            egui::vec2(pill_w, ITEM_H),
        );
        let id = ui.id().with(id_key);
        let resp = ui.interact(pill_rect, id, egui::Sense::click());

        let bg = if resp.hovered() { BG_TERTIARY } else { BG_SECONDARY };
        let bd = if resp.hovered() { BORDER_HOVER } else { BORDER };
        ui.painter().rect_filled(pill_rect, 6.0, bg);
        ui.painter()
            .rect_stroke(pill_rect, 6.0, egui::Stroke::new(0.5, bd));

        let mut px = pill_rect.min.x + 8.0;
        let py = pill_rect.center().y;

        if let Some(color) = dot_color {
            ui.painter()
                .circle_filled(egui::pos2(px + 4.0, py), 3.5, color);
            px += 12.0;
        }

        ui.painter().text(
            egui::pos2(px, py),
            egui::Align2::LEFT_CENTER,
            label,
            font,
            TEXT_PRIMARY,
        );

        // Chevron
        ui.painter().text(
            egui::pos2(pill_rect.max.x - 9.0, py),
            egui::Align2::RIGHT_CENTER,
            "\u{02C5}",
            egui::FontId::proportional(10.0),
            TEXT_TERTIARY,
        );

        pill_rect.max.x + 4.0
    }

    fn ghost_btn(
        &self,
        ui: &mut egui::Ui,
        x: f32,
        cy: f32,
        label: &str,
        primary: bool,
        id_key: &str,
    ) -> (egui::Rect, bool) {
        let font = egui::FontId::proportional(12.0);
        let label_w = ui
            .fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), TEXT_SECONDARY).size().x);
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
            (ACCENT_SEL_BG, ACCENT_BORDER, ACCENT_TEXT)
        } else if hov {
            (BG_SECONDARY, BORDER, TEXT_PRIMARY)
        } else {
            (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT, TEXT_SECONDARY)
        };

        if bg != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(btn_rect, 6.0, bg);
        }
        if bd != egui::Color32::TRANSPARENT {
            ui.painter()
                .rect_stroke(btn_rect, 6.0, egui::Stroke::new(0.5, bd));
        }
        ui.painter()
            .text(btn_rect.center(), egui::Align2::CENTER_CENTER, label, font, tc);

        (btn_rect, clk)
    }

    fn vsep(&self, ui: &mut egui::Ui, x: f32, cy: f32) -> f32 {
        ui.painter().vline(
            x + 4.0,
            (cy - 9.0)..=(cy + 9.0),
            egui::Stroke::new(0.5, BORDER),
        );
        x + 12.0
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
