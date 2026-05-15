//! Reusable UI components for Git Gud
//!
//! This module contains reusable UI components that can be used
//! in different parts of the application.

mod branch_list;
mod command_log;
mod commit_graph;
mod commit_panel;
mod enhanced_diff_viewer;
mod error_dialog;
mod file_dialog;
mod file_history;
mod file_list;
mod passphrase_dialog;
mod recent_repos;
mod toolbar;
mod virtual_scroll;

pub use branch_list::BranchList;
pub use command_log::CommandLog;
pub use commit_graph::CommitGraph;
pub use commit_panel::CommitPanel;
pub use enhanced_diff_viewer::EnhancedDiffViewer;
pub use error_dialog::ErrorDialog;
pub use file_dialog::FileDialog;
pub use file_history::FileHistoryPanel;
pub use file_list::{open_in_explorer, FileList};
pub use passphrase_dialog::PassphraseDialog;
pub use recent_repos::RecentRepos;
pub use toolbar::{Toolbar, ToolbarAction};
pub use virtual_scroll::VirtualScroll;
