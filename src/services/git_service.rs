//! Git service for Git Gud application
//!
//! This module provides core Git operations using git2-rs.

use crate::models;
use anyhow::{Result, anyhow};
use git2::{BranchType, Commit, DiffOptions, ErrorCode, Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

/// Git service for performing Git operations
pub struct GitService;

impl GitService {
    /// Initialize a new Git repository
    pub fn init_repository(path: &Path) -> Result<Repository> {
        log::info!("Initializing Git repository at: {:?}", path);
        let repo = Repository::init(path)?;
        log::info!("Repository initialized successfully");
        Ok(repo)
    }

    /// Open an existing Git repository
    pub fn open_repository(path: &Path) -> Result<Repository> {
        log::info!("Opening Git repository at: {:?}", path);
        let repo = Repository::open(path)?;
        log::info!("Repository opened successfully");
        Ok(repo)
    }

    /// Check if a path contains a Git repository
    pub fn is_repository(path: &Path) -> bool {
        Repository::open(path).is_ok()
    }

    /// Get repository status (unstaged and staged files)
    pub fn get_status(
        repo: &Repository,
    ) -> Result<(Vec<models::FileChange>, Vec<models::FileChange>)> {
        log::info!("Getting repository status");

        let mut unstaged_files = Vec::new();
        let mut staged_files = Vec::new();

        // Get status entries
        let statuses = repo.statuses(Some(
            StatusOptions::new()
                .include_untracked(true)
                .renames_from_rewrites(true)
                .renames_head_to_index(true)
                .recurse_untracked_dirs(true),
        ))?;

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("").to_string();

            // Determine if file is in index (staged) or working tree (unstaged)
            let is_staged = status.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            );

            let is_unstaged = status.intersects(
                Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            );

            // Convert git2 status to our FileStatus
            let file_status = if status.contains(Status::WT_NEW)
                || status.contains(Status::INDEX_NEW)
            {
                if is_staged {
                    models::FileStatus::Added
                } else {
                    models::FileStatus::Untracked
                }
            } else if status.contains(Status::WT_MODIFIED)
                || status.contains(Status::INDEX_MODIFIED)
            {
                models::FileStatus::Modified
            } else if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED)
            {
                models::FileStatus::Deleted
            } else if status.contains(Status::WT_RENAMED) || status.contains(Status::INDEX_RENAMED)
            {
                models::FileStatus::Renamed
            } else if status.contains(Status::WT_TYPECHANGE)
                || status.contains(Status::INDEX_TYPECHANGE)
            {
                models::FileStatus::Modified
            } else if status.contains(Status::IGNORED) {
                models::FileStatus::Ignored
            } else {
                models::FileStatus::Unmodified
            };

            let file_change = models::FileChange {
                path: PathBuf::from(path),
                status: file_status,
                diff: None, // Will be loaded lazily when needed
            };

            // Add to appropriate list
            if is_staged {
                staged_files.push(file_change.clone());
            }
            if is_unstaged {
                unstaged_files.push(file_change);
            }
        }

        log::debug!(
            "Found {} unstaged files, {} staged files",
            unstaged_files.len(),
            staged_files.len()
        );

        Ok((unstaged_files, staged_files))
    }

    /// Get list of branches
    pub fn get_branches(repo: &Repository) -> Result<Vec<models::Branch>> {
        log::info!("Getting branches");

        let mut branches = Vec::new();
        let local_branches = repo.branches(Some(BranchType::Local))?;
        let current_branch = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()));

        for branch_result in local_branches {
            let (branch, branch_type) = branch_result?;
            let name = branch.name()?.unwrap_or("").to_string();
            let is_current = current_branch.as_ref().map_or(false, |cb| cb == &name);

            let commit_id = branch
                .get()
                .peel_to_commit()
                .map(|c| c.id().to_string())
                .unwrap_or_else(|_| "".to_string());

            branches.push(models::Branch {
                name,
                is_remote: branch_type == BranchType::Remote,
                is_current,
                commit_id,
            });
        }

        // Sort: current branch first, then alphabetically
        branches.sort_by(|a, b| {
            if a.is_current && !b.is_current {
                std::cmp::Ordering::Less
            } else if !a.is_current && b.is_current {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        log::debug!("Found {} branches", branches.len());
        Ok(branches)
    }

    /// Get HEAD commit
    pub fn get_head_commit(repo: &Repository) -> Result<models::Commit> {
        log::info!("Getting HEAD commit");

        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        Ok(Self::commit_to_model(&commit))
    }

    /// Stage files
    pub fn stage_files(repo: &Repository, paths: &[PathBuf]) -> Result<()> {
        log::info!("Staging {} files: {:?}", paths.len(), paths);

        let mut index = repo.index()?;
        let repo_path = repo
            .workdir()
            .unwrap_or_else(|| repo.path().parent().unwrap_or(Path::new(".")));

        for path in paths {
            // Try to convert absolute path to relative path within repository
            let relative_path = if path.is_absolute() {
                if let Ok(rel) = path.strip_prefix(repo_path) {
                    rel
                } else {
                    // If we can't strip the prefix, use the file name as a fallback
                    path.file_name()
                        .map(Path::new)
                        .unwrap_or_else(|| path.as_path())
                }
            } else {
                path.as_path()
            };

            let path_str = relative_path.to_string_lossy();
            if let Err(e) = index.add_path(relative_path) {
                // If adding fails, try to update (for modified files)
                if e.code() == ErrorCode::NotFound {
                    if let Err(e2) = index.update_all(&[relative_path], None) {
                        return Err(anyhow!("Failed to stage file {}: {}", path_str, e2));
                    }
                } else {
                    return Err(anyhow!("Failed to stage file {}: {}", path_str, e));
                }
            }
        }

        index.write()?;
        log::info!("Files staged successfully");
        Ok(())
    }

    /// Unstage files
    pub fn unstage_files(repo: &Repository, paths: &[PathBuf]) -> Result<()> {
        log::info!("Unstaging {} files: {:?}", paths.len(), paths);

        let repo_path = repo
            .workdir()
            .unwrap_or_else(|| repo.path().parent().unwrap_or(Path::new(".")));

        // Convert paths to string pathspecs
        let mut pathspecs = Vec::new();
        for path in paths {
            // Try to convert absolute path to relative path within repository
            let relative_path = if path.is_absolute() {
                if let Ok(rel) = path.strip_prefix(repo_path) {
                    rel
                } else {
                    // If we can't strip the prefix, use the file name as a fallback
                    path.file_name()
                        .map(Path::new)
                        .unwrap_or_else(|| path.as_path())
                }
            } else {
                path.as_path()
            };

            pathspecs.push(relative_path.to_string_lossy().to_string());
        }

        // Use reset_default to unstage files (equivalent to git reset HEAD -- <file>)
        // This handles all cases: modified, added, deleted files
        if let Some(head) = repo.head().ok() {
            let target = head.peel_to_commit()?.into_object();

            // Build array of pathspecs
            let pathspec_array: Vec<&str> = pathspecs.iter().map(|s| s.as_str()).collect();

            // Reset the specified paths from HEAD
            repo.reset_default(Some(&target), &pathspec_array)?;
            log::info!("Files unstaged successfully using reset_default");
        } else {
            // No HEAD (empty repository), just remove new files from index
            let mut index = repo.index()?;
            for pathspec in &pathspecs {
                index.remove_path(Path::new(pathspec))?;
            }
            index.write()?;
            log::info!("Files unstaged successfully (empty repo)");
        }

        Ok(())
    }

    /// Create a commit
    pub fn create_commit(repo: &Repository, message: &str) -> Result<()> {
        log::info!("Creating commit: {}", message);

        // Get signature (author/committer)
        let sig = repo.signature()?;

        // Get index and write tree
        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;

        // Get parent commit (HEAD)
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents = parent.as_ref().map_or(vec![], |p| vec![p]);

        // Create commit
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

        log::info!("Commit created successfully");
        Ok(())
    }

    /// Checkout a branch
    pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<()> {
        log::info!("Checking out branch: {}", branch_name);

        let (object, reference) = repo.revparse_ext(branch_name)?;
        repo.checkout_tree(&object, None)?;

        // Move HEAD to the branch
        match reference {
            Some(gref) => repo.set_head(gref.name().unwrap_or("")),
            None => repo.set_head_detached(object.id()),
        }?;

        log::info!("Checked out branch successfully");
        Ok(())
    }

    /// Get diff for a file
    pub fn get_file_diff(repo: &Repository, path: &Path) -> Result<String> {
        log::info!("Getting diff for file: {:?}", path);

        let repo_path = repo
            .workdir()
            .unwrap_or_else(|| repo.path().parent().unwrap_or(Path::new(".")));
        let relative_path = if path.is_absolute() {
            if let Ok(rel) = path.strip_prefix(repo_path) {
                rel
            } else {
                // If we can't strip the prefix, use the file name as a fallback
                path.file_name().map(Path::new).unwrap_or_else(|| path)
            }
        } else {
            path
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(relative_path);

        // Diff between HEAD and index (staged changes)
        let head = repo.head().ok();
        let head_tree = head.and_then(|h| h.peel_to_tree().ok());
        let index = repo.index()?;
        let diff =
            repo.diff_tree_to_index(head_tree.as_ref(), Some(&index), Some(&mut diff_opts))?;

        // If no staged changes, diff between index and working directory
        let diff = if diff.deltas().len() == 0 {
            let diff = repo.diff_index_to_workdir(Some(&index), Some(&mut diff_opts))?;
            diff
        } else {
            diff
        };

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            match line.origin() {
                '+' | '-' | ' ' => diff_text.push_str(&format!("{}{}", line.origin(), content)),
                _ => {}
            }
            true
        })?;

        log::debug!("Generated diff ({} bytes)", diff_text.len());
        Ok(diff_text)
    }

    /// Convert git2::Commit to models::Commit
    fn commit_to_model(commit: &Commit) -> models::Commit {
        models::Commit {
            id: commit.id().to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            email: commit.author().email().unwrap_or("").to_string(),
            message: commit.message().unwrap_or("").to_string(),
            timestamp: commit.time().seconds(),
            parents: commit.parents().map(|p| p.id().to_string()).collect(),
        }
    }
}
