use notify::{Watcher, RecursiveMode};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use crate::models::WatcherState;

pub fn start_watching_repo(repo_path: &str, app_handle: AppHandle, state: State<'_, WatcherState>) -> Result<(), String> {
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
