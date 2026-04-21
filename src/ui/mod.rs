//! UI components for Git Gud application
//!
//! This module contains the egui-based user interface components
//! that make up the Git Gud GUI.

pub mod components;
pub mod main_window;
pub mod repository_view;
pub mod commit_view;

/// Re-exports for convenience
pub use components::{BranchList, FileList, DiffViewer, EnhancedDiffViewer, CommitPanel, ErrorDialog, FileDialog, RecentRepos};
pub use main_window::MainWindow;
pub use repository_view::RepositoryView;
pub use commit_view::CommitView;