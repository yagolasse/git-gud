//! Unit tests for UI components

use crate::ui::ErrorDialog;

/// Test ErrorDialog basic functionality
#[test]
fn test_error_dialog_basics() {
    let mut dialog = ErrorDialog::new();

    // Test initial state
    assert!(!dialog.is_visible());

    // Test showing error
    dialog.show_error("Test error message".to_string());
    assert!(dialog.is_visible());

    // Test hiding
    dialog.hide();
    assert!(!dialog.is_visible());
}

/// Test ErrorDialog with long error message
#[test]
fn test_error_dialog_long_message() {
    let mut dialog = ErrorDialog::new();

    // Create a long error message
    let long_message = "Error: ".to_string() + &"x".repeat(200);
    dialog.show_error(long_message.clone());

    assert!(dialog.is_visible());
    // Note: We can't test the UI rendering without a GUI context
}

/// Test ErrorDialog default implementation
#[test]
fn test_error_dialog_default() {
    let dialog = ErrorDialog::default();
    assert!(!dialog.is_visible());
}

/// Test staging/unstaging notification mechanism
#[test]
fn test_staging_unstaging_notification() {
    use crate::state::UIState;

    let mut ui_state = UIState::new();

    // Initially should be false
    assert!(!ui_state.check_and_reset_staged_unstaged());

    // Mark as staged/unstaged
    ui_state.mark_files_staged_or_unstaged();

    // Should return true and reset
    assert!(ui_state.check_and_reset_staged_unstaged());

    // Should be false again after reset
    assert!(!ui_state.check_and_reset_staged_unstaged());

    // Test reset method clears the flag
    ui_state.mark_files_staged_or_unstaged();
    ui_state.reset();
    assert!(!ui_state.check_and_reset_staged_unstaged());
}
