use git2::Repository;
use std::path::Path;
use crate::models::{FileStatus, RepoInfo};

pub fn get_repository(path: &str) -> Result<Repository, String> {
    Repository::discover(path).map_err(|e| format!("Failed to find repository: {}", e))
}

pub fn get_repo_status(repo: &Repository) -> Result<Vec<FileStatus>, String> {
    let mut statuses = Vec::new();

    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(true);
    status_options.recurse_untracked_dirs(true);

    let repo_statuses = repo.statuses(Some(&mut status_options))
        .map_err(|e| format!("Failed to get statuses: {}", e))?;

    for entry in repo_statuses.iter() {
        let path = entry.path().unwrap_or("unknown").to_string();
        let status_bits = entry.status();

        let status_str = if status_bits.is_index_new() || status_bits.is_wt_new() {
            "Added"
        } else if status_bits.is_index_modified() || status_bits.is_wt_modified() {
            "Modified"
        } else if status_bits.is_index_deleted() || status_bits.is_wt_deleted() {
            "Deleted"
        } else if status_bits.is_index_renamed() || status_bits.is_wt_renamed() {
            "Renamed"
        } else {
            "Other"
        };

        let is_staged = status_bits.is_index_new() || 
                        status_bits.is_index_modified() || 
                        status_bits.is_index_deleted() || 
                        status_bits.is_index_renamed() || 
                        status_bits.is_index_typechange();

        statuses.push(FileStatus {
            path,
            status: status_str.to_string(),
            staged: is_staged,
        });
    }

    Ok(statuses)
}

pub fn get_repo_info(repo: &Repository) -> Result<RepoInfo, String> {
    let workdir = repo.workdir()
        .ok_or_else(|| "Repository has no working directory".to_string())?
        .to_path_buf();
    
    let name = workdir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let head = repo.head().ok();
    let head_shorthand = head.as_ref().and_then(|h| h.shorthand().map(|s| s.to_string()));
    
    Ok(RepoInfo {
        path: workdir.to_string_lossy().to_string(),
        name,
        current_branch: head_shorthand.clone().unwrap_or_else(|| "DETACHED HEAD".to_string()),
        head_shorthand,
    })
}

pub fn stage_files(repo: &Repository, file_paths: Vec<String>) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Failed to open index: {}", e))?;
    
    for file_path in file_paths {
        index.add_path(Path::new(&file_path))
            .map_err(|e| format!("Failed to add file {} to index: {}", file_path, e))?;
    }
    
    index.write().map_err(|e| format!("Failed to write index: {}", e))?;
    Ok(())
}

pub fn unstage_files(repo: &Repository, file_paths: Vec<String>) -> Result<(), String> {
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    repo.reset_default(Some(commit.as_object()), &file_paths)
        .map_err(|e| format!("Failed to unstage files: {}", e))?;
        
    Ok(())
}

pub fn rename_branch(repo: &Repository, old_name: &str, new_name: &str) -> Result<(), String> {
    let mut branch = repo.find_branch(old_name, git2::BranchType::Local)
        .map_err(|e| format!("Failed to find branch {}: {}", old_name, e))?;
    
    branch.rename(new_name, false)
        .map_err(|e| format!("Failed to rename branch to {}: {}", new_name, e))?;
        
    Ok(())
}

pub fn commit_changes(repo: &Repository, message: &str, amend: bool) -> Result<(), String> {
    let mut index = repo.index().map_err(|e| format!("Failed to open index: {}", e))?;
    let tree_id = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
    let tree = repo.find_tree(tree_id).map_err(|e| format!("Failed to find tree: {}", e))?;
    
    let signature = repo.signature().map_err(|e| format!("Failed to get default signature: {}", e))?;
    
    if amend {
        let head = repo.head().map_err(|e| format!("Failed to get HEAD for amend: {}", e))?;
        let parent = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
        
        parent.amend(Some("HEAD"), Some(&signature), Some(&signature), None, Some(message), Some(&tree))
            .map_err(|e| format!("Failed to amend commit: {}", e))?;
    } else {
        let parent_commit = match repo.head() {
            Ok(head) => Some(head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD: {}", e))?),
            Err(_) => None, // Initial commit
        };

        let parents = match &parent_commit {
            Some(c) => vec![c],
            None => vec![],
        };

        repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &parents)
            .map_err(|e| format!("Failed to create commit: {}", e))?;
    }
    
    Ok(())
}

pub fn get_last_commit_message(repo: &Repository) -> Result<String, String> {
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    Ok(commit.message().unwrap_or("").to_string())
}

pub fn discard_unstaged_changes(repo: &Repository, file_paths: Vec<String>) -> Result<(), String> {
    let mut checkout_opts = git2::build::CheckoutBuilder::new();
    checkout_opts.force();
    
    for path in &file_paths {
        checkout_opts.path(path);
    }

    repo.checkout_index(None, Some(&mut checkout_opts))
        .map_err(|e| format!("Failed to discard changes for files: {}", e))?;
        
    Ok(())
}

pub fn get_file_diff(repo: &Repository, file_path: &str, staged: bool) -> Result<String, String> {
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    opts.context_lines(3);
    opts.interhunk_lines(1);

    let diff = if staged {
        let head = repo.head().ok();
        let tree = match head {
            Some(h) => Some(h.peel_to_tree().map_err(|e| format!("Failed to peel HEAD to tree: {}", e))?),
            None => None,
        };
        repo.diff_tree_to_index(tree.as_ref(), None, Some(&mut opts))
            .map_err(|e| format!("Failed to get staged diff: {}", e))?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| format!("Failed to get unstaged diff: {}", e))?
    };

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        match origin {
            '+' | '-' | ' ' => {
                diff_text.push(origin);
                diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            }
            'H' => {
                diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            }
            _ => {}
        }
        true
    }).map_err(|e| format!("Failed to format diff: {}", e))?;

    Ok(diff_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_repo_status_empty() {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let statuses = get_repo_status(&repo).unwrap();
        assert!(statuses.is_empty());
    }

    #[test]
    fn test_repo_status_with_untracked() {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello Git").unwrap();

        let statuses = get_repo_status(&repo).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].path, "test.txt");
        assert_eq!(statuses[0].status, "Added");
        assert_eq!(statuses[0].staged, false);
    }
}
