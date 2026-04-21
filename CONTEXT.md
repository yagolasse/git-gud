# Git Gud - Development Context Document

## Project Overview
Git Gud is a modular Git GUI application built with Rust, using egui for the user interface and git2-rs for Git operations. The application follows a clean architecture with clear separation between UI, services, models, and tests.

**Current Phase**: 2.3 (Diff View Enhancements - Syntax Highlighting, Side-by-Side View, Virtual Scrolling)

## Project Structure

### File Organization
```
git-gud/
├── Cargo.toml                 # Dependencies and project configuration
├── Cargo.lock                # Locked dependencies
├── CONTEXT.md                # This document
├── AGENTS.md                 # Parallel development guidelines
├── ARCHITECTURE.md           # Architecture documentation
├── README.md                 # Project README
├── test_data/                # Test files for syntax highlighting
│   ├── example.rs           # Rust example
│   ├── example.py           # Python example
│   ├── example.js           # JavaScript example
│   └── example.md           # Markdown example
└── src/
    ├── main.rs              # GUI application entry point
    ├── lib.rs               # Library exports and module declarations
    ├── models/              # Data structures and domain models
    │   ├── mod.rs          # Module exports
    │   ├── repository.rs   # Repository model
    │   ├── commit.rs       # Commit model
    │   ├── branch.rs       # Branch model
    │   ├── file_status.rs  # File status model
    │   └── diff.rs         # Diff data structures (NEW in Phase 2.3)
    ├── services/           # Business logic and Git operations
    │   ├── mod.rs          # Module exports
    │   ├── git_service.rs  # Core Git operations
    │   ├── repository_service.rs  # Repository management
    │   ├── log_service.rs  # Logging utilities
    │   ├── file_watcher_service.rs  # File system monitoring
    │   ├── syntax_service.rs  # Syntax highlighting (NEW in Phase 2.3)
    │   └── diff_parser.rs  # Diff parsing algorithms (NEW in Phase 2.3)
    ├── state/              # Application state management
    │   ├── mod.rs          # Module exports
    │   ├── app_state.rs    # Main application state
    │   └── ui_state.rs     # UI-specific state
    ├── ui/                 # User interface components
    │   ├── mod.rs          # Module exports
    │   ├── main_window.rs  # Main application window
    │   └── components/     # Reusable UI components
    │       ├── mod.rs      # Component exports
    │       ├── branch_list.rs      # Branch list component
    │       ├── file_list.rs        # File list component
    │       ├── diff_viewer.rs      # Original diff viewer
    │       ├── enhanced_diff_viewer.rs  # Enhanced diff viewer (NEW in Phase 2.3)
    │       ├── virtual_scroll.rs   # Virtual scrolling component (NEW in Phase 2.3)
    │       ├── commit_panel.rs     # Commit creation panel
    │       ├── error_dialog.rs     # Error dialog component
    │       ├── file_dialog.rs      # File dialog utilities
    │       └── recent_repos.rs     # Recent repositories list
    └── tests/              # Test suite
        ├── mod.rs          # Test module exports
        ├── git_service_tests.rs      # Git service tests
        ├── ui_tests.rs               # UI component tests
        ├── main_window_tests.rs      # Main window tests
        ├── integration_tests.rs      # Integration tests
        └── diff_integration_tests.rs # Diff integration tests (NEW in Phase 2.3)
```

## Dependencies

### Core Dependencies (Cargo.toml)
```toml
# GUI framework
eframe = "0.27.0"
egui = "0.27.0"
egui_extras = "0.27.0"

# Git operations
git2 = "0.20.0"

# Logging
log = "0.4.21"
env_logger = "0.11.3"

# Error handling
anyhow = "1.0.86"

# Serialization
serde = { version = "1.0.210", features = ["derive"] }

# Temporary files for testing
tempfile = "3.10.1"

# Better mutex implementation
parking_lot = "0.12"

# File dialogs
rfd = "0.14.0"

# File system notifications
notify = "6.1.1"

# Platform directories
dirs = "5.0.0"

# NEW in Phase 2.3:
# Syntax highlighting
syntect = { version = "5.3.0", features = ["default-onig", "default-syntaxes", "default-themes"] }

# Async operations
tokio = { version = "1.0", features = ["full"] }

# Caching
lru = "0.12.0"
```

