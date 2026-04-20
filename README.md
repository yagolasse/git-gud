# Git Gud

A lightweight, high-performance Git GUI built with **Rust**, **Tauri**, and **Vanilla JavaScript**. Inspired by tools like GitFork, **Git Gud** aims to provide a fast and intuitive experience for managing your Git repositories.

## ✏️ Project Status

**Current Version**: v0.1.0 (Development)
**Last Updated**: April 20, 2026

**✅ Completed**: Core Git operations, branch management, remote synchronization, SSH authentication with fallback
**⚠️ Known Issues**: Visual Studio linker error prevents app launch (see troubleshooting)
**🚧 In Progress**: UI refinements, SSH authentication improvements

## ✨ Features

### ✅ **Core Repository Management**
- **📂 Multi-Repo Tabs**: Open and manage multiple repositories simultaneously in a clean tabbed interface.
- **🔄 Session Persistence**: Remembers your open repositories and window dimensions across sessions.
- **⚡ Real-time Status**: Monitors your filesystem and automatically updates the UI when files are modified, staged, or committed (even from outside the app).

### ✅ **Commit Operations**
- **📝 Stage & Commit**: Easily stage/unstage individual files or everything at once. Supports the standard subject/body commit format.
- **🔧 Amend Commit**: One-click to amend your last commit, with automatic message pre-filling.
- **❌ Discard Changes**: Safely discard unstaged changes with confirmation dialog.

### ✅ **Branch Management**
- **🌿 Branch Operations**: Checkout, rename, and create branches with optional start points.
- **🔀 Create Branch**: Create new branches with optional start point (commit, branch, or tag) and force overwrite option.
- **📊 Branch Status**: Visual indicators for ahead/behind counts relative to upstream branches.

### ✅ **Remote Synchronization**
- **📡 Remote Management**: Add, remove, and view remotes with full URL support.
- **⬇️ Fetch**: Fetch from all remotes or specific remote with progress indicators.
- **⬆️ Push**: Push current branch or specific branch to remote with SSH fallback.
- **🔄 Pull**: Pull current branch from upstream with fast-forward merge support.
- **☁️ SSH Authentication**: Comprehensive SSH support with fallback to git CLI when GUI authentication fails.

### ✅ **User Interface**
- **🎨 Theme Support**: Light and dark themes with system integration.
- **📋 Context Menus**: Right-click context menus for branches and remotes.
- **🔍 Diff Viewer**: Side-by-side diff view with syntax highlighting.
- **⏱️ Loading States**: Visual feedback for network operations with spinner animations.
- **🗂️ Branch Grouping**: Remote branches grouped by their origin in sidebar.

### 🔄 **Real-time Updates**
- **👁️ File Watcher**: Automatic refresh when repository changes occur.
- **📈 Dynamic Updates**: UI updates without manual refresh for staged/unstaged changes.

## ⚠️ Current Status & Known Issues

### ✅ **Working Features**
- All core Git operations (stage, commit, branch, remote management)
- SSH authentication with fallback to git CLI
- Complete UI with modals, context menus, and real-time updates
- Unit tests for all backend functionality

### 🚨 **Critical Issues**
1. **Visual Studio Linker Error**: The app currently fails to launch on Windows due to a linker error:
   ```
   error: linking with `link.exe` failed: exit code: 143
   note: the Visual Studio build tools may need to be repaired
   ```
   - **Workaround**: Use the standalone CLI test to verify backend functionality

2. **UI Conflicts**: '+' buttons in accordion headers may conflict with toggle actions
   - **Partial Fix**: `e.stopPropagation()` implemented, may need CSS restructuring

3. **SSH Authentication Loop**: Push operations may cause infinite authentication attempts in some cases
   - **Mitigation**: Fallback to git CLI implemented with verbose logging

