//! UI components for Git Gud application
//!
//! This module contains the egui-based user interface components
//! that make up the Git Gud GUI.

pub mod colors;
pub mod components;
pub mod main_window;

/// Re-exports for convenience
pub use components::{
    BranchList, CommandLog, CommitGraph, CommitPanel, EnhancedDiffViewer, ErrorDialog, FileDialog,
    FileHistoryPanel, FileList, PassphraseDialog, RecentRepos, Toolbar,
};
pub use components::ToolbarAction;
pub use main_window::MainWindow;
