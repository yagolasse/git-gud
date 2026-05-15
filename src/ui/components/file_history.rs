use crate::models::diff::LineChangeType;
use crate::models::Commit;
use crate::services::{DiffParser, GitService};
use crate::state::AppState;
use crate::ui::colors::Palette;
use eframe::egui;
use std::path::PathBuf;

pub struct FileHistoryPanel {
    commits: Vec<Commit>,
    last_path: Option<PathBuf>,
    selected_commit_id: Option<String>,
    pending_commit: Option<String>,
    diff_parser: DiffParser,
    unified_diff: Option<crate::models::diff::UnifiedDiff>,
}

impl FileHistoryPanel {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            last_path: None,
            selected_commit_id: None,
            pending_commit: None,
            diff_parser: DiffParser::new(),
            unified_diff: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !state.ui_state.show_file_history {
            return;
        }
        let Some(file_path) = state.ui_state.file_history_path.clone() else {
            return;
        };
        let Some(repo_state) = &state.repository_state else {
            return;
        };

        // Load commit list when the file changes
        if self.last_path.as_ref() != Some(&file_path) {
            self.commits.clear();
            self.selected_commit_id = None;
            self.pending_commit = None;
            self.unified_diff = None;
            match GitService::get_file_history(&repo_state.repository, &file_path, 300) {
                Ok(commits) => self.commits = commits,
                Err(e) => log::error!("get_file_history failed: {}", e),
            }
            self.last_path = Some(file_path.clone());
        }

        // Load diff when a commit was selected last frame
        if let Some(commit_id) = self.pending_commit.take() {
            match GitService::get_file_diff_at_commit(
                &repo_state.repository,
                &file_path,
                &commit_id,
            ) {
                Ok(text) => {
                    let mut ud = self.diff_parser.parse_unified(&text);
                    self.diff_parser.apply_word_diffs(&mut ud.lines);
                    self.unified_diff = Some(ud);
                }
                Err(e) => {
                    log::error!("get_file_diff_at_commit failed: {}", e);
                    self.unified_diff = None;
                }
            }
            self.selected_commit_id = Some(commit_id);
        }

        let p = crate::ui::colors::get(state.dark_mode);
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.display().to_string());

        let mut open = true;
        let mut newly_selected: Option<String> = None;

        egui::Window::new(format!("History — {}", file_name))
            .id(egui::Id::new("file_history_window"))
            .default_size([800.0, 500.0])
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                newly_selected = self.render(ui, p);
            });

        if let Some(id) = newly_selected {
            self.pending_commit = Some(id);
        }
        if !open {
            state.ui_state.show_file_history = false;
        }
    }

    fn render(&self, ui: &mut egui::Ui, p: &'static Palette) -> Option<String> {
        let mut newly_selected = None;

        ui.columns(2, |cols| {
            egui::ScrollArea::vertical()
                .id_salt("file_history_commits")
                .auto_shrink([false, false])
                .show(&mut cols[0], |ui| {
                    if self.commits.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("No commits found for this file")
                                .color(p.text_tertiary),
                        );
                        return;
                    }
                    for commit in &self.commits {
                        let is_sel = self.selected_commit_id.as_deref() == Some(&commit.id);
                        let short_id = &commit.id[..7.min(commit.id.len())];
                        let summary: String =
                            commit.message.lines().next().unwrap_or("").chars().take(55).collect();
                        let resp =
                            ui.selectable_label(is_sel, format!("{} {}", short_id, summary));
                        ui.label(
                            egui::RichText::new(format!(
                                "  {} · {}",
                                commit.author,
                                age(commit.timestamp)
                            ))
                            .color(p.text_tertiary)
                            .size(10.0),
                        );
                        if resp.clicked() && !is_sel {
                            newly_selected = Some(commit.id.clone());
                        }
                    }
                });

            egui::ScrollArea::vertical()
                .id_salt("file_history_diff")
                .auto_shrink([false, false])
                .show(&mut cols[1], |ui| {
                    match &self.unified_diff {
                        None if self.selected_commit_id.is_none() => {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Select a commit to see the diff")
                                    .color(p.text_tertiary),
                            );
                        }
                        None => {
                            ui.label(
                                egui::RichText::new("No diff available").color(p.text_tertiary),
                            );
                        }
                        Some(ud) if ud.lines.is_empty() => {
                            ui.label(
                                egui::RichText::new("No changes at this commit")
                                    .color(p.text_tertiary),
                            );
                        }
                        Some(ud) => {
                            for line in &ud.lines {
                                let (color, pfx) = match line.change_type {
                                    LineChangeType::Added => (p.diff_add_text, "+"),
                                    LineChangeType::Removed => (p.diff_rem_text, "-"),
                                    LineChangeType::HunkHeader | LineChangeType::FileHeader => {
                                        (p.diff_hunk_text, "")
                                    }
                                    _ => (p.diff_context_text, " "),
                                };
                                ui.label(
                                    egui::RichText::new(format!("{}{}", pfx, line.content))
                                        .color(color)
                                        .monospace()
                                        .size(11.0),
                                );
                            }
                        }
                    }
                });
        });

        newly_selected
    }
}

impl Default for FileHistoryPanel {
    fn default() -> Self {
        Self::new()
    }
}

fn age(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs = now - ts;
    if secs < 60 {
        "just now".into()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 86400 * 30 {
        format!("{}d ago", secs / 86400)
    } else if secs < 86400 * 365 {
        format!("{}mo ago", secs / (86400 * 30))
    } else {
        format!("{}y ago", secs / (86400 * 365))
    }
}
