use git2::Repository;
use notify::{Watcher, RecursiveMode, Config, EventKind};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
struct FileStatus {
    path: String,
    status: String,
    staged: bool,
}

#[derive(Serialize, Clone)]
struct RepoInfo {
    path: String,
    name: String,
    current_branch: String,
    head_shorthand: Option<String>,
}

#[derive(Default)]
struct WatcherState {
    watchers: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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

    // Setup watcher for this repository's .git directory
    start_watching_repo(&info.path, app_handle, state)?;

    Ok(info)
}

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

#[tauri::command]
fn unstage_files(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    repo.reset_default(Some(commit.as_object()), &file_paths)
        .map_err(|e| format!("Failed to unstage files: {}", e))?;
        
    Ok(())
}

#[tauri::command]
fn rename_branch(repo_path: String, old_name: String, new_name: String) -> Result<(), String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let mut branch = repo.find_branch(&old_name, git2::BranchType::Local)
        .map_err(|e| format!("Failed to find branch {}: {}", old_name, e))?;
    
    branch.rename(&new_name, false)
        .map_err(|e| format!("Failed to rename branch to {}: {}", new_name, e))?;
        
    Ok(())
}

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

#[tauri::command]
fn get_last_commit_message(repo_path: String) -> Result<String, String> {
    let repo = Repository::discover(&repo_path).map_err(|e| format!("Failed to find repository: {}", e))?;
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel HEAD to commit: {}", e))?;
    
    Ok(commit.message().unwrap_or("").to_string())
}

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
