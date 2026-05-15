//! Git service for Git Gud application
//!
//! This module provides core Git operations using git2-rs.

use crate::models;
use anyhow::{Result, anyhow};
use git2::{BranchType, Commit, DiffOptions, ErrorCode, Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

const MAX_DIFF_LINES: usize = 5_000;

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

            // Conflicted files take priority — show them before any other classification
            if status.contains(Status::CONFLICTED) {
                unstaged_files.push(models::FileChange {
                    path: PathBuf::from(&path),
                    status: models::FileStatus::Conflicted,
                    diff: None,
                    conflict_count: None,
                });
                continue;
            }

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
                diff: None,
                conflict_count: None,
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
        let all_branches = repo.branches(None)?;
        let current_branch = repo
            .head()
            .ok()
            .and_then(|r| r.shorthand().map(|s| s.to_string()));

        for branch_result in all_branches {
            let (branch, branch_type) = branch_result?;
            let name = branch.name()?.unwrap_or("").to_string();
            let is_current = current_branch.as_ref() == Some(&name);

            let commit_id = branch
                .get()
                .peel_to_commit()
                .map(|c| c.id().to_string())
                .unwrap_or_else(|_| "".to_string());

            let (ahead, behind) = if branch_type == BranchType::Local {
                branch.upstream().ok()
                    .and_then(|upstream| {
                        let local_oid = branch.get().peel_to_commit().ok()?.id();
                        let upstream_oid = upstream.get().peel_to_commit().ok()?.id();
                        repo.graph_ahead_behind(local_oid, upstream_oid).ok()
                    })
                    .map(|(a, b)| (a as u32, b as u32))
                    .unwrap_or((0, 0))
            } else {
                (0, 0)
            };

            branches.push(models::Branch {
                name,
                is_remote: branch_type == BranchType::Remote,
                is_current,
                commit_id,
                ahead,
                behind,
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

    /// Get ahead/behind counts for the current branch relative to its upstream.
    /// Returns (ahead, behind), or (0, 0) if no upstream is configured.
    pub fn get_ahead_behind(repo: &Repository) -> (usize, usize) {
        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => return (0, 0),
        };

        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return (0, 0),
        };

        let upstream_oid = match repo
            .branch_upstream_name(head.name().unwrap_or(""))
            .ok()
            .and_then(|name| name.as_str().map(|s| s.to_string()))
            .and_then(|name| repo.refname_to_id(&name).ok())
        {
            Some(oid) => oid,
            None => return (0, 0),
        };

        repo.graph_ahead_behind(local_oid, upstream_oid).unwrap_or((0, 0))
    }

    /// Get list of tags with their target commit IDs
    pub fn get_tags(repo: &Repository) -> Result<Vec<models::Tag>> {
        log::info!("Getting tags");

        let tag_names = repo.tag_names(None)?;
        let mut tags = Vec::new();
        for name in tag_names.iter().flatten() {
            let refname = format!("refs/tags/{}", name);
            if let Ok(reference) = repo.find_reference(&refname)
                && let Ok(commit) = reference.peel_to_commit() {
                    tags.push(models::Tag {
                        name: name.to_string(),
                        commit_id: commit.id().to_string(),
                    });
                }
        }

        log::debug!("Found {} tags", tags.len());
        Ok(tags)
    }

    /// Create an annotated tag at HEAD
    pub fn create_tag(repo: &Repository, name: &str, message: &str) -> Result<()> {
        log::info!("Creating tag: {}", name);

        let head = repo.head()?;
        let target = head.peel_to_commit()?.into_object();
        let sig = repo.signature()?;

        repo.tag(name, &target, &sig, message, false)?;

        log::info!("Tag '{}' created", name);
        Ok(())
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
                    if let Err(e2) = index.update_all([relative_path], None) {
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
        if let Ok(head) = repo.head() {
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

    /// Delete a local branch (equivalent to `git branch -d`; errors if branch is not fully merged)
    pub fn delete_branch(repo: &Repository, name: &str) -> Result<()> {
        let mut branch = repo
            .find_branch(name, BranchType::Local)
            .map_err(|e| anyhow!("Branch '{}' not found: {}", name, e))?;
        branch.delete().map_err(|e| anyhow!("Failed to delete branch '{}': {}", name, e))?;
        log::info!("Branch '{}' deleted", name);
        Ok(())
    }

    /// Rename a local branch
    pub fn rename_branch(repo: &Repository, old_name: &str, new_name: &str) -> Result<()> {
        let mut branch = repo
            .find_branch(old_name, BranchType::Local)
            .map_err(|e| anyhow!("Branch '{}' not found: {}", old_name, e))?;
        branch
            .rename(new_name, false)
            .map_err(|e| anyhow!("Failed to rename branch '{}': {}", old_name, e))?;
        log::info!("Branch '{}' renamed to '{}'", old_name, new_name);
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

    /// Discard working-directory changes for a file.
    /// For tracked files: restores the index version (equivalent to `git checkout -- <path>`).
    /// For untracked files: deletes the file.
    pub fn discard_changes(repo: &Repository, path: &Path) -> Result<()> {
        let workdir = repo.workdir().ok_or_else(|| anyhow!("bare repository"))?;
        let relative = if path.is_absolute() { path.strip_prefix(workdir).unwrap_or(path) } else { path };

        let mut index = repo.index()?;
        if index.get_path(relative, 0).is_some() {
            // Tracked: restore from index
            let mut checkout_opts = git2::build::CheckoutBuilder::new();
            checkout_opts.path(relative).force();
            repo.checkout_index(Some(&mut index), Some(&mut checkout_opts))?;
        } else {
            // Untracked: just delete
            let abs = workdir.join(relative);
            if abs.is_file() {
                std::fs::remove_file(&abs)?;
            } else if abs.is_dir() {
                std::fs::remove_dir_all(&abs)?;
            }
        }
        log::info!("Discarded changes for {:?}", relative);
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
            repo.diff_index_to_workdir(Some(&index), Some(&mut diff_opts))?
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

        // Fallback for untracked (new, unstaged) files: neither diff above covers them.
        // Read the file directly and format as a new-file unified diff.
        if diff_text.is_empty() {
            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                repo_path.join(relative_path)
            };
            if full_path.is_file() {
                match std::fs::read_to_string(&full_path) {
                    Ok(content) => {
                        let path_str = relative_path.to_string_lossy();
                        let line_count = content.lines().count().max(1);
                        diff_text.push_str(&format!(
                            "--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n",
                            path_str, line_count
                        ));
                        for line in content.lines() {
                            diff_text.push('+');
                            diff_text.push_str(line);
                            diff_text.push('\n');
                        }
                    }
                    Err(_) => {
                        diff_text.push_str("(binary file)\n");
                    }
                }
            }
        }

        let lines: Vec<&str> = diff_text.lines().collect();
        let diff_text = if lines.len() > MAX_DIFF_LINES {
            format!("@@ File too large — showing first {} lines @@\n{}", MAX_DIFF_LINES, lines[..MAX_DIFF_LINES].join("\n"))
        } else {
            diff_text
        };

        log::debug!("Generated diff ({} bytes)", diff_text.len());
        Ok(diff_text)
    }

    /// Amend the HEAD commit with new message and current index state
    pub fn amend_commit(repo: &Repository, summary: &str, description: &str) -> Result<()> {
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        let full_message = if description.is_empty() {
            summary.to_string()
        } else {
            format!("{}\n\n{}", summary, description)
        };

        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;

        head_commit.amend(Some("HEAD"), None, None, None, Some(&full_message), Some(&tree))?;
        log::info!("Commit amended successfully");
        Ok(())
    }

    /// Create a new branch, optionally checking it out
    pub fn create_branch(repo: &Repository, name: &str, checkout: bool) -> Result<()> {
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        repo.branch(name, &head_commit, false)
            .map_err(|e| anyhow!("Failed to create branch '{}': {}", name, e))?;

        if checkout {
            let refname = format!("refs/heads/{}", name);
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }

        log::info!("Branch '{}' created", name);
        Ok(())
    }

    /// Save current working tree changes to the stash
    pub fn stash_save(repo: &mut Repository, message: &str) -> Result<()> {
        let sig = repo.signature()?;
        repo.stash_save(&sig, message, None)
            .map_err(|e| anyhow!("Failed to stash: {}", e))?;
        log::info!("Stash saved: {}", message);
        Ok(())
    }

    /// List all stash entries
    pub fn stash_list(repo: &mut Repository) -> Result<Vec<models::StashEntry>> {
        let mut entries = Vec::new();
        repo.stash_foreach(|index, message, _oid| {
            entries.push(models::StashEntry {
                index,
                message: message.to_string(),
            });
            true
        })?;
        Ok(entries)
    }

    /// Apply the stash at `index` and remove it from the stash list
    pub fn stash_pop(repo: &mut Repository, index: usize) -> Result<()> {
        match repo.stash_pop(index, None) {
            Ok(()) => {
                log::info!("Stash {} popped", index);
                Ok(())
            }
            Err(e) => {
                let msg = e.message().to_lowercase();
                if msg.contains("conflict") {
                    // Apply without dropping so the stash is preserved
                    let _ = repo.stash_apply(index, None);
                    Err(anyhow!("Stash applied with conflicts — stash kept at index {}. Resolve conflicts then drop the stash manually.", index))
                } else {
                    Err(anyhow!("Failed to pop stash {}: {}", index, e))
                }
            }
        }
    }

    /// Apply a stash entry without removing it from the stash list
    pub fn stash_apply(repo: &mut Repository, index: usize) -> Result<()> {
        repo.stash_apply(index, None)
            .map_err(|e| anyhow!("Failed to apply stash {}: {}", index, e))?;
        log::info!("Stash {} applied (kept)", index);
        Ok(())
    }

    /// Remove the stash at `index` without applying it
    pub fn stash_drop(repo: &mut Repository, index: usize) -> Result<()> {
        repo.stash_drop(index)
            .map_err(|e| anyhow!("Failed to drop stash {}: {}", index, e))?;
        log::info!("Stash {} dropped", index);
        Ok(())
    }

    /// Pull from a remote using the system git binary (fast-forward only).
    pub fn pull(repo: &Repository, remote_name: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;

        let head = repo.head()?;
        let branch = head.shorthand()
            .ok_or_else(|| anyhow!("No current branch"))?.to_string();

        log::info!("Pulling {} from {}", branch, remote_name);
        crate::services::git_command::run_blocking(
            workdir,
            &["pull", "--ff-only", remote_name, &branch],
        )
        .map_err(|e| anyhow!("{}", e))?;

        Ok(())
    }

    /// Push a branch to a remote using the system git binary.
    /// Automatically adds --set-upstream when the branch has no tracking ref.
    pub fn push(repo: &Repository, remote_name: &str, branch_name: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;

        let has_upstream = repo
            .branch_upstream_name(&format!("refs/heads/{}", branch_name))
            .is_ok();

        log::info!("Pushing {} to {} (set-upstream: {})", branch_name, remote_name, !has_upstream);
        let mut args = vec!["push"];
        if !has_upstream {
            args.push("--set-upstream");
        }
        args.push(remote_name);
        args.push(branch_name);

        crate::services::git_command::run_blocking(workdir,&args)
            .map_err(|e| anyhow!("{}", e))?;

        Ok(())
    }

    /// Fetch from a remote using the system git binary.
    pub fn fetch(repo: &Repository, remote_name: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        log::info!("Fetching from {}", remote_name);
        crate::services::git_command::run_blocking(workdir,&["fetch", remote_name])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Cherry-pick a commit onto the current branch.
    pub fn cherry_pick(repo: &Repository, commit_id: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        log::info!("Cherry-picking {}", commit_id);
        crate::services::git_command::run_blocking(workdir,&["cherry-pick", commit_id])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    pub fn cherry_pick_skip(repo: &Repository) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        crate::services::git_command::run_blocking(workdir,&["cherry-pick", "--skip"])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    pub fn resolve_conflict_ours(repo: &Repository, path: &std::path::Path) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        let path_str = path.to_string_lossy();
        crate::services::git_command::run_blocking(workdir,&["checkout", "--ours", &path_str])
            .map_err(|e| anyhow!("{}", e))?;
        crate::services::git_command::run_blocking(workdir,&["add", &path_str])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    pub fn resolve_conflict_theirs(repo: &Repository, path: &std::path::Path) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        let path_str = path.to_string_lossy();
        crate::services::git_command::run_blocking(workdir,&["checkout", "--theirs", &path_str])
            .map_err(|e| anyhow!("{}", e))?;
        crate::services::git_command::run_blocking(workdir,&["add", &path_str])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Merge a branch into the current branch.
    pub fn merge_branch(repo: &Repository, branch_name: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        log::info!("Merging {}", branch_name);
        crate::services::git_command::run_blocking(workdir,&["merge", branch_name])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Push a tag to a remote using the system git binary.
    pub fn push_tag(repo: &Repository, remote_name: &str, tag_name: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;

        let refspec = format!("refs/tags/{}", tag_name);
        log::info!("Pushing tag {} to {}", tag_name, remote_name);
        crate::services::git_command::run_blocking(
            workdir,
            &["push", remote_name, &refspec],
        )
        .map_err(|e| anyhow!("{}", e))?;

        Ok(())
    }

    /// Cherry-pick a commit onto the current branch without creating a commit.
    pub fn cherry_pick_no_commit(repo: &Repository, commit_id: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        log::info!("Cherry-picking --no-commit {}", commit_id);
        crate::services::git_command::run_blocking(workdir,&["cherry-pick", "--no-commit", commit_id])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// List git worktrees for this repository.
    pub fn list_worktrees(repo: &Repository) -> Result<Vec<crate::models::WorktreeEntry>> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        let output = crate::services::git_command::run_blocking(workdir, &["worktree", "list", "--porcelain"])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(parse_worktrees(&output, workdir))
    }

    /// Add a git worktree at `path` checking out `branch`.
    pub fn add_worktree(repo: &Repository, path: &std::path::Path, branch: &str) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        crate::services::git_command::run_blocking(workdir,&["worktree", "add", &path.to_string_lossy(), branch])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Remove a git worktree at `path`.
    pub fn remove_worktree(repo: &Repository, path: &std::path::Path) -> Result<()> {
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow!("Not a working repository"))?;
        crate::services::git_command::run_blocking(workdir,&["worktree", "remove", &path.to_string_lossy()])
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    /// Get recent commits via RevWalk (topological + time order), all branches.
    pub fn get_commits(repo: &Repository, limit: usize) -> Result<Vec<models::Commit>> {
        let mut walk = repo.revwalk()?;
        walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

        // Push all local branch tips so every branch appears in the graph
        let mut any_pushed = false;
        if let Ok(refs) = repo.references() {
            for r in refs.flatten() {
                if r.is_branch()
                    && let Some(oid) = r.target() {
                        let _ = walk.push(oid);
                        any_pushed = true;
                    }
            }
        }
        if !any_pushed {
            if repo.head().is_ok() {
                walk.push_head()?;
            } else {
                return Ok(Vec::new());
            }
        }

        let mut commits = Vec::with_capacity(limit.min(256));
        for oid_result in walk.take(limit) {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            commits.push(Self::commit_to_model(&commit));
        }
        Ok(commits)
    }

    /// Get commits that touched a specific file (`git log -- <path>`).
    pub fn get_file_history(repo: &Repository, path: &Path, limit: usize) -> Result<Vec<models::Commit>> {
        let mut walk = repo.revwalk()?;
        walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        if repo.head().is_ok() {
            walk.push_head()?;
        } else {
            return Ok(Vec::new());
        }

        let repo_path = repo.workdir().unwrap_or_else(|| Path::new("."));
        let relative = if path.is_absolute() { path.strip_prefix(repo_path).unwrap_or(path) } else { path };

        let mut results = Vec::new();
        let walk_limit = (limit * 20).max(2000);
        for oid_result in walk.take(walk_limit) {
            if results.len() >= limit { break; }
            let oid = match oid_result { Ok(o) => o, Err(_) => continue };
            let commit = repo.find_commit(oid)?;
            if Self::commit_touches_path(repo, &commit, relative) {
                results.push(Self::commit_to_model(&commit));
            }
        }
        Ok(results)
    }

    fn commit_touches_path(repo: &Repository, commit: &git2::Commit<'_>, path: &Path) -> bool {
        let Ok(tree) = commit.tree() else { return false; };
        let mut opts = DiffOptions::new();
        opts.pathspec(path);
        let parent_commit = commit.parents().next();
        let parent_tree = parent_commit.as_ref().and_then(|p| p.tree().ok());
        repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))
            .is_ok_and(|d| d.deltas().len() > 0)
    }

    /// Get the diff of a specific file at a specific commit.
    pub fn get_file_diff_at_commit(repo: &Repository, path: &Path, commit_id: &str) -> Result<String> {
        let oid = git2::Oid::from_str(commit_id)?;
        let commit = repo.find_commit(oid)?;
        let commit_tree = commit.tree()?;

        let repo_path = repo.workdir().unwrap_or_else(|| Path::new("."));
        let relative = if path.is_absolute() { path.strip_prefix(repo_path).unwrap_or(path) } else { path };

        let mut opts = DiffOptions::new();
        opts.pathspec(relative);
        let parent_commit = commit.parents().next();
        let parent_tree = parent_commit.as_ref().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), Some(&mut opts))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            match line.origin() {
                '+' | '-' | ' ' => diff_text.push_str(&format!("{}{}", line.origin(), content)),
                _ => diff_text.push_str(content),
            }
            true
        })?;
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

fn parse_worktrees(output: &str, main_workdir: &std::path::Path) -> Vec<crate::models::WorktreeEntry> {
    let mut entries = Vec::new();
    let mut path: Option<std::path::PathBuf> = None;
    let mut branch: Option<String> = None;

    let flush = |path: &mut Option<std::path::PathBuf>, branch: &mut Option<String>, entries: &mut Vec<crate::models::WorktreeEntry>, main_workdir: &std::path::Path| {
        if let Some(p) = path.take() {
            let is_current = p == main_workdir || p == main_workdir.parent().unwrap_or(main_workdir);
            entries.push(crate::models::WorktreeEntry { path: p, branch: branch.take(), is_current });
        }
    };

    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("worktree ") {
            flush(&mut path, &mut branch, &mut entries, main_workdir);
            path = Some(std::path::PathBuf::from(stripped.trim()));
        } else if let Some(stripped) = line.strip_prefix("branch ") {
            let b = stripped.trim();
            let b = b.strip_prefix("refs/heads/").unwrap_or(b);
            branch = Some(b.to_string());
        }
    }
    flush(&mut path, &mut branch, &mut entries, main_workdir);

    // Mark the first entry (main worktree) as current if none were marked
    if !entries.is_empty() && !entries.iter().any(|e| e.is_current) {
        entries[0].is_current = true;
    }

    entries
}
