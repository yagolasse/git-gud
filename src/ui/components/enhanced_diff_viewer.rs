//! Enhanced diff viewer component for Git Gud
//!
//! This component provides advanced diff viewing capabilities including
//! side-by-side diff view and syntax highlighting.

use crate::models::diff::{DiffConfig, DiffDisplayMode, SideBySideDiff, UnifiedDiff};
use crate::services::{DiffParser, SyntaxService};
use crate::state::AppState;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

/// Enhanced diff viewer UI component
pub struct EnhancedDiffViewer {
    /// Current diff configuration
    config: DiffConfig,

    /// Current unified diff
    unified_diff: Option<UnifiedDiff>,

    /// Current side-by-side diff
    side_by_side_diff: Option<SideBySideDiff>,

    /// Diff parser service
    diff_parser: DiffParser,

    /// Syntax highlighting service
    syntax_service: Arc<SyntaxService>,

    /// Last selected file path (to detect changes)
    last_selected_file: Option<PathBuf>,

    /// Scroll position for unified view
    unified_scroll_offset: f32,

    /// Scroll position for left side of side-by-side view
    side_by_side_left_scroll: f32,

    /// Scroll position for right side of side-by-side view
    side_by_side_right_scroll: f32,

    /// Whether scroll positions are synchronized
    scroll_synced: bool,
}

impl EnhancedDiffViewer {
    /// Create a new enhanced diff viewer component
    pub fn new() -> Self {
        Self {
            config: DiffConfig::default(),
            unified_diff: None,
            side_by_side_diff: None,
            diff_parser: DiffParser::new(),
            syntax_service: Arc::new(SyntaxService::new()),
            last_selected_file: None,
            unified_scroll_offset: 0.0,
            side_by_side_left_scroll: 0.0,
            side_by_side_right_scroll: 0.0,
            scroll_synced: true,
        }
    }

    /// Create a new enhanced diff viewer with custom syntax service
    pub fn with_syntax_service(syntax_service: Arc<SyntaxService>) -> Self {
        Self {
            config: DiffConfig::default(),
            unified_diff: None,
            side_by_side_diff: None,
            diff_parser: DiffParser::new(),
            syntax_service,
            last_selected_file: None,
            unified_scroll_offset: 0.0,
            side_by_side_left_scroll: 0.0,
            side_by_side_right_scroll: 0.0,
            scroll_synced: true,
        }
    }

    /// Show the enhanced diff viewer component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Enhanced Diff View");

        // Check if we need to refresh the diff
        let current_selected_file = state.ui_state.selected_file_path().cloned();
        let needs_refresh = current_selected_file != self.last_selected_file;

        // Also check if files have been staged/unstaged
        let files_changed = state.ui_state.check_and_reset_staged_unstaged();

        if needs_refresh || files_changed {
            self.refresh_diff(state);
            self.last_selected_file = current_selected_file;
        }

        // Options toolbar
        self.show_toolbar(ui, state);

        ui.separator();

        // Show appropriate view based on configuration
        if !state.has_repository() {
            ui.label("No repository loaded");
            return;
        }

        if !state.ui_state.has_file_selection() {
            ui.label("Select a file to view diff");
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap();
        ui.label(format!("Selected: {}", selected_file.display()));

        // Show diff based on current mode
        match self.config.mode {
            DiffDisplayMode::Unified => self.show_unified_view(ui, selected_file),
            DiffDisplayMode::SideBySide => self.show_side_by_side_view(ui, selected_file),
            DiffDisplayMode::WordLevel => {
                // Word level not implemented yet, fall back to side-by-side
                ui.label("Word-level diff view not yet implemented");
                self.show_side_by_side_view(ui, selected_file);
            }
        }
    }

