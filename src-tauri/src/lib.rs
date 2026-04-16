use git2::Repository;
use serde::Serialize;

#[derive(Serialize)]
struct FileStatus {
    path: String,
    status: String,
    staged: bool,
}

#[derive(Serialize)]
struct RepoInfo {
    path: String,
    name: String,
    current_branch: String,
    head_shorthand: Option<String>,
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
fn open_repository(path: String) -> Result<RepoInfo, String> {
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
    
    Ok(RepoInfo {
        path: workdir.to_string_lossy().to_string(),
        name,
        current_branch: head_shorthand.clone().unwrap_or_else(|| "DETACHED HEAD".to_string()),
        head_shorthand,
    })
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            open_repository, 
            get_repo_status,
            stage_files,
            unstage_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
