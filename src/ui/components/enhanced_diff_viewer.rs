use crate::models::diff::{DiffConfig, DiffDisplayMode, LineChangeType, SideBySideDiff, UnifiedDiff};
use crate::services::{DiffParser, SyntaxService};
use crate::state::AppState;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

// Chrome (header + action bar) — light
const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 244);
const BG_TERTIARY: egui::Color32 = egui::Color32::from_rgb(235, 235, 234);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(26, 26, 24);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(95, 94, 90);
const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(136, 135, 128);
const BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 38);

// Diff content area — dark
const DIFF_CONTENT_BG: egui::Color32 = egui::Color32::from_rgb(15, 16, 17); // #0f1011
const DIFF_CONTEXT_TEXT: egui::Color32 = egui::Color32::from_rgb(168, 168, 156); // #a8a89c
const DIFF_HUNK_BG: egui::Color32 = egui::Color32::from_rgb(26, 28, 30); // #1a1c1e
const DIFF_HUNK_TEXT: egui::Color32 = egui::Color32::from_rgb(124, 129, 137); // #7c8189

const DIFF_ADD_BG: egui::Color32 = egui::Color32::from_rgb(26, 58, 26);
const DIFF_ADD_TEXT: egui::Color32 = egui::Color32::from_rgb(115, 201, 145);
const DIFF_ADD_GUTTER: egui::Color32 = egui::Color32::from_rgb(30, 63, 30);
const DIFF_REM_BG: egui::Color32 = egui::Color32::from_rgb(58, 26, 26);
const DIFF_REM_TEXT: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);
const DIFF_REM_GUTTER: egui::Color32 = egui::Color32::from_rgb(63, 30, 30);

const STATUS_MODIFIED: egui::Color32 = egui::Color32::from_rgb(226, 167, 75);
const STATUS_ADDED: egui::Color32 = egui::Color32::from_rgb(115, 201, 145);
const STATUS_DELETED: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);

// Gutter layout constants
const GUTTER_OLD: f32 = 32.0; // width reserved for old line number
const GUTTER_NEW: f32 = 32.0; // width reserved for new line number
const GUTTER_PFX: f32 = 12.0; // width reserved for +/- prefix
const GUTTER_TOTAL: f32 = GUTTER_OLD + GUTTER_NEW + GUTTER_PFX;
const ROW_HEIGHT: f32 = 18.0;

pub struct EnhancedDiffViewer {
    config: DiffConfig,
    unified_diff: Option<UnifiedDiff>,
    side_by_side_diff: Option<SideBySideDiff>,
    diff_parser: DiffParser,
    syntax_service: Arc<SyntaxService>,
    last_selected_file: Option<PathBuf>,
    scroll_synced: bool,
    sync_scroll_y: f32,
}

impl EnhancedDiffViewer {
    pub fn new() -> Self {
        Self {
            config: DiffConfig::default(),
            unified_diff: None,
            side_by_side_diff: None,
            diff_parser: DiffParser::new(),
            syntax_service: Arc::new(SyntaxService::new()),
            last_selected_file: None,
            scroll_synced: true,
            sync_scroll_y: 0.0,
        }
    }

