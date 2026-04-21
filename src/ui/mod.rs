//! UI components for Git Gud application
//!
//! This module contains the egui-based user interface components
//! that make up the Git Gud GUI.

pub mod commit_view;
pub mod components;
pub mod main_window;
pub mod repository_view;

pub use commit_view::CommitView;
/// Re-exports for convenience
pub use components::{
    BranchList, CommitPanel, DiffViewer, EnhancedDiffViewer, ErrorDialog, FileDialog, FileList,
    RecentRepos,
};
pub use main_window::MainWindow;
pub use repository_view::RepositoryView;
