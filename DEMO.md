# Git Gud - Repository Opening Feature Demo

## Overview
This document demonstrates the repository opening feature implemented in Git Gud. The feature allows users to open Git repositories through both CLI and GUI interfaces.

## Features Implemented

### 1. GUI Repository Open Dialog
- **Modal dialog** appears on startup or via "File → Open Repository..."
- **Path input field** with text editing
- **Browse button** for native file dialog integration
- **Recent repositories** list (click to select)
- **Open/Cancel buttons**

### 2. CLI to GUI Path Passing
- `git-gud gui /path/to/repo` - Opens GUI with repository pre-loaded
- `git-gud gui` - Opens GUI with open dialog

### 3. Error Handling
- **Error dialog** for failed repository loading
- **Error logging** to console
- **Details toggle** for long error messages

### 4. UI Components
- **Three-panel layout** (Branches, Files, Diff/Commit)
- **Empty state** when no repository loaded
- **Menu bar** with File menu (Open, Close, Exit)

## How to Test

### Prerequisites
```bash
# Build the project
cargo build
```

### Test 1: CLI with Repository Path
```bash
# Create a test repository
mkdir -p /tmp/test-repo
cd /tmp/test-repo
git init
echo "Test" > file.txt
git add file.txt
git commit -m "Initial commit"

# Open GUI with repository
cd /Users/yagolasse/git-gud
cargo run -- gui "/tmp/test-repo"
```

**Expected behavior:**
- GUI launches without open dialog
- Repository is loaded and displayed
- Three-panel layout shows repository data
- Menu bar shows repository path

### Test 2: GUI without Path
```bash
# Open GUI without repository
cargo run -- gui
```

**Expected behavior:**
- GUI launches with open dialog
- "No repository loaded" empty state
- Can enter path manually or use Browse button
- Recent repositories list (if any)

### Test 3: Error Handling
```bash
# Try to open non-existent repository
cargo run -- gui "/tmp/non-existent-repo"
```

**Expected behavior:**
- GUI launches with open dialog
- Error dialog appears with error message
- Open dialog remains visible

### Test 4: Menu Bar Operations
1. Launch GUI with `cargo run -- gui`
2. Use "File → Open Repository..." to open dialog
3. Use "File → Close Repository" to return to empty state
4. Use "File → Exit" to close application

## Architecture Notes

### Components Created
1. **ErrorDialog** - Reusable error dialog component
2. **FileDialog** - Native file dialog wrapper
3. **RecentRepos** - Recent repositories manager
4. **MainWindow** - Updated with all features

### State Management
- **AppState** - Manages repository state and errors
- **PendingAction** - Handles UI actions that modify state
- **SharedAppState** - Thread-safe state sharing

### Testing
- **11 unit tests** covering all components
- **Mock testing** for state transitions
- **Integration-ready** architecture

## Code Structure

```
src/
├── main.rs              # CLI/GUI entry points
├── cli.rs               # CLI command parsing
├── ui/
│   ├── main_window.rs   # Main GUI window
│   └── components/      # Reusable UI components
│       ├── error_dialog.rs
│       ├── file_dialog.rs
│       ├── recent_repos.rs
│       └── ... (existing components)
└── tests/               # Unit tests
```

## Future Enhancements

1. **Persistent recent repositories** - Save to disk
2. **Repository validation** - Pre-check before opening
3. **Drag-and-drop** - Open repositories by dragging
4. **Repository favorites** - Star/pin important repos
5. **Multi-repository support** - Tabs or workspace concept

## Troubleshooting

### Common Issues

1. **GUI doesn't launch**: Check egui/eframe dependencies
2. **Repository not loading**: Verify path and Git permissions
3. **Native dialog not working**: Platform-specific rfd issues
4. **Errors not showing**: Check log output for details

### Debugging
```bash
# Enable verbose logging
cargo run -- --verbose gui /path/to/repo

# Check logs
tail -f /tmp/git-gud.log  # If log file configured
```

## Conclusion
The repository opening feature provides a complete workflow for users to interact with Git repositories through both CLI and GUI interfaces. The implementation follows modular architecture principles and includes comprehensive error handling and user feedback.