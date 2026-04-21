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

/// Test unstaging behavior for modified files
#[test]
fn test_unstage_modified_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create a file and add it to the repository
    let test_file_path = repo_path.join("test.txt");
    std::fs::write(&test_file_path, "initial content")?;
    
    // Add and commit the file
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Initial commit")?;
    
    // Modify the file
    std::fs::write(&test_file_path, "modified content")?;
    
    // Stage the modification
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after staging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert!(unstaged.is_empty()); // Should be no unstaged changes
    assert_eq!(staged.len(), 1); // File should be staged
    assert!(matches!(staged[0].status, crate::models::FileStatus::Modified));
    
    // Unstage the file
    GitService::unstage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after unstaging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("DEBUG: After unstaging modified file:");
    println!("  Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("    - {:?}: {:?}", file.path, file.status);
    }
    println!("  Staged files: {}", staged.len());
    for file in &staged {
        println!("    - {:?}: {:?}", file.path, file.status);
    }
    
    assert_eq!(unstaged.len(), 1); // File should be unstaged
    assert!(staged.is_empty(), "Staged should be empty but has {} files: {:?}", staged.len(), staged); // Should be no staged changes
    assert!(matches!(unstaged[0].status, crate::models::FileStatus::Modified));
    
    Ok(())
}

/// Test unstaging behavior for partially staged files
#[test]
fn test_unstage_partially_staged_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create a file and add it to the repository
    let test_file_path = repo_path.join("test.txt");
    std::fs::write(&test_file_path, "line1\nline2\nline3\n")?;
    
    // Add and commit the file
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Initial commit")?;
    
    // Modify the file twice (create two separate changes)
    std::fs::write(&test_file_path, "line1\nline2 modified\nline3\n")?;
    
    // Stage the first modification
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    
    // Make another modification (now file is partially staged)
    std::fs::write(&test_file_path, "line1\nline2 modified\nline3 modified\n")?;
    
    // Check status - file should appear in both staged and unstaged
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus of partially staged file:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // File should appear in both lists (partially staged)
    assert!(unstaged.len() >= 1);
    assert!(staged.len() >= 1);
    
    // Unstage the file
    GitService::unstage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after unstaging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after unstaging partially staged file:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // After unstaging, all changes should be unstaged
    assert_eq!(unstaged.len(), 1);
    assert!(staged.is_empty());
    assert!(matches!(unstaged[0].status, crate::models::FileStatus::Modified));
    
    Ok(())
}

/// Test unstaging behavior for new files
#[test]
fn test_unstage_new_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create a new (untracked) file
    let new_file_path = repo_path.join("new.txt");
    std::fs::write(&new_file_path, "new file content")?;
    
    // Stage the new file
    GitService::stage_files(&repo, &[new_file_path.clone()])?;
    
    // Check status after staging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert!(unstaged.is_empty()); // Should be no unstaged changes
    assert_eq!(staged.len(), 1); // File should be staged
    assert!(matches!(staged[0].status, crate::models::FileStatus::Added));
    
    // Unstage the new file
    GitService::unstage_files(&repo, &[new_file_path.clone()])?;
    
    // Check status after unstaging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert_eq!(unstaged.len(), 1); // File should be unstaged
    assert!(staged.is_empty()); // Should be no staged changes
    assert!(matches!(unstaged[0].status, crate::models::FileStatus::Untracked));
    
    Ok(())
}

/// Test "Unstage All" with mixed file types (reproduces the reported issue)
#[test]
fn test_unstage_all_mixed_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    
    // Create and commit initial file
    let file1_path = repo_path.join("file1.txt");
    std::fs::write(&file1_path, "initial content")?;
    GitService::stage_files(&repo, &[file1_path.clone()])?;
    GitService::create_commit(&repo, "Initial commit")?;
    
    // Create multiple files with different statuses
    // 1. Modified existing file
    std::fs::write(&file1_path, "modified content")?;
    
    // 2. New file
    let file2_path = repo_path.join("file2.txt");
    std::fs::write(&file2_path, "new file content")?;
    
    // 3. Another new file
    let file3_path = repo_path.join("file3.txt");
    std::fs::write(&file3_path, "another new file")?;
    
    // Stage all files
    GitService::stage_files(&repo, &[file1_path.clone(), file2_path.clone(), file3_path.clone()])?;
    
    // Check status after staging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after staging all files:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    assert!(unstaged.is_empty());
    assert_eq!(staged.len(), 3);
    
    // Unstage all files (simulating "Unstage All" button)
    GitService::unstage_files(&repo, &[file1_path.clone(), file2_path.clone(), file3_path.clone()])?;
    
    // Check status after unstaging all
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after unstaging all files:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // After unstaging all:
    // - Modified file should be unstaged with status Modified
    // - New files should be unstaged with status Untracked
    // - No files should be staged
    assert_eq!(unstaged.len(), 3);
    assert!(staged.is_empty(), "Staged should be empty but has {} files", staged.len());
    
    // Check each file's status
    let mut found_modified = false;
    let mut found_untracked = 0;
    
    for file in &unstaged {
        match file.status {
            crate::models::FileStatus::Modified => {
                // Check if this is the modified file (compare filenames since paths are relative)
                if file.path.file_name() == file1_path.file_name() {
                    found_modified = true;
                }
            }
            crate::models::FileStatus::Untracked => {
                found_untracked += 1;
            }
            _ => panic!("Unexpected file status: {:?}", file.status),
        }
    }
    
    assert!(found_modified, "Modified file not found in unstaged list");
    assert_eq!(found_untracked, 2, "Expected 2 untracked files, found {}", found_untracked);
    
    println!("\n✓ All files unstaged correctly with proper statuses");
    println!("  - Modified file: shows as 'Modified' (not 'Deleted' or 'Added')");
    println!("  - New files: show as 'Untracked' (not in staged list)");
    println!("  - No files appear in both staged and unstaged lists");
    
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