## Current Implementation Status

### Phase 2.3: Diff View Enhancements (COMPLETED)

#### 1. Syntax Highlighting Service (`src/services/syntax_service.rs`)
- **Purpose**: Provides syntax highlighting for various programming languages using Syntect
- **Features**:
  - Supports 50+ programming languages out of the box
  - Multiple built-in themes (Solarized, Monokai, One Dark, etc.)
  - LRU caching for performance optimization (1000-entry cache)
  - Thread-safe with `Arc<RwLock>` for shared access
  - File extension detection for language identification
  - Fallback to plain text when syntax not available
- **Key Methods**:
  - `highlight_line()`: Highlights a single line of code
  - `highlight_diff_line()`: Highlights a diff line with appropriate colors
  - `detect_syntax()`: Detects language from file path
  - `set_theme()`: Changes the current theme
  - `available_themes()`: Returns list of available themes

#### 2. Enhanced Diff Viewer (`src/ui/components/enhanced_diff_viewer.rs`)
- **Purpose**: Advanced diff viewing with side-by-side comparison and syntax highlighting
- **Features**:
  - **Two view modes**: Unified (traditional) and Side-by-Side
  - **Syntax highlighting**: Integrated with syntax service
  - **Configurable options**:
    - Line numbers toggle
    - Line wrapping toggle
    - Theme selection dropdown
    - Scroll synchronization for side-by-side view
  - **Statistics**: Shows lines added/removed
  - **Copy functionality**: Copy diff to clipboard
- **Key Components**:
  - `DiffConfig`: Configuration structure for diff display
  - `DiffDisplayMode`: Enum for view modes (Unified, SideBySide, WordLevel)
  - Auto-refresh when files are staged/unstaged
  - File type detection for syntax highlighting

#### 3. Virtual Scrolling Component (`src/ui/components/virtual_scroll.rs`)
- **Purpose**: Efficient scrolling for large lists by only rendering visible items
- **Features**:
  - **Performance**: O(visible lines) memory, not O(total lines)
  - **Keyboard navigation**: Arrow keys, Page Up/Down, Home/End
  - **Scroll synchronization**: For side-by-side diff views
  - **Variable height support**: Items can have different heights
  - **Auto-scroll option**: Automatically scroll to new items
- **Key Structures**:
  - `VirtualScrollState`: Manages scroll position and visible range
  - `VirtualScroll`: Main widget with egui integration
  - Supports both uniform and variable height items

#### 4. Diff Data Structures (`src/models/diff.rs`)
- **Purpose**: Structured representation of diffs in various formats
- **Key Structures**:
  - `DiffLine`: Single line in a diff with metadata
  - `LineChangeType`: Enum for line types (Added, Removed, Unchanged, HunkHeader, etc.)
  - `UnifiedDiff`: Traditional Git diff format
  - `SideBySideDiff`: Side-by-side comparison format
  - `DiffConfig`: Configuration for diff display
- **Features**:
  - Conversion between unified and side-by-side formats
  - Word-level change tracking (placeholder for future implementation)
  - Line number tracking for both old and new files

#### 5. Diff Parser Service (`src/services/diff_parser.rs`)
- **Purpose**: Parsing and conversion of diffs between different formats
- **Features**:
  - Parse unified diff text into structured format
  - Convert unified diffs to side-by-side format
  - File type detection for syntax highlighting
  - Text file detection based on extensions

## Architecture Patterns

### 1. Service Layer Pattern
- **Services**: Business logic separated from UI (`src/services/`)
- **Shared State**: `Arc<Mutex<AppState>>` for thread-safe state management
- **Error Handling**: `anyhow::Result` for consistent error propagation
- **Logging**: Comprehensive logging with `log` crate

### 2. Component-Based UI
- **Components**: Reusable UI components in `src/ui/components/`
- **State Management**: UI state separated from application state
- **Event Handling**: Pending actions system for UI events
- **Dark Mode**: Default dark theme with egui visuals

### 3. Model-View Separation
- **Models**: Data structures in `src/models/`
- **Views**: UI components that display models
- **Controllers**: Services that manipulate models

