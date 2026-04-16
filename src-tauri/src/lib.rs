use git2::Repository;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct RepoInfo {
    path: String,
    name: String,
    current_branch: String,
    head_shorthand: Option<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn open_repository(path: String) -> Result<RepoInfo, String> {
    // Repository::discover searches for a .git folder in the path or its parents
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_repository])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
