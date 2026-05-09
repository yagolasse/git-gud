---
project_name: 'git-gud'
user_name: 'Yago Lasse'
date: '2026-05-05'
sections_completed: ['technology_stack', 'language_rules', 'framework_rules', 'testing_rules', 'code_quality', 'workflow', 'dont_miss']
status: 'complete'
rule_count: 42
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

| Technology | Version | Purpose |
|-----------|---------|---------|
| Rust | edition 2024 | Systems language |
| egui / eframe | 0.27.0 | Immediate-mode GUI framework |
| egui_extras | 0.27.0 | Image loaders, extra widgets |
| git2 | 0.20.0 | libgit2 bindings |
| parking_lot | 0.12 | Faster mutex (never std::sync::Mutex) |
| anyhow | 1.0.86 | Error handling |
| log + env_logger | 0.4.21 / 0.11.3 | Logging (Info level default) |
| serde (derive) | 1.0.210 | JSON serialization |
| syntect | 5.3.0 | Syntax highlighting (default-onig, default-syntaxes, default-themes) |
| tokio | 1.0 (full) | Async runtime |
| notify | 6.1.1 | File system watching |
| rfd | 0.14.0 | Native file dialogs |
| dirs | 5.0.0 | Config directory paths |
| lru | 0.12.0 | LRU cache |
| tempfile | 3.10.1 | Temporary dirs (tests) |

**Build:** Release LTO enabled, single codegen unit, panic=abort.

## Critical Implementation Rules

### Rust Language Rules

- Error type: `anyhow::Result<T>` (re-exported from `src/lib.rs`)
- Create errors: `anyhow::anyhow!("message {}", args)`
- Imports: `use crate::module::Thing;` cross-module, `use super::` for siblings
- Files: `snake_case`, Structs: `PascalCase`, Functions: `snake_case`
- Common derives: `Debug, Clone` everywhere; `Serialize, Deserialize` on models
- `impl Default` calls `Self::new()` — never use `#[derive(Default)]`

### egui/eframe UI Rules

- Component signature: `pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState)`
- State: `SharedAppState = Arc<parking_lot::Mutex<AppState>>`, lock per frame
- Panel order: side panels before CentralPanel for correct z-ordering
- Dark theme: `cc.egui_ctx.set_visuals(egui::Visuals::dark())` at startup
- Frame colors: `Color32::from_rgb(30,30,35)` left, `(35,35,40)` top, `(40,40,45)` bottom
- eframe entry: wrap `MainWindow` in a struct, delegate `update()` to `show(ctx, frame)`

### Testing Rules

- Git tests: always `tempfile::TempDir` for repos, auto-cleanup on drop
- Test returns: `anyhow::Result<()>` for I/O tests, `()` for pure logic
- Naming: `test_snake_case_description()`, files: `*_tests.rs`
- Integration tests: `src/tests/mod.rs` declares `#[cfg(test)] mod file;`
- Test runner: `cargo test` only, no external frameworks
- Always run `cargo test` after any code change (88+ tests must pass)

### Code Quality & Style Rules

- Stateless services: `pub struct Name;` with associated functions
- Stateful services: `pub struct Name { fields }` + `pub fn new() -> Self`
- Logging: `log::info!()` info, `log::error!()` errors, `log::debug!()` details
- Module exports: `pub mod submod; pub use submod::Type;` in each `mod.rs`
- Format: `cargo fmt` standard style, `cargo clippy --all-targets` linting
- Verify: `cargo check --all-targets` + `cargo test` after every change
- No inline comments unless explicitly requested

### Development Workflow Rules

- After every change: `cargo check --all-targets` + `cargo test` (88+ pass)
- Never run `cargo run` — the user tests UI themselves
- File locking: Red zone (`Cargo.toml`, `lib.rs`, `main.rs`, `mod.rs`) single-agent only
- Error flow: `set_error()` surfaces to UI + logs, `handle_pending_actions()` processes deferred ops
- File watcher: `start_watching()` on load, `stop_watching()` on close, `should_refresh()` per frame

### Critical Don't-Miss Rules

- Mutex: `parking_lot::Mutex` only, never `std::sync::Mutex`
- Components: never store state references, pass `&mut AppState` per frame
- Guard: check `state.has_repository()` before accessing repo data
- Closures: clone data before entering closures that borrow `ui`
- Secrets: never commit credentials or tokens
- Input validation: all user-supplied paths and names must be validated
- Red zone files: single agent only (`Cargo.toml`, `lib.rs`, `main.rs`, all `mod.rs`)

---

## Usage Guidelines

**For AI Agents:**
- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**
- Keep this file lean and focused on agent needs
- Update when technology stack changes
- Review quarterly for outdated rules
- Remove rules that become obvious over time

Last Updated: 2026-05-05
