//! Recent repositories component for Git Gud
//!
//! This component manages and displays recently opened repositories.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

/// Recent repositories manager
pub struct RecentRepos {
    /// List of recent repository paths (most recent first)
    repos: VecDeque<PathBuf>,
    
    /// Maximum number of recent repos to keep
    max_count: usize,
}

impl RecentRepos {
    /// Create a new recent repos manager
    pub fn new(max_count: usize) -> Self {
        Self {
            repos: VecDeque::with_capacity(max_count),
            max_count,
        }
    }
    
    /// Add a repository to the recent list
    pub fn add(&mut self, path: &Path) {
        // Remove if already exists
        self.repos.retain(|p| p != path);
        
        // Add to front
        self.repos.push_front(path.to_path_buf());
        
        // Trim to max count
        if self.repos.len() > self.max_count {
            self.repos.pop_back();
        }
    }
    
    /// Get the list of recent repositories (most recent first)
    pub fn get(&self) -> Vec<&PathBuf> {
        self.repos.iter().collect()
    }
    
    /// Check if a repository is in the recent list
    pub fn contains(&self, path: &Path) -> bool {
        self.repos.iter().any(|p| p == path)
    }
    
    /// Clear all recent repositories
    pub fn clear(&mut self) {
        self.repos.clear();
    }
    
    /// Get the number of recent repositories
    pub fn len(&self) -> usize {
        self.repos.len()
    }
    
    /// Check if there are no recent repositories
    pub fn is_empty(&self) -> bool {
        self.repos.is_empty()
    }
}

impl Default for RecentRepos {
    fn default() -> Self {
        Self::new(10)
    }
}