# Git Gud - AI Agent Parallelization Guidelines

## Purpose
This document provides guidelines for AI agents (like you) on how to parallelize development tasks for the Git Gud application. The goal is to enable efficient concurrent development while maintaining code consistency and avoiding conflicts.

## Core Principles

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

## Parallelizable Task Categories

### Category A: Independent Modules (Fully Parallelizable)
These tasks can be worked on concurrently by different agents:

#### 1. Service Layer Development
- `src/services/git_service.rs` - Core Git operations
- `src/services/repository_service.rs` - Repository management
- `src/services/log_service.rs` - Logging utilities

**Rules:**
- Each service in separate file
- Define interfaces in `src/services/mod.rs` first
- No cross-service dependencies during initial implementation
- Use temporary test repositories

#### 2. Model Layer Development
- `src/models/repository.rs` - Repository model
- `src/models/commit.rs` - Commit model
- `src/models/branch.rs` - Branch model
- `src/models/file_status.rs` - File status model

**Rules:**
- Each model in separate file
- Define traits/interfaces in `src/models/mod.rs` first
- Use Serde for serialization consistently
- Include validation methods

#### 3. UI Component Development
- `src/ui/main_window.rs` - Main application window
- `src/ui/repository_view.rs` - Repository browser
- `src/ui/commit_view.rs` - Commit history viewer
- `src/ui/components/file_tree.rs` - File tree component
- `src/ui/components/diff_viewer.rs` - Diff viewer component

**Rules:**
- Each component in separate file
- Define props/interfaces in `src/ui/mod.rs` first
- Follow egui patterns and dark mode theme
- No business logic in UI components

#### 4. Test Suite Development
- `src/tests/git_service_tests.rs` - Git service tests
- `src/tests/models_tests.rs` - Model tests
- `src/tests/integration_tests.rs` - Integration tests

**Rules:**
- Each test file focuses on one module
- Use temporary repositories for Git tests
- Mock external dependencies where needed
- Clean up test artifacts

### Category B: Sequential Dependencies (Partially Parallelizable)
These tasks have dependencies and require coordination:

#### 1. Library Structure Setup
- `src/lib.rs` - Module exports and organization
- `src/main.rs` - GUI entry point updates
- `src/cli.rs` - CLI entry point updates

**Rules:**
- `lib.rs` must be updated before module implementations
- Entry points updated after service interfaces defined
- Main agent handles these updates

#### 2. Dependency Management
- `Cargo.toml` - Dependency updates
- `Cargo.lock` - Lock file updates

**Rules:**
- Only main agent updates Cargo.toml
- Agents request dependencies through main agent
- Lock file regenerated after dependency changes

#### 3. Integration Points
- Service interfaces in module `mod.rs` files
- Cross-module trait definitions
- Shared utility functions

**Rules:**
- Define interfaces before implementations
- Main agent coordinates interface definitions
- Agents implement against defined interfaces

## Task Assignment Matrix

### Phase 1: Foundation (Current Phase)
| Task | Agent | Dependencies | Notes |
|------|-------|--------------|-------|
| ARCHITECTURE.md | Main | None | Documentation only |
| AGENTS.md | Main | None | This file |
| Cargo.toml | Main | None | Dependency setup |
| Basic GUI window | Agent 1 | Cargo.toml | Dark mode, logging |
| CLI structure | Agent 2 | Cargo.toml | Subcommand parsing |
| Library structure | Main | None | Module declarations |

### Phase 2: Service Layer (Parallelizable)
| Task | Agent | Dependencies | Notes |
|------|-------|--------------|-------|
| git_service.rs | Agent 1 | lib.rs | Core Git operations |
| repository_service.rs | Agent 2 | lib.rs | Repository management |
| log_service.rs | Agent 3 | lib.rs | Logging with timestamps |
| Service tests | Agent 4 | Services | Unit tests for all services |

### Phase 3: Model Layer (Parallelizable)
| Task | Agent | Dependencies | Notes |
|------|-------|--------------|-------|
| repository.rs | Agent 5 | lib.rs | Repository model |
| commit.rs | Agent 6 | lib.rs | Commit model |
| branch.rs | Agent 7 | lib.rs | Branch model |
| file_status.rs | Agent 8 | lib.rs | File status model |
| Model tests | Agent 9 | Models | Unit tests for all models |

