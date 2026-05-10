use crate::models::diff::{DiffConfig, DiffDisplayMode, LineChangeType, SideBySideDiff, UnifiedDiff};
use crate::services::{DiffParser, SyntaxService};
use crate::state::AppState;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

// Spec color tokens (dark-mode)
const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(30, 30, 35);
const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(150, 150, 155);
const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(95, 95, 100);
const TEXT_CONTEXT: egui::Color32 = egui::Color32::from_rgb(180, 180, 185);
const ACCENT_SEL_BG: egui::Color32 = egui::Color32::from_rgb(9, 71, 113);
const ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(100, 170, 240);

const DIFF_ADD_BG: egui::Color32 = egui::Color32::from_rgb(26, 58, 26);
const DIFF_ADD_TEXT: egui::Color32 = egui::Color32::from_rgb(115, 201, 145);
const DIFF_ADD_GUTTER: egui::Color32 = egui::Color32::from_rgb(30, 63, 30);
const DIFF_REM_BG: egui::Color32 = egui::Color32::from_rgb(58, 26, 26);
const DIFF_REM_TEXT: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);
const DIFF_REM_GUTTER: egui::Color32 = egui::Color32::from_rgb(63, 30, 30);

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

        // Header bar — file path
        Self::show_header_bar(ui, &selected_file);

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

    fn show_header_bar(ui: &mut egui::Ui, file_path: &std::path::Path) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 26.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, BG_SECONDARY);
            ui.painter().text(
                egui::pos2(rect.min.x + 8.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                file_path.display().to_string(),
                egui::FontId::monospace(11.0),
                TEXT_SECONDARY,
            );
        }
    }

    fn show_action_bar(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 26.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(35, 35, 42));

            let font = egui::FontId::proportional(11.0);
            let btn_h = 20.0;
            let btn_y = rect.min.y + (26.0 - btn_h) / 2.0;
            let mut x = rect.min.x + 8.0;

            for (label, mode) in [
                ("Unified", DiffDisplayMode::Unified),
                ("Split", DiffDisplayMode::SideBySide),
            ] {
                let galley =
                    ui.fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), TEXT_SECONDARY));
                let btn_w = galley.size().x + 12.0;
                let btn_rect =
                    egui::Rect::from_min_size(egui::pos2(x, btn_y), egui::vec2(btn_w, btn_h));
                let btn_id = ui.id().with("action_bar").with(label);
                let btn = ui.interact(btn_rect, btn_id, egui::Sense::click());

                let active = self.config.mode == mode;
                let bg = if active {
                    ACCENT_SEL_BG
                } else if btn.hovered() {
                    egui::Color32::from_rgb(50, 50, 60)
                } else {
                    egui::Color32::TRANSPARENT
                };
                let text_color = if active { ACCENT_TEXT } else { TEXT_SECONDARY };

                if ui.is_rect_visible(btn_rect) {
                    ui.painter().rect_filled(btn_rect, 4.0, bg);
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        font.clone(),
                        text_color,
                    );
                }

                if btn.clicked() {
                    self.config.mode = mode;
                }

                x += btn_w + 4.0;
            }
        }
    }

    fn show_unified_view(&mut self, ui: &mut egui::Ui, file_path: &std::path::Path) {
        let lines = match &self.unified_diff {
            Some(ud) if ud.is_binary => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Binary file").color(TEXT_TERTIARY));
                return;
            }
            Some(ud) if ud.lines.is_empty() => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No changes").color(TEXT_TERTIARY));
                return;
            }
            Some(ud) => &ud.lines[..],
            None => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No diff loaded").color(TEXT_TERTIARY));
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
        let Some(ref side_by_side_diff) = self.side_by_side_diff else {
            ui.label(egui::RichText::new("No diff loaded").color(TEXT_TERTIARY));
            return;
        };

        if side_by_side_diff.is_binary {
            ui.label(egui::RichText::new("Binary file").color(TEXT_TERTIARY));
            return;
        }

        if side_by_side_diff.is_empty() {
            ui.label(egui::RichText::new("No changes").color(TEXT_TERTIARY));
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
            (BG_SECONDARY, BG_SECONDARY, TEXT_TERTIARY)
        }
        _ => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT, TEXT_CONTEXT),
    }
}
