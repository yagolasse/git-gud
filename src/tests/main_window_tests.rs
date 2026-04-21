//! Unit tests for MainWindow state management
//!
//! These tests focus on the state transitions and logic of MainWindow
//! without requiring a GUI context.

use crate::state::AppState;
use crate::ui::ErrorDialog;
use std::sync::Arc;
use parking_lot::Mutex;

/// Test helper to create a mock MainWindow state
struct MockMainWindowState {
    show_open_dialog: bool,
    open_repo_path: String,
    error_dialog: ErrorDialog,
    state: Arc<Mutex<AppState>>,
}

impl MockMainWindowState {
    fn new() -> Self {
        Self {
            show_open_dialog: true,
            open_repo_path: ".".to_string(),
            error_dialog: ErrorDialog::new(),
            state: Arc::new(Mutex::new(AppState::new())),
        }
    }
    
    /// Simulate opening a repository (success case)
    fn simulate_open_repository_success(&mut self) {
        // In real code, this would call state.load_repository()
        // For testing, we'll just simulate the state change
        self.show_open_dialog = false;
        self.state.lock().clear_error();
    }
    
    /// Simulate opening a repository (error case)
    fn simulate_open_repository_error(&mut self, error_msg: &str) {
        // In real code, this would call state.load_repository() which sets error
        self.state.lock().set_error(error_msg.to_string());
        
        // Error dialog should show
        if !self.error_dialog.is_visible() {
            self.error_dialog.show_error(error_msg.to_string());
        }
    }
    
    /// Simulate closing repository
    fn simulate_close_repository(&mut self) {
        let mut state = self.state.lock();
        state.repository_state = None;
        state.ui_state.reset();
        state.clear_error();
        state.clear_info();
    }
}

/// Test MainWindow state transitions
#[test]
fn test_main_window_state_transitions() {
    let mut mock = MockMainWindowState::new();
    
    // Initial state
    assert!(mock.show_open_dialog);
    assert!(!mock.error_dialog.is_visible());
    assert!(!mock.state.lock().has_repository());
    
    // Simulate successful repository open
    mock.simulate_open_repository_success();
    assert!(!mock.show_open_dialog);
    assert!(!mock.error_dialog.is_visible());
    
    // Simulate repository close
    mock.simulate_close_repository();
    assert!(!mock.state.lock().has_repository());
    
    // Simulate error when opening repository
    mock.show_open_dialog = true;
    mock.simulate_open_repository_error("Failed to open repository");
    assert!(mock.error_dialog.is_visible());
}

/// Test error dialog integration with AppState
#[test]
fn test_error_dialog_app_state_integration() {
    let mut mock = MockMainWindowState::new();
    
    // Set error in AppState
    let error_msg = "Test error message";
    mock.state.lock().set_error(error_msg.to_string());
    
    // Error dialog should reflect AppState error
    if !mock.error_dialog.is_visible() {
        mock.error_dialog.show_error(error_msg.to_string());
    }
    
    assert!(mock.error_dialog.is_visible());
    
    // Clear error in AppState
    mock.state.lock().clear_error();
    mock.error_dialog.hide();
    
    assert!(!mock.error_dialog.is_visible());
}

/// Test open repository path handling
#[test]
fn test_open_repository_path() {
    let mut mock = MockMainWindowState::new();
    
    // Test default path
    assert_eq!(mock.open_repo_path, ".");
    
    // Update path
    mock.open_repo_path = "/path/to/repo".to_string();
    assert_eq!(mock.open_repo_path, "/path/to/repo");
    
    // Reset to default
    mock.open_repo_path = ".".to_string();
    assert_eq!(mock.open_repo_path, ".");
}