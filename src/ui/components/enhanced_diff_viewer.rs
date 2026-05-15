use crate::models::diff::{DiffConfig, DiffDisplayMode, DiffLine, LineChangeType, SideBySideDiff, UnifiedDiff};
use crate::services::{DiffParser, SyntaxService};
use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

const GUTTER_OLD: f32 = 32.0;
const GUTTER_NEW: f32 = 32.0;
const GUTTER_PFX: f32 = 12.0;
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
        Self { syntax_service, ..Self::new() }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        let p = crate::ui::colors::get(state.dark_mode);

        let theme = if state.dark_mode { "base16-ocean.dark" } else { "InspiredGitHub" };
        self.syntax_service.set_theme(theme);

        let current_selected_file = state.ui_state.selected_file_path().cloned();
        let needs_refresh = current_selected_file != self.last_selected_file;
        let files_changed = state.ui_state.check_and_reset_staged_unstaged();
        if needs_refresh || files_changed {
            self.refresh_diff(state);
            self.last_selected_file = current_selected_file;
        }

        if !state.has_repository() || !state.ui_state.has_file_selection() {
            let center = ui.available_rect_before_wrap().center();
            ui.allocate_space(ui.available_size());
            if ui.is_rect_visible(ui.max_rect()) {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "No file selected",
                    egui::FontId::proportional(13.0),
                    p.text_tertiary,
                );
            }
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap().to_path_buf();

        let status_badge = state.repository_state.as_ref().and_then(|r| {
            r.staged_files
                .iter()
                .chain(r.unstaged_files.iter())
                .find(|f| f.path == selected_file)
                .map(|f| match f.status {
                    crate::models::FileStatus::Modified  => ("M", p.status_modified),
                    crate::models::FileStatus::Added     => ("A", p.status_added),
                    crate::models::FileStatus::Deleted   => ("D", p.status_deleted),
                    crate::models::FileStatus::Untracked => ("U", p.status_added),
                    crate::models::FileStatus::Renamed   => ("R", p.status_modified),
                    _                                    => ("·", p.text_tertiary),
                })
        });

        let (added_lines, removed_lines) = match &self.unified_diff {
            Some(ud) => {
                let a = ud.lines.iter().filter(|l| l.change_type == LineChangeType::Added).count();
                let r = ud.lines.iter().filter(|l| l.change_type == LineChangeType::Removed).count();
                (a, r)
            }
            None => (0, 0),
        };

        Self::show_header_bar(ui, p, &selected_file, status_badge, added_lines, removed_lines);
        self.show_action_bar(ui, p);

        let pending = match self.config.mode {
            DiffDisplayMode::Unified | DiffDisplayMode::WordLevel => {
                self.show_unified_view(ui, p, &selected_file)
            }
            DiffDisplayMode::SideBySide => {
                self.show_side_by_side_view(ui, p, &selected_file);
                None
            }
        };
        if let Some(action) = pending {
            state.ui_state.pending_action = Some(action);
        }
    }

    fn show_header_bar(
        ui: &mut egui::Ui,
        p: &Palette,
        file_path: &std::path::Path,
        status_badge: Option<(&str, egui::Color32)>,
        added: usize,
        removed: usize,
    ) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 28.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) { return; }

        ui.painter().rect_filled(rect, 0.0, p.bg_secondary);
        ui.painter().hline(
            rect.min.x..=rect.max.x,
            rect.max.y - 0.5,
            egui::Stroke::new(0.5, p.border),
        );

        let cy = rect.center().y;
        let mut x = rect.min.x + 10.0;

        if let Some((letter, color)) = status_badge {
            let sq = egui::Rect::from_center_size(egui::pos2(x + 7.0, cy), egui::vec2(14.0, 14.0));
            ui.painter().rect_filled(sq, 3.0, color);
            ui.painter().text(
                sq.center(), egui::Align2::CENTER_CENTER, letter,
                egui::FontId::monospace(9.0), egui::Color32::WHITE,
            );
            x += 20.0;
        }

        ui.painter().text(
            egui::pos2(x, cy),
            egui::Align2::LEFT_CENTER,
            file_path.display().to_string(),
            egui::FontId::monospace(11.0),
            p.text_secondary,
        );

        let mono = egui::FontId::monospace(11.0);
        let rem_text = format!("-{}", removed);
        let rem_galley = ui.fonts(|f| f.layout_no_wrap(rem_text.clone(), mono.clone(), p.diff_rem_text));
        let rx = rect.max.x - 10.0 - rem_galley.size().x;
        ui.painter().text(egui::pos2(rx, cy), egui::Align2::LEFT_CENTER, rem_text, mono.clone(), p.diff_rem_text);

        let add_text = format!("+{}", added);
        let add_galley = ui.fonts(|f| f.layout_no_wrap(add_text.clone(), mono.clone(), p.diff_add_text));
        let ax = rx - add_galley.size().x - 8.0;
        ui.painter().text(egui::pos2(ax, cy), egui::Align2::LEFT_CENTER, add_text, mono, p.diff_add_text);
    }

    fn show_action_bar(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, 32.0), egui::Sense::hover());

        if !ui.is_rect_visible(rect) { return; }

        ui.painter().rect_filled(rect, 0.0, p.bg_primary);
        ui.painter().hline(
            rect.min.x..=rect.max.x,
            rect.max.y - 0.5,
            egui::Stroke::new(0.5, p.border),
        );

        let cy = rect.center().y;
        let btn_h = 24.0;
        let font = egui::FontId::proportional(11.0);

        let labels = [
            ("Unified", DiffDisplayMode::Unified),
            ("Split", DiffDisplayMode::SideBySide),
            ("Word", DiffDisplayMode::WordLevel),
        ];
        let widths: Vec<f32> = labels.iter().map(|(lbl, _)| {
            ui.fonts(|f| f.layout_no_wrap(lbl.to_string(), font.clone(), p.text_secondary).size().x) + 20.0
        }).collect();
        let total_w: f32 = widths.iter().sum();

        let group_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 10.0, cy - btn_h / 2.0),
            egui::vec2(total_w, btn_h),
        );
        ui.painter().rect_stroke(group_rect, 6.0, egui::Stroke::new(0.5, p.border));

        let last_idx = labels.len() - 1;
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
                } else if i == last_idx {
                    egui::Rounding { nw: 0.0, sw: 0.0, ne: 6.0, se: 6.0 }
                } else {
                    egui::Rounding::ZERO
                };
                ui.painter().rect_filled(btn_rect, rounding, p.bg_tertiary);
            }

            ui.painter().text(
                btn_rect.center(), egui::Align2::CENTER_CENTER, label, font.clone(),
                if active { p.text_primary } else { p.text_secondary },
            );

            if i < labels.len() - 1 {
                ui.painter().vline(
                    btn_rect.max.x,
                    (group_rect.min.y)..=(group_rect.max.y),
                    egui::Stroke::new(0.5, p.border),
                );
            }
            if btn_resp.clicked() { self.config.mode = *mode; }
            bx += widths[i];
        }

        let gear_rect = egui::Rect::from_center_size(
            egui::pos2(rect.max.x - 10.0 - 13.0, cy),
            egui::vec2(26.0, btn_h),
        );
        let gear_id = ui.id().with("diff_gear");
        let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
        if gear_resp.hovered() {
            ui.painter().rect_filled(gear_rect, 4.0, p.bg_secondary);
        }
        paint_gear(ui.painter(), gear_rect.center(), if gear_resp.hovered() { p.text_primary } else { p.text_secondary });

        let popup_id = ui.make_persistent_id("diff_settings_popup");
        if gear_resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }
        egui::popup_below_widget(ui, popup_id, &gear_resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
            ui.set_min_width(160.0);
            ui.add(egui::Slider::new(&mut self.config.context_lines, 1..=10).text("Context lines"));
            ui.checkbox(&mut self.config.ignore_whitespace, "Ignore whitespace");
            ui.checkbox(&mut self.config.show_line_numbers, "Show line numbers");
            ui.checkbox(&mut self.config.wrap_lines, "Wrap lines");
            ui.checkbox(&mut self.config.syntax_highlighting, "Syntax highlighting");
        });
    }

    fn show_unified_view(&mut self, ui: &mut egui::Ui, p: &'static Palette, file_path: &std::path::Path) -> Option<crate::state::PendingAction> {
        let content_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(content_rect, 0.0, p.diff_content_bg);

        let lines = match &self.unified_diff {
            Some(ud) if ud.is_binary => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Binary file").color(p.diff_context_text));
                return None;
            }
            Some(ud) if ud.lines.is_empty() => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No changes").color(p.diff_context_text));
                return None;
            }
            Some(ud) => &ud.lines[..],
            None => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No diff loaded").color(p.diff_context_text));
                return None;
            }
        };

        let syntax_svc = std::sync::Arc::clone(&self.syntax_service);
        let file_path_owned = file_path.to_path_buf();
        let mut pending: Option<crate::state::PendingAction> = None;
        let word_level = self.config.mode == DiffDisplayMode::WordLevel;

        egui::ScrollArea::vertical()
            .id_salt(egui::Id::new("unified_diff").with(file_path))
            .auto_shrink([false, false])
            .show_rows(ui, ROW_HEIGHT, lines.len(), |ui, row_range| {
                let syntax = syntax_svc.detect_syntax(&file_path_owned);
                for i in row_range {
                    let use_word = word_level && !lines[i].word_changes.is_empty();
                    let job = if !use_word && !lines[i].content.is_empty() && lines[i].change_type != LineChangeType::ConflictSeparator {
                        Some(syntax_svc.highlight_line(&lines[i].content, syntax))
                    } else {
                        None
                    };
                    if let Some(action) = Self::show_diff_row(ui, p, &lines[i], job, &file_path_owned, word_level) {
                        pending = Some(action);
                    }
                }
            });

        pending
    }

    fn show_diff_row(
        ui: &mut egui::Ui,
        p: &Palette,
        line: &crate::models::diff::DiffLine,
        highlight: Option<egui::text::LayoutJob>,
        file_path: &std::path::Path,
        word_level: bool,
    ) -> Option<crate::state::PendingAction> {
        let is_conflict_sep = line.change_type == LineChangeType::ConflictSeparator;
        // Conflict separator rows are taller to accommodate the Accept buttons
        let row_h = if is_conflict_sep { 22.0 } else { ROW_HEIGHT };
        let available_width = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available_width, row_h), egui::Sense::hover());

        if !ui.is_rect_visible(rect) { return None; }

        let (bg, gutter_bg, text_color) = row_colors(&line.change_type, p);

        let painter = ui.painter().with_clip_rect(rect);

        if bg != egui::Color32::TRANSPARENT {
            painter.rect_filled(rect, 0.0, bg);
        }
        if gutter_bg != egui::Color32::TRANSPARENT {
            let gutter_rect = egui::Rect::from_min_size(rect.min, egui::vec2(GUTTER_TOTAL, row_h));
            painter.rect_filled(gutter_rect, 0.0, gutter_bg);
        }

        let y = rect.center().y;
        let mono = egui::FontId::monospace(11.0);

        if let Some(n) = line.left_line_num {
            painter.text(
                egui::pos2(rect.min.x + GUTTER_OLD - 2.0, y),
                egui::Align2::RIGHT_CENTER,
                n.to_string(), mono.clone(), p.text_tertiary,
            );
        }
        if let Some(n) = line.right_line_num {
            painter.text(
                egui::pos2(rect.min.x + GUTTER_OLD + GUTTER_NEW - 2.0, y),
                egui::Align2::RIGHT_CENTER,
                n.to_string(), mono.clone(), p.text_tertiary,
            );
        }
        if line.prefix != ' ' && !is_conflict_sep {
            let pfx_color = match line.change_type {
                LineChangeType::Added   => p.diff_add_text,
                LineChangeType::Removed => p.diff_rem_text,
                _                       => p.text_tertiary,
            };
            painter.text(
                egui::pos2(rect.min.x + GUTTER_OLD + GUTTER_NEW + GUTTER_PFX / 2.0, y),
                egui::Align2::CENTER_CENTER,
                line.prefix.to_string(), mono.clone(), pfx_color,
            );
        }

        // For conflict separator lines that start with "<<<<<<<", render Accept buttons
        if is_conflict_sep && line.content.starts_with("<<<") {
            painter.text(
                egui::pos2(rect.min.x + GUTTER_TOTAL + 4.0, y),
                egui::Align2::LEFT_CENTER,
                &line.content, mono.clone(), text_color,
            );
            let btn_font = egui::FontId::proportional(10.0);
            let btn_h = 16.0;
            let ours_label = "Accept Ours";
            let theirs_label = "Accept Theirs";
            let ours_w = ui.fonts(|f| f.layout_no_wrap(ours_label.into(), btn_font.clone(), p.text_primary).size().x) + 10.0;
            let theirs_w = ui.fonts(|f| f.layout_no_wrap(theirs_label.into(), btn_font.clone(), p.text_primary).size().x) + 10.0;
            let theirs_rect = egui::Rect::from_min_size(
                egui::pos2(rect.max.x - theirs_w - 4.0, y - btn_h / 2.0),
                egui::vec2(theirs_w, btn_h),
            );
            let ours_rect = egui::Rect::from_min_size(
                egui::pos2(theirs_rect.min.x - ours_w - 4.0, y - btn_h / 2.0),
                egui::vec2(ours_w, btn_h),
            );
            let ours_id = ui.id().with("ours").with(line.left_line_num);
            let theirs_id = ui.id().with("theirs").with(line.left_line_num);
            let ours_resp = ui.interact(ours_rect, ours_id, egui::Sense::click());
            let theirs_resp = ui.interact(theirs_rect, theirs_id, egui::Sense::click());
            let painter2 = ui.painter();
            painter2.rect_filled(ours_rect, 3.0, if ours_resp.hovered() { p.diff_add_bg } else { p.bg_tertiary });
            painter2.rect_stroke(ours_rect, 3.0, egui::Stroke::new(0.5, p.border));
            painter2.text(ours_rect.center(), egui::Align2::CENTER_CENTER, ours_label, btn_font.clone(), p.text_primary);
            painter2.rect_filled(theirs_rect, 3.0, if theirs_resp.hovered() { p.diff_rem_bg } else { p.bg_tertiary });
            painter2.rect_stroke(theirs_rect, 3.0, egui::Stroke::new(0.5, p.border));
            painter2.text(theirs_rect.center(), egui::Align2::CENTER_CENTER, theirs_label, btn_font, p.text_primary);
            if ours_resp.clicked() {
                return Some(crate::state::PendingAction::ResolveOurs(file_path.to_path_buf()));
            }
            if theirs_resp.clicked() {
                return Some(crate::state::PendingAction::ResolveTheirs(file_path.to_path_buf()));
            }
            return None;
        }

        if !line.content.is_empty() {
            let content_x = rect.min.x + GUTTER_TOTAL + 2.0;
            if word_level && !line.word_changes.is_empty() {
                let job = build_word_diff_job(p, line, &mono, text_color);
                let galley = ui.fonts(|f| f.layout_job(job));
                let pos = egui::pos2(content_x, rect.min.y + (row_h - galley.size().y) / 2.0);
                painter.galley(pos, galley, text_color);
            } else if let Some(job) = highlight {
                let galley = ui.fonts(|f| f.layout_job(job));
                let pos = egui::pos2(content_x, rect.min.y + (row_h - galley.size().y) / 2.0);
                painter.galley(pos, galley, text_color);
            } else {
                painter.text(
                    egui::pos2(content_x, y),
                    egui::Align2::LEFT_CENTER,
                    &line.content, mono, text_color,
                );
            }
        }

        None
    }

    fn show_side_by_side_view(&mut self, ui: &mut egui::Ui, p: &'static Palette, file_path: &std::path::Path) {
        let content_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(content_rect, 0.0, p.diff_content_bg);

        let Some(ref side_by_side_diff) = self.side_by_side_diff else {
            ui.label(egui::RichText::new("No diff loaded").color(p.diff_context_text));
            return;
        };

        if side_by_side_diff.is_binary {
            ui.label(egui::RichText::new("Binary file").color(p.diff_context_text));
            return;
        }
        if side_by_side_diff.is_empty() {
            ui.label(egui::RichText::new("No changes").color(p.diff_context_text));
            return;
        }

        let scroll_synced = self.scroll_synced;
        let sync_y = self.sync_scroll_y;
        let left_id = egui::Id::new("left_diff").with(file_path);
        let right_id = egui::Id::new("right_diff").with(file_path);
        let left_lines = side_by_side_diff.left_lines.clone();
        let right_lines = side_by_side_diff.right_lines.clone();
        let mut new_sync_y = sync_y;

        let syntax_svc_l = std::sync::Arc::clone(&self.syntax_service);
        let syntax_svc_r = std::sync::Arc::clone(&self.syntax_service);
        let file_path_owned = file_path.to_path_buf();

        ui.columns(2, |columns| {
            let fp = file_path_owned.clone();
            let fp2 = file_path_owned.clone();
            let left_out = egui::ScrollArea::vertical()
                .id_salt(left_id)
                .auto_shrink([false, false])
                .show_rows(&mut columns[0], ROW_HEIGHT, left_lines.len(), |ui, range| {
                    let syntax = syntax_svc_l.detect_syntax(&fp);
                    for i in range {
                        let job = if !left_lines[i].content.is_empty() {
                            Some(syntax_svc_l.highlight_line(&left_lines[i].content, syntax))
                        } else {
                            None
                        };
                        Self::show_diff_row(ui, p, &left_lines[i], job, &fp, false);
                    }
                });
            new_sync_y = left_out.state.offset.y;

            let mut right_scroll = egui::ScrollArea::vertical()
                .id_salt(right_id)
                .auto_shrink([false, false]);
            if scroll_synced {
                right_scroll = right_scroll.scroll_offset(egui::Vec2::new(0.0, sync_y));
            }
            right_scroll.show_rows(&mut columns[1], ROW_HEIGHT, right_lines.len(), |ui, range| {
                let syntax = syntax_svc_r.detect_syntax(&file_path_owned);
                for i in range {
                    let job = if !right_lines[i].content.is_empty() {
                        Some(syntax_svc_r.highlight_line(&right_lines[i].content, syntax))
                    } else {
                        None
                    };
                    Self::show_diff_row(ui, p, &right_lines[i], job, &fp2, false);
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
            let is_conflicted = repo_state.unstaged_files.iter()
                .any(|f| f.path == selected_file && f.status == crate::models::FileStatus::Conflicted);

            if is_conflicted {
                let abs = repo_state.repository.workdir()
                    .map(|wd| wd.join(&selected_file));
                let unified = abs
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|raw| parse_conflict_file(&raw))
                    .unwrap_or_else(|| UnifiedDiff {
                        lines: Vec::new(),
                        old_file_path: None,
                        new_file_path: None,
                        is_binary: false,
                        lines_added: 0,
                        lines_removed: 0,
                    });
                self.unified_diff = Some(unified);
                self.side_by_side_diff = None;
                return;
            }

            match crate::services::GitService::get_file_diff(&repo_state.repository, &selected_file) {
                Ok(diff_text) => {
                    let mut unified = self.diff_parser.parse_unified(&diff_text);
                    self.diff_parser.apply_word_diffs(&mut unified.lines);
                    self.unified_diff = Some(unified.clone());
                    self.side_by_side_diff = Some(self.diff_parser.unified_to_side_by_side(&unified));
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

    pub fn config(&self) -> &DiffConfig { &self.config }
    pub fn config_mut(&mut self) -> &mut DiffConfig { &mut self.config }
    pub fn syntax_service(&self) -> &Arc<SyntaxService> { &self.syntax_service }
}

impl Default for EnhancedDiffViewer {
    fn default() -> Self { Self::new() }
}

fn build_word_diff_job(
    p: &Palette,
    line: &DiffLine,
    mono: &egui::FontId,
    text_color: egui::Color32,
) -> egui::text::LayoutJob {
    let highlight_bg = match line.change_type {
        LineChangeType::Added => p.diff_word_add_bg,
        LineChangeType::Removed => p.diff_word_rem_bg,
        _ => return egui::text::LayoutJob::default(),
    };
    let content = &line.content;
    let mut job = egui::text::LayoutJob::default();
    let mut pos = 0usize;
    for wc in &line.word_changes {
        if content[wc.start..wc.end].trim().is_empty() {
            continue;
        }
        if pos < wc.start {
            job.append(&content[pos..wc.start], 0.0, egui::text::TextFormat {
                font_id: mono.clone(),
                color: text_color,
                ..Default::default()
            });
        }
        job.append(&content[wc.start..wc.end], 0.0, egui::text::TextFormat {
            font_id: mono.clone(),
            color: text_color,
            background: highlight_bg,
            ..Default::default()
        });
        pos = wc.end;
    }
    if pos < content.len() {
        job.append(&content[pos..], 0.0, egui::text::TextFormat {
            font_id: mono.clone(),
            color: text_color,
            ..Default::default()
        });
    }
    job
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

fn row_colors(
    change_type: &LineChangeType,
    p: &Palette,
) -> (egui::Color32, egui::Color32, egui::Color32) {
    match change_type {
        LineChangeType::Added   => (p.diff_add_bg, p.diff_add_gutter, p.diff_add_text),
        LineChangeType::Removed => (p.diff_rem_bg, p.diff_rem_gutter, p.diff_rem_text),
        LineChangeType::HunkHeader | LineChangeType::FileHeader => {
            (p.diff_hunk_bg, p.diff_hunk_bg, p.diff_hunk_text)
        }
        LineChangeType::ConflictOurs => (p.diff_add_bg, p.diff_add_gutter, p.diff_add_text),
        LineChangeType::ConflictTheirs => (p.diff_rem_bg, p.diff_rem_gutter, p.diff_rem_text),
        LineChangeType::ConflictSeparator => (p.diff_hunk_bg, p.diff_hunk_bg, p.diff_hunk_text),
        _ => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT, p.diff_context_text),
    }
}

/// Parse a raw file with conflict markers into a UnifiedDiff for display.
fn parse_conflict_file(raw: &str) -> UnifiedDiff {
    let mut lines = Vec::new();
    #[derive(PartialEq)]
    enum Side { Context, Ours, Theirs }
    let mut side = Side::Context;
    for (i, text) in raw.lines().enumerate() {
        let ln = i + 1;
        let (change_type, content) = if text.starts_with("<<<<<<<") {
            side = Side::Ours;
            (LineChangeType::ConflictSeparator, text.to_owned())
        // TODO: handle diff3 ||||||| markers (merge.conflictstyle = diff3)
        } else if text.starts_with("=======") {
            side = Side::Theirs;
            (LineChangeType::ConflictSeparator, text.to_owned())
        } else if text.starts_with(">>>>>>>") {
            side = Side::Context;
            (LineChangeType::ConflictSeparator, text.to_owned())
        } else {
            match side {
                Side::Ours    => (LineChangeType::ConflictOurs,   text.to_owned()),
                Side::Theirs  => (LineChangeType::ConflictTheirs, text.to_owned()),
                Side::Context => (LineChangeType::Unchanged,       text.to_owned()),
            }
        };
        lines.push(DiffLine::new(Some(ln), Some(ln), content, change_type, ' '));
    }
    UnifiedDiff {
        lines,
        old_file_path: None,
        new_file_path: None,
        is_binary: false,
        lines_added: 0,
        lines_removed: 0,
    }
}
