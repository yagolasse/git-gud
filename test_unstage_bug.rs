use git_gud::services::GitService;
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> anyhow::Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    println!("Testing unstaging behavior...");
    println!("Repository path: {:?}", repo_path);
    
    // Initialize repository
    let repo = GitService::init_repository(repo_path)?;
    println!("Repository initialized");
    
    // Create a file and add it to the repository
    let test_file_path = repo_path.join("test.txt");
    fs::write(&test_file_path, "initial content")?;
    
    // Add and commit the file
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    GitService::create_commit(&repo, "Initial commit")?;
    println!("Initial commit created");
    
    // Modify the file
    fs::write(&test_file_path, "modified content")?;
    println!("File modified");
    
    // Stage the modification
    GitService::stage_files(&repo, &[test_file_path.clone()])?;
    println!("File staged");
    
    // Check status after staging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after staging:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // Unstage the file
    println!("\nUnstaging file...");
    GitService::unstage_files(&repo, &[test_file_path.clone()])?;
    
    // Check status after unstaging
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after unstaging:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // Now test with a new (untracked) file
    println!("\n\nTesting with new (untracked) file...");
    let new_file_path = repo_path.join("new.txt");
    fs::write(&new_file_path, "new file content")?;
    
    // Stage the new file
    GitService::stage_files(&repo, &[new_file_path.clone()])?;
    println!("New file staged");
    
    // Check status
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after staging new file:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    // Unstage the new file
    println!("\nUnstaging new file...");
    GitService::unstage_files(&repo, &[new_file_path.clone()])?;
    
    // Check status after unstaging new file
    let (unstaged, staged) = GitService::get_status(&repo)?;
    println!("\nStatus after unstaging new file:");
    println!("Unstaged files: {}", unstaged.len());
    for file in &unstaged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    println!("Staged files: {}", staged.len());
    for file in &staged {
        println!("  - {:?}: {:?}", file.path, file.status);
    }
    
    Ok(())
}