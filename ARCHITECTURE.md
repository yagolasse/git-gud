# Git Gud - Architecture Documentation

## Overview
Git Gud is a modular Git GUI application built with Rust, using egui for the user interface and git2-rs for Git operations. The application follows a clean architecture with clear separation between UI, services, models, and tests.

## Design Principles

### 1. Modularity
- Each feature resides in its own file
- UI components separate from business logic
- Models separate from services
- Tests separate from implementation

### 2. Dual Interface
- GUI and CLI share the same service layer
- CLI provides equivalent functionality for testing
- Both interfaces log all operations consistently

### 3. Comprehensive Logging
- Log all Git operations with parameters
- Log UI interactions and state changes
- Use plain text format with timestamps
- Multiple log levels (debug, info, error, warn)

### 4. Error Handling
- Use `anyhow::Result` for consistent error handling
- Provide context for all errors
- Log errors with stack traces in debug mode

### 5. Testing Strategy
- Use temporary files for Git operations
- Unit tests for services and models
- Integration tests for CLI/GUI parity
- Built-in Rust testing framework

## File Structure

```
src/
├── main.rs              # GUI application entry point
├── cli.rs              # CLI application entry point
├── lib.rs              # Library exports and module declarations
├── models/             # Data structures and domain models
│   ├── mod.rs         # Module exports
│   ├── repository.rs  # Repository model
│   ├── commit.rs      # Commit model
│   ├── branch.rs      # Branch model
│   └── file_status.rs # File status model
├── services/           # Business logic and Git operations
│   ├── mod.rs         # Module exports
│   ├── git_service.rs # Core Git operations
│   ├── repository_service.rs # Repository management
│   └── log_service.rs # Logging utilities
├── ui/                 # GUI components
│   ├── mod.rs         # Module exports
│   ├── main_window.rs # Main application window
│   ├── repository_view.rs # Repository browser
│   ├── commit_view.rs # Commit history viewer
│   └── components/    # Reusable UI components
│       ├── mod.rs
│       ├── file_tree.rs
│       └── diff_viewer.rs
└── tests/             # Test suites
    ├── mod.rs        # Test module exports
    ├── git_service_tests.rs
    ├── models_tests.rs
    └── integration_tests.rs
```

## Module Responsibilities

### Models (`src/models/`)
- Define data structures for Git concepts
- Serialization/deserialization with Serde
- Validation and transformation logic
- No business logic or UI dependencies

### Services (`src/services/`)
- Implement Git operations using git2-rs
- Handle errors and provide context
- Log all operations with parameters
- No UI dependencies, CLI-compatible

### UI (`src/ui/`)
- egui-based user interface components
- State management for UI interactions
- Event handling and user input
- Visual representation of models

### Tests (`src/tests/`)
- Unit tests for services and models
- Integration tests with temporary repositories
- CLI command testing
- UI component testing (where applicable)

## Dependencies

### Core Dependencies
- `egui` + `eframe`: GUI framework
- `git2`: Git operations library
- `log` + `env_logger`: Logging framework
- `clap`: CLI argument parsing
- `anyhow`: Error handling
- `serde`: Serialization/deserialization
- `tempfile`: Temporary file management for testing

### Development Dependencies
- Built-in Rust testing framework
- No external test runners required

## Logging Architecture

### Log Format
```
[YYYY-MM-DD HH:MM:SS] [LEVEL] [MODULE] Message
```

### Log Levels
- `ERROR`: Critical failures that prevent operation
- `WARN`: Non-critical issues that should be reviewed
- `INFO`: Normal operational messages (Git commands, UI actions)
- `DEBUG`: Detailed information for troubleshooting

### Log Sources
1. **Git Operations**: All git2-rs calls with parameters
2. **UI Interactions**: Button clicks, navigation, state changes
3. **Application Lifecycle**: Startup, shutdown, configuration
4. **Error Context**: Stack traces and error details

## CLI/GUI Parity

### Command Mapping
Every GUI action should have an equivalent CLI command:

| GUI Action | CLI Command | Purpose |
|------------|-------------|---------|
| Open Repository | `git-gud open <path>` | Open existing repository |
| Initialize Repo | `git-gud init <path>` | Create new repository |
| Check Status | `git-gud status` | Show repository status |
| Stage Files | `git-gud add <files...>` | Add files to staging |
| Create Commit | `git-gud commit -m <message>` | Commit staged changes |
| View History | `git-gud log` | Show commit history |
| Branch Operations | `git-gud branch <name>` | List/create branches |

### Testing Strategy
1. Implement service layer first
2. Test via CLI commands
3. Build UI on top of tested services
4. Verify GUI/CLI produce same results

## Development Workflow

### Adding New Features
1. Define model in `src/models/`
2. Implement service in `src/services/`
3. Write unit tests in `src/tests/`
4. Test via CLI commands
5. Create UI component in `src/ui/`
6. Integrate into main application

### Code Organization Guidelines
- Keep files small and focused (< 500 lines)
- One responsibility per file
- Clear module boundaries
- Consistent naming conventions
- Comprehensive documentation

## Error Handling Strategy

### Error Types
1. **Git Operations**: Repository errors, network issues
2. **File System**: Permission errors, missing files
3. **User Input**: Invalid commands, malformed data
4. **Application**: Configuration errors, state corruption

### Error Reporting
- Log errors with full context
- Provide user-friendly messages in UI
- Include recovery suggestions where possible
- Preserve original error chain

## Testing Guidelines

### Unit Tests
- Test services with temporary repositories
- Mock external dependencies where appropriate
- Cover error cases and edge conditions
- Run quickly and independently

### Integration Tests
- Test CLI commands end-to-end
- Verify GUI/CLI parity
- Test with real Git repositories
- Clean up test artifacts

### Test Data
- Use `tempfile` crate for temporary repositories
- Generate test commits with realistic data
- Test across different Git scenarios
- Include merge conflicts and edge cases

## Performance Considerations

### Memory Management
- Avoid large allocations in UI thread
- Use streaming for large diffs
- Cache repository state where appropriate
- Clean up temporary resources

### Responsiveness
- Perform Git operations in background threads
- Update UI incrementally
- Provide progress indicators for long operations
- Handle cancellation gracefully

## Security Considerations

### File System Access
- Validate repository paths
- Handle permission errors gracefully
- Sanitize user input for Git commands
- Avoid path traversal vulnerabilities

### Data Privacy
- Don't log sensitive file contents
- Handle .gitignore patterns correctly
- Respect repository privacy settings
- Clean up temporary files securely