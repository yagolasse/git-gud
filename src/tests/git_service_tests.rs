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
    assert!(matches!(
        unstaged[0].status,
        crate::models::FileStatus::Untracked
    ));

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
    assert!(matches!(
        staged[0].status,
        crate::models::FileStatus::Modified
    ));

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
    assert!(
        staged.is_empty(),
        "Staged should be empty but has {} files: {:?}",
        staged.len(),
        staged
    ); // Should be no staged changes
    assert!(matches!(
        unstaged[0].status,
        crate::models::FileStatus::Modified
    ));

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
    assert!(matches!(
        unstaged[0].status,
        crate::models::FileStatus::Modified
    ));

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
    assert!(matches!(
        unstaged[0].status,
        crate::models::FileStatus::Untracked
    ));

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
    GitService::stage_files(
        &repo,
        &[file1_path.clone(), file2_path.clone(), file3_path.clone()],
    )?;

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
    GitService::unstage_files(
        &repo,
        &[file1_path.clone(), file2_path.clone(), file3_path.clone()],
    )?;

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
    assert!(
        staged.is_empty(),
        "Staged should be empty but has {} files",
        staged.len()
    );

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
    assert_eq!(
        found_untracked, 2,
        "Expected 2 untracked files, found {}",
        found_untracked
    );

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

/// Configure git identity on a repo so signature() works in tests
fn setup_identity(repo: &git2::Repository) -> Result<()> {
    let mut cfg = repo.config()?;
    cfg.set_str("user.name", "Test User")?;
    cfg.set_str("user.email", "test@example.com")?;
    Ok(())
}

/// Test amend: create a commit, amend with new message, verify HEAD message updated
#[test]
fn test_amend_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    let repo = GitService::init_repository(repo_path)?;
    setup_identity(&repo)?;

    let file = repo_path.join("a.txt");
    fs::write(&file, "hello")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "original message")?;

    // Amend with new message
    GitService::amend_commit(&repo, "amended summary", "amended body")?;

    let head = GitService::get_head_commit(&repo)?;
    assert!(head.message.starts_with("amended summary"));
    assert!(head.message.contains("amended body"));

    Ok(())
}

/// Test amend without description
#[test]
fn test_amend_commit_summary_only() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("b.txt");
    fs::write(&file, "data")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "first")?;

    GitService::amend_commit(&repo, "second", "")?;
    let head = GitService::get_head_commit(&repo)?;
    assert_eq!(head.message.trim(), "second");

    Ok(())
}

/// Test create_branch: branch exists after creation
#[test]
fn test_create_branch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("f.txt");
    fs::write(&file, "data")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "init")?;

    GitService::create_branch(&repo, "feature", false)?;

    let branch = repo.find_branch("feature", git2::BranchType::Local)?;
    assert!(!branch.is_head());

    Ok(())
}

/// Test create_branch with checkout: HEAD moves to new branch
#[test]
fn test_create_branch_with_checkout() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("g.txt");
    fs::write(&file, "data")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "init")?;

    GitService::create_branch(&repo, "new-feature", true)?;

    let head = repo.head()?;
    let branch_name = head.shorthand().unwrap_or("");
    assert_eq!(branch_name, "new-feature");

    Ok(())
}

/// Test stash save and list
#[test]
fn test_stash_save_and_list() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    // Initial commit
    let file = temp_dir.path().join("x.txt");
    fs::write(&file, "original")?;
    GitService::stage_files(&repo, &[file.clone()])?;
    GitService::create_commit(&repo, "init")?;

    // Dirty working tree
    fs::write(&file, "modified")?;

    GitService::stash_save(&mut repo, "my stash")?;

    // Working tree should be clean after stash
    let (unstaged, staged) = GitService::get_status(&repo)?;
    assert!(unstaged.is_empty(), "expected clean working tree after stash");
    assert!(staged.is_empty());

    let stashes = GitService::stash_list(&mut repo)?;
    assert_eq!(stashes.len(), 1);
    assert!(stashes[0].message.contains("my stash"));

    Ok(())
}

/// Test stash pop: working tree is restored after pop
#[test]
fn test_stash_pop() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("y.txt");
    fs::write(&file, "original")?;
    GitService::stage_files(&repo, &[file.clone()])?;
    GitService::create_commit(&repo, "init")?;

    fs::write(&file, "modified")?;
    GitService::stash_save(&mut repo, "pop test")?;

    // Verify clean
    let (u, _) = GitService::get_status(&repo)?;
    assert!(u.is_empty());

    // Pop
    GitService::stash_pop(&mut repo, 0)?;

    // Working tree should be dirty again
    let (unstaged, _) = GitService::get_status(&repo)?;
    assert!(!unstaged.is_empty(), "expected dirty working tree after pop");

    let stashes = GitService::stash_list(&mut repo)?;
    assert!(stashes.is_empty(), "stash list should be empty after pop");

    Ok(())
}

/// Test stash drop: entry removed, working tree unchanged
#[test]
fn test_stash_drop() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("z.txt");
    fs::write(&file, "original")?;
    GitService::stage_files(&repo, &[file.clone()])?;
    GitService::create_commit(&repo, "init")?;

    fs::write(&file, "modified")?;
    GitService::stash_save(&mut repo, "drop test")?;

    GitService::stash_drop(&mut repo, 0)?;

    let stashes = GitService::stash_list(&mut repo)?;
    assert!(stashes.is_empty());

    // Working tree was NOT restored (drop without apply)
    let content = fs::read_to_string(&file)?;
    assert_eq!(content, "original", "working tree should stay clean after drop");

    Ok(())
}

