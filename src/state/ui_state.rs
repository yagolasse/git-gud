use std::path::PathBuf;

/// Pending actions that need to be executed after UI rendering
#[derive(Debug)]
pub enum PendingAction {
    StageAll(Vec<PathBuf>),
    UnstageAll(Vec<PathBuf>),
    StageSelected(Vec<PathBuf>),
    UnstageSelected(Vec<PathBuf>),
    CheckoutBranch(String),
    CreateCommit(String),
    Pull,
    Push,
}

/// UI-specific state
#[derive(Default)]
pub struct UIState {
    /// Selected branch name
    pub selected_branch: Option<String>,

    /// Selected file path (for diff view)
    pub selected_file: Option<PathBuf>,

    /// Commit message summary (first line)
    pub commit_summary: String,

    /// Commit message description (body)
    pub commit_description: String,

    /// UI panel sizes (for persistence)
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub middle_top_height: f32,
    pub middle_bottom_height: f32,

    /// Pending action to execute after UI rendering
    pub pending_action: Option<PendingAction>,

    /// Whether files have been staged/unstaged since last diff refresh
    pub files_staged_or_unstaged: bool,

    /// Create-branch dialog visibility and input
    pub show_create_branch_dialog: bool,
    pub new_branch_name: String,
    pub new_branch_checkout: bool,

    /// Stash-save dialog visibility and input
    pub show_stash_save_dialog: bool,
    pub stash_message: String,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            selected_branch: None,
            selected_file: None,
            commit_summary: String::new(),
            commit_description: String::new(),
            left_panel_width: 250.0,
            right_panel_width: 400.0,
            middle_top_height: 300.0,
            middle_bottom_height: 300.0,
            pending_action: None,
            files_staged_or_unstaged: false,
            show_create_branch_dialog: false,
            new_branch_name: String::new(),
            new_branch_checkout: true,
            show_stash_save_dialog: false,
            stash_message: String::new(),
        }
    }

    pub fn commit_message(&self) -> String {
        if self.commit_description.is_empty() {
            self.commit_summary.clone()
        } else {
            format!("{}\n\n{}", self.commit_summary, self.commit_description)
        }
    }

    pub fn set_commit_message(&mut self, message: &str) {
        let lines: Vec<&str> = message.lines().collect();
        if lines.is_empty() {
            self.commit_summary.clear();
            self.commit_description.clear();
        } else {
            self.commit_summary = lines[0].to_string();
            if lines.len() > 1 {
                let description_start = lines
                    .iter()
                    .skip(1)
                    .position(|line| !line.trim().is_empty())
                    .map(|pos| pos + 1)
                    .unwrap_or(1);
                self.commit_description = lines[description_start..].join("\n");
            } else {
                self.commit_description.clear();
            }
        }
    }

    pub fn clear_commit_message(&mut self) {
        self.commit_summary.clear();
        self.commit_description.clear();
    }

    pub fn is_commit_message_valid(&self) -> bool {
        !self.commit_summary.trim().is_empty()
    }

    pub fn select_file(&mut self, path: PathBuf) {
        self.selected_file = Some(path);
    }

    pub fn clear_file_selection(&mut self) {
        self.selected_file = None;
    }

    pub fn has_file_selection(&self) -> bool {
        self.selected_file.is_some()
    }

    pub fn selected_file_path(&self) -> Option<&PathBuf> {
        self.selected_file.as_ref()
    }

    pub fn select_branch(&mut self, branch_name: String) {
        self.selected_branch = Some(branch_name);
    }

    pub fn clear_branch_selection(&mut self) {
        self.selected_branch = None;
    }

    pub fn has_branch_selection(&self) -> bool {
        self.selected_branch.is_some()
    }

    pub fn selected_branch_name(&self) -> Option<&str> {
        self.selected_branch.as_deref()
    }

    pub fn mark_files_staged_or_unstaged(&mut self) {
        self.files_staged_or_unstaged = true;
    }

    pub fn check_and_reset_staged_unstaged(&mut self) -> bool {
        let result = self.files_staged_or_unstaged;
        self.files_staged_or_unstaged = false;
        result
    }

    pub fn reset(&mut self) {
        self.selected_branch = None;
        self.selected_file = None;
        self.clear_commit_message();
        self.pending_action = None;
        self.files_staged_or_unstaged = false;
    }
}
