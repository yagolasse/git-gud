//! Data models for Git Gud application
//!
//! This module contains the data structures that represent Git concepts
//! such as repositories, commits, branches, and file statuses.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub path: PathBuf,
    pub name: String,
    pub is_bare: bool,
    pub head: Option<String>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: i64,
    pub parents: Vec<String>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_remote: bool,
    pub is_current: bool,
    pub commit_id: String,
}

/// File status in working directory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
}

/// File change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub status: FileStatus,
    pub diff: Option<String>,
}