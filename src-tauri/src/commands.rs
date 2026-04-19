use tauri::{AppHandle, State};
use crate::models::{FileStatus, RepoInfo, WatcherState};
use crate::git;
use crate::watcher;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_repo_status(path: String) -> Result<Vec<FileStatus>, String> {
    let repo = git::get_repository(&path)?;
    git::get_repo_status(&repo)
}

#[tauri::command]
pub fn get_branches(repo_path: String) -> Result<Vec<crate::models::BranchInfo>, String> {
    let repo = git::get_repository(&repo_path)?;
    git::get_branches(&repo)
}

#[tauri::command]
pub fn get_stashes(repo_path: String) -> Result<Vec<crate::models::StashInfo>, String> {
    let mut repo = git::get_repository(&repo_path)?;
    git::get_stashes(&mut repo)
}

#[tauri::command]
pub fn get_remotes(repo_path: String) -> Result<Vec<crate::models::RemoteInfo>, String> {
    let repo = git::get_repository(&repo_path)?;
    git::get_remotes(&repo)
}

#[tauri::command]
pub fn open_repository(path: String, app_handle: AppHandle, state: State<'_, WatcherState>) -> Result<RepoInfo, String> {
    let repo = git::get_repository(&path)?;
    let info = git::get_repo_info(&repo)?;

    watcher::start_watching_repo(&info.path, app_handle, state)?;

    Ok(info)
}

#[tauri::command]
pub fn stage_files(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::stage_files(&repo, file_paths)
}

#[tauri::command]
pub fn unstage_files(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::unstage_files(&repo, file_paths)
}

#[tauri::command]
pub fn rename_branch(repo_path: String, old_name: String, new_name: String) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::rename_branch(&repo, &old_name, &new_name)
}

#[tauri::command]
pub fn commit_changes(repo_path: String, message: String, amend: bool) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::commit_changes(&repo, &message, amend)
}

#[tauri::command]
pub fn get_last_commit_message(repo_path: String) -> Result<String, String> {
    let repo = git::get_repository(&repo_path)?;
    git::get_last_commit_message(&repo)
}

#[tauri::command]
pub fn discard_unstaged_changes(repo_path: String, file_paths: Vec<String>) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::discard_unstaged_changes(&repo, file_paths)
}

#[tauri::command]
pub fn get_file_diff(repo_path: String, file_path: String, staged: bool) -> Result<String, String> {
    let repo = git::get_repository(&repo_path)?;
    git::get_file_diff(&repo, &file_path, staged)
}

#[tauri::command]
pub fn checkout_branch(repo_path: String, branch_name: String, is_remote: bool) -> Result<(), String> {
    let repo = git::get_repository(&repo_path)?;
    git::checkout_branch(&repo, &branch_name, is_remote)
}
