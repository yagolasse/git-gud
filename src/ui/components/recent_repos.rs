//! Recent repositories component for Git Gud
//!
//! This component manages and displays recently opened repositories.

use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

    /// Save recent repositories to a file
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let content = self
            .repos
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(path, content)
    }

    /// Load recent repositories from a file
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let repos = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                PathBuf::from_str(line.trim())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid path"))
            })
            .collect::<Result<VecDeque<_>, _>>()?;

        Ok(Self {
            repos,
            max_count: 10,
        })
    }

    /// Get the default path for storing recent repositories
    pub fn default_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("git-gud");
        path.push("recent_repos.txt");
        path
    }

    /// Load recent repositories from default location
    pub fn load_default() -> Self {
        let path = Self::default_path();
        match Self::load_from_file(&path) {
            Ok(repos) => repos,
            Err(_) => Self::default(),
        }
    }

    /// Save recent repositories to default location
    pub fn save_default(&self) -> io::Result<()> {
        let path = Self::default_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.save_to_file(&path)
    }
}

impl Default for RecentRepos {
    fn default() -> Self {
        Self::new(10)
    }
}
