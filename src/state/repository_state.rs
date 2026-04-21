//! Repository-specific state for Git Gud
//!
//! This struct holds state specific to a loaded Git repository,
//! including branches, file status, and repository metadata.

use crate::models;
use crate::services;
use git2::Repository;
use std::path::PathBuf;

/// Repository-specific state
pub struct RepositoryState {
    /// The underlying git2 repository
    pub repository: Repository,
    
    /// Repository path
    pub path: PathBuf,
    
    /// Repository model
    pub model: models::Repository,
    
    /// List of branches
    pub branches: Vec<models::Branch>,
    
    /// Unstaged files (working directory changes)
    pub unstaged_files: Vec<models::FileChange>,
    
    /// Staged files (index changes)
    pub staged_files: Vec<models::FileChange>,
    
    /// Current HEAD commit
    pub head_commit: Option<models::Commit>,
    
    /// Remote repositories
    pub remotes: Vec<String>,
}

impl RepositoryState {
    /// Create a new repository state from a git2 repository
    pub fn new(repository: Repository, path: PathBuf) -> anyhow::Result<Self> {
        log::info!("Creating repository state for: {:?}", path);
        
        // Create repository model
        let model = models::Repository {
            path: path.clone(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            is_bare: repository.is_bare(),
            head: repository.head().ok().map(|r| r.shorthand().unwrap_or("").to_string()),
        };
        
        let mut state = Self {
            repository,
            path,
            model,
            branches: Vec::new(),
            unstaged_files: Vec::new(),
            staged_files: Vec::new(),
            head_commit: None,
            remotes: Vec::new(),
        };
        
        // Load initial data
        state.refresh()?;
        
        Ok(state)
    }
    
    /// Refresh repository data (branches, status, etc.)
    pub fn refresh(&mut self) -> anyhow::Result<()> {
        log::debug!("Refreshing repository state");
        
        // Load branches
        self.load_branches()?;
        
        // Load file status
        self.load_file_status()?;
        
        // Load HEAD commit
        self.load_head_commit()?;
        
        // Load remotes
        self.load_remotes()?;
        
        Ok(())
    }
    
    /// Load branches from repository
    fn load_branches(&mut self) -> anyhow::Result<()> {
        self.branches = services::GitService::get_branches(&self.repository)?;
        log::debug!("Loaded {} branches", self.branches.len());
        Ok(())
    }
    
    /// Load file status (unstaged and staged files)
    fn load_file_status(&mut self) -> anyhow::Result<()> {
        let (unstaged, staged) = services::GitService::get_status(&self.repository)?;
        self.unstaged_files = unstaged;
        self.staged_files = staged;
        log::debug!("Loaded {} unstaged files, {} staged files", 
                   self.unstaged_files.len(), self.staged_files.len());
        Ok(())
    }
    
    /// Load HEAD commit
    fn load_head_commit(&mut self) -> anyhow::Result<()> {
        self.head_commit = services::GitService::get_head_commit(&self.repository).ok();
        if let Some(commit) = &self.head_commit {
            log::debug!("HEAD commit: {} - {}", commit.id, commit.message);
        }
        Ok(())
    }
    
    /// Load remote repositories
    fn load_remotes(&mut self) -> anyhow::Result<()> {
        self.remotes = self.repository.remotes()?
            .iter()
            .filter_map(|name| name.map(|s| s.to_string()))
            .collect();
        log::debug!("Loaded {} remotes", self.remotes.len());
        Ok(())
    }
    
    /// Stage files
    pub fn stage_files(&mut self, paths: &[PathBuf]) -> anyhow::Result<()> {
        log::info!("Staging {} files", paths.len());
        services::GitService::stage_files(&self.repository, paths)?;
        
        // Refresh file status after staging
        self.load_file_status()?;
        
        Ok(())
    }
    
    /// Unstage files
    pub fn unstage_files(&mut self, paths: &[PathBuf]) -> anyhow::Result<()> {
        log::info!("Unstaging {} files", paths.len());
        services::GitService::unstage_files(&self.repository, paths)?;
        
        // Refresh file status after unstaging
        self.load_file_status()?;
        
        Ok(())
    }
    
    /// Create a commit
    pub fn create_commit(&mut self, message: &str) -> anyhow::Result<()> {
        log::info!("Creating commit: {}", message);
        services::GitService::create_commit(&self.repository, message)?;
        
        // Refresh repository state after commit
        self.refresh()?;
        
        Ok(())
    }
    
    /// Checkout a branch
    pub fn checkout_branch(&mut self, branch_name: &str) -> anyhow::Result<()> {
        log::info!("Checking out branch: {}", branch_name);
        services::GitService::checkout_branch(&self.repository, branch_name)?;
        
        // Refresh repository state after checkout
        self.refresh()?;
        
        Ok(())
    }
    
    /// Get the current branch name
    pub fn current_branch(&self) -> Option<&str> {
        self.branches
            .iter()
            .find(|b| b.is_current)
            .map(|b| b.name.as_str())
    }
    
    /// Check if repository has unstaged changes
    pub fn has_unstaged_changes(&self) -> bool {
        !self.unstaged_files.is_empty()
    }
    
    /// Check if repository has staged changes
    pub fn has_staged_changes(&self) -> bool {
        !self.staged_files.is_empty()
    }
}