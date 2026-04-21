//! Service layer for Git Gud application
//!
//! This module contains the business logic and Git operations
//! that power both the CLI and GUI interfaces.

pub mod git_service;
pub mod repository_service;
pub mod log_service;
pub mod file_watcher_service;

/// Re-exports for convenience
pub use git_service::GitService;
pub use repository_service::RepositoryService;
pub use log_service::LogService;
pub use file_watcher_service::{FileWatcherService, SharedFileWatcher};