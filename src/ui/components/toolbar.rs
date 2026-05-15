use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::path::PathBuf;

const ITEM_H: f32 = 26.0;
const TOOLBAR_H: f32 = 38.0;

pub enum ToolbarAction {
    OpenRepo(PathBuf),
    ShowOpenDialog,
}

pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self { Self }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        recent_repos: &[PathBuf],
    ) -> Option<ToolbarAction> {
        let p = crate::ui::colors::get(state.dark_mode);

        let available_width = ui.available_width();
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, TOOLBAR_H), egui::Sense::hover());
        if !ui.is_rect_visible(bar_rect) { return None; }

        ui.painter().rect_filled(bar_rect, 0.0, p.bg_primary);
        ui.painter().hline(
            bar_rect.min.x..=bar_rect.max.x,
            bar_rect.max.y - 0.5,
            egui::Stroke::new(0.5, p.border),
        );

        let cy = bar_rect.center().y;
        let mut x = bar_rect.min.x + 10.0;
        let mut action: Option<ToolbarAction> = None;

        let repo_name = state.repository_state.as_ref().map(|r| r.model.name.clone()).unwrap_or_default();
        if !repo_name.is_empty() {
            let (pill_rect, pill_resp) = self.pill_button(ui, p, x, cy, &repo_name, None, "pill_repo");
            x = pill_rect.max.x + 4.0;

            let popup_id = ui.id().with("repo_dropdown");
            if pill_resp.clicked() {
                ui.memory_mut(|m| {
                    if m.is_popup_open(popup_id) { m.close_popup(); } else { m.open_popup(popup_id); }
                });
            }

            if ui.memory(|m| m.is_popup_open(popup_id)) {
                let area_resp = egui::Area::new(popup_id)
                    .fixed_pos(pill_rect.left_bottom())
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            ui.set_min_width(pill_rect.width().max(160.0));
                            for path in recent_repos {
                                let name = path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                                if ui.button(&name).clicked() {
                                    action = Some(ToolbarAction::OpenRepo(path.clone()));
                                    ui.memory_mut(|m| m.close_popup());
                                }
                            }
                            if !recent_repos.is_empty() { ui.separator(); }
                            if ui.button("Open other...").clicked() {
                                action = Some(ToolbarAction::ShowOpenDialog);
                                ui.memory_mut(|m| m.close_popup());
                            }
                        });
                    });

                let interact_pos = ui.ctx().input(|i| i.pointer.interact_pos().unwrap_or_default());
                if ui.ctx().input(|i| i.pointer.any_click())
                    && !area_resp.response.rect.contains(interact_pos)
                    && !pill_rect.contains(interact_pos)
                {
                    ui.memory_mut(|m| m.close_popup());
                }
            }

            x = self.vsep(ui, p, x, cy);
        }

        // Snapshot running state so we can disable the relevant button and inline the spinner.
        let (fetch_running, pull_running, push_running, net_progress, net_last_line) =
            match &state.network_status {
                crate::state::NetworkStatus::Running { operation, progress, lines, .. } => {
                    let op = operation.as_str();
                    (op == "Fetch", op == "Pull", op == "Push", *progress, lines.last().cloned())
                }
                _ => (false, false, false, -1.0, None),
            };

        let (fetch_r, fetch_clk) = self.ghost_btn(ui, p, x, cy, "Fetch", false, fetch_running, "btn_fetch");
        ui.interact(fetch_r, ui.id().with("btn_fetch_hover"), egui::Sense::hover())
            .on_hover_text("Ctrl+Shift+F");
        x = fetch_r.max.x + 4.0;
        if fetch_clk && state.has_repository() {
            state.ui_state.pending_action = Some(crate::state::PendingAction::Fetch);
        }
        if fetch_running {
            x = inline_spinner(ui, p, x, cy, net_progress, net_last_line.as_deref());
        }

        let (pull_r, pull_clk) = self.ghost_btn(ui, p, x, cy, "Pull", false, pull_running, "btn_pull");
        ui.interact(pull_r, ui.id().with("btn_pull_hover"), egui::Sense::hover())
            .on_hover_text("Ctrl+Shift+L");
        x = pull_r.max.x + 4.0;
        if pull_clk && state.has_repository() {
            state.ui_state.pending_action = Some(crate::state::PendingAction::Pull);
        }
        if pull_running {
            x = inline_spinner(ui, p, x, cy, net_progress, net_last_line.as_deref());
        }

        let (push_r, push_clk) = self.ghost_btn(ui, p, x, cy, "Push", false, push_running, "btn_push");
        ui.interact(push_r, ui.id().with("btn_push_hover"), egui::Sense::hover())
            .on_hover_text("Ctrl+Shift+P");
        x = push_r.max.x + 4.0;
        if push_clk && state.has_repository() {
            state.ui_state.pending_action = Some(crate::state::PendingAction::Push);
        }
        if push_running {
            x = inline_spinner(ui, p, x, cy, net_progress, net_last_line.as_deref());
        }

        if fetch_running || pull_running || push_running {
            ui.ctx().request_repaint();
        }

        x = self.vsep(ui, p, x, cy);

        let (nb_r, nb_clk) = self.ghost_btn(ui, p, x, cy, "New branch", false, false, "btn_newbranch");
        ui.interact(nb_r, ui.id().with("btn_newbranch_hover"), egui::Sense::hover())
            .on_hover_text("Create a new branch");
        x = nb_r.max.x + 4.0;
        if nb_clk && state.has_repository() {
            state.ui_state.show_create_branch_dialog = true;
        }

        let (st_r, st_clk) = self.ghost_btn(ui, p, x, cy, "Stash", false, false, "btn_stash");
        ui.interact(st_r, ui.id().with("btn_stash_hover"), egui::Sense::hover())
            .on_hover_text("Stash working directory changes");
        x = st_r.max.x + 4.0;
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

        let (_, nt_clk) = self.ghost_btn(ui, p, x, cy, "New tag", false, false, "btn_newtag");
        if nt_clk && state.has_repository() {
            state.ui_state.show_create_tag_dialog = true;
        }

        action
    }

    #[allow(clippy::too_many_arguments)]
    fn pill_button(
        &self,
        ui: &mut egui::Ui,
        p: &Palette,
        x: f32,
        cy: f32,
        label: &str,
        dot_color: Option<egui::Color32>,
        id_key: &str,
    ) -> (egui::Rect, egui::Response) {
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

        (pill_rect, resp)
    }

    #[allow(clippy::too_many_arguments)]
    fn ghost_btn(
        &self,
        ui: &mut egui::Ui,
        p: &Palette,
        x: f32,
        cy: f32,
        label: &str,
        primary: bool,
        disabled: bool,
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

        if disabled {
            ui.painter().text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font,
                p.text_tertiary,
            );
            return (btn_rect, false);
        }

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

