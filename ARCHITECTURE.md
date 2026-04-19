# Git Gud - Architecture Documentation

## Overview

Git Gud is a Tauri-based desktop Git GUI application with a Rust backend and vanilla JavaScript frontend. The application provides a multi-tab interface for managing Git repositories with real-time file system monitoring.

**Tech Stack:**
- **Backend**: Rust (git2-rs for Git operations, notify for file watching)
- **Frontend**: Vanilla JavaScript, CSS3, HTML5
- **Framework**: Tauri v2
- **Build Tool**: Cargo (Rust)

## Project Structure

```
git-gud/
├── src/                    # Frontend (web assets)
│   ├── assets/            # Fonts, icons
│   ├── index.html         # Main HTML document
│   ├── main.js            # Frontend application logic
│   └── styles.css         # CSS styles with light/dark theme
├── src-tauri/             # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs        # Application entry point
│   │   ├── lib.rs         # Module declarations and Tauri setup
│   │   ├── commands.rs    # Tauri command handlers
│   │   ├── git.rs         # Git operations using git2-rs
│   │   ├── models.rs      # Data structures (serializable)
│   │   └── watcher.rs     # File system watcher for .git changes
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri configuration
│   └── capabilities/      # Tauri permissions
├── .vscode/               # VS Code settings
├── README.md
├── ARCHITECTURE.md        # This file
└── .gitignore
```

## Frontend (src/)

### index.html
- **Purpose**: Main application UI structure
- **Key Sections**:
  - Menu bar (File, View dropdowns)
  - Repository tabs container
  - Main content area with:
    - Left sidebar (Branches, Remotes, Stashes)
    - Middle changes panel (Unstaged/Staged files)
    - Right diff viewer and commit section
  - Modals (rename branch)

**Adding UI elements**:
- Add HTML structure within appropriate sections
- Assign unique IDs for JavaScript targeting
- Use existing CSS classes for consistency

### main.js
- **Purpose**: Frontend application logic and Tauri IPC
- **Key Functions**:
  - `handleOpenRepo()`: Opens repository via native dialog
  - `refreshEverything()`: Fetches all repo data
  - `createFileItem()`: Creates DOM element for file changes
  - Event listeners for UI interactions
  - Theme management

**Adding new features**:
1. Add new event listeners in `setupEventListeners()`
2. Create corresponding async functions that call Tauri commands via `invoke()`
3. Update UI elements using DOM manipulation
4. Add any new state variables at the top of the file

**IPC with Backend**:
```javascript
const result = await invoke("command_name", { param1: value1 });
```

### styles.css
- **Purpose**: Styling with CSS custom properties for theming
- **Key Features**:
  - CSS variables for light/dark themes
  - Responsive layout with Flexbox
  - Status colors for Git operations
  - Diff viewer styling

**Adding styles**:
- Use existing CSS variables (`--bg-primary`, `--text-primary`, etc.)
- Follow naming conventions (BEM-like)
- Add dark theme overrides in `.dark-theme` block

## Backend (src-tauri/)

### lib.rs
- **Purpose**: Tauri application builder and command registration
- **Key Sections**:
  - Module declarations
  - `WatcherState` management
  - Plugin initialization
  - Command handler registration via `invoke_handler![]`

**Adding new commands**:
1. Add command function to `commands.rs`
2. Add command to `invoke_handler![]` macro in `lib.rs`

### commands.rs
- **Purpose**: Tauri command handlers (bridge between frontend and git operations)
- **Structure**: Each command:
  - Takes parameters from frontend
  - Calls corresponding function in `git.rs`
  - Returns `Result<T, String>` (success or error message)

**Adding new commands**:
```rust
#[tauri::command]
pub fn new_command(param: String) -> Result<ReturnType, String> {
    let repo = git::get_repository(&param)?;
    git::new_operation(&repo)
}
```

### git.rs
- **Purpose**: Git operations using git2-rs library
- **Key Functions**:
  - `get_repository()`: Opens repository
  - `get_repo_status()`: Gets file statuses
  - `stage_files()` / `unstage_files()`: Stage management
  - `commit_changes()`: Creates commits
  - `checkout_branch()`: Branch operations

**Adding new Git operations**:
1. Add public function in `git.rs`
2. Use `git2::Repository` methods
3. Return `Result<T, String>` with descriptive errors
4. Add corresponding test in the `tests` module at bottom

### models.rs
- **Purpose**: Data structures shared between frontend and backend
- **Key Structs**:
  - `FileStatus`: Git file status (path, status, staged)
  - `RepoInfo`: Repository metadata
  - `BranchInfo`, `StashInfo`, `RemoteInfo`
  - `WatcherState`: File watcher management