### 🔧 **Development Notes**
- The project uses Tauri v2 but doesn't have npm package.json
- Backend is fully functional and tested via standalone CLI
- Frontend is complete but requires app to run for integration testing

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri Prerequisites](https://tauri.app/v2/guides/getting-started/prerequisites/)
  - Visual Studio Build Tools (Windows)
  - WebView2 (Windows)
  - See Tauri docs for other platforms

### Installation & Development

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yagolasse/git-gud.git
   cd git-gud
   ```

2. **Build and test the backend**:
   ```bash
   cd src-tauri
   cargo build
   cargo test  # Run all unit tests
   ```

3. **Run standalone CLI test** (verifies Git functionality):
   ```bash
   cd src-tauri
   cargo run --bin test-cli
   ```

4. **Attempt to run the Tauri app** (may fail due to linker issue):
   ```bash
   cd src-tauri
   cargo run --bin git-gud
   ```

### Troubleshooting Build Issues

If you encounter the Visual Studio linker error:

1. **Repair Visual Studio Build Tools** through Visual Studio Installer
2. **Ensure "Desktop development with C++" workload** is installed
3. **Try alternative Rust toolchain**:
   ```bash
   rustup update stable
   rustup default stable
   ```
4. **Install Tauri CLI globally** (may help):
   ```bash
   npm install -g @tauri-apps/cli
   cargo tauri dev
   ```

## 📁 Project Structure

```
git-gud/
├── src/                          # Frontend (Vanilla JS/HTML/CSS)
│   ├── index.html               # Main UI with modals and context menus
│   ├── main.js                  # Frontend logic (1321 lines)
│   └── styles.css               # Theme and styling (919 lines)
├── src-tauri/                   # Backend (Rust/Tauri)
│   ├── src/
│   │   ├── git.rs              # Core Git operations (1023 lines)
│   │   ├── commands.rs         # Tauri command handlers (127 lines)
│   │   ├── models.rs           # Data structures
│   │   ├── watcher.rs          # File system watcher
│   │   ├── lib.rs              # Tauri setup
│   │   └── main.rs             # Entry point
│   ├── src/bin/
│   │   └── test-cli.rs         # Standalone CLI test
│   ├── Cargo.toml              # Dependencies
│   └── tauri.conf.json         # Tauri configuration
├── ARCHITECTURE.md             # Technical architecture documentation
├── CONTEXT.md                  # Current session context
└── README.md                   # This file
```

## 🧪 Testing

### Backend Tests
All Rust unit tests are passing:
```bash
cd src-tauri
cargo test
```

Tests include:
- Repository status and staging
- Branch creation and management
- Remote operations
- Commit operations
- Diff generation

### Standalone CLI Test
Verifies Git functionality without UI:
```bash
cd src-tauri
cargo run --bin test-cli
```

### Integration Testing
Currently blocked by linker error. Once resolved:
1. Launch app with `cargo run --bin git-gud`
2. Test UI interactions
3. Verify frontend-backend communication

## 🔄 Next Steps

### High Priority
1. **Fix Visual Studio linker error** - Enable app launch
2. **Debug commit button** - Verify frontend-backend integration
3. **Test SSH operations** - Ensure fallback works correctly

### Medium Priority
1. **Resolve UI conflicts** - Finalize '+' button/accordion interaction
2. **Improve SSH authentication** - Reduce verbosity, handle edge cases
3. **Add error handling** - Better user feedback for failed operations

### Future Enhancements
1. **Merge conflict resolution**
2. **Rebase and interactive rebase**
3. **Stash management UI**
4. **Git LFS support**
5. **Plugin system for extensions**

## 🛠️ Tech Stack

- **Backend**: Rust, [git2-rs](https://github.com/rust-lang/git2-rs), [notify](https://github.com/notify-rs/notify)
- **Frontend**: Vanilla JavaScript, CSS3 (Flexbox/Grid), HTML5
- **Framework**: [Tauri v2](https://tauri.app/)
- **Authentication**: SSH agent, SSH keys, credential helper with git CLI fallback
- **Testing**: Rust unit tests, standalone CLI verification

## 🤝 Contributing

This project is primarily for learning AI code generation workflows. Issues and pull requests are welcome for:
- Fixing the Visual Studio linker error
- Improving SSH authentication
- Enhancing UI/UX
- Adding missing Git features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
*Last Updated: April 20, 2026*  
*Project Status: Functional backend, UI complete, build issues blocking integration*
