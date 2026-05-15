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

    /// Stash entries
    pub stashes: Vec<models::StashEntry>,

    /// Tags with target commit IDs
    pub tags: Vec<crate::models::Tag>,

    /// Ahead/behind counts for the current branch
    pub ahead: usize,
    pub behind: usize,

    /// Recent commits (up to 500)
    pub commits: Vec<models::Commit>,

    /// Worktrees associated with this repository
    pub worktrees: Vec<models::WorktreeEntry>,
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
            head: repository
                .head()
                .ok()
                .map(|r| r.shorthand().unwrap_or("").to_string()),
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
            stashes: Vec::new(),
            tags: Vec::new(),
            ahead: 0,
            behind: 0,
            commits: Vec::new(),
            worktrees: Vec::new(),
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

        // Load stashes
        self.stashes = services::GitService::stash_list(&mut self.repository).unwrap_or_default();

        // Load tags
        self.tags = services::GitService::get_tags(&self.repository).unwrap_or_default();

        // Load ahead/behind
        let (ahead, behind) = services::GitService::get_ahead_behind(&self.repository);
        self.ahead = ahead;
        self.behind = behind;

        // Load recent commits
        self.commits = services::GitService::get_commits(&self.repository, 500).unwrap_or_default();

        // Load worktrees
        self.worktrees = services::GitService::list_worktrees(&self.repository).unwrap_or_default();

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
        let (mut unstaged, staged) = services::GitService::get_status(&self.repository)?;
        // Populate conflict_count for conflicted files
        if let Some(workdir) = self.repository.workdir() {
            for file in unstaged.iter_mut() {
                if file.status == crate::models::FileStatus::Conflicted {
                    let abs = workdir.join(&file.path);
                    let count = std::fs::read_to_string(&abs)
                        .map(|s| s.lines().filter(|l| l.starts_with("<<<<<<<")).count())
                        .unwrap_or(0);
                    file.conflict_count = Some(count);
                }
            }
        }
        self.unstaged_files = unstaged;
        self.staged_files = staged;
        log::debug!(
            "Loaded {} unstaged files, {} staged files",
            self.unstaged_files.len(),
            self.staged_files.len()
        );
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
        self.remotes = self
            .repository
            .remotes()?
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

    /// Amend the HEAD commit
    pub fn amend_commit(&mut self, summary: &str, description: &str) -> anyhow::Result<()> {
        services::GitService::amend_commit(&self.repository, summary, description)?;
        self.refresh()?;
        Ok(())
    }

    /// Delete a local branch
    pub fn delete_branch(&mut self, name: &str) -> anyhow::Result<()> {
        services::GitService::delete_branch(&self.repository, name)?;
        self.refresh()?;
        Ok(())
    }

    /// Rename a local branch
    pub fn rename_branch(&mut self, old_name: &str, new_name: &str) -> anyhow::Result<()> {
        services::GitService::rename_branch(&self.repository, old_name, new_name)?;
        self.refresh()?;
        Ok(())
    }

    /// Create a new local branch
    pub fn create_branch(&mut self, name: &str, checkout: bool) -> anyhow::Result<()> {
        services::GitService::create_branch(&self.repository, name, checkout)?;
        self.refresh()?;
        Ok(())
    }

    /// Create an annotated tag at HEAD
    pub fn create_tag(&mut self, name: &str, message: &str) -> anyhow::Result<()> {
        services::GitService::create_tag(&self.repository, name, message)?;
        self.refresh()?;
        Ok(())
    }

    /// Push a tag to a remote
    pub fn push_tag(&self, remote_name: &str, tag_name: &str) -> anyhow::Result<()> {
        services::GitService::push_tag(&self.repository, remote_name, tag_name)
    }

    /// Save working tree changes to the stash
    pub fn stash_save(&mut self, message: &str) -> anyhow::Result<()> {
        services::GitService::stash_save(&mut self.repository, message)?;
        self.refresh()?;
        Ok(())
    }

    /// Apply and remove a stash entry
    pub fn stash_pop(&mut self, index: usize) -> anyhow::Result<()> {
        services::GitService::stash_pop(&mut self.repository, index)?;
        self.refresh()?;
        Ok(())
    }

    /// Apply a stash entry without removing it
    pub fn stash_apply(&mut self, index: usize) -> anyhow::Result<()> {
        services::GitService::stash_apply(&mut self.repository, index)?;
        self.refresh()?;
        Ok(())
    }

    /// Remove a stash entry without applying it
    pub fn stash_drop(&mut self, index: usize) -> anyhow::Result<()> {
        services::GitService::stash_drop(&mut self.repository, index)?;
        self.stashes = services::GitService::stash_list(&mut self.repository).unwrap_or_default();
        Ok(())
    }

    /// Pull from a remote (fast-forward only)
    pub fn pull(&mut self, remote_name: &str) -> anyhow::Result<()> {
        services::GitService::pull(&self.repository, remote_name)?;
        self.refresh()?;
        Ok(())
    }

    /// Push the current branch to a remote
    pub fn push(&mut self, remote_name: &str, branch_name: &str) -> anyhow::Result<()> {
        services::GitService::push(&self.repository, remote_name, branch_name)
    }

    /// Cherry-pick a commit onto the current branch
    pub fn cherry_pick(&mut self, commit_id: &str) -> anyhow::Result<()> {
        services::GitService::cherry_pick(&self.repository, commit_id)?;
        self.refresh()?;
        Ok(())
    }

    pub fn cherry_pick_skip(&mut self) -> anyhow::Result<()> {
        services::GitService::cherry_pick_skip(&self.repository)?;
        self.refresh()?;
        Ok(())
    }

    pub fn cherry_pick_no_commit(&mut self, commit_id: &str) -> anyhow::Result<()> {
        services::GitService::cherry_pick_no_commit(&self.repository, commit_id)?;
        self.refresh()?;
        Ok(())
    }

    pub fn revert_commit(&mut self, commit_id: &str) -> anyhow::Result<()> {
        services::GitService::revert_commit(&self.repository, commit_id)?;
        self.refresh()?;
        Ok(())
    }

    pub fn create_branch_at(&mut self, name: &str, commit_id: &str, checkout: bool) -> anyhow::Result<()> {
        services::GitService::create_branch_at(&self.repository, name, commit_id, checkout)?;
        self.refresh()?;
        Ok(())
    }

    pub fn add_worktree(&mut self, path: &std::path::Path, branch: &str) -> anyhow::Result<()> {
        services::GitService::add_worktree(&self.repository, path, branch)?;
        self.refresh()?;
        Ok(())
    }

    pub fn remove_worktree(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        services::GitService::remove_worktree(&self.repository, path)?;
        self.refresh()?;
        Ok(())
    }

    pub fn resolve_ours(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        services::GitService::resolve_conflict_ours(&self.repository, path)?;
        self.refresh()?;
        Ok(())
    }

    pub fn resolve_theirs(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        services::GitService::resolve_conflict_theirs(&self.repository, path)?;
        self.refresh()?;
        Ok(())
    }

    /// Merge a branch into the current branch
    pub fn merge_branch(&mut self, branch_name: &str) -> anyhow::Result<()> {
        services::GitService::merge_branch(&self.repository, branch_name)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::GitService;
    use std::fs;
    use tempfile::TempDir;

    fn setup_identity(repo: &git2::Repository) -> anyhow::Result<()> {
        let mut cfg = repo.config()?;
        cfg.set_str("user.name", "Test User")?;
        cfg.set_str("user.email", "test@example.com")?;
        Ok(())
    }

    #[test]
    fn test_repository_state_new() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = GitService::init_repository(repo_path)?;

        // Create repository state
        let state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        assert_eq!(state.path, repo_path);
        assert_eq!(
            state.model.name,
            repo_path.file_name().unwrap().to_string_lossy()
        );
        assert!(!state.model.is_bare);
        assert!(state.branches.is_empty()); // No commits yet, so no branches
        assert!(state.unstaged_files.is_empty());
        assert!(state.staged_files.is_empty());
        assert!(state.head_commit.is_none());
        assert!(state.remotes.is_empty());

        Ok(())
    }

    #[test]
    fn test_repository_state_refresh() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository and create initial commit
        let repo = GitService::init_repository(repo_path)?;
        setup_identity(&repo)?;
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        GitService::stage_files(&repo, &[test_file.clone()])?;
        GitService::create_commit(&repo, "Initial commit")?;

        // Create repository state
        let mut state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        // Initial state should have branches (main/master)
        assert!(!state.branches.is_empty());
        assert!(state.current_branch().is_some());
        assert!(state.head_commit.is_some());
        assert!(state.unstaged_files.is_empty());
        assert!(state.staged_files.is_empty());

        // Modify file to create unstaged changes
        fs::write(&test_file, "modified content")?;
        state.refresh()?;

        assert!(state.has_unstaged_changes());
        assert!(!state.has_staged_changes());
        assert_eq!(state.unstaged_files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_stage_and_unstage_files() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = GitService::init_repository(repo_path)?;

        // Create repository state (no files yet)
        let mut state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        // Initially no changes
        assert!(!state.has_unstaged_changes());
        assert!(!state.has_staged_changes());

        // Create a file
        let new_file = repo_path.join("new.txt");
        fs::write(&new_file, "new file")?;

        // Refresh to detect unstaged file
        state.refresh()?;
        assert!(state.has_unstaged_changes());
        assert!(!state.has_staged_changes());

        // Stage the file
        state.stage_files(&[new_file.clone()])?;
        assert!(!state.has_unstaged_changes());
        assert!(state.has_staged_changes());

        // Unstage the file
        state.unstage_files(&[new_file.clone()])?;
        assert!(state.has_unstaged_changes());
        assert!(!state.has_staged_changes());

        Ok(())
    }

    #[test]
    fn test_create_commit() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = GitService::init_repository(repo_path)?;
        setup_identity(&repo)?;
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        // Create repository state
        let mut state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        // Stage and commit
        state.stage_files(&[test_file.clone()])?;
        state.create_commit("Test commit")?;

        // After commit, should have no staged/unstaged changes
        assert!(!state.has_unstaged_changes());
        assert!(!state.has_staged_changes());
        assert!(state.head_commit.is_some());
        assert_eq!(state.head_commit.as_ref().unwrap().message, "Test commit");

        Ok(())
    }

    #[test]
    fn test_current_branch() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository with initial commit
        let repo = GitService::init_repository(repo_path)?;
        setup_identity(&repo)?;
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        GitService::stage_files(&repo, &[test_file.clone()])?;
        GitService::create_commit(&repo, "Initial commit")?;

        // Create repository state
        let state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        // Should have a current branch (main or master)
        assert!(state.current_branch().is_some());
        let branch_name = state.current_branch().unwrap();
        assert!(branch_name == "main" || branch_name == "master");

        Ok(())
    }

    #[test]
    fn test_has_unstaged_and_staged_changes() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = GitService::init_repository(repo_path)?;
        setup_identity(&repo)?;
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content")?;

        // Stage and commit the file to start with clean repo
        GitService::stage_files(&repo, &[test_file.clone()])?;
        GitService::create_commit(&repo, "Initial commit")?;

        // Create repository state after commit
        let mut state = RepositoryState::new(repo, repo_path.to_path_buf())?;

        // Initially no changes
        assert!(!state.has_unstaged_changes());
        assert!(!state.has_staged_changes());

        // Modify file
        fs::write(&test_file, "modified")?;
        state.refresh()?;

        // Should have unstaged changes
        assert!(state.has_unstaged_changes());
        assert!(!state.has_staged_changes());

        // Stage the change
        state.stage_files(&[test_file.clone()])?;

        // Should have staged changes, no unstaged
        assert!(!state.has_unstaged_changes());
        assert!(state.has_staged_changes());

        Ok(())
    }

    #[test]
    fn test_tags_loaded_on_refresh() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        let repo = GitService::init_repository(repo_path)?;
        let mut cfg = repo.config()?;
        cfg.set_str("user.name", "Test User")?;
        cfg.set_str("user.email", "test@example.com")?;

        let file = repo_path.join("tag_test.txt");
        fs::write(&file, "tagged")?;
        GitService::stage_files(&repo, &[file.clone()])?;
        GitService::create_commit(&repo, "commit for tag")?;

        let head = repo.head()?.peel_to_commit()?.id();
        repo.tag("test-tag", &repo.find_object(head, None)?, &repo.signature()?, "test", false)?;

        let state = RepositoryState::new(repo, repo_path.to_path_buf())?;
        assert!(state.tags.iter().any(|t| t.name == "test-tag"));

        Ok(())
    }
}
