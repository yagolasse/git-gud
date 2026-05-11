# Git Gud — Claude Code Guide

## Project Overview

**Git Gud** is a desktop Git GUI built with Rust, using egui/eframe for the UI and git2-rs for Git operations. It follows a clean architecture with separation between models, services, state, and UI.

## Tech Stack

| Crate | Version | Purpose |
|-------|---------|---------|
| `eframe` / `egui` | 0.27.0 | Immediate-mode GUI framework |
| `git2` | 0.20.0 | Git operations |
| `parking_lot` | 0.12 | Mutex for shared state |
| `anyhow` | 1.0 | Error handling |
| `log` / `env_logger` | 0.4 / 0.11 | Logging |
| `rfd` | 0.14 | Native file dialogs |
| `notify` | 6.1 | File system watcher for auto-refresh |
| `syntect` | 5.3 | Syntax highlighting in diff view |
| `tempfile` | 3.10 | Temporary repos in tests |
| `dirs` | 5.0 | Platform directories |

## File Structure

```
src/
├── main.rs                    # Entry point — launches egui app
├── lib.rs                     # Crate root, module declarations
├── models/
│   ├── mod.rs                 # Re-exports all models
│   ├── branch.rs              # Branch model
│   ├── commit.rs              # Commit model
│   ├── diff.rs                # DiffLine, UnifiedDiff, SideBySideDiff, DiffConfig
│   ├── file_status.rs         # FileStatus enum
│   └── repository.rs          # Repository model
├── services/
│   ├── mod.rs
│   ├── git_service.rs         # All git2 operations (status, stage, unstage, commit, checkout)
│   ├── diff_parser.rs         # Parse unified diff text → structured types
│   ├── file_watcher_service.rs# notify-based auto-refresh
│   ├── log_service.rs         # Logging helpers
│   ├── repository_service.rs  # Repository discovery helpers (partially stubbed)
│   └── syntax_service.rs      # syntect integration, LRU cache
├── state/
│   ├── mod.rs
│   ├── app_state.rs           # AppState, AppConfig, LogEntry/LogLevel, dark_mode toggle
│   ├── prefs.rs               # AppPrefs — persists dark_mode + last_repo to disk
│   ├── repository_state.rs    # RepositoryState (staged/unstaged files, branches, commits)
│   └── ui_state.rs            # UIState (selections, commit text, pending actions)
└── ui/
    ├── mod.rs
    ├── colors.rs              # Shared Palette struct — LIGHT and DARK consts, get(dark) fn
    ├── main_window.rs         # MainWindow — layout, menus, panels, dark mode sync
    └── components/
        ├── mod.rs
        ├── branch_list.rs     # Sidebar branch/remote/tag sections
        ├── command_log.rs     # Floating session log window (View menu)
        ├── commit_panel.rs    # Summary + description + Commit button
        ├── enhanced_diff_viewer.rs # Unified + split diff, dark content area
        ├── error_dialog.rs    # Modal error dialog
        ├── file_dialog.rs     # rfd native file dialog wrapper
        ├── file_list.rs       # Staged/Changes sections with stage/unstage actions
        ├── recent_repos.rs    # Persisted recent repository list
        ├── toolbar.rs         # Repo + branch pills, fetch/pull/push, sync counter
        └── virtual_scroll.rs  # Virtual scrolling helper for large diffs
```

## Architecture Patterns

### Shared State
`AppState` lives behind `Arc<parking_lot::Mutex<AppState>>` (`SharedAppState`). In the render loop, always **clone what you need before rendering** to avoid holding the lock during UI calls:
```rust
let entries = { self.state.lock().command_log.clone() };
// Now render with `entries` — lock is released
```

### Color System
All colors live in `src/ui/colors.rs`. Components call `crate::ui::colors::get(state.dark_mode)` at the top of `show()` to get a `&'static Palette`. Never hardcode colors in components; always use the palette.

`dark_mode: bool` lives on `AppState`. Toggle via `state.toggle_dark_mode()`. The View menu exposes "Switch to Dark/Light Mode". `MainWindow::show()` calls `ctx.set_visuals()` each frame to keep egui built-in widgets in sync.

### Geometric Icons
egui's bundled fonts do not cover Dingbats (✓ ⚙), Spacing Modifier Letters (˅), or Small Geometric Shapes (▾ ▸). All icon drawing uses painter primitives instead:
- Chevrons → `egui::Shape::convex_polygon` (filled triangles)
- Checkmarks → `painter.line_segment` (two strokes)
- Gear/settings → `painter.line_segment` (three horizontal lines)

### Pending Actions
UI events that mutate state set `ui_state.pending_action` and return. `AppState::handle_pending_actions()` is called at the start of the next frame. This avoids borrow conflicts in immediate-mode rendering.

### Component Pattern
Each component is a struct with a `show(&mut self, ui: &mut egui::Ui, state: &mut AppState)` method. Internal helpers are free functions that accept `&Palette` and specific data — no stored color state.

## Code Conventions

- **Error handling:** `anyhow::Result<T>` with `.context()` on all fallible operations
- **Logging:** `log::info!` / `log::error!` / `log::debug!` — never `println!` in library code
- **Models:** `#[derive(Debug, Clone)]` on all model types. State types that cross the mutex boundary also need `Clone`.
- **Tests:** Use `tempfile::TempDir` for any test that touches a real Git repo. Each test gets its own temp dir.
- **No comments** unless the *why* is non-obvious. No docstrings on private helpers.
- **Always use `rtk` prefix** for all shell commands (see global CLAUDE.md for RTK token-saving tool).

## Build & Test Commands

```powershell
rtk cargo check          # Fast type-check (no binary)
rtk cargo build          # Full build
rtk cargo test           # Run all tests (87 expected to pass)
rtk cargo clippy         # Lint
```

## Agent Guidelines

### File Safety
- **Red zone** (coordinate before editing): `Cargo.toml`, `src/lib.rs`, `src/main.rs`, any `mod.rs`
- **Yellow zone** (high-contention, touch carefully): `src/state/app_state.rs`, `src/state/repository_state.rs`, `src/ui/colors.rs`, `src/ui/main_window.rs`
- **Green zone** (one agent per file, otherwise free): All component files, all service files, all test files

### Always After Every Change
1. `rtk cargo check` — must pass with zero errors
2. `rtk cargo test` — all 87 tests must still pass
3. Never run the GUI yourself — the user tests the UI

### Interface First
When adding a feature that touches multiple files, define the types/signatures in models/state first, then implement.

## Known TODOs / Unimplemented

| Feature | Location | Notes |
|---------|----------|-------|
| Pull / Push | `toolbar.rs` | Buttons exist, show "not yet implemented" |
| New Branch | `toolbar.rs` | Button exists, not wired |
| Stash | `toolbar.rs` | Button exists, not wired |
| Amend commit | `commit_panel.rs` | Checkbox exists, not implemented |
| Create branch (+ button) | `branch_list.rs` | Shows "not yet implemented" |
| Commit graph | `main_window.rs` History tab | Placeholder only |
| Word-level diff | `enhanced_diff_viewer.rs` | `DiffDisplayMode::WordLevel` falls through to unified |
| Search within diff | `enhanced_diff_viewer.rs` | Not started |
| Settings dialog | toolbar gear icon | Not started |
| Show in File Explorer | File menu | Not started |
| `repository_service.rs` stubs | `repository_service.rs` | `discover_repositories`, `get_repository_info` return empty |