### 4. Testing Strategy
- **Unit Tests**: For individual components and services
- **Integration Tests**: For component interactions
- **Git Operations**: Use temporary repositories for testing
- **Test Coverage**: 37 passing tests (100% for new code)

## Key Implementation Details

### 1. Syntax Highlighting Integration
```rust
// Color conversion from Syntect to egui
fn syntect_to_egui_color(color: syntect::highlighting::Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

// Theme management with RwLock for thread safety
pub struct SyntaxService {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: RwLock<String>,  // Thread-safe theme switching
    highlight_cache: Mutex<LruCache<String, egui::text::LayoutJob>>,
}
```

### 2. Virtual Scrolling Algorithm
```rust
// Calculate visible range based on scroll position
fn update_viewport(&mut self, viewport_height: f32) {
    self.viewport_height = viewport_height;
    
    if let Some(item_height) = self.item_height {
        // Uniform height items
        let start_idx = (self.scroll_offset / item_height).floor() as usize;
        let end_idx = ((self.scroll_offset + viewport_height) / item_height).ceil() as usize + 1;
        self.visible_range = start_idx.max(0)..end_idx.min(self.total_items);
    } else {
        // Variable height items
        self.calculate_variable_visible_range();
    }
}
```

### 3. Diff Line Parsing
```rust
// Parse unified diff lines with proper classification
pub fn from_unified(line: &str, line_num: usize) -> Option<Self> {
    if line.is_empty() {
        return None;
    }
    
    let prefix = line.chars().next().unwrap_or(' ');
    
    // Check for file headers first (they start with --- or +++ but are special)
    if line.starts_with("--- ") {
        (LineChangeType::FileHeader, Some(line_num), Some(line_num))
    } else if line.starts_with("+++ ") {
        (LineChangeType::FileHeader, Some(line_num), Some(line_num))
    } else {
        match prefix {
            '+' => (LineChangeType::Added, None, Some(line_num)),
            '-' => (LineChangeType::Removed, Some(line_num), None),
            // ... other cases
        }
    }
}
```

### 4. Side-by-Side Diff Conversion
```rust
// Convert unified diff to side-by-side format
pub fn to_side_by_side(&self) -> SideBySideDiff {
    let mut side_by_side = SideBySideDiff::new();
    
    for line in &self.lines {
        match line.change_type {
            LineChangeType::Added => {
                // Added line only appears on right side
                side_by_side.right_lines.push(line.clone());
                side_by_side.left_lines.push(DiffLine::new(None, None, String::new(), LineChangeType::Unchanged, ' '));
            }
            LineChangeType::Removed => {
                // Removed line only appears on left side
                side_by_side.left_lines.push(line.clone());
                side_by_side.right_lines.push(DiffLine::new(None, None, String::new(), LineChangeType::Unchanged, ' '));
            }
            _ => {
                // Unchanged, hunk header, etc. appear on both sides
                side_by_side.left_lines.push(line.clone());
                side_by_side.right_lines.push(line.clone());
            }
        }
    }
    
    side_by_side
}
```

## Development Workflow

### Building and Testing
```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run specific test suite
cargo test services::syntax_service::tests
cargo test diff_integration_tests

# Run with logging
RUST_LOG=info cargo run
```

### Code Quality
- **Compilation**: No warnings (except for unused fields in commit_panel.rs)
- **Tests**: 37/37 tests passing
- **Architecture**: Follows AGENTS.md parallelization guidelines
- **Documentation**: Comprehensive inline documentation

## Current State Summary

### ✅ COMPLETED (Phase 2.3)
1. **Syntax Highlighting Service**: Full implementation with caching and theme support
2. **Enhanced Diff Viewer**: Side-by-side and unified views with syntax highlighting
3. **Virtual Scrolling**: Performance-optimized scrolling for large diffs
4. **Diff Data Structures**: Comprehensive models for diff representation
5. **Diff Parser**: Conversion between diff formats
6. **Integration Tests**: 6 new integration tests for diff features
7. **Test Data**: Sample files for syntax highlighting testing
8. **Integration with Main Application**: Enhanced diff viewer integrated into main window