**Adding new data types**:
- Derive `Serialize` and `Clone` for frontend compatibility
- Use `Option<T>` for nullable fields
- Add to appropriate modules

### watcher.rs
- **Purpose**: Monitors `.git` directory for changes
- **Functionality**: Watches for changes to `index`, `HEAD`, `refs/*` files
- **Integration**: Emits `repo-updated` event to frontend via Tauri

## Adding New Features

### Frontend Feature Workflow
1. **Add UI elements** to `index.html`
2. **Add styles** in `styles.css`
3. **Add JavaScript logic** in `main.js`:
   - Event listeners in `setupEventListeners()`
   - Async functions calling Tauri commands
   - UI update functions
4. **Add persistent state** if needed (localStorage)

### Backend Feature Workflow
1. **Add data structures** in `models.rs` (if needed)
2. **Add Git operation** in `git.rs` with proper error handling
3. **Add command handler** in `commands.rs`
4. **Register command** in `lib.rs` `invoke_handler![]`
5. **Add unit tests** in `git.rs` tests module

### Example: Adding "Fetch from Remote" Feature
1. Frontend: Add fetch button in remotes section
2. JavaScript: `handleFetchRemote()` calling `invoke("fetch_remote", {repoPath, remoteName})`
3. Backend: Add `fetch_remote` command in `commands.rs`
4. Git operation: Add `fetch_remote()` in `git.rs` using `git2::Repository::find_remote()` and `fetch()`
5. Tests: Add test for fetch operation

## Testing

### Running Tests
```powershell
# Run all Rust unit tests
cd src-tauri
cargo test

# Run specific test module
cargo test --test git

# Run tests with verbose output
cargo test -- --nocapture

# Run tests for a specific function
cargo test test_function_name
```

### Adding Unit Tests
Tests are located at the bottom of `git.rs` in the `#[cfg(test)]` module.

**Test Structure:**
```rust
#[test]
fn test_feature_name() {
    // Setup: Create temp directory and repository
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    
    // Action: Call the function being tested
    let result = function_being_tested(&repo);
    
    // Assert: Verify expected behavior
    assert!(result.is_ok());
    // Additional assertions...
}
```

**Test Guidelines:**
1. Use `tempfile::tempdir()` for isolated test environments
2. Create realistic Git scenarios (commits, branches, etc.)
3. Test both success and error cases
4. Clean up assertions (no leftover files)

### Test Dependencies
- `tempfile`: For temporary test directories
- Available in `[dev-dependencies]` in `Cargo.toml`

## Development Environment

*Note: This project uses Tauri v2 with Rust backend and vanilla JavaScript frontend. No Node.js/npm build step is required.*

### Prerequisites (Windows)
1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
   ```powershell
   # Verify installation
   rustc --version
   cargo --version
   ```
2. **Tauri CLI**: Install via Cargo
   ```powershell
   cargo install tauri-cli
   ```
3. **Git**: Required for git2-rs backend (usually installed with Git for Windows)
4. **WebView2**: Already included in Windows 10/11

### Development Commands

**All commands should be run from the project root (`C:\Projects\git-gud`)** unless specified.

#### Building and Running
```powershell
# Development mode (hot reload)
cargo tauri dev

# Build for production (release mode)
cargo tauri build

# Build without bundling
cargo build

# Clean build artifacts
cargo clean
```

#### Testing
```powershell
# Run all Rust unit tests (from project root or src-tauri/)
cd src-tauri
cargo test

# Run tests with detailed output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name

# Run tests for a specific module
cargo test --test git

# Check test coverage (requires cargo-tarpaulin)
cargo tarpaulin --ignore-tests
```

#### Code Quality
```powershell
# Check for compilation errors
cargo check

# Format Rust code (requires rustfmt)
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings
```

#### Frontend Development
- No build step required (vanilla JS/CSS/HTML)
- Browser DevTools available via Tauri's webview inspector (F12)
- Use `console.log()` for debugging
- Live reload when changing frontend files

### Debugging Tips

**Rust Debugging:**
- Use `println!()` or `dbg!()` macros for quick debugging
- VS Code with Rust Analyzer extension recommended
- Set breakpoints in VS Code for Rust code

**JavaScript Debugging:**
- Press F12 in Tauri window to open DevTools
- Console logs appear in DevTools console
- Network tab shows Tauri IPC calls

**Tauri Events:**
- Listen to events via `window.__TAURI__.event.listen()`
- Emit events from Rust with `app_handle.emit()`

