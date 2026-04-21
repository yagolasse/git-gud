//! Integration tests for Git Gud
//!
//! These tests verify the integration between different components
//! without requiring a GUI context.

use crate::services::GitService;
use crate::ui::RecentRepos;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Test the integration between GitService and RecentRepos
#[test]
fn test_git_service_recent_repos_integration() -> Result<()> {
    // Create a test repository
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;

    // Create a test file
    let test_file_path = repo_path.join("test.txt");
    fs::write(&test_file_path, "Test content")?;

    // Stage and commit the file
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Test commit")?;

    // Test RecentRepos integration
    let mut recent_repos = RecentRepos::new(5);

    // Add repository to recent repos
    recent_repos.add(repo_path);

    // Verify it was added
    assert!(recent_repos.contains(repo_path));
    assert_eq!(recent_repos.len(), 1);

    // Get recent repos
    let repos = recent_repos.get();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0], repo_path);

    // Test max count
    for i in 0..10 {
        let path = format!("/tmp/repo-{}", i);
        recent_repos.add(path.as_ref());
    }

    assert_eq!(recent_repos.len(), 5); // Should be limited to max_count

    Ok(())
}

/// Test error handling integration
#[test]
fn test_error_handling_integration() -> Result<()> {
    // Create RecentRepos
    let mut recent_repos = RecentRepos::default();

    // Add some repositories
    recent_repos.add("/path/to/repo1".as_ref());
    recent_repos.add("/path/to/repo2".as_ref());

    // Verify they were added
    assert_eq!(recent_repos.len(), 2);

    // Clear and verify
    recent_repos.clear();
    assert!(recent_repos.is_empty());

    Ok(())
}

/// Test path normalization
#[test]
fn test_path_handling_integration() -> Result<()> {
    let mut recent_repos = RecentRepos::new(3);

    // Add same path with different representations
    recent_repos.add("/home/user/repo".as_ref());
    recent_repos.add("/home/user/./repo".as_ref()); // Should be treated as same

    // With trailing slash
    recent_repos.add("/home/user/repo/".as_ref());

    // Recent repos should deduplicate
    assert_eq!(recent_repos.len(), 1);

    Ok(())
}