/// Render a small inline spinner just after a button, returning the new x position.
fn inline_spinner(
    ui: &mut egui::Ui,
    p: &Palette,
    x: f32,
    cy: f32,
    progress: f32,
    tooltip: Option<&str>,
) -> f32 {
    let r = 7.0_f32;
    let center = egui::pos2(x + r, cy);
    let t = ui.ctx().input(|i| i.time) as f32;
    paint_spinner(ui.painter(), center, r, progress, t, p.bg_tertiary, p.accent_text);

    if let Some(text) = tooltip {
        let area_id = ui.id().with("net_spinner_tip").with(x as i32);
        let area = egui::Rect::from_center_size(center, egui::vec2(r * 2.0 + 4.0, ITEM_H));
        ui.interact(area, area_id, egui::Sense::hover()).on_hover_text(text);
    }

    x + r * 2.0 + 4.0
}

/// Draw a circular progress indicator.
/// `progress < 0` → indeterminate spinning arc; `0..=1` → filled arc clockwise from top.
fn paint_spinner(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    progress: f32,
    t: f32,
    track_color: egui::Color32,
    fill_color: egui::Color32,
) {
    use std::f32::consts::TAU;
    const SEGS: usize = 48;
    let stroke_w = 2.0_f32;

    // Track (full circle)
    let track_stroke = egui::Stroke::new(stroke_w, track_color);
    for i in 0..SEGS {
        let a1 = (i as f32 / SEGS as f32) * TAU;
        let a2 = ((i + 1) as f32 / SEGS as f32) * TAU;
        painter.line_segment(
            [
                egui::pos2(center.x + radius * a1.cos(), center.y + radius * a1.sin()),
                egui::pos2(center.x + radius * a2.cos(), center.y + radius * a2.sin()),
            ],
            track_stroke,
        );
    }

    // Arc (indeterminate or determinate)
    let fill_stroke = egui::Stroke::new(stroke_w, fill_color);
    if progress < 0.0 {
        // Rotating 120° arc
        let arc = TAU / 3.0;
        let start = t * TAU * 0.8; // one rotation every ~1.25 s
        for i in 0..SEGS {
            let frac = i as f32 / SEGS as f32;
            if frac >= arc / TAU { break; }
            let a1 = start + frac * TAU;
            let a2 = start + (frac + 1.0 / SEGS as f32) * TAU;
            painter.line_segment(
                [
                    egui::pos2(center.x + radius * a1.cos(), center.y + radius * a1.sin()),
                    egui::pos2(center.x + radius * a2.cos(), center.y + radius * a2.sin()),
                ],
                fill_stroke,
            );
        }
    } else {
        // Clockwise from top (-π/2), filling to `progress` fraction
        let frac = progress.clamp(0.0, 1.0);
        let start = -TAU / 4.0;
        let arc_segs = ((frac * SEGS as f32) as usize).min(SEGS);
        for i in 0..arc_segs {
            let a1 = start + (i as f32 / SEGS as f32) * TAU;
            let a2 = start + ((i + 1) as f32 / SEGS as f32) * TAU;
            painter.line_segment(
                [
                    egui::pos2(center.x + radius * a1.cos(), center.y + radius * a1.sin()),
                    egui::pos2(center.x + radius * a2.cos(), center.y + radius * a2.sin()),
                ],
                fill_stroke,
            );
        }
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
