use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents the status of a single file in the repository.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct FileStatus {
    /// The relative path to the file from the repository root.
    pub path: String,
    /// A human-readable status string (e.g., "Added", "Modified", "Deleted").
    pub status: String,
    /// Whether the file is currently staged in the Git index.
    pub staged: bool,
}

/// Basic metadata about an opened repository.
#[derive(Serialize, Clone, Debug)]
pub struct RepoInfo {
    /// Absolute path to the repository's working directory.
    pub path: String,
    /// The name of the repository (usually the directory name).
    pub name: String,
    /// The name of the current branch or "DETACHED HEAD".
    pub current_branch: String,
    /// The shorthand name of the HEAD reference, if available.
    pub head_shorthand: Option<String>,
}

/// Represents a Git branch.
#[derive(Serialize, Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

/// Represents a Git stash.
#[derive(Serialize, Clone, Debug)]
pub struct StashInfo {
    pub index: usize,
    pub message: String,
}

/// Represents a Git remote.
#[derive(Serialize, Clone, Debug)]
pub struct RemoteInfo {
    pub name: String,
    pub url: Option<String>,
}

/// Global state to manage active file system watchers for each open repository.
#[derive(Default)]
pub struct WatcherState {
    /// A map of repository paths to their respective `notify` watchers.
    pub watchers: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}
