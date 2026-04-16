use git2::Repository;
use notify::{Watcher, RecursiveMode};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// Represents the status of a single file in the repository.
#[derive(Serialize, Clone)]
struct FileStatus {
    /// The relative path to the file from the repository root.
    path: String,
    /// A human-readable status string (e.g., "Added", "Modified", "Deleted").
    status: String,
    /// Whether the file is currently staged in the Git index.
    staged: bool,
}

/// Basic metadata about an opened repository.
#[derive(Serialize, Clone)]
struct RepoInfo {
    /// Absolute path to the repository's working directory.
    path: String,
    /// The name of the repository (usually the directory name).
    name: String,
    /// The name of the current branch or "DETACHED HEAD".
    current_branch: String,
    /// The shorthand name of the HEAD reference, if available.
    head_shorthand: Option<String>,
}

/// Global state to manage active file system watchers for each open repository.
#[derive(Default)]
struct WatcherState {
    /// A map of repository paths to their respective `notify` watchers.
    watchers: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

/// A simple greeting command for testing Tauri's IPC.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Returns a list of all changed files in the repository, including untracked ones.
/// 
/// # Arguments
/// * `path` - The path to the repository or any subdirectory within it.
#[tauri::command]
fn get_repo_status(path: String) -> Result<Vec<FileStatus>, String> {
    let repo = Repository::discover(&path).map_err(|e| format!("Failed to find repository: {}", e))?;
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

/// Opens a repository, extracts its metadata, and starts a background file system watcher.
/// 
/// # Arguments
/// * `path` - The path to open.
/// * `app_handle` - Tauri's application handle for emitting events.
/// * `state` - The managed watcher state.
#[tauri::command]
fn open_repository(path: String, app_handle: AppHandle, state: State<'_, WatcherState>) -> Result<RepoInfo, String> {
    let repo = Repository::discover(&path).map_err(|e| format!("Failed to find repository: {}", e))?;
    
    let workdir = repo.workdir()
        .ok_or_else(|| "Repository has no working directory".to_string())?
        .to_path_buf();
    
    let name = workdir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let head = repo.head().ok();
    let head_shorthand = head.as_ref().and_then(|h| h.shorthand().map(|s| s.to_string()));
    
    let info = RepoInfo {
        path: workdir.to_string_lossy().to_string(),
        name,
        current_branch: head_shorthand.clone().unwrap_or_else(|| "DETACHED HEAD".to_string()),
        head_shorthand,
    };

    // Setup watcher for this repository's .git directory to detect background changes
    start_watching_repo(&info.path, app_handle, state)?;

    Ok(info)
}

/// Initializes a file system watcher for a repository's `.git` directory.
/// 
/// This watcher emits a `repo-updated` event to the frontend whenever staging,
/// commits, or branch changes are detected.
fn start_watching_repo(repo_path: &str, app_handle: AppHandle, state: State<'_, WatcherState>) -> Result<(), String> {
    let mut watchers = state.watchers.lock().unwrap();
    if watchers.contains_key(repo_path) {
        return Ok(());
    }

    let git_dir = PathBuf::from(repo_path).join(".git");
    if !git_dir.exists() {
        return Ok(());
    }

    let repo_path_owned = repo_path.to_string();
    let app_handle_clone = app_handle.clone();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            // Check if it's a file modification/creation in relevant git files
            let mut should_refresh = false;
            for path in event.paths {
                if let Some(filename) = path.file_name() {
                    let name = filename.to_string_lossy();
                    // Watching index (staged changes), HEAD (branch/commits), and refs (branch updates)
                    if name == "index" || name == "HEAD" || name.starts_with("refs") {
                        should_refresh = true;
                        break;
                    }
                }
            }

            if should_refresh {
                let _ = app_handle_clone.emit("repo-updated", repo_path_owned.clone());
            }
        }
    }).map_err(|e| e.to_string())?;

    watcher.watch(&git_dir, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
    watchers.insert(repo_path.to_string(), watcher);

    Ok(())
}

/// Stages a list of files by adding them to the Git index.
#[tauri::command]
fn stage_files(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let mut index = repo.index().map_err(|e| format!("Failed to open index: {}", e))?;
    
    for file_path in file_paths {
        index.add_path(std::path::Path::new(&file_path))
            .map_err(|e| format!("Failed to add file {} to index: {}", file_path, e))?;
    }
    
    index.write().map_err(|e| format!("Failed to write index: {}", e))?;
    Ok(())
}

/// Unstages a list of files by resetting them in the Git index to the state of HEAD.
#[tauri::command]
fn unstage_files(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    repo.reset_default(Some(commit.as_object()), &file_paths)
        .map_err(|e| format!("Failed to unstage files: {}", e))?;
        
    Ok(())
}

/// Renames a local branch.
#[tauri::command]
fn rename_branch(repo_path: String, old_name: String, new_name: String) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let mut branch = repo.find_branch(&old_name, git2::BranchType::Local)
        .map_err(|e| format!("Failed to find branch {}: {}", old_name, e))?;
    
    branch.rename(&new_name, false)
        .map_err(|e| format!("Failed to rename branch to {}: {}", new_name, e))?;
        
    Ok(())
}

/// Commits the currently staged changes.
/// 
/// # Arguments
/// * `repo_path` - Path to the repository.
/// * `message` - The commit message.
/// * `amend` - If true, the last commit will be replaced with these changes.
#[tauri::command]
fn commit_changes(repo_path: String, message: String, amend: bool) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let mut index = repo.index().map_err(|e| format!("Failed to open index: {}", e))?;
    let tree_id = index.write_tree().map_err(|e| format!("Failed to write tree: {}", e))?;
    let tree = repo.find_tree(tree_id).map_err(|e| format!("Failed to find tree: {}", e))?;
    
    let signature = repo.signature().map_err(|e| format!("Failed to get default signature: {}", e))?;
    
    if amend {
        let head = repo.head().map_err(|e| format!("Failed to get HEAD for amend: {}", e))?;
        let parent = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
        
        parent.amend(Some("HEAD"), Some(&signature), Some(&signature), None, Some(&message), Some(&tree))
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

        repo.commit(Some("HEAD"), &signature, &signature, &message, &tree, &parents)
            .map_err(|e| format!("Failed to create commit: {}", e))?;
    }
    
    Ok(())
}

/// Returns the commit message of the current HEAD.
#[tauri::command]
fn get_last_commit_message(repo_path: String) -> Result<String, String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    Ok(commit.message().unwrap_or("").to_string())
}

/// Main entry point for the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet, 
            open_repository, 
            get_repo_status,
            stage_files,
            unstage_files,
            rename_branch,
            commit_changes,
            get_last_commit_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
