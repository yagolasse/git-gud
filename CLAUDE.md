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
| `uuid` | 1.23 | UUID generation for IPC tokens |
| `dirs` | 5.0 | Platform directories |

## File Structure

```
src/
├── main.rs                    # Entry point — launches egui app, askpass subcommand
├── lib.rs                     # Crate root, module declarations
├── models/
│   ├── mod.rs                 # Re-exports all models
│   ├── diff.rs                # DiffLine, UnifiedDiff, SideBySideDiff, DiffConfig
│   └── ssh_config.rs          # SshConfig + SshHost — parses ~/.ssh/config
├── services/
│   ├── mod.rs
│   ├── askpass.rs             # TCP-loopback IPC bridge for GIT_ASKPASS
│   ├── git_command.rs         # Git binary abstraction (config, env, error classification)
│   ├── git_service.rs         # All git2 operations (status, stage, unstage, commit, checkout)
│   ├── diff_parser.rs         # Parse unified diff text → structured types
│   ├── file_watcher_service.rs# notify-based auto-refresh
│   ├── log_service.rs         # Logging helpers
│   ├── repository_service.rs  # Repository discovery helpers (partially stubbed)
│   └── syntax_service.rs      # syntect integration, LRU cache
├── state/
│   ├── mod.rs
│   ├── app_state.rs           # AppState, AppConfig, NetworkStatus, LogEntry/LogLevel
│   ├── prefs.rs               # AppPrefs — persists dark_mode + last_repo to disk
│   ├── repository_state.rs    # RepositoryState (staged/unstaged files, branches, commits, tags)
│   └── ui_state.rs            # UIState (selections, commit text, pending actions, passphrase state)
└── ui/
    ├── mod.rs
    ├── colors.rs              # Shared Palette struct — LIGHT and DARK consts, get(dark) fn
    ├── main_window.rs         # MainWindow — layout, menus, panels, dark mode sync, passphrase poll
    └── components/
        ├── mod.rs
        ├── branch_list.rs     # Sidebar branch/remote/tag sections
        ├── command_log.rs     # Floating session log window (View menu)
        ├── commit_graph.rs    # Lane-based commit DAG
        ├── commit_panel.rs    # Summary + description + Commit button
        ├── enhanced_diff_viewer.rs # Unified + split diff, dark content area
        ├── error_dialog.rs    # Modal error dialog
        ├── file_dialog.rs     # rfd native file dialog wrapper
        ├── file_list.rs       # Staged/Changes sections with stage/unstage actions
        ├── passphrase_dialog.rs # Modal passphrase input for SSH key auth
        ├── recent_repos.rs    # Persisted recent repository list
        ├── toolbar.rs         # Repo + branch pills, fetch/pull/push, network indicator
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

### Pending Actions
UI events that mutate state set `ui_state.pending_action` and return. `AppState::handle_pending_actions()` is called at the start of the next frame. This avoids borrow conflicts in immediate-mode rendering.

### Network Operations (Pull / Push)

All remote git commands route through `git_command.rs`:
- `git_command::run_blocking()` — synchronous, used during the transition
- `git_command::run_async()` — tokio-based for future async integration
- `git_command::run_streaming()` — line-by-line streaming via mpsc channels

`GitConfig` (set via `init_config()` at startup, default-lazy otherwise) configures:
- `binary: PathBuf` — path to the git binary (default `"git"`, settable for vendored git)
- `askpass: Option<PathBuf>` — path to the `GIT_ASKPASS` helper binary
- `askpass_port: Option<u16>` — TCP port for askpass IPC

Error classification in `GitCommandError` distinguishes:
- `Auth` — permission denied, host key verification failed
- `Network` — DNS failure, connection refused
- `RemoteRejected` — non-fast-forward push rejections
- `Transport` / `Exit` / `Spawn` — other failures

### SSH / Credential Auth (Askpass IPC)

When system git needs an SSH passphrase or HTTPS credential, it invokes the `GIT_ASKPASS` program. The IPC flow:

```
system git → git-gud askpass "prompt" → TCP 127.0.0.1:<port> → GUI dialog → passphrase → stdout
```

1. **GUI startup** (`main.rs:run_gui_with_path`): spawns TCP listener on `127.0.0.1:0`, stores port, sets `GIT_ASKPASS=<current-exe>` and `GIT_GUD_ASKPASS_PORT=<port>` in `GitConfig`
2. **Askpass subcommand** (`main.rs`): when `git-gud askpass <prompt>` is invoked, connects to TCP, sends prompt, reads response, prints to stdout
3. **Server** (`askpass.rs:start_server`): accepts connections in a background thread, stores `PassphraseRequest`, polls `response` field
4. **GUI poll** (`main_window.rs`): `PassphraseDialog::poll_and_show()` checks `AskpassState` for pending requests each frame, shows modal password dialog
5. **User submits**: passphrase written to `AskpassRequests.response`, server reads it, sends back to client

Cross-platform: TCP loopback (`127.0.0.1`) is identical on Windows, macOS, and Linux — no per-platform API surface.

### SSH Config Parser

`models/ssh_config.rs` parses `~/.ssh/config` at startup (no external crates):
- `SshConfig::load()` — reads the file, parses `Host`, `HostName`, `Port`, `User`, `IdentityFile`
- `SshConfig::find_host(alias)` — matches a hostname against wildcard patterns
- Stored in `AppState.ssh_config`, available for UI display or identity file validation

### Network Status

`AppState.network_status: NetworkStatus` tracks ongoing remote operations:
- `Idle` — no operation running
- `Running { operation, lines, progress, lines_rx, is_pull }` — pull/push runs in a background thread; toolbar shows an inline circular spinner next to the active button; `poll_network()` drains the mpsc channel each frame

`NetworkStatus` manually implements `Clone` (returns `Idle`) because `Receiver<StreamLine>` is not `Clone`.

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
rtk cargo test           # Run all tests (101 expected to pass)
rtk cargo clippy         # Lint
```

## Agent Guidelines

### File Safety
- **Red zone** (coordinate before editing): `Cargo.toml`, `src/lib.rs`, `src/main.rs`, any `mod.rs`
- **Yellow zone** (high-contention, touch carefully): `src/state/app_state.rs`, `src/state/repository_state.rs`, `src/ui/colors.rs`, `src/ui/main_window.rs`
- **Green zone** (one agent per file, otherwise free): All component files, all service files, all test files

### Always After Every Change
1. `rtk cargo check` — must pass with zero errors
2. `rtk cargo test` — all 101 tests must still pass
3. Never run the GUI yourself — the user tests the UI

### Interface First
When adding a feature that touches multiple files, define the types/signatures in models/state first, then implement.

## Known TODOs / Unimplemented

| Feature | Location | Notes |
|---------|----------|-------|
| Amend commit | `commit_panel.rs` | Checkbox exists, not implemented |
| Word-level diff | `enhanced_diff_viewer.rs` | `DiffDisplayMode::WordLevel` falls through to unified |
| Search within diff | `enhanced_diff_viewer.rs` | Not started |
| Show in File Explorer | File menu | Not started |
| `repository_service.rs` stubs | `repository_service.rs` | `discover_repositories`, `get_repository_info` return empty |
| Push to upstream-less branch | `git_service.rs` `push()` | Needs `--set-upstream` / `-u` flag when no tracking branch exists |
| Fetch (without merge) | `toolbar.rs` | Button calls `refresh_repository()`; should run `git fetch origin` |
| Add remote | `toolbar.rs` | No UI yet to add/configure remotes |
