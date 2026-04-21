//! Service layer for Git Gud application
//!
//! This module contains the business logic and Git operations
//! that power both the CLI and GUI interfaces.

pub mod git_service;
pub mod repository_service;
pub mod log_service;
pub mod file_watcher_service;
pub mod syntax_service;
pub mod diff_parser;

/// Re-exports for convenience
pub use git_service::GitService;
pub use repository_service::RepositoryService;
pub use log_service::LogService;
pub use file_watcher_service::{FileWatcherService, SharedFileWatcher};
pub use syntax_service::SyntaxService;
pub use diff_parser::DiffParser;

/// Result type for service operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;