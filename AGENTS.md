# Git Gud - AI Agent Parallelization Guidelines

## Purpose
This document provides guidelines for AI agents on how to parallelize development tasks for the Git Gud application. The goal is to enable efficient concurrent development while maintaining code consistency and avoiding conflicts.

## Project Overview

**Git Gud** is a desktop Git GUI built with Rust, egui/eframe, and git2-rs. Architecture:
- **`src/models/`** — Data structures (Repository, Commit, Branch, FileStatus, Diff) in `mod.rs` + `diff.rs`
- **`src/services/`** — Business logic (git, file watcher, syntax highlighting, diff parsing, logging)
- **`src/state/`** — Application state management (app_state, repository_state, ui_state)
- **`src/ui/`** — egui components (main_window, components/*)
- **`src/tests/`** — Unit and integration tests

**Current status:** Phase 2.3 complete. 83/83 tests passing. Build compiles with no errors.

## Core Principles

### 0. Always Verify
- **ALWAYS** run `cargo check --all-targets` after every code change
- **ALWAYS** run `cargo test` after every code change (83+ tests must pass)
- **NEVER** run the GUI (`cargo run`) — the user will test the UI themselves
- **FAILING** builds or tests must be fixed immediately, before any other work

### 1. File Safety First
- **NEVER** edit the same file with multiple agents concurrently
- **ALWAYS** check if a file is being modified before editing
- **USE** atomic operations where possible
- **AVOID** overlapping changes to shared dependencies

### 2. Task Decomposition
- Break features into independent, parallelizable units
- Define clear interfaces before implementation
- Create stub functions/methods for integration points
- Document dependencies between tasks

### 3. Coordination Strategy
- Main agent orchestrates task assignment
- Agents report completion and conflicts
- Sequential tasks wait for dependencies
- Merge conflicts resolved by main agent

## File Locking Rules

### Red Zone (No Concurrent Access — Main Agent Only)
- `Cargo.toml` — Dependency management
- `src/lib.rs` — Module declarations
- `src/main.rs` — Entry point
- `src/models/mod.rs` — All model definitions (consolidated)
- `src/services/mod.rs` — Service exports
- `src/state/mod.rs` — State exports
- `src/ui/mod.rs` — UI exports
- `src/ui/components/mod.rs` — Component exports

### Yellow Zone (Coordinated Access)
- `src/state/app_state.rs` — Shared app state, touches all layers
- `src/state/repository_state.rs` — Core repo refresh/staging logic

### Green Zone (Free Access — One Agent Per File)
- `src/services/git_service.rs`
- `src/services/repository_service.rs`
- `src/services/log_service.rs`
- `src/services/file_watcher_service.rs`
- `src/services/syntax_service.rs`
- `src/services/diff_parser.rs`
- `src/state/ui_state.rs`
- `src/models/diff.rs`
- `src/ui/main_window.rs`
- `src/ui/repository_view.rs`
- `src/ui/commit_view.rs`
- `src/ui/components/branch_list.rs`
- `src/ui/components/file_list.rs`
- `src/ui/components/diff_viewer.rs`
- `src/ui/components/enhanced_diff_viewer.rs`
- `src/ui/components/virtual_scroll.rs`
- `src/ui/components/commit_panel.rs`
- `src/ui/components/error_dialog.rs`
- `src/ui/components/file_dialog.rs`
- `src/ui/components/recent_repos.rs`
- `src/tests/*.rs` — All test files

## Current Task Backlog

### Stubs to Implement
| File | Status | Description |
|------|--------|-------------|
| `src/services/repository_service.rs` | STUB | `discover_repositories()`, `get_repository_info()`, `cleanup_temp_repositories()` all return empty/Ok |
| `src/ui/repository_view.rs` | STUB | Only renders "Repository View" label |
| `src/ui/commit_view.rs` | STUB | Only renders "Commit View" label |

### TODOs / Missing Features
| Feature | Location | Notes |
|---------|----------|-------|
| "Edit", "View", "Help" menus | `src/ui/main_window.rs` | Show "coming soon" — wire up actual functionality |
| "Show in File Explorer" | `src/ui/main_window.rs` | Menu item exists, not implemented |
| "Commit History" button | `src/ui/main_window.rs` | Shows "not yet implemented" info |
| Word-level diff highlighting | `src/models/diff.rs`, `src/services/diff_parser.rs` | Planned Phase 2.4 |
| Search within diffs | `src/ui/components/enhanced_diff_viewer.rs` | Not started |
| Config persistence | `src/state/app_state.rs` | User prefs not saved between sessions |
| Advanced Git ops (merge, rebase, stash) | Future | Phase 3.0 |
| Commit graph visualization | Future | Phase 3.0 |

## Parallelizable Task Assignments

### Batch A: Implement Stubs (Can Run 2 Agents in Parallel)
| Agent | Task | File | Dependencies |
|-------|------|------|--------------|
| Agent 1 | Implement `repository_service.rs` | `src/services/repository_service.rs` | `git2` crate, `src/models/mod.rs` models |
| Agent 2 | Implement `repository_view.rs` | `src/ui/repository_view.rs` | Main window integration slot |
| Agent 3 | Implement `commit_view.rs` | `src/ui/commit_view.rs` | `git_service.rs`, main window integration slot |

**Note:** Agents 2 and 3 can work in parallel (different files). Agent 1 is independent of 2 and 3.

### Batch B: Feature Work (Sequential or Parallel Depending on Scope)
| Agent | Task | File(s) | Dependencies |
|-------|------|---------|--------------|
| Agent 1 | Wire up Edit/View/Help menus | `src/ui/main_window.rs` | None |
| Agent 2 | "Show in File Explorer" | `src/ui/main_window.rs` | Must wait for Agent 1 if same file |
| Agent 3 | Word-level diff highlighting | `src/models/diff.rs`, `src/services/diff_parser.rs`, `src/ui/components/enhanced_diff_viewer.rs` | None |
| Agent 4 | Search within diffs | `src/ui/components/enhanced_diff_viewer.rs` | Must wait for Agent 3 if same file |
| Agent 5 | Config persistence | `src/state/app_state.rs` | `serde`, `dirs` |
| Agent 6 | Tests for any new feature | `src/tests/` (new file) | Feature being tested |

## Communication Protocol

### Agent Status Updates
```
[AGENT_ID] [TASK] [STATUS] [BLOCKERS]
```

### Task Completion Checklist
- [ ] `cargo check --all-targets` passes
- [ ] `cargo test` passes (all 37 existing + new tests)
- [ ] `cargo clippy --all-targets` produces no new warnings
- [ ] Follows existing code conventions (anyhow::Result, log crate, dark theme visuals)
- [ ] No secrets or credentials committed

## Code Conventions

- **Error handling:** `anyhow::Result<T>` with `.context()` for all fallible operations
- **Logging:** `log::info!()` / `log::error!()` / `log::debug!()` — use appropriate levels
- **State sharing:** `Arc<parking_lot::Mutex<AppState>>` for thread-safe shared state
- **UI pattern:** Components take `&mut egui::Ui`, state passed via function arguments, not stored in components
- **Models:** All in `src/models/mod.rs` — add new models here, use `#[derive(Debug, Clone, Serialize, Deserialize)]`
- **Testing:** Use `tempfile::TempDir` for git repo tests, clean up after each test
- **Dark mode:** Default egui visuals are dark — follow existing color patterns

## Build & Test Commands

```powershell
# Check compilation
cargo check --all-targets

# Run all tests
cargo test

# Run with logging
$env:RUST_LOG="info"; cargo run

# Lint
cargo clippy --all-targets

# Format
cargo fmt
```

## Conflict Resolution Protocol

1. **Prevention:** Clear task boundaries, defined interfaces first, atomic file operations
2. **Detection:** Git status before edits, build verification after each change
3. **Resolution:**
   - Minor conflicts: Main agent merges
   - Interface conflicts: Redefine interface, notify dependent agents
   - Dependency conflicts: Update Cargo.toml, regenerate lock file
   - Build failures: Fix immediately, don't proceed with broken code

## Security

- Don't commit secrets or credentials
- Use temporary test repositories (`tempfile`)
- Validate all user input (paths, branch names, commit messages)
- Log securely (no sensitive file contents)
- Clean up temporary files
