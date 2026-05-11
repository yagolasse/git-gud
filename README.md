# Git Gud

A native desktop Git GUI built with Rust. Focuses on the everyday staging-and-committing workflow with a clean, fast interface.

## Features

- **Three-panel layout** — branches on the left, staged/unstaged files in the center, diff on the right
- **Stage / unstage** individual files or everything at once, with context-menu support
- **Commit panel** — summary + description fields, character-count indicator, amend checkbox
- **Diff viewer** — unified and side-by-side modes, syntax highlighting via syntect
- **Branch list** — local branches, remotes, checkout on double-click, filterable
- **Auto-refresh** — file watcher detects external changes and refreshes the working tree automatically
- **Session command log** — floating window (View → Show Command Log) capturing every operation with timestamp and result
- **Light / dark mode** — toggle from the View menu, persisted across restarts
- **Recent repositories** — reopens the last-used repository on startup

## Requirements

- [Rust](https://rustup.rs) stable (1.75+)
- Git installed and on `PATH`

## Build

```powershell
cargo build --release
```

The binary is written to `target/release/git-gud.exe` (Windows) or `target/release/git-gud` (macOS/Linux).

## Run

```powershell
# Open with the file-chooser dialog
cargo run

# Open a specific repository directly
cargo run -- /path/to/repo
```

## Development

```powershell
cargo check       # fast type-check
cargo test        # 87 tests
cargo clippy      # lint
```

All tests use temporary Git repositories via `tempfile`; no side effects on the real filesystem.

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` 0.27 | Immediate-mode GUI |
| `git2` 0.20 | Git operations |
| `syntect` 5.3 | Syntax highlighting |
| `notify` 6.1 | File system watcher |
| `parking_lot` 0.12 | Mutex |
| `rfd` 0.14 | Native file dialogs |
| `dirs` 5.0 | Platform config directory |
| `anyhow` 1.0 | Error handling |

## License

TBD