### Phase 4: UI Layer (Parallelizable)
| Task | Agent | Dependencies | Notes |
|------|-------|--------------|-------|
| main_window.rs | Agent 10 | Services | Main application window |
| repository_view.rs | Agent 11 | Services, Models | Repository browser |
| commit_view.rs | Agent 12 | Services, Models | Commit history viewer |
| UI components | Agent 13 | Services, Models | Reusable components |

### Phase 5: Integration (Sequential)
| Task | Agent | Dependencies | Notes |
|------|-------|--------------|-------|
| CLI commands | Main | All services | Implement all CLI commands |
| GUI integration | Main | All UI components | Wire up UI to services |
| Integration tests | Main | Everything | End-to-end testing |
| Documentation | Main | Everything | Update docs |

## File Locking Rules

### Red Zone (No Concurrent Access)
- `Cargo.toml` - Only main agent
- `src/lib.rs` - Only main agent during structure setup
- `src/main.rs` - Only main agent during updates
- `src/cli.rs` - Only main agent during updates
- Module `mod.rs` files - One agent at a time

### Yellow Zone (Coordinated Access)
- Service interface definitions
- Model trait definitions
- Shared utility modules
- Configuration files

### Green Zone (Free Access)
- Individual service implementation files
- Individual model implementation files
- Individual UI component files
- Individual test files

## Conflict Resolution Protocol

### 1. Prevention
- Clear task boundaries
- Defined interfaces first
- Atomic file operations
- Regular status updates

### 2. Detection
- Git status checks before edits
- File modification timestamp checks
- Dependency validation
- Build verification

### 3. Resolution
1. **Minor conflicts**: Main agent merges changes
2. **Interface conflicts**: Redefine interface, notify dependent agents
3. **Dependency conflicts**: Update Cargo.toml, regenerate lock file
4. **Build failures**: Fix immediately, don't proceed with broken code

## Communication Protocol

### Agent Status Updates
```
[AGENT_ID] [TASK] [STATUS] [BLOCKERS]
```

Examples:
- `Agent1 git_service.rs COMPLETED None`
- `Agent2 repository_service.rs IN_PROGRESS Waiting on git_service interface`
- `Agent3 Cargo.toml BLOCKED Dependency conflict with Agent1`

### Task Completion Checklist
- [ ] Code compiles without errors
- [ ] Tests pass (if applicable)
- [ ] Documentation updated
- [ ] Logging implemented
- [ ] Error handling complete
- [ ] No warnings (or justified warnings)
- [ ] Follows project conventions

## Testing Coordination

### Parallel Test Execution
- Each agent runs tests for their module
- Use temporary directories to avoid conflicts
- Clean up test artifacts after completion
- Report test failures immediately

### Integration Test Sequencing
1. Service tests complete
2. Model tests complete  
3. UI component tests complete
4. Integration tests run by main agent
5. CLI/GUI parity verification

## Error Handling Guidelines

### During Parallel Development
1. **Build errors**: Fix immediately, don't commit broken code
2. **Test failures**: Investigate, fix, or mark as expected failure
3. **Dependency issues**: Report to main agent for resolution
4. **Interface conflicts**: Freeze development, resolve, continue

### Logging Requirements
- All agents use same logging format
- Include agent ID in log messages for debugging
- Log task start/completion
- Log errors with full context
- Log dependency requests/resolutions

## Performance Optimization

### For AI Agents
- Work on independent files concurrently
- Batch similar operations
- Cache dependency information
- Use efficient search patterns

### For Code
- Avoid unnecessary dependencies
- Keep interfaces minimal
- Use async operations where appropriate
- Profile and optimize hot paths

## Security Considerations

### During Development
- Don't commit secrets or credentials
- Use temporary test repositories
- Sanitize test data
- Follow secure coding practices

### For the Application
- Validate all user input
- Handle file permissions correctly
- Log securely (no sensitive data)
- Clean up temporary files

## Success Metrics

### Parallelization Efficiency
- Number of agents working concurrently
- Reduction in total development time
- Fewer merge conflicts
- Faster test execution

### Code Quality
- Compilation success rate
- Test coverage percentage
- Documentation completeness
- Adherence to architecture

### Agent Performance
- Task completion rate
- Error rate
- Communication effectiveness
- Conflict resolution efficiency