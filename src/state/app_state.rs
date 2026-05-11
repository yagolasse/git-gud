//! Main application state for Git Gud
//!
//! This struct holds the global application state that is shared
//! between all UI components and services.

use crate::services;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

use super::{RepositoryState, UIState};

/// Severity of a command log entry
#[derive(Clone)]
pub enum LogLevel {
    Info,
    Error,
}

/// A single entry in the session command log
#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

/// Main application state
pub struct AppState {
    /// Repository-specific state (None if no repository is loaded)
    pub repository_state: Option<RepositoryState>,

    /// UI state (selections, input fields, etc.)
    pub ui_state: UIState,

    /// Application configuration
    pub config: AppConfig,

    /// Error message to display to user (if any)
    pub error_message: Option<String>,

    /// Information message to display to user (if any)
    pub info_message: Option<String>,

    /// Session-scoped log of git operations
    pub command_log: Vec<LogEntry>,

    /// Whether the UI is in dark mode
    pub dark_mode: bool,
}

/// Application configuration
pub struct AppConfig {
    /// Default repository path
    pub default_repository_path: PathBuf,

    /// Whether to show hidden files
    pub show_hidden_files: bool,

    /// Diff view style (unified, side-by-side, etc.)
    pub diff_style: DiffStyle,

    /// Whether to auto-refresh repository status
    pub auto_refresh: bool,

    /// Refresh interval in seconds
    pub refresh_interval: u64,
}