    pub fn with_syntax_service(syntax_service: Arc<SyntaxService>) -> Self {
        Self {
            syntax_service,
            ..Self::new()
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        // Detect file changes
        let current_selected_file = state.ui_state.selected_file_path().cloned();
        let needs_refresh = current_selected_file != self.last_selected_file;
        let files_changed = state.ui_state.check_and_reset_staged_unstaged();
        if needs_refresh || files_changed {
            self.refresh_diff(state);
            self.last_selected_file = current_selected_file;
        }

        // Empty state
        if !state.has_repository() || !state.ui_state.has_file_selection() {
            let center = ui.available_rect_before_wrap().center();
            ui.allocate_space(ui.available_size());
            if ui.is_rect_visible(ui.max_rect()) {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "No file selected",
                    egui::FontId::proportional(13.0),
                    TEXT_TERTIARY,
                );
            }
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap().to_path_buf();

        // Compute status badge info and +/- counts
        let status_badge = state.repository_state.as_ref().and_then(|r| {
            r.staged_files
                .iter()
                .chain(r.unstaged_files.iter())
                .find(|f| f.path == selected_file)
                .map(|f| match f.status {
                    crate::models::FileStatus::Modified => ("M", STATUS_MODIFIED),
                    crate::models::FileStatus::Added => ("A", STATUS_ADDED),
                    crate::models::FileStatus::Deleted => ("D", STATUS_DELETED),
                    crate::models::FileStatus::Untracked => ("U", STATUS_ADDED),
                    crate::models::FileStatus::Renamed => ("R", STATUS_MODIFIED),
                    _ => ("·", TEXT_TERTIARY),
                })
        });
        let (added_lines, removed_lines) = match &self.unified_diff {
            Some(ud) => {
                let a = ud
                    .lines
                    .iter()
                    .filter(|l| l.change_type == crate::models::diff::LineChangeType::Added)
                    .count();
                let r = ud
                    .lines
                    .iter()
                    .filter(|l| l.change_type == crate::models::diff::LineChangeType::Removed)
                    .count();
                (a, r)
            }
            None => (0, 0),
        };

        // Header bar — file path + status badge + +/- counts
        Self::show_header_bar(ui, &selected_file, status_badge, added_lines, removed_lines);

        // Action bar — Unified / Split
        self.show_action_bar(ui);

        // Diff content
        match self.config.mode {
            DiffDisplayMode::Unified | DiffDisplayMode::WordLevel => {
                self.show_unified_view(ui, &selected_file);
            }
            DiffDisplayMode::SideBySide => {
                self.show_side_by_side_view(ui, &selected_file);
            }
        }
    }

    fn show_header_bar(
        ui: &mut egui::Ui,
        file_path: &std::path::Path,
        status_badge: Option<(&str, egui::Color32)>,
        added: usize,
        removed: usize,
    ) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 28.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        ui.painter().rect_filled(rect, 0.0, BG_SECONDARY);
        ui.painter().hline(
            rect.min.x..=rect.max.x,
            rect.max.y - 0.5,
            egui::Stroke::new(0.5, BORDER),
        );

        let cy = rect.center().y;
        let mut x = rect.min.x + 10.0;

        // Status badge
        if let Some((letter, color)) = status_badge {
            let sq = egui::Rect::from_center_size(egui::pos2(x + 7.0, cy), egui::vec2(14.0, 14.0));
            ui.painter().rect_filled(sq, 3.0, color);
            ui.painter().text(
                sq.center(),
                egui::Align2::CENTER_CENTER,
                letter,
                egui::FontId::monospace(9.0),
                egui::Color32::WHITE,
            );
            x += 20.0;
        }

        ui.painter().text(
            egui::pos2(x, cy),
            egui::Align2::LEFT_CENTER,
            file_path.display().to_string(),
            egui::FontId::monospace(11.0),
            TEXT_SECONDARY,
        );

        // +N / −N counts (right-aligned)
        let mono = egui::FontId::monospace(11.0);
        let rem_text = format!("\u{2212}{}", removed);
        let rem_galley =
            ui.fonts(|f| f.layout_no_wrap(rem_text.clone(), mono.clone(), DIFF_REM_TEXT));
        let rx = rect.max.x - 10.0 - rem_galley.size().x;
        ui.painter()
            .text(egui::pos2(rx, cy), egui::Align2::LEFT_CENTER, rem_text, mono.clone(), DIFF_REM_TEXT);

        let add_text = format!("+{}", added);
        let add_galley =
            ui.fonts(|f| f.layout_no_wrap(add_text.clone(), mono.clone(), DIFF_ADD_TEXT));
        let ax = rx - add_galley.size().x - 8.0;
        ui.painter()
            .text(egui::pos2(ax, cy), egui::Align2::LEFT_CENTER, add_text, mono, DIFF_ADD_TEXT);
    }

