//! UI state for Git Gud
//!
//! This struct holds UI-specific state like selections,
//! input field values, and UI preferences.

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
    
    /// Whether to show only staged files
    pub show_only_staged: bool,
    
    /// Whether to show only unstaged files
    pub show_only_unstaged: bool,
    
    /// File filter string
    pub file_filter: String,
    
    /// Branch filter string
    pub branch_filter: String,
    
    /// Selected remote for fetch/pull/push
    pub selected_remote: Option<String>,
    
    /// UI panel sizes (for persistence)
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub middle_top_height: f32,
    pub middle_bottom_height: f32,
    
    /// Whether the application is in dark mode
    pub dark_mode: bool,
    
    /// Font size scaling
    pub font_scale: f32,
    
    /// Pending action to execute after UI rendering
    pub pending_action: Option<PendingAction>,
}

impl UIState {
    /// Create a new UI state with default values
    pub fn new() -> Self {
        Self {
            selected_branch: None,
            selected_file: None,
            commit_summary: String::new(),
            commit_description: String::new(),
            show_only_staged: false,
            show_only_unstaged: false,
            file_filter: String::new(),
            branch_filter: String::new(),
            selected_remote: None,
            left_panel_width: 250.0,
            right_panel_width: 400.0,
            middle_top_height: 300.0,
            middle_bottom_height: 300.0,
            dark_mode: true,
            font_scale: 1.0,
            pending_action: None,
        }
    }
    
    /// Get the full commit message (summary + description)
    pub fn commit_message(&self) -> String {
        if self.commit_description.is_empty() {
            self.commit_summary.clone()
        } else {
            format!("{}\n\n{}", self.commit_summary, self.commit_description)
        }
    }
    
    /// Set the commit message (parses summary and description)
    pub fn set_commit_message(&mut self, message: &str) {
        let lines: Vec<&str> = message.lines().collect();
        
        if lines.is_empty() {
            self.commit_summary.clear();
            self.commit_description.clear();
        } else {
            self.commit_summary = lines[0].to_string();
            
            if lines.len() > 1 {
                // Skip empty lines between summary and description
                let description_start = lines.iter()
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
    
    /// Clear the commit message
    pub fn clear_commit_message(&mut self) {
        self.commit_summary.clear();
        self.commit_description.clear();
    }
    
    /// Check if commit message is valid (non-empty summary)
    pub fn is_commit_message_valid(&self) -> bool {
        !self.commit_summary.trim().is_empty()
    }
    
    /// Select a file for diff viewing
    pub fn select_file(&mut self, path: PathBuf) {
        self.selected_file = Some(path);
    }
    
    /// Clear file selection
    pub fn clear_file_selection(&mut self) {
        self.selected_file = None;
    }
    
    /// Check if a file is selected
    pub fn has_file_selection(&self) -> bool {
        self.selected_file.is_some()
    }
    
    /// Get the selected file path
    pub fn selected_file_path(&self) -> Option<&PathBuf> {
        self.selected_file.as_ref()
    }
    
    /// Select a branch
    pub fn select_branch(&mut self, branch_name: String) {
        self.selected_branch = Some(branch_name);
    }
    
    /// Clear branch selection
    pub fn clear_branch_selection(&mut self) {
        self.selected_branch = None;
    }
    
    /// Check if a branch is selected
    pub fn has_branch_selection(&self) -> bool {
        self.selected_branch.is_some()
    }
    
    /// Get the selected branch name
    pub fn selected_branch_name(&self) -> Option<&str> {
        self.selected_branch.as_deref()
    }
    
    /// Apply file filter to a list of file paths
    pub fn filter_files<'a>(&self, files: &'a [PathBuf]) -> Vec<&'a PathBuf> {
        if self.file_filter.is_empty() {
            return files.iter().collect();
        }
        
        let filter_lower = self.file_filter.to_lowercase();
        files.iter()
            .filter(|path| {
                path.to_string_lossy().to_lowercase().contains(&filter_lower)
            })
            .collect()
    }
    
    /// Apply branch filter to a list of branch names
    pub fn filter_branches<'a>(&self, branches: &'a [String]) -> Vec<&'a String> {
        if self.branch_filter.is_empty() {
            return branches.iter().collect();
        }
        
        let filter_lower = self.branch_filter.to_lowercase();
        branches.iter()
            .filter(|name| name.to_lowercase().contains(&filter_lower))
            .collect()
    }
    
    /// Reset UI state to defaults (except for panel sizes)
    pub fn reset(&mut self) {
        self.selected_branch = None;
        self.selected_file = None;
        self.clear_commit_message();
        self.show_only_staged = false;
        self.show_only_unstaged = false;
        self.file_filter.clear();
        self.branch_filter.clear();
        self.selected_remote = None;
        self.pending_action = None;
    }
}