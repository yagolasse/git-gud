//! Reusable UI components for Git Gud
//!
//! This module contains reusable UI components that can be used
//! in different parts of the application.

mod branch_list;
mod file_list;
mod diff_viewer;
mod commit_panel;
mod error_dialog;
mod file_dialog;
mod recent_repos;

pub use branch_list::BranchList;
pub use file_list::FileList;
pub use diff_viewer::DiffViewer;
pub use commit_panel::CommitPanel;
pub use error_dialog::ErrorDialog;
pub use file_dialog::FileDialog;
pub use recent_repos::RecentRepos;