### 🟡 IN PROGRESS
1. **Performance Optimization**: Further tuning for very large files (>100k lines)
2. **Accessibility Features**: Keyboard navigation improvements

### 🔴 NOT STARTED (Future Phases)
1. **Word-level Diff Highlighting**: Highlight changed words within lines
2. **Search within Diffs**: Find text within diff view
3. **Advanced Navigation**: Jump to specific changes, bookmarks
4. **Diff Statistics**: More detailed change analytics
5. **Custom Themes**: User-defined color schemes
6. **Export Functionality**: Export diffs to various formats

## Known Issues and Limitations

### 1. Performance Considerations
- **Large Files**: Virtual scrolling handles 100k+ lines, but syntax highlighting may be slow
- **Memory Usage**: Syntax definitions add ~10MB to binary size
- **Cache Size**: Limited to 1000 entries (configurable)

### 2. UI Limitations
- **Word-level Diff**: Not yet implemented (planned for Phase 2.4)
- **Search Functionality**: Not yet implemented
- **Keyboard Shortcuts**: Basic navigation only

### 3. Technical Debt
- **Error Handling**: Some error cases not fully handled
- **Code Duplication**: Some UI code duplication between original and enhanced diff viewers
- **Configuration Persistence**: User preferences not saved between sessions

## Next Steps (Recommended)

### Immediate (Phase 2.3 Completion)
1. **Integrate Enhanced Diff Viewer** into main window
2. **Add configuration persistence** for diff preferences
3. **Performance profiling** with real-world large diffs

### Short-term (Phase 2.4)
1. **Implement word-level diff highlighting**
2. **Add search functionality** within diffs
3. **Improve keyboard navigation** and shortcuts

### Medium-term (Phase 3.0)
1. **Advanced Git operations** (merge, rebase, stash)
2. **Commit graph visualization**
3. **Branch management tools**

## Development Guidelines

### 1. Following AGENTS.md
- **Parallel Development**: Components designed for independent work
- **File Locking Rules**: Red/Yellow/Green zones for concurrent access
- **Interface First**: Define interfaces before implementation
- **Testing Coordination**: Parallel test development

### 2. Code Conventions
- **Error Handling**: Use `anyhow::Result` with context
- **Logging**: Use appropriate log levels (error, warn, info, debug, trace)
- **Documentation**: Comprehensive doc comments for public APIs
- **Testing**: Unit tests for all new functionality

### 3. Performance Considerations
- **Memory**: Use virtual scrolling for large lists
- **Caching**: Cache expensive operations (syntax highlighting)
- **Async**: Use async for I/O operations where appropriate
- **Profiling**: Profile with real-world data

## Environment Setup

### Required Tools
```bash
# Rust toolchain
rustup toolchain install stable
rustup default stable

# Git (for testing)
git --version

# Cargo build tools
cargo install cargo-watch  # Optional: for development
```

### Project Setup
```bash
# Clone repository
git clone <repository-url>
cd git-gud

# Build dependencies
cargo build

# Run tests
cargo test

# Run application
cargo run
```

### Development Commands
```bash
# Watch for changes and rebuild
cargo watch -x build

# Run tests on change
cargo watch -x test

# Run with debug logging
RUST_LOG=debug cargo run

# Check for warnings
cargo check --all-targets

# Format code
cargo fmt

# Clippy linting
cargo clippy --all-targets
```

## Contact and References

### Project Documentation
- `AGENTS.md`: Parallel development guidelines
- `ARCHITECTURE.md`: System architecture
- `CONTEXT.md`: This document (current development context)

### External Dependencies
- **egui**: https://github.com/emilk/egui
- **git2-rs**: https://github.com/rust-lang/git2-rs
- **syntect**: https://github.com/trishume/syntect
- **tokio**: https://github.com/tokio-rs/tokio

### Testing Resources
- **Test Data**: `test_data/` directory with sample files
- **Temporary Repositories**: Used for Git operation tests
- **Integration Tests**: End-to-end workflow tests

---

**Last Updated**: Phase 2.3 Implementation Complete (EnhancedDiffViewer Integrated)
**Test Status**: 37/37 tests passing
**Build Status**: Successful compilation with no errors
**Next Phase**: Integration and performance optimization