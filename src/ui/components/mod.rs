//! Reusable UI components for Git Gud
//!
//! This module contains reusable UI components that can be used
//! in different parts of the application.

mod branch_list;
mod commit_panel;
mod diff_viewer;
mod enhanced_diff_viewer;
mod error_dialog;
mod file_dialog;
mod file_list;
mod recent_repos;
mod virtual_scroll;

pub use branch_list::BranchList;
pub use commit_panel::CommitPanel;
pub use diff_viewer::DiffViewer;
pub use enhanced_diff_viewer::EnhancedDiffViewer;
pub use error_dialog::ErrorDialog;
pub use file_dialog::FileDialog;
pub use file_list::FileList;
pub use recent_repos::RecentRepos;
pub use virtual_scroll::VirtualScroll;
