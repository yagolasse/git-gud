mod models;
mod git;
mod watcher;
mod commands;

use models::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::greet, 
            commands::open_repository, 
            commands::get_repo_status,
            commands::get_branches,
            commands::get_stashes,
            commands::get_remotes,
            commands::stage_files,
            commands::unstage_files,
            commands::discard_unstaged_changes,
            commands::rename_branch,
            commands::commit_changes,
            commands::get_last_commit_message,
            commands::get_file_diff,
            commands::checkout_branch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