/// Diff view style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStyle {
    Unified,
    SideBySide,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_repository_path: PathBuf::from("."),
            show_hidden_files: false,
            diff_style: DiffStyle::Unified,
            auto_refresh: true,
            refresh_interval: 5,
        }
    }
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self {
            repository_state: None,
            ui_state: UIState::default(),
            config: AppConfig::default(),
            error_message: None,
            info_message: None,
            command_log: Vec::new(),
            dark_mode: false,
        }
    }

    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }

    /// Check if a repository is loaded
    pub fn has_repository(&self) -> bool {
        self.repository_state.is_some()
    }

    /// Get the repository state (panics if no repository is loaded)
    pub fn repository_state(&self) -> &RepositoryState {
        self.repository_state
            .as_ref()
            .expect("No repository loaded")
    }

    /// Get mutable repository state (panics if no repository is loaded)
    pub fn repository_state_mut(&mut self) -> &mut RepositoryState {
        self.repository_state
            .as_mut()
            .expect("No repository loaded")
    }

    /// Set an error message to display to the user
    pub fn set_error(&mut self, message: String) {
        self.command_log.push(LogEntry {
            timestamp: now_str(),
            level: LogLevel::Error,
            message: message.clone(),
        });
        log::error!("User error: {}", message);
        self.error_message = Some(message);
    }

    /// Clear the current error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set an info message to display to the user
    pub fn set_info(&mut self, message: String) {
        self.command_log.push(LogEntry {
            timestamp: now_str(),
            level: LogLevel::Info,
            message: message.clone(),
        });
        log::info!("User info: {}", message);
        self.info_message = Some(message);
    }

    /// Clear the current info message
    pub fn clear_info(&mut self) {
        self.info_message = None;
    }

    /// Clear the session command log
    pub fn clear_command_log(&mut self) {
        self.command_log.clear();
    }

    /// Load a repository into the application state
    pub fn load_repository(&mut self, path: PathBuf) -> anyhow::Result<()> {
        log::info!("Loading repository from: {:?}", path);

        // Try to open the repository
        let repo = match services::GitService::open_repository(&path) {
            Ok(repo) => repo,
            Err(e) => {
                let error_msg = format!("Failed to open repository: {}", e);
                self.set_error(error_msg.clone());
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        // Create repository state
        let repository_state = RepositoryState::new(repo, path.clone())?;

        // Update application state
        self.repository_state = Some(repository_state);
        self.ui_state.selected_branch = self
            .repository_state()
            .branches
            .iter()
            .find(|b| b.is_current)
            .map(|b| b.name.clone());

        self.set_info(format!("Repository loaded: {:?}", path.clone()));

        Ok(())
    }

    /// Pre-populate the commit message fields from the HEAD commit (used when amend is toggled on)
    pub fn prefill_amend_message(&mut self) {
        if let Some(repo_state) = &self.repository_state {
            if let Some(head) = &repo_state.head_commit {
                self.ui_state.set_commit_message(&head.message);
            }
        }
    }

    /// Refresh repository status (unstaged/staged files, branches, etc.)
    pub fn refresh_repository(&mut self) -> anyhow::Result<()> {
        if let Some(repo_state) = &mut self.repository_state {
            repo_state.refresh()?;
            log::debug!("Repository refreshed");
        }
        self.validate_file_selection();
        Ok(())
    }

    /// Ensure the selected file is still present in the staged/unstaged lists.
    /// If not, auto-select the first available file or clear the selection.
    pub fn validate_file_selection(&mut self) {
        if let Some(repo_state) = &self.repository_state {
            let selected = match &self.ui_state.selected_file {
                Some(p) => p.clone(),
                None => return,
            };
            let still_present = repo_state.staged_files.iter().any(|f| f.path == selected)
                || repo_state.unstaged_files.iter().any(|f| f.path == selected);
            if !still_present {
                self.ui_state.selected_file = repo_state
                    .staged_files
                    .first()
                    .or_else(|| repo_state.unstaged_files.first())
                    .map(|f| f.path.clone());
            }
        }
    }

    /// Handle pending actions from UI
    pub fn handle_pending_actions(&mut self) {
        if let Some(action) = self.ui_state.pending_action.take() {
            match action {
                super::ui_state::PendingAction::StageAll(paths) => {
                    if let Err(e) = self.repository_state_mut().stage_files(&paths) {
                        self.set_error(format!("Failed to stage all files: {}", e));
                    } else {
                        self.set_info(format!("Staged {} files", paths.len()));
                        self.ui_state.mark_files_staged_or_unstaged();
                    }
                }
                super::ui_state::PendingAction::UnstageAll(paths) => {
                    if let Err(e) = self.repository_state_mut().unstage_files(&paths) {
                        self.set_error(format!("Failed to unstage all files: {}", e));
                    } else {
                        self.set_info(format!("Unstaged {} files", paths.len()));
                        self.ui_state.mark_files_staged_or_unstaged();
                    }
                }
                super::ui_state::PendingAction::StageSelected(paths) => {
                    if let Err(e) = self.repository_state_mut().stage_files(&paths) {
                        self.set_error(format!("Failed to stage selected files: {}", e));
                    } else {
                        self.set_info(format!("Staged {} files", paths.len()));
                        self.ui_state.mark_files_staged_or_unstaged();
                    }
                }
                super::ui_state::PendingAction::UnstageSelected(paths) => {
                    if let Err(e) = self.repository_state_mut().unstage_files(&paths) {
                        self.set_error(format!("Failed to unstage selected files: {}", e));
                    } else {
                        self.set_info(format!("Unstaged {} files", paths.len()));
                        self.ui_state.mark_files_staged_or_unstaged();
                    }
                }
                super::ui_state::PendingAction::CheckoutBranch(branch_name) => {
                    if let Err(e) = self.repository_state_mut().checkout_branch(&branch_name) {
                        self.set_error(format!("Failed to checkout branch {}: {}", branch_name, e));
                    } else {
                        self.set_info(format!("Checked out branch: {}", branch_name));
                    }
                }
                super::ui_state::PendingAction::CreateCommit(message) => {
                    if let Err(e) = self.repository_state_mut().create_commit(&message) {
                        self.set_error(format!("Failed to create commit: {}", e));
                    } else {
                        self.set_info("Commit created successfully".to_string());
                        self.ui_state.clear_commit_message();
                    }
                }
                super::ui_state::PendingAction::Pull => {
                    match self.repository_state_mut().pull("origin") {
                        Ok(()) => self.set_info("Pull successful".to_string()),
                        Err(e) => self.set_error(format!("Pull failed: {}", e)),
                    }
                }
                super::ui_state::PendingAction::Push => {
                    let branch = self.repository_state
                        .as_ref()
                        .and_then(|r| r.current_branch().map(|b| b.to_string()))
                        .unwrap_or_else(|| "main".to_string());
                    match self.repository_state_mut().push("origin", &branch) {
                        Ok(()) => self.set_info("Push successful".to_string()),
                        Err(e) => self.set_error(format!("Push failed: {}", e)),
                    }
                }
            }
            self.validate_file_selection();
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for shared application state
pub type SharedAppState = Arc<Mutex<AppState>>;

fn now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{:02}:{:02}:{:02}", (s % 86400) / 3600, (s % 3600) / 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::GitService;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(!state.has_repository());
        assert!(state.repository_state.is_none());
        assert!(state.error_message.is_none());
        assert!(state.info_message.is_none());
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.has_repository());
        assert!(state.error_message.is_none());
        assert!(state.info_message.is_none());
    }

    #[test]
    fn test_has_repository() {
        let state = AppState::new();
        assert!(!state.has_repository());

        // Simulate having a repository (we can't easily create a real one here)
        // This is tested in test_load_repository
    }

    #[test]
    fn test_set_and_clear_error() {
        let mut state = AppState::new();
        assert!(state.error_message.is_none());

        state.set_error("Test error".to_string());
        assert!(state.error_message.is_some());
        assert_eq!(state.error_message.as_ref().unwrap(), "Test error");

        state.clear_error();
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_set_and_clear_info() {
        let mut state = AppState::new();
        assert!(state.info_message.is_none());

        state.set_info("Test info".to_string());
        assert!(state.info_message.is_some());
        assert_eq!(state.info_message.as_ref().unwrap(), "Test info");

        state.clear_info();
        assert!(state.info_message.is_none());
    }

    #[test]
    fn test_load_repository_success() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize a git repository
        GitService::init_repository(repo_path)?;

        // Create a test file to have something in the repo
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        let mut state = AppState::new();
        assert!(!state.has_repository());

        // Load the repository
        state.load_repository(repo_path.to_path_buf())?;

        assert!(state.has_repository());
        assert!(state.repository_state.is_some());
        assert!(state.info_message.is_some()); // Should have info message

        Ok(())
    }

    #[test]
    fn test_load_repository_failure() {
        let mut state = AppState::new();

        // Try to load a non-existent repository
        let result = state.load_repository("/nonexistent/path".into());
        assert!(result.is_err());
        assert!(state.error_message.is_some()); // Should have error message
        assert!(!state.has_repository());
    }

    #[test]
    fn test_refresh_repository_without_repo() -> anyhow::Result<()> {
        let mut state = AppState::new();
        // Should not panic when no repository loaded
        state.refresh_repository()?;
        Ok(())
    }

    #[test]
    fn test_handle_pending_actions_no_action() {
        let mut state = AppState::new();
        // Should not panic when no pending action
        state.handle_pending_actions();
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.default_repository_path, PathBuf::from("."));
        assert!(!config.show_hidden_files);
        assert_eq!(config.diff_style, DiffStyle::Unified);
        assert!(config.auto_refresh);
        assert_eq!(config.refresh_interval, 5);
    }

    #[test]
    fn test_diff_style_equality() {
        assert_eq!(DiffStyle::Unified, DiffStyle::Unified);
        assert_eq!(DiffStyle::SideBySide, DiffStyle::SideBySide);
        assert_ne!(DiffStyle::Unified, DiffStyle::SideBySide);
    }

    #[test]
    fn test_shared_app_state() {
        let shared_state = SharedAppState::new(Mutex::new(AppState::new()));
        let state = shared_state.lock();
        assert!(!state.has_repository());
    }
}
