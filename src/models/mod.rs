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

// Re-export diff module
pub mod diff;
pub use diff::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::path::Path;

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            path: Path::new("/tmp/repo").to_path_buf(),
            name: "test-repo".to_string(),
            is_bare: false,
            head: Some("main".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&repo).unwrap();
        let deserialized: Repository = serde_json::from_str(&json).unwrap();

        assert_eq!(repo.name, deserialized.name);
        assert_eq!(repo.is_bare, deserialized.is_bare);
        assert_eq!(repo.head, deserialized.head);
    }

    #[test]
    fn test_commit_serialization() {
        let commit = Commit {
            id: "abc123".to_string(),
            author: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            message: "Initial commit".to_string(),
            timestamp: 1234567890,
            parents: vec![],
        };

        let json = serde_json::to_string(&commit).unwrap();
        let deserialized: Commit = serde_json::from_str(&json).unwrap();

        assert_eq!(commit.id, deserialized.id);
        assert_eq!(commit.author, deserialized.author);
        assert_eq!(commit.message, deserialized.message);
        assert_eq!(commit.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_branch_serialization() {
        let branch = Branch {
            name: "feature-branch".to_string(),
            is_remote: false,
            is_current: true,
            commit_id: "def456".to_string(),
        };

        let json = serde_json::to_string(&branch).unwrap();
        let deserialized: Branch = serde_json::from_str(&json).unwrap();

        assert_eq!(branch.name, deserialized.name);
        assert_eq!(branch.is_remote, deserialized.is_remote);
        assert_eq!(branch.is_current, deserialized.is_current);
        assert_eq!(branch.commit_id, deserialized.commit_id);
    }

    #[test]
    fn test_file_status_enum() {
        // Test equality and serialization for each variant
        let variants = vec![
            FileStatus::Unmodified,
            FileStatus::Modified,
            FileStatus::Added,
            FileStatus::Deleted,
            FileStatus::Renamed,
            FileStatus::Copied,
            FileStatus::Untracked,
            FileStatus::Ignored,
        ];

        for (_i, variant) in variants.iter().enumerate() {
            // Test PartialEq
            assert_eq!(variant, variant);

            // Test serialization round-trip
            let json = serde_json::to_string(variant).unwrap();
            let deserialized: FileStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, &deserialized);
        }
    }

    #[test]
    fn test_file_change_serialization() {
        let file_change = FileChange {
            path: Path::new("src/main.rs").to_path_buf(),
            status: FileStatus::Modified,
            diff: Some("--- a/src/main.rs\n+++ b/src/main.rs".to_string()),
        };

        let json = serde_json::to_string(&file_change).unwrap();
        let deserialized: FileChange = serde_json::from_str(&json).unwrap();

        assert_eq!(file_change.path, deserialized.path);
        assert_eq!(file_change.status, deserialized.status);
        assert_eq!(file_change.diff, deserialized.diff);
    }

    #[test]
    fn test_file_change_without_diff() {
        let file_change = FileChange {
            path: Path::new("README.md").to_path_buf(),
            status: FileStatus::Untracked,
            diff: None,
        };

        let json = serde_json::to_string(&file_change).unwrap();
        let deserialized: FileChange = serde_json::from_str(&json).unwrap();

        assert_eq!(file_change.path, deserialized.path);
        assert_eq!(file_change.status, deserialized.status);
        assert!(deserialized.diff.is_none());
    }
}