    fn show_action_bar(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 32.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        ui.painter().rect_filled(rect, 0.0, BG_PRIMARY);
        ui.painter().hline(
            rect.min.x..=rect.max.x,
            rect.max.y - 0.5,
            egui::Stroke::new(0.5, BORDER),
        );

        let cy = rect.center().y;
        let btn_h = 24.0;
        let font = egui::FontId::proportional(11.0);

        // Toggle group: border around both buttons together
        let labels = [
            ("Unified", DiffDisplayMode::Unified),
            ("Split", DiffDisplayMode::SideBySide),
        ];
        let widths: Vec<f32> = labels
            .iter()
            .map(|(lbl, _)| {
                ui.fonts(|f| {
                    f.layout_no_wrap(lbl.to_string(), font.clone(), TEXT_SECONDARY)
                        .size()
                        .x
                }) + 20.0
            })
            .collect();
        let total_w: f32 = widths.iter().sum();
        let group_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 10.0, cy - btn_h / 2.0),
            egui::vec2(total_w, btn_h),
        );

        // Outer border
        ui.painter()
            .rect_stroke(group_rect, 6.0, egui::Stroke::new(0.5, BORDER));

        let mut bx = group_rect.min.x;
        for (i, (label, mode)) in labels.iter().enumerate() {
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(bx, group_rect.min.y),
                egui::vec2(widths[i], btn_h),
            );
            let btn_id = ui.id().with("action_bar").with(i);
            let btn_resp = ui.interact(btn_rect, btn_id, egui::Sense::click());

            let active = self.config.mode == *mode;
            if active {
                let rounding = if i == 0 {
                    egui::Rounding { nw: 6.0, sw: 6.0, ne: 0.0, se: 0.0 }
                } else {
                    egui::Rounding { nw: 0.0, sw: 0.0, ne: 6.0, se: 6.0 }
                };
                ui.painter().rect_filled(btn_rect, rounding, BG_TERTIARY);
            }

            ui.painter().text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font.clone(),
                if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            );

            // Divider between buttons
            if i < labels.len() - 1 {
                ui.painter().vline(
                    btn_rect.max.x,
                    (group_rect.min.y)..=(group_rect.max.y),
                    egui::Stroke::new(0.5, BORDER),
                );
            }

            if btn_resp.clicked() {
                self.config.mode = *mode;
            }

            bx += widths[i];
        }

        // Settings icon — far right
        let gear_rect = egui::Rect::from_center_size(
            egui::pos2(rect.max.x - 10.0 - 13.0, cy),
            egui::vec2(26.0, btn_h),
        );
        let gear_id = ui.id().with("diff_gear");
        let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
        if gear_resp.hovered() {
            ui.painter().rect_filled(gear_rect, 4.0, BG_SECONDARY);
        }
        ui.painter().text(
            gear_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{2699}",
            egui::FontId::proportional(13.0),
            if gear_resp.hovered() { TEXT_PRIMARY } else { TEXT_SECONDARY },
        );
    }

    fn show_unified_view(&mut self, ui: &mut egui::Ui, file_path: &std::path::Path) {
        // Fill the entire content rect with the dark diff background
        let content_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(content_rect, 0.0, DIFF_CONTENT_BG);

        let lines = match &self.unified_diff {
            Some(ud) if ud.is_binary => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Binary file").color(DIFF_CONTEXT_TEXT));
                return;
            }
            Some(ud) if ud.lines.is_empty() => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No changes").color(DIFF_CONTEXT_TEXT));
                return;
            }
            Some(ud) => &ud.lines[..],
            None => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No diff loaded").color(DIFF_CONTEXT_TEXT));
                return;
            }
        };

        egui::ScrollArea::vertical()
            .id_source(egui::Id::new("unified_diff").with(file_path))
            .auto_shrink([false, false])
            .show_rows(ui, ROW_HEIGHT, lines.len(), |ui, row_range| {
                for i in row_range {
                    Self::show_diff_row(ui, &lines[i]);
                }
            });
    }

    fn show_diff_row(ui: &mut egui::Ui, line: &crate::models::diff::DiffLine) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, ROW_HEIGHT), egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let (bg, gutter_bg, text_color) = row_colors(&line.change_type);

        // Row background
        if bg != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 0.0, bg);
        }

        // Gutter background
        if gutter_bg != egui::Color32::TRANSPARENT {
            let gutter_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(GUTTER_TOTAL, ROW_HEIGHT),
            );
            ui.painter().rect_filled(gutter_rect, 0.0, gutter_bg);
        }

        let y = rect.center().y;
        let mono = egui::FontId::monospace(11.0);

        // Old line number (right-aligned)
        if let Some(n) = line.left_line_num {
            ui.painter().text(
                egui::pos2(rect.min.x + GUTTER_OLD - 2.0, y),
                egui::Align2::RIGHT_CENTER,
                n.to_string(),
                mono.clone(),
                TEXT_TERTIARY,
            );
        }

        // New line number (right-aligned)
        if let Some(n) = line.right_line_num {
            ui.painter().text(
                egui::pos2(rect.min.x + GUTTER_OLD + GUTTER_NEW - 2.0, y),
                egui::Align2::RIGHT_CENTER,
                n.to_string(),
                mono.clone(),
                TEXT_TERTIARY,
            );
        }

        // Prefix (+/-)
        if line.prefix != ' ' {
            let pfx_color = match line.change_type {
                LineChangeType::Added => DIFF_ADD_TEXT,
                LineChangeType::Removed => DIFF_REM_TEXT,
                _ => TEXT_TERTIARY,
            };
            ui.painter().text(
                egui::pos2(rect.min.x + GUTTER_OLD + GUTTER_NEW + GUTTER_PFX / 2.0, y),
                egui::Align2::CENTER_CENTER,
                line.prefix.to_string(),
                mono.clone(),
                pfx_color,
            );
        }

        // Content
        if !line.content.is_empty() {
            ui.painter().text(
                egui::pos2(rect.min.x + GUTTER_TOTAL + 2.0, y),
                egui::Align2::LEFT_CENTER,
                &line.content,
                mono,
                text_color,
            );
        }
    }

    fn show_side_by_side_view(&mut self, ui: &mut egui::Ui, file_path: &std::path::Path) {
        let content_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(content_rect, 0.0, DIFF_CONTENT_BG);

        let Some(ref side_by_side_diff) = self.side_by_side_diff else {
            ui.label(egui::RichText::new("No diff loaded").color(DIFF_CONTEXT_TEXT));
            return;
        };

        if side_by_side_diff.is_binary {
            ui.label(egui::RichText::new("Binary file").color(DIFF_CONTEXT_TEXT));
            return;
        }

        if side_by_side_diff.is_empty() {
            ui.label(egui::RichText::new("No changes").color(DIFF_CONTEXT_TEXT));
            return;
        }

        let scroll_synced = self.scroll_synced;
        let sync_y = self.sync_scroll_y;
        let left_id = egui::Id::new("left_diff").with(file_path);
        let right_id = egui::Id::new("right_diff").with(file_path);

        let left_lines = side_by_side_diff.left_lines.clone();
        let right_lines = side_by_side_diff.right_lines.clone();
        let mut new_sync_y = sync_y;

        ui.columns(2, |columns| {
            let left_out = egui::ScrollArea::vertical()
                .id_source(left_id)
                .auto_shrink([false, false])
                .show_rows(&mut columns[0], ROW_HEIGHT, left_lines.len(), |ui, range| {
                    for i in range {
                        Self::show_diff_row(ui, &left_lines[i]);
                    }
                });
            new_sync_y = left_out.state.offset.y;

            let mut right_scroll = egui::ScrollArea::vertical()
                .id_source(right_id)
                .auto_shrink([false, false]);
            if scroll_synced {
                right_scroll = right_scroll.scroll_offset(egui::Vec2::new(0.0, sync_y));
            }
            right_scroll.show_rows(&mut columns[1], ROW_HEIGHT, right_lines.len(), |ui, range| {
                for i in range {
                    Self::show_diff_row(ui, &right_lines[i]);
                }
            });
        });

        self.sync_scroll_y = new_sync_y;
    }

    fn refresh_diff(&mut self, state: &mut AppState) {
        if !state.has_repository() || !state.ui_state.has_file_selection() {
            self.unified_diff = None;
            self.side_by_side_diff = None;
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap().to_path_buf();

        if let Some(repo_state) = &state.repository_state {
            match crate::services::GitService::get_file_diff(
                &repo_state.repository,
                &selected_file,
            ) {
                Ok(diff_text) => {
                    let unified = self.diff_parser.parse_unified(&diff_text);
                    self.unified_diff = Some(unified.clone());
                    self.side_by_side_diff =
                        Some(self.diff_parser.unified_to_side_by_side(&unified));
                }
                Err(e) => {
                    self.unified_diff = None;
                    self.side_by_side_diff = None;
                    log::error!("Failed to load diff for {:?}: {}", selected_file, e);
                    state.set_error(format!("Failed to load diff: {}", e));
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.unified_diff = None;
        self.side_by_side_diff = None;
        self.last_selected_file = None;
    }

    pub fn force_refresh(&mut self) {
        self.last_selected_file = None;
    }

    pub fn config(&self) -> &DiffConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DiffConfig {
        &mut self.config
    }

    pub fn syntax_service(&self) -> &Arc<SyntaxService> {
        &self.syntax_service
    }
}

impl Default for EnhancedDiffViewer {
    fn default() -> Self {
        Self::new()
    }
}

fn row_colors(
    change_type: &LineChangeType,
) -> (egui::Color32, egui::Color32, egui::Color32) {
    // (bg, gutter_bg, text_color)
    match change_type {
        LineChangeType::Added => (DIFF_ADD_BG, DIFF_ADD_GUTTER, DIFF_ADD_TEXT),
        LineChangeType::Removed => (DIFF_REM_BG, DIFF_REM_GUTTER, DIFF_REM_TEXT),
        LineChangeType::HunkHeader | LineChangeType::FileHeader => {
            (DIFF_HUNK_BG, DIFF_HUNK_BG, DIFF_HUNK_TEXT)
        }
        _ => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT, DIFF_CONTEXT_TEXT),
    }
}