    /// Show the toolbar with configuration options
    fn show_toolbar(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.horizontal(|ui| {
            // View mode selection
            ui.label("View:");
            ui.radio_value(&mut self.config.mode, DiffDisplayMode::Unified, "Unified");
            ui.radio_value(
                &mut self.config.mode,
                DiffDisplayMode::SideBySide,
                "Side-by-side",
            );

            ui.separator();

            // Display options
            ui.checkbox(&mut self.config.show_line_numbers, "Line numbers");
            ui.checkbox(&mut self.config.wrap_lines, "Wrap lines");
            ui.checkbox(&mut self.config.syntax_highlighting, "Syntax");

            if self.config.syntax_highlighting {
                // Theme selection dropdown
                ui.label("Theme:");
                let themes = self.syntax_service.available_themes();
                egui::ComboBox::from_id_source("theme_select")
                    .selected_text(&self.config.theme)
                    .show_ui(ui, |ui| {
                        for theme in themes {
                            if ui
                                .selectable_value(&mut self.config.theme, theme.clone(), &theme)
                                .changed()
                            {
                                self.syntax_service.set_theme(&theme);
                                self.config.theme = theme;
                            }
                        }
                    });
            }

            ui.separator();

            // Action buttons
            if ui.button("Copy").clicked() && let Some(ref unified) = self.unified_diff {
                let diff_text = unified
                    .lines
                    .iter()
                    .map(|line| format!("{}{}", line.prefix, line.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                ui.output_mut(|o| o.copied_text = diff_text);
                state.set_info("Diff copied to clipboard".to_string());
            }

            if ui.button("Refresh").clicked() {
                self.refresh_diff(state);
            }

            // Scroll sync toggle for side-by-side view
            if self.config.mode == DiffDisplayMode::SideBySide {
                ui.checkbox(&mut self.scroll_synced, "Sync scroll");
            }
        });
    }

    /// Show unified diff view
    fn show_unified_view(&mut self, ui: &mut egui::Ui, file_path: &std::path::Path) {
        if let Some(ref unified_diff) = self.unified_diff {
            if unified_diff.is_binary {
                ui.label("Binary files differ");
                return;
            }

            if unified_diff.lines.is_empty() {
                ui.label("No changes");
                return;
            }

            // Show diff statistics
            ui.horizontal(|ui| {
                ui.label(format!(
                    "+{} -{}",
                    unified_diff.lines_added, unified_diff.lines_removed
                ));
            });

            // Create scroll area for unified view
            egui::ScrollArea::vertical()
                .id_source("unified_diff_scroll")
                .scroll_offset(egui::Vec2::new(0.0, self.unified_scroll_offset))
                .show(ui, |ui| {
                    let mut job = egui::text::LayoutJob::default();

                    // Build highlighted layout job
                    let syntax = if self.config.syntax_highlighting {
                        self.syntax_service.detect_syntax(file_path)
                    } else {
                        None
                    };

                    for line in &unified_diff.lines {
                        let line_job = if self.config.syntax_highlighting {
                            self.syntax_service.highlight_diff_line(line, syntax)
                        } else {
                            // Basic highlighting without syntax
                            let color = match line.change_type {
                                crate::models::diff::LineChangeType::Added => {
                                    egui::Color32::DARK_GREEN
                                }
                                crate::models::diff::LineChangeType::Removed => {
                                    egui::Color32::DARK_RED
                                }
                                crate::models::diff::LineChangeType::HunkHeader => {
                                    egui::Color32::BLUE
                                }
                                crate::models::diff::LineChangeType::FileHeader => {
                                    egui::Color32::GRAY
                                }
                                _ => egui::Color32::WHITE,
                            };

                            let mut job = egui::text::LayoutJob::default();
                            if line.prefix != ' ' {
                                job.append(
                                    &line.prefix.to_string(),
                                    0.0,
                                    egui::TextFormat::simple(egui::FontId::monospace(12.0), color),
                                );
                            }
                            job.append(
                                &line.content,
                                0.0,
                                egui::TextFormat::simple(egui::FontId::monospace(12.0), color),
                            );
                            job
                        };

                        // Add line number if enabled
                        if self.config.show_line_numbers {
                            let line_num = line
                                .left_line_num
                                .or(line.right_line_num)
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "".to_string());

                            let line_num_text = format!("{:>4}: ", line_num);
                            job.append(
                                &line_num_text,
                                0.0,
                                egui::TextFormat::simple(
                                    egui::FontId::monospace(12.0),
                                    egui::Color32::GRAY,
                                ),
                            );
                        }

                        // Add the line content
                        for section in line_job.sections {
                            // Extract text from the line_job's text
                            let byte_range = section.byte_range.clone();
                            if byte_range.end <= line_job.text.len() {
                                let text = &line_job.text[byte_range];
                                job.append(text, section.leading_space, section.format.clone());
                            } else {
                                // Fallback: use the full line content
                                job.append(
                                    &line.content,
                                    section.leading_space,
                                    section.format.clone(),
                                );
                            }
                        }

                        // Add newline
                        job.append("\n", 0.0, egui::TextFormat::default());
                    }

                    // Display the job
                    ui.add(
                        egui::TextEdit::multiline(&mut String::new())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .frame(false)
                            .layouter(&mut |ui, _text, wrap_width| {
                                let mut layout_job = job.clone();
                                layout_job.wrap.max_width = wrap_width;
                                if !self.config.wrap_lines {
                                    layout_job.wrap.max_width = f32::INFINITY;
                                }
                                ui.fonts(|f| f.layout_job(layout_job))
                            }),
                    );
                });
        } else {
            ui.label("No diff loaded");
        }
    }

    /// Show side-by-side diff view
    fn show_side_by_side_view(&mut self, ui: &mut egui::Ui, file_path: &std::path::Path) {
        if let Some(ref side_by_side_diff) = self.side_by_side_diff {
            if side_by_side_diff.is_binary {
                ui.label("Binary files differ");
                return;
            }

            if side_by_side_diff.is_empty() {
                ui.label("No changes");
                return;
            }

            // Show diff statistics
            ui.horizontal(|ui| {
                ui.label(format!(
                    "+{} -{}",
                    side_by_side_diff.lines_added, side_by_side_diff.lines_removed
                ));
            });

            // Create columns for side-by-side view
            let mut left_scroll = self.side_by_side_left_scroll;
            let mut right_scroll = self.side_by_side_right_scroll;
            let scroll_synced = self.scroll_synced;

            ui.columns(2, |columns| {
                // Left column (old file)
                Self::show_diff_column(
                    &self.config,
                    &self.syntax_service,
                    &mut columns[0],
                    "Left (Old)",
                    &side_by_side_diff.left_lines,
                    file_path,
                    &mut left_scroll,
                    "left_diff_scroll",
                );

                // Right column (new file)
                Self::show_diff_column(
                    &self.config,
                    &self.syntax_service,
                    &mut columns[1],
                    "Right (New)",
                    &side_by_side_diff.right_lines,
                    file_path,
                    &mut right_scroll,
                    "right_diff_scroll",
                );
            });

            // Sync scroll positions if enabled
            if scroll_synced {
                let avg_scroll = (left_scroll + right_scroll) / 2.0;
                left_scroll = avg_scroll;
                right_scroll = avg_scroll;
            }

            // Update scroll positions
            self.side_by_side_left_scroll = left_scroll;
            self.side_by_side_right_scroll = right_scroll;
        } else {
            ui.label("No diff loaded");
        }
    }

    /// Show a single column in side-by-side view
    #[allow(clippy::too_many_arguments)]
    fn show_diff_column(
        config: &DiffConfig,
        syntax_service: &Arc<SyntaxService>,
        ui: &mut egui::Ui,
        title: &str,
        lines: &[crate::models::diff::DiffLine],
        file_path: &std::path::Path,
        scroll_offset: &mut f32,
        scroll_id: &str,
    ) {
        ui.heading(title);

        egui::ScrollArea::vertical()
            .id_source(scroll_id)
            .scroll_offset(egui::Vec2::new(0.0, *scroll_offset))
            .show(ui, |ui| {
                let mut job = egui::text::LayoutJob::default();

                // Build highlighted layout job
                let syntax = if config.syntax_highlighting {
                    syntax_service.detect_syntax(file_path)
                } else {
                    None
                };

                for line in lines {
                    // Skip empty placeholder lines
                    if line.content.is_empty() && !line.is_content() {
                        job.append("\n", 0.0, egui::TextFormat::default());
                        continue;
                    }

                    // Add line number if enabled
                    if config.show_line_numbers {
                        let line_num = line
                            .left_line_num
                            .or(line.right_line_num)
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "".to_string());

                        let line_num_text = format!("{:>4}: ", line_num);
                        job.append(
                            &line_num_text,
                            0.0,
                            egui::TextFormat::simple(
                                egui::FontId::monospace(12.0),
                                egui::Color32::GRAY,
                            ),
                        );
                    }

                    // Add the line content
                    let line_job = if config.syntax_highlighting && line.should_highlight {
                        syntax_service.highlight_diff_line(line, syntax)
                    } else {
                        // Basic highlighting without syntax
                        let color = match line.change_type {
                            crate::models::diff::LineChangeType::Added => egui::Color32::DARK_GREEN,
                            crate::models::diff::LineChangeType::Removed => egui::Color32::DARK_RED,
                            crate::models::diff::LineChangeType::HunkHeader => egui::Color32::BLUE,
                            crate::models::diff::LineChangeType::FileHeader => egui::Color32::GRAY,
                            _ => egui::Color32::WHITE,
                        };

                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            &line.content,
                            0.0,
                            egui::TextFormat::simple(egui::FontId::monospace(12.0), color),
                        );
                        job
                    };

                    for section in line_job.sections {
                        // Extract text from the line_job's text
                        let byte_range = section.byte_range.clone();
                        if byte_range.end <= line_job.text.len() {
                            let text = &line_job.text[byte_range];
                            job.append(text, section.leading_space, section.format.clone());
                        } else {
                            // Fallback: use the full line content
                            job.append(
                                &line.content,
                                section.leading_space,
                                section.format.clone(),
                            );
                        }
                    }

                    // Add newline
                    job.append("\n", 0.0, egui::TextFormat::default());
                }

                // Display the job
                ui.add(
                    egui::TextEdit::multiline(&mut String::new())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .frame(false)
                        .layouter(&mut |ui, _text, wrap_width| {
                            let mut layout_job = job.clone();
                            layout_job.wrap.max_width = wrap_width;
                            if !config.wrap_lines {
                                layout_job.wrap.max_width = f32::INFINITY;
                            }
                            ui.fonts(|f| f.layout_job(layout_job))
                        }),
                );
            });
    }

    /// Refresh the diff for the currently selected file
    fn refresh_diff(&mut self, state: &mut AppState) {
        if !state.has_repository() || !state.ui_state.has_file_selection() {
            self.unified_diff = None;
            self.side_by_side_diff = None;
            return;
        }

        let selected_file = state.ui_state.selected_file_path().unwrap();

        // Get actual diff from Git service
        if let Some(repo_state) = &state.repository_state {
            match crate::services::GitService::get_file_diff(&repo_state.repository, selected_file)
            {
                Ok(diff_text) => {
                    // Parse unified diff
                    let unified = self.diff_parser.parse_unified(&diff_text);
                    self.unified_diff = Some(unified.clone());

                    // Convert to side-by-side
                    let side_by_side = self.diff_parser.unified_to_side_by_side(&unified);
                    self.side_by_side_diff = Some(side_by_side);

                    log::debug!("Loaded diff for file: {:?}", selected_file);
                }
                Err(e) => {
                    self.unified_diff = None;
                    self.side_by_side_diff = None;
                    log::error!("Failed to load diff for file {:?}: {}", selected_file, e);
                    state.set_error(format!("Failed to load diff: {}", e));
                }
            }
        } else {
            self.unified_diff = None;
            self.side_by_side_diff = None;
        }
    }

    /// Clear the current diff
    pub fn clear(&mut self) {
        self.unified_diff = None;
        self.side_by_side_diff = None;
        self.last_selected_file = None;
    }

    /// Force refresh of the diff
    pub fn force_refresh(&mut self) {
        self.last_selected_file = None;
    }

    /// Get the current configuration
    pub fn config(&self) -> &DiffConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut DiffConfig {
        &mut self.config
    }

    /// Get the syntax service
    pub fn syntax_service(&self) -> &Arc<SyntaxService> {
        &self.syntax_service
    }
}

impl Default for EnhancedDiffViewer {
    fn default() -> Self {
        Self::new()
    }
}
