//! Diff viewer component for Git Gud
//!
//! This component displays diffs for selected files.

use crate::state::AppState;
use eframe::egui;

/// Diff viewer UI component
pub struct DiffViewer {
    /// Current diff text
    diff_text: String,
    
    /// Whether to show line numbers
    show_line_numbers: bool,
    
    /// Whether to wrap lines
    wrap_lines: bool,
}

impl DiffViewer {
    /// Create a new diff viewer component
    pub fn new() -> Self {
        Self {
            diff_text: String::new(),
            show_line_numbers: true,
            wrap_lines: false,
        }
    }
    
    /// Show the diff viewer component
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("Diff View");
        
        // Options toolbar
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_line_numbers, "Line numbers");
            ui.checkbox(&mut self.wrap_lines, "Wrap lines");
            
            if ui.button("Copy").clicked() && !self.diff_text.is_empty() {
                ui.output_mut(|o| o.copied_text = self.diff_text.clone());
                state.set_info("Diff copied to clipboard".to_string());
            }
            
            if ui.button("Refresh").clicked() {
                self.refresh_diff(state);
            }
        });
        
        ui.separator();
        
        // Simple version for now
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
        ui.label("Diff viewer will be implemented in Slice 3");
    }
    
    /// Refresh the diff for the currently selected file
    fn refresh_diff(&mut self, state: &mut AppState) {
        if !state.has_repository() || !state.ui_state.has_file_selection() {
            self.diff_text.clear();
            return;
        }
        
        let selected_file = state.ui_state.selected_file_path().unwrap();
        let _repo_state = state.repository_state();
        
        // TODO: Load actual diff from Git service
        // For now, show a placeholder
        self.diff_text = format!(
            "Diff for: {}\n\n\
            --- a/{}\n\
            +++ b/{}\n\
            @@ -1,3 +1,4 @@\n\
            // Example diff\n\
            -removed line\n\
            +added line\n\
             unchanged line\n\
            +another added line",
            selected_file.display(),
            selected_file.display(),
            selected_file.display()
        );
        
        log::debug!("Refreshed diff for file: {:?}", selected_file);
    }
    
    /// Display diff text with syntax highlighting
    fn show_diff_text(&mut self, ui: &mut egui::Ui) {
        let mut job = egui::text::LayoutJob::default();
        
        for (line_num, line) in self.diff_text.lines().enumerate() {
            if self.show_line_numbers {
                let line_num_text = format!("{:4}: ", line_num + 1);
                job.append(
                    &line_num_text,
                    0.0,
                    egui::TextFormat::simple(
                        egui::FontId::monospace(12.0),
                        egui::Color32::GRAY,
                    ),
                );
            }
            
            // Color code diff lines
            let (color, text) = if line.starts_with('+') {
                (egui::Color32::DARK_GREEN, line)
            } else if line.starts_with('-') {
                (egui::Color32::DARK_RED, line)
            } else if line.starts_with('@') {
                (egui::Color32::BLUE, line)
            } else {
                (egui::Color32::WHITE, line)
            };
            
            job.append(
                text,
                0.0,
                egui::TextFormat::simple(
                    egui::FontId::monospace(12.0),
                    color,
                ),
            );
            
            job.append("\n", 0.0, egui::TextFormat::default());
        }
        
        ui.add(
            egui::TextEdit::multiline(&mut self.diff_text)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .frame(false)
                .layouter(&mut |ui, text, wrap_width| {
                    let mut layout_job = job.clone();
                    layout_job.wrap.max_width = wrap_width;
                    if !self.wrap_lines {
                        layout_job.wrap.max_width = f32::INFINITY;
                    }
                    ui.fonts(|f| f.layout_job(layout_job))
                })
        );
    }
    
    /// Clear the current diff
    pub fn clear(&mut self) {
        self.diff_text.clear();
    }
}

impl Default for DiffViewer {
    fn default() -> Self {
        Self::new()
    }
}