//! Service layer for Git Gud application
//!
//! This module contains the business logic and Git operations
//! that power both the CLI and GUI interfaces.

pub mod diff_parser;
pub mod file_watcher_service;
pub mod git_service;
pub mod log_service;
pub mod repository_service;
pub mod syntax_service;

pub use diff_parser::DiffParser;
pub use file_watcher_service::{FileWatcherService, SharedFileWatcher};
/// Re-exports for convenience
pub use git_service::GitService;
pub use log_service::LogService;
pub use repository_service::RepositoryService;
pub use syntax_service::SyntaxService;

/// Result type for service operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;
