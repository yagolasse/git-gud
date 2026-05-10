//! UI components for Git Gud application
//!
//! This module contains the egui-based user interface components
//! that make up the Git Gud GUI.

pub mod components;
pub mod main_window;

/// Re-exports for convenience
pub use components::{
    BranchList, CommandLog, CommitPanel, EnhancedDiffViewer, ErrorDialog, FileDialog, FileList,
    RecentRepos, Toolbar,
};
pub use main_window::MainWindow;