/// Test multiple stashes: count and drop-by-index
#[test]
fn test_stash_multiple() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("m.txt");
    fs::write(&file, "v0")?;
    GitService::stage_files(&repo, &[file.clone()])?;
    GitService::create_commit(&repo, "init")?;

    for i in 1..=3usize {
        fs::write(&file, format!("v{}", i))?;
        GitService::stash_save(&mut repo, &format!("stash {}", i))?;
    }

    let stashes = GitService::stash_list(&mut repo)?;
    assert_eq!(stashes.len(), 3);

    // Drop stash at index 1 (middle)
    GitService::stash_drop(&mut repo, 1)?;
    let stashes = GitService::stash_list(&mut repo)?;
    assert_eq!(stashes.len(), 2);

    Ok(())
}

/// Test push and pull using a local bare repository as remote
#[test]
fn test_push_and_pull() -> Result<()> {
    let bare_dir = TempDir::new()?;
    let local_dir = TempDir::new()?;
    let local2_dir = TempDir::new()?;

    // Create bare remote — use three slashes so Windows paths like C:/ are valid
    let bare_url = format!("file:///{}", bare_dir.path().to_string_lossy().replace('\\', "/"));
    git2::Repository::init_bare(bare_dir.path())?;

    // Set up local repo A, add remote, push initial commit
    let repo_a = GitService::init_repository(local_dir.path())?;
    setup_identity(&repo_a)?;
    let file = local_dir.path().join("r.txt");
    fs::write(&file, "hello remote")?;
    GitService::stage_files(&repo_a, &[file])?;
    GitService::create_commit(&repo_a, "initial")?;
    repo_a.remote("origin", &bare_url)?;
    GitService::push(&repo_a, "origin", "master")
        .or_else(|_| GitService::push(&repo_a, "origin", "main"))?;

    // Set up local repo B from a clone
    let repo_b = git2::build::RepoBuilder::new().clone(&bare_url, local2_dir.path())?;
    setup_identity(&repo_b)?;

    // Pull in repo B (should be up-to-date or succeed)
    let result = GitService::pull(&repo_b, "origin");
    // Up-to-date is also success
    assert!(result.is_ok(), "pull failed: {:?}", result);

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

/// Test that get_branches includes remote branches after setting up a remote
#[test]
fn test_get_branches_includes_remotes() -> Result<()> {
    let local_dir = TempDir::new()?;
    let bare_dir = TempDir::new()?;

    let bare_url = format!("file:///{}", bare_dir.path().to_string_lossy().replace('\\', "/"));
    git2::Repository::init_bare(bare_dir.path())?;

    let repo = GitService::init_repository(local_dir.path())?;
    setup_identity(&repo)?;

    let file = local_dir.path().join("f.txt");
    fs::write(&file, "hello")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "init")?;

    repo.remote("origin", &bare_url)?;
    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or("main");
    GitService::push(&repo, "origin", branch)?;

    let branches = GitService::get_branches(&repo)?;
    assert!(
        branches.iter().any(|b| b.is_remote),
        "expected at least one remote branch"
    );

    Ok(())
}

/// Test that get_tags returns tag names
#[test]
fn test_get_tags_empty_and_populated() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("t.txt");
    fs::write(&file, "tagged")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "init")?;

    let tags = GitService::get_tags(&repo)?;
    assert!(tags.is_empty());

    let head = repo.head()?.peel_to_commit()?.id();
    repo.tag("v1.0", &repo.find_object(head, None)?, &repo.signature()?, "first tag", false)?;
    repo.tag("v2.0", &repo.find_object(head, None)?, &repo.signature()?, "second tag", false)?;
    repo.tag("lightweight", &repo.find_object(head, None)?, &repo.signature()?, "", false)?;

    let tags = GitService::get_tags(&repo)?;
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"v1.0".to_string()));
    assert!(tags.contains(&"v2.0".to_string()));
    assert!(tags.contains(&"lightweight".to_string()));

    Ok(())
}

/// Test creating a tag via GitService and verifying it is listed
#[test]
fn test_create_tag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo = GitService::init_repository(temp_dir.path())?;
    setup_identity(&repo)?;

    let file = temp_dir.path().join("c.txt");
    fs::write(&file, "tag test")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "commit for tag")?;

    GitService::create_tag(&repo, "release-1.0", "First release")?;

    let tags = GitService::get_tags(&repo)?;
    assert!(tags.contains(&"release-1.0".to_string()));

    Ok(())
}

/// Test pushing a tag to a remote
#[test]
fn test_push_tag() -> Result<()> {
    let local_dir = TempDir::new()?;
    let bare_dir = TempDir::new()?;

    let bare_url = format!("file:///{}", bare_dir.path().to_string_lossy().replace('\\', "/"));
    git2::Repository::init_bare(bare_dir.path())?;

    let repo = GitService::init_repository(local_dir.path())?;
    setup_identity(&repo)?;

    let file = local_dir.path().join("p.txt");
    fs::write(&file, "push tag test")?;
    GitService::stage_files(&repo, &[file])?;
    GitService::create_commit(&repo, "init")?;

    repo.remote("origin", &bare_url)?;
    let branch = repo.head()?.shorthand().unwrap_or("main").to_string();
    GitService::push(&repo, "origin", &branch)?;

    GitService::create_tag(&repo, "v1.0", "version 1.0")?;
    GitService::push_tag(&repo, "origin", "v1.0")?;

    Ok(())
}