### Windows-Specific Notes
- **Path Separators**: Use `/` or `\\` in Rust strings for Windows paths
- **Git Installation**: Ensure `git` is in PATH for git2-rs to work
- **Antivirus**: May slow down compilation; add `target/` to exclusions
- **File Watching**: Uses Windows ReadDirectoryChangesW API via `notify` crate

## Code Style Guidelines

### Rust
- **Error Handling**: Use `Result<T, String>` with descriptive error messages
- **Imports**: Group standard library, external crates, internal modules
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Documentation**: Use doc comments `///` for public functions

### JavaScript
- **Variables**: `camelCase`, `const` for constants, `let` for mutables
- **Functions**: Async/await for Tauri IPC calls
- **DOM Manipulation**: Use `document.getElementById()` and `element.addEventListener()`
- **Comments**: JSDoc style for function documentation

### CSS
- **Variables**: Use CSS custom properties for theming
- **Classes**: Descriptive names with consistent naming convention
- **Organization**: Group related styles, use comments for sections

## Git Operations Reference

### Supported Operations
- Open repository and get basic info
- Get file status (staged/unstaged)
- Stage/unstage files
- Discard unstaged changes
- Commit changes (with amend support)
- Get branches (local/remote)
- Checkout branches
- Rename branches
- Get stashes
- Get remotes
- Get file diffs (staged/unstaged)

### Planned/Extension Operations
- Push/pull to remotes
- Create/delete branches
- Merge/rebase operations
- Stash create/apply/drop
- Tag management
- Submodule support

## File Watcher System

The application monitors `.git` directory for changes to:
- `index` (staging area changes)
- `HEAD` (branch changes)
- `refs/*` (branch/tag updates)

When changes are detected, the backend emits a `repo-updated` event with the repository path. The frontend listens for this event and refreshes the UI accordingly.

## State Management

### Frontend State
- `repositories[]`: Array of open repositories
- `activeTabIndex`: Currently selected repository index
- `currentUnstagedPaths` / `currentStagedPaths`: Visible file paths for bulk operations
- LocalStorage: Persists open repo paths and theme preference

### Backend State
- `WatcherState`: Manages file system watchers per repository
- No persistent database - all Git state is managed by Git itself

## Error Handling

### Frontend
- Tauri command errors are caught with `try/catch`
- User-facing alerts for critical errors
- Console logging for debugging

### Backend
- All Git operations return `Result<T, String>`
- Error messages should be descriptive for debugging
- Use `map_err()` to convert git2 errors to strings

## Performance Considerations

- File watchers are created per repository and cleaned up on app close
- Git operations are synchronous but fast for typical repository sizes
- UI updates are batched where possible
- Diff rendering uses pre-formatted text for simplicity

## Contributing Checklist

When adding a new feature:

- [ ] Add frontend UI elements (HTML)
- [ ] Style the UI (CSS)
- [ ] Add JavaScript event handlers and logic
- [ ] Add Rust command handler
- [ ] Implement Git operation
- [ ] Add unit tests
- [ ] Update documentation if needed
- [ ] Test with multiple repositories
- [ ] Verify light/dark theme compatibility

## Troubleshooting

### "cargo tauri dev" fails with "failed to run custom build command for ..."
- Ensure Rust is up to date: `rustup update`
- Clean and rebuild: `cargo clean && cargo tauri dev`
- Check Windows build tools: Install Visual Studio Build Tools with C++ workload

### Git operations fail with "Failed to find repository"
- Ensure the path is a valid Git repository
- Check that `git` is in PATH: `git --version`
- Repository might be bare or corrupted

### File watcher not detecting changes
- Watcher only monitors `.git` directory changes
- Some Git operations may not trigger file events (e.g., fast-forward merges)
- Manual refresh via UI may be needed

### UI not updating after Git operations
- Check browser DevTools console for errors
- Ensure Tauri event listeners are properly set up
- Verify `repo-updated` events are being emitted

### High CPU usage
- File watchers can be intensive on large repositories
- Consider excluding certain directories in future improvements
- Monitor with Task Manager

## Common Pitfalls

1. **Git2-rs Paths**: Use relative paths from repository root for Git operations
2. **Tauri Commands**: Ensure command names match between Rust and JavaScript
3. **File Watching**: Watcher only monitors `.git` directory, not working tree
4. **Theme CSS**: Always check dark theme overrides for new UI elements
5. **Error Messages**: Provide helpful error messages for users

---

*Last Updated: April 19, 2026*  
*For questions or updates to this document, refer to the codebase structure and existing patterns.*  
*Note: This document assumes Windows PowerShell environment.*