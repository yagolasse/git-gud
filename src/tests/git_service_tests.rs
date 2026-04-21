//! Tests for Git service functionality

use crate::services::GitService;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Test basic Git repository operations
#[test]
fn test_git_repository_operations() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Test repository initialization
    let repo = GitService::init_repository(repo_path)?;
    assert!(repo_path.join(".git").exists());
    
    // Test opening the repository
    let opened_repo = GitService::open_repository(repo_path)?;
    assert_eq!(repo.path(), opened_repo.path());
    
    // Test checking if it's a repository
    assert!(GitService::is_repository(repo_path));
    
    Ok(())
}

/// Test file staging and unstaging
#[test]
fn test_file_staging_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create a test file
    let test_file_path = repo_path.join("test.txt");
    fs::write(&test_file_path, "Hello, Git!")?;
    
    // Get initial status (should have untracked file)
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert_eq!(unstaged.len(), 1); // File should be untracked (unstaged)
    assert!(staged.is_empty());
    
    // Stage the file
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after staging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert!(unstaged.is_empty()); // Should be no unstaged changes
    assert_eq!(staged.len(), 1); // File should be staged
    assert_eq!(staged[0].path.file_name(), test_file_path.file_name());
    assert!(matches!(staged[0].status, crate::models::FileStatus::Added));
    
    // Unstage the file
    GitService::unstage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after unstaging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert_eq!(unstaged.len(), 1); // File should be unstaged
    assert!(staged.is_empty()); // Should be no staged changes
    assert_eq!(unstaged[0].path.file_name(), test_file_path.file_name());
    assert!(matches!(unstaged[0].status, crate::models::FileStatus::Untracked));
    
    Ok(())
}

/// Test branch operations
#[test]
fn test_branch_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create initial commit
    let test_file_path = repo_path.join("initial.txt");
    fs::write(&test_file_path, "Initial commit")?;
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Initial commit")?;
    
    // Get branches
    let branches = GitService::get_branches(&repo)?;
    assert_eq!(branches.len(), 1); // Should have master/main branch
    assert!(branches[0].is_current); // Should be current branch
    
    Ok(())
}

/// Test commit creation
#[test]
fn test_commit_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create and stage a file
    let test_file_path = repo_path.join("commit_test.txt");
    fs::write(&test_file_path, "Test commit")?;
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    
    // Create a commit
    GitService::create_commit(&repo, "Test commit message")?;
    
    // Get HEAD commit
    let head_commit = GitService::get_head_commit(&repo)?;
    assert_eq!(head_commit.message, "Test commit message");
    assert!(!head_commit.id.is_empty());
    
    Ok(())
}

/// Test file diff generation
#[test]
fn test_file_diff() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create initial file and commit
    let test_file_path = repo_path.join("diff_test.txt");
    fs::write(&test_file_path, "Line 1\nLine 2\nLine 3")?;
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Initial content")?;
    
    // Modify the file
    fs::write(&test_file_path, "Line 1\nModified Line 2\nLine 3\nLine 4")?;
    
    // Get diff
    let diff = GitService::get_file_diff(&repo, &test_file_path)?;
    assert!(!diff.is_empty());
    assert!(diff.contains("Modified"));
    
    Ok(())
}