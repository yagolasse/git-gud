use git2::{Repository, Signature};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Git functionality ===");
    
    // Create a temporary directory for our test repo
    let dir = tempdir()?;
    println!("Created temp directory: {:?}", dir.path());
    
    // Initialize a new git repository
    let repo = Repository::init(dir.path())?;
    println!("Initialized git repository");
    
    // Create a test file
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "Hello, Git!")?;
    println!("Created test file: {:?}", file_path);
    
    // Stage the file
    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.write()?;
    println!("Staged test.txt");
    
    // Get signature for commit
    let signature = Signature::now("Test User", "test@example.com")?;
    
    // Create tree from index
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    
    // Create initial commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )?;
    println!("Created initial commit");
    
    // Modify the file
    writeln!(file, "Modified content")?;
    drop(file); // Close file
    
    // Stage the modification
    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.write()?;
    println!("Staged modification");
    
    // Create tree for new commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    
    // Get parent commit
    let head = repo.head()?;
    let parent_commit = head.peel_to_commit()?;
    
    // Create second commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Second commit",
        &tree,
        &[&parent_commit],
    )?;
    println!("Created second commit");
    
    // Check that we have 2 commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let commit_count = revwalk.count();
    println!("Total commits in repo: {}", commit_count);
    
    assert_eq!(commit_count, 2);
    println!("✓ All tests passed!");
    
    Ok(())
}