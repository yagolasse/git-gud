---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
lastStep: 14
inputDocuments:
  - _bmad-output/project-context.md
  - ARCHITECTURE.md
  - CONTEXT.md
  - AGENTS.md
---

# UX Design Specification for git-gud

**Author:** Yago Lasse
**Date:** 2026-05-05

---

## Executive Summary

### Project Vision

An open-source desktop Git GUI that matches the polish and performance of Fork,
providing developers with a clear, fast, visual interface for staging files,
reviewing diffs, and managing branches — without the sluggishness of Electron-based
alternatives.

### Target Users

Experienced developers who currently use Fork or similar paid Git GUIs and want
a high-quality open-source alternative. They know Git well but prefer visual
workflows for staging, diffing, and committing. They value performance
(snappy interactions, no UI lag) and clarity (clean layout, no clutter).

### Key Design Challenges

1. **Three-panel density** — Branches, files, and diff+commit must coexist without
   visual clutter, even with large repositories
2. **Performance perception** — Every interaction (stage, unstage, switch branch,
   view diff) must feel instant; virtual scrolling solves large diffs but all Git
   operations must be sub-frame responsive
3. **Visual hierarchy** — Users must scan effortlessly: current branch → changed
   files → diff details → commit action — the flow should be self-evident

### Design Opportunities

1. **Fork-caliber polish as open source** — Become the de facto Git GUI for
   developers who reject Electron bloat and proprietary tools
2. **egui's immediate-mode advantage** — Zero-framework-overhead rendering means
   inherently lower latency than DOM-based GUIs
3. **Syntax-highlighted diffs** — A differentiator; most GUIs show plain text diffs
4. **Scalability by design** — Virtual scrolling + LRU caching means even
   100k-line diffs stay responsive
---

## Core User Experience

### Defining Experience

The core loop is **files → stage → commit**: users open a repo, scan changed
files, click to review diffs, check boxes to stage, write a message, and commit.
Everything in the UI must serve this loop — branches, diff viewing, commit
panel — all orbit around the staging workflow.

### Platform Strategy

**Desktop-first, cross-platform.** Windows initially, but designed for macOS
and Linux from the start. No web version — native desktop performance is a
core differentiator. Mouse/trackpad primary with full keyboard shortcut support
as a first-class interaction mode (Fork users expect it).

### Effortless Interactions

1. **Branch switching** — One click or keystroke, instant checkout. No loading
   spinner, no confirmation dialog. The file list and diff update immediately.
2. **Seeing what changed** — Click a file, diff appears. No lag, no manual
   refresh. File watcher handles external changes automatically.
3. **Staging** — Checkbox or keyboard shortcut. Undo is instant and obvious.
   No partial staging complexity unless explicitly requested.

### Critical Success Moments

1. **First impression for Fork users** — They open a repo and immediately see
   their branches, changed files, and a clean dark UI. No setup, no config
   wizard, no Electron splash screen. Just works.
2. **Diff performance** — Scrolling through a large diff must feel native.
   If diff rendering lags, the app fails its core promise.
3. **Commit completion** — The commit button results in an instant success
   indicator, not a buried status message.

### Experience Principles

1. **Performance is UX** — No operation may block the UI. Git ops must be
   sub-frame. Virtual scrolling for large data. No spinners, no waits.
2. **Visibility over navigation** — Current branch, staged count, diff content
   are always visible. No digging through menus for core info.
3. **Keyboard parity** — Every mouse action has a keyboard equivalent. Power
   users never need to leave the keyboard.
4. **Zero-friction open source** — No license popup, no telemetry nag, no
   account required. Open a repo and start working.
---

## Desired Emotional Response

### Primary Emotional Goals

1. **Confident and in control** — Users feel the tool responds instantly to every
   action. No guesswork about what's staged, what branch they're on, or what
   changed. The UI is transparent about Git state.
2. **Free and unconstrained** — No license dialogs, no feature gates, no account
   prompts. Open source means the tool serves the user, not a business model.
3. **Productive momentum** — The tool fades into the background. Users flow
   through files→stage→commit without friction, maintaining their train of thought.

### Emotional Journey Mapping

| Moment | Desired Feeling |
|--------|----------------|
| First launch / open repo | Relief — "It just works, no setup" |
| Viewing changed files | Orientation — "I can see everything at a glance" |
| Clicking a file for diff | Trust — "The diff is instant and clear" |
| Staging files | Certainty — "I know exactly what I'm committing" |
| After commit | Accomplishment — "Done, next task" |
| Error occurs | Clarity — "I know what went wrong and how to fix it" |
| Returning next session | Welcome — "It remembered my recent repos" |

### Micro-Emotions

- **Trust over skepticism** — Every action must produce a visible, immediate
  result. Staging checkmarks, diff updates, branch changes — all instantaneous.
- **Clarity over confusion** — Git state (branch, staged count, dirty files)
  is always visible. Errors explain what happened and suggest next steps.
- **Satisfaction over frustration** — No operation blocks the UI. No stale data.
  File watcher means the repo state is always current.

### Design Implications

- **Responsiveness → Instant feedback** — Every click/keystroke produces an
  immediate visual result. No spinners for operations under 100ms. Progress
  indicators only for operations that genuinely take time (large repo clone).
- **No license → Clean onboarding** — First launch skips directly to "Open
  Repository". No registration, no welcome wizard, no "pro features" upsells.
- **Productive → Minimal chrome** — UI elements exist only to serve the core
  loop. No decorative elements, no feature showcase panels, no tips of the day.
- **Be clear → Helpful errors** — Error messages explain what went wrong in
  plain language and suggest a fix. No raw Git error output shown to users.

### Emotional Design Principles

1. **Invisible when working** — The best Git GUI is one you don't notice using.
2. **Trustworthy at all times** — Never show stale data or ambiguous state.
3. **Respectful of attention** — No popups, no toasts for routine success. Errors
   only when action is needed.
---

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

**Fork (git-fork.com)**
- **What it does well:** Three-panel layout (branches | files | diff) establishes
  a clear visual hierarchy. The staging area uses checkboxes that make staging
  feel deliberate and reviewable. Branch context menu gives power without clutter.
- **Key pattern:** Every Git concept has a direct visual representation — no
  translation between CLI model and GUI display.
- **Takeaway for git-gud:** Adopt the three-panel layout structure and checkbox
  staging. Improve on Fork by adding syntax highlighting to diffs.

**GitHub Desktop**
- **What it does well:** Minimalist onboarding — open a repo, see changes, commit.
  No configuration required. Visual design uses ample whitespace and clear
  typography to reduce cognitive load.
- **Key pattern:** Progressive disclosure — advanced features exist but don't
  compete with the primary workflow.
- **Takeaway for git-gud:** Match the "just open and work" simplicity. Hide
  advanced Git operations behind discoverable but unobtrusive access points.

### Transferable UX Patterns

**Navigation Patterns:**
- **Three-panel layout (Fork)** — Branches left, files center, diff+commit right.
  Already implemented. Each panel independently resizable and scrollable.
- **Recently opened list (GitHub Desktop)** — No "welcome screen" with
  marketing. Just a list of your repos. Implemented via recent_repos component.

**Interaction Patterns:**
- **Checkbox staging (Fork)** — Click to stage, click to unstage. No drag-and-drop
  complexity. Implemented in file_list component.
- **Context menu on right-click (Fork)** — Branch operations, file operations
  accessible via context menu, not buried in menu bar. Partially implemented.
- **Command palette (VS Code)** — Consider for future: Ctrl+Shift+P for any
  action without navigating menus.

**Visual Patterns:**
- **Dark theme by default** — Both Fork and GitHub Desktop offer dark mode.
  git-gud enforces it. Reduces eye strain for long coding sessions.
- **Syntax-highlighted diffs** — A git-gud differentiator. Neither Fork nor
  GitHub Desktop highlight code in diffs. This is a competitive advantage.

### Anti-Patterns to Avoid

1. **Nested menus** — Never more than one level of submenu. Any command
   reachable in two clicks or one shortcut. Flat is fast.
2. **Blocking operations** — Never freeze the UI for Git operations. All Git
   calls must be non-blocking or so fast they feel non-blocking (< 16ms).
3. **Feature overload in main view** — The primary panel shows branches, files,
   diff, commit — and nothing else. Stash, rebase, merge are secondary views
   or context-menu items, never inline.
4. **Stale data** — Never show Git state that doesn't match the filesystem.
   File watcher ensures auto-refresh. Manual refresh is failure recovery, not
   normal operation.

### Design Inspiration Strategy

**Adopt directly:**
- Three-panel layout from Fork (already implemented)
- Checkbox staging from Fork (already implemented)
- Recent repos list from GitHub Desktop (already implemented)

**Adapt and improve:**
- Branch context menu — add keyboard shortcuts for each action
- Diff viewer — add syntax highlighting (Fork's weakness, our strength)
- Error handling — clearer, more actionable than Fork's raw error display

**Avoid entirely:**
- Nested menus beyond one level
- Modal wizards or setup flows
- Any UI element that could block or lag
- Marketing upsells, feature gates, license reminders
---

## Design System Foundation

### Design System Choice

**egui-native design system.** Since git-gud is a native Rust desktop app using
egui (immediate-mode GUI), the design system is egui itself. There is no external
component library to adopt — egui provides all widgets, theming, and layout out
of the box.

### Rationale

egui's immediate-mode architecture is the project's core technical choice and
directly enables the performance UX principle. No DOM, no virtual tree diffing,
no CSS — every frame rebuilds the UI from state. This means:
- Zero framework overhead per frame
- No style cascade to debug
- Every visual decision is explicit in code

### Implementation Approach

| Aspect | Implementation |
|--------|---------------|
| Theme | `egui::Visuals::dark()` at startup |
| Panel backgrounds | `egui::Frame` with `Color32::from_rgb()` fills |
| Layout | `SidePanel` + `CentralPanel` + `TopBottomPanel` |
| Typography | egui default proportional font (no custom font loaded) |
| Widgets | Stock egui: `Button`, `TextEdit`, `Checkbox`, `CollapsingHeader`, `menu::bar` |
| Icons | Unicode emoji (💾, 🗑️, 🔄, 📜) — no icon library |
| Colors | Dark theme: backgrounds ~RGB(30-40), text white, green for staged, yellow for warnings |

### Customization Strategy

- **Current:** Minimal customization — dark visuals + frame fills. Emoji as
  button icons. This is clean and fast.
- **Near-term:** Define semantic color tokens (e.g., `STAGED_GREEN`, `ERROR_RED`,
  `BRANCH_BLUE`) instead of inline `Color32::from_rgb()` calls scattered across
  components.
- **Long-term:** Consider a custom egui style with branded accent colors and
  improved typography once the core workflow is stable.
---

## Core User Experience

### Defining Experience

**"Stage and commit in one view."** Users open a repository and immediately see
their branches, changed files, and a diff in a single screen. They click files
to review changes, check boxes to stage, write a message, and commit.
The terminal stays closed.

### User Mental Model

Developers bring existing Git CLI knowledge — they understand staging areas,
branches, and commits. git-gud doesn't teach Git; it makes Git visual and fast.
The mental model is: "what Fork does, but open source and faster." Users expect:
- Changed files listed immediately (not after loading spinner)
- Diff visible on click (not after a render delay)
- Staging as checkboxes (familiar from Fork)
- Commit as a text field + button (familiar from GitHub Desktop)

### Success Criteria

| Criterion | Target |
|-----------|--------|
| Open repo → see changes | < 1 second |
| Click file → diff visible | Instant (same frame) |
| Stage checkbox → visual feedback | Same frame |
| Commit button → success message | Same frame |
| Branch switch → all panels refresh | < 2 seconds |
| 100k-line diff scrolling | No perceived lag (virtual scroll) |

### Novel UX Patterns

**Established (adopted):**
- Three-panel layout (branches | files | diff+commit) — from Fork
- Checkbox staging with staged/unstaged split — from Fork
- Recent repositories list — from GitHub Desktop

**Novel (differentiators):**
- **Syntax-highlighted diffs** — Neither Fork nor GitHub Desktop highlights code
  within diffs. git-gud uses syntect for language-aware coloring in both unified
  and side-by-side views.
- **Virtual scrolling for large diffs** — Most Git GUIs stall on large diffs.
  git-gud renders only visible lines, making 100k-line diffs as fast as 50-line
  ones.

### Experience Mechanics

**1. Initiation:** User opens a repo (File → Open, or recent repos list, or CLI
path argument). No wizard, no config.

**2. Interaction:**
- Left panel: click branch to switch, double-click to checkout, right-click for
  context menu
- Center top: checkbox to stage/unstage, click file name to view diff
- Center bottom: review staged files, unstage if needed
- Right top: unified or side-by-side diff with syntax highlighting
- Right bottom: write commit summary + description, click Commit

**3. Feedback:**
- Stage: file instantly moves from unstaged to staged list, count updates
- Diff: appears instantly, syntax-highlighted, scrollable via virtual scroll
- Commit: success info bar, staged count resets, commit message clears
- Error: modal with explanation and suggested fix, not raw error dump

**4. Completion:** After commit, the user returns to the changed files view —
ready for the next change. The tool stays invisible between commits.
---

## Visual Design Foundation

### Color System

**Dark theme** — the only theme. Chosen for eye comfort during long coding
sessions and consistency with IDE environments (VS Code, IntelliJ, terminals).

| Context | Color | Usage |
|---------|-------|-------|
| Left panel background | RGB(30, 30, 35) | Branches sidebar |
| Top panel background | RGB(35, 35, 40) | Unstaged files |
| Bottom panel background | RGB(40, 40, 45) | Staged files |
| Right panel background | RGB(30, 30, 35) | Diff + commit |
| Menu/top bar | egui default dark | Menu bar |
| Text primary | White (egui default) | All text |
| Staged / success | GREEN | Ready to commit, commit success |
| Unstaged / warning | YELLOW | No staged changes |
| Error | RED | Error state, invalid input |

**Semantic color targets (near-term):** Extract inline `Color32::from_rgb()`
calls into named constants to ensure consistency and allow future theming.

### Typography System

- **Font:** egui default proportional font. No custom fonts loaded.
- **Rationale:** This is a developer tool, not a consumer app. Custom fonts add
  binary size and complexity with minimal UX benefit. Developers already spend
  hours in egui-default-styled tools.
- **Scale:** Default egui heading/body/monospace sizes.
- **Monospace:** egui default for code in diffs — syntax highlighting via syntect
  adds font styling (bold, italic, underline) per syntax rules.

### Spacing & Layout Foundation

**Dense and efficient.** Developer tools prioritize information density over
breathing room. The three-panel layout maximizes usable space.

- **Panel margins:** Zero (`egui::Margin::ZERO`) — frame fills provide visual
  separation instead of margins
- **Panel sizes:** Resizable, with defaults (left 250px, right 400px) and
  minimums (150px)
- **List spacing:** Compact — files and branches listed with minimal padding
  to maximize visible items
- **Form spacing:** Standard egui widget spacing for commit panel fields
- **Grid:** None — egui's immediate-mode layout handles positioning. No CSS
  grid or flexbox equivalent needed.

### Accessibility Considerations

- **Contrast:** Dark backgrounds with white text meet AA contrast ratios by
  default via egui's dark visuals
- **Resizable panels:** Users can adjust panel widths to their preference
- **Keyboard navigation:** Arrow keys for lists (implemented in virtual scroll),
  planned: full keyboard shortcut parity
- **No color-only signaling:** Status indicators use text labels alongside
  color (e.g., "✓ Ready to commit" in green, not just green dot)
- **Future:** Configurable font sizes for accessibility, high-contrast theme
  option
---

## Design Direction

### Single Design Direction: Dark Three-Panel Desktop

git-gud follows a single, focused design direction rather than multiple
variations. The direction is already implemented and validated:

**Layout:** Three resizable panels with zero-margin frames — branches (left),
files (center), diff + commit (right). Menu bar at top. No sidebar toggling,
no tabs, no floating windows.

**Visual identity:** Dark egui visuals with layered panel backgrounds
distinguishing functional zones. The color progression from darkest (branches)
to lightest (staged files) creates natural visual hierarchy without borders or
separators.

**Interaction model:** Immediate-mode GUI — every frame rebuilds from state.
No DOM, no virtual DOM, no style cascade. This enables the performance UX
principle directly: zero framework overhead means sub-frame responsiveness.

**Why a single direction:** egui's constraints (no custom styling engine,
immediate-mode rendering) and the project's maturity (Phase 2.3, UI already
built) make multiple design explorations unnecessary. The direction is set —
the work is in refinement, not exploration.
---

## User Journey Flows

### Journey 1: Open Repository & Browse

1. Launch git-gud → "Open Repository" dialog with recent repos list
2. Select repo (click recent, type path, or Browse) → panels populate
3. Left panel shows branches (current highlighted), center shows changed files, right shows empty diff
4. User scans files list, sees staged/unstaged counts
5. Ready for next journey

**Entry points:** Recent repos click, path text entry, native file dialog, CLI path argument

### Journey 2: Review & Stage Changes

1. User clicks a changed file in center panel
2. Right panel instantly shows syntax-highlighted diff (unified or side-by-side)
3. User reviews changes — scrolls, switches view mode, toggles line numbers
4. User checks the checkbox next to the file → file moves from unstaged to staged list
5. Staged count increments, diff remains visible for further review
6. Repeat for additional files

**Decision point:** Toggle between unified/side-by-side diff view per file type preference
**Error recovery:** Uncheck staged file to move back to unstaged (instant undo)

### Journey 3: Commit Flow

1. User writes commit summary in right-bottom panel (required, single line)
2. User optionally writes description (multi-line body)
3. Character counter and validity indicator show status
4. "Commit" button enabled only when: staged files > 0 AND summary is non-empty
5. User clicks Commit → success info bar, message clears, staged count resets
6. User returns to files list, ready for next change

**Error recovery:** Failed commit shows modal with explanation and suggested fix

### Journey 4: Branch Switch

1. User clicks a branch in left panel → diff/file list updates instantly
2. Or user types in branch filter to find a specific branch
3. Or user uses context menu (right-click) for branch operations
4. Or double-clicks branch name to checkout

**Edge case:** Uncommitted changes — git2 handles checkout safety

### Journey Patterns

- **Immediate feedback:** Every action produces same-frame visual result
- **No confirmation dialogs:** Actions are reversible (unstage) or committed via deliberate button press
- **Progressive disclosure:** Branch filter is a text field, not a modal. Commit history is a button, not a default panel.
- **State always visible:** Current branch, staged count, change count always displayed

### Flow Optimization Principles

1. **One view for the entire loop** — never navigate away from the three-panel view
2. **Click path ≤ 2 for any action** — stage: 1 click, commit: type + 1 click, switch branch: 1 click
3. **No intermediate states** — no "loading..." between actions that complete in < 100ms
---

## Component Strategy

### Design System Components (egui built-in)

| egui Widget | Usage in git-gud |
|-------------|-----------------|
| `SidePanel::left` | Branches panel |
| `SidePanel::right` | Diff + commit panel |
| `CentralPanel` | File lists (unstaged + staged) |
| `TopBottomPanel::top` | Menu bar, unstaged files section, diff section |
| `TopBottomPanel::bottom` | Staged files section, commit section |
| `Button` | Commit, Refresh, Cancel, History, clear buttons |
| `TextEdit::singleline` | Commit summary, author fields, path input, branch filter |
| `TextEdit::multiline` | Commit description |
| `Checkbox` | File staging, advanced options toggles |
| `CollapsingHeader` | Advanced commit options |
| `menu::bar` | File/Edit/Repository/View/Help menus |
| `Window` | Repository open dialog |
| `Label` / `heading` | All text content |

### Custom Components (already implemented)

| Component | File | Status | Purpose |
|-----------|------|--------|---------|
| `BranchList` | `components/branch_list.rs` | Complete | Branch list with filter and context menu |
| `FileList` | `components/file_list.rs` | Complete | File list with checkboxes, staging, filtering |
| `EnhancedDiffViewer` | `components/enhanced_diff_viewer.rs` | Complete | Unified + side-by-side diff with syntax HL |
| `DiffViewer` | `components/diff_viewer.rs` | Complete | Simple unified diff (original) |
| `CommitPanel` | `components/commit_panel.rs` | Complete | Commit message input and commit action |
| `VirtualScroll` | `components/virtual_scroll.rs` | Complete | Efficient scrolling for large lists |
| `ErrorDialog` | `components/error_dialog.rs` | Complete | Modal error dialog with details toggle |
| `FileDialog` | `components/file_dialog.rs` | Complete | Native file/folder picker wrapper |
| `RecentRepos` | `components/recent_repos.rs` | Complete | Recent repos with disk persistence |

### Stub Components (placeholder, not yet implemented)

| Component | File | Status |
|-----------|------|--------|
| `RepositoryView` | `repository_view.rs` | Stub — shows "Repository View" label only |
| `CommitView` | `commit_view.rs` | Stub — shows "Commit View" label only |

### Component Implementation Strategy

**Pattern:** Every component follows the same signature:
```rust
pub struct ComponentName { /* internal state */ }
impl ComponentName {
    pub fn new() -> Self { ... }
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState) { ... }
}
```

**State flow:** `MainWindow` locks `SharedAppState` once per panel, passes
`&mut AppState` to each component's `show()` method. Components borrow the
lock guard for one frame — no state references stored between frames.

### Implementation Roadmap

| Phase | Components | Status |
|-------|-----------|--------|
| Phase 2.1-2.2 | Core: `BranchList`, `FileList`, `DiffViewer`, `CommitPanel`, `MainWindow` | Complete |
| Phase 2.3 | Enhanced: `EnhancedDiffViewer`, `VirtualScroll`, `SyntaxService`, `DiffParser` | Complete |
| Phase 2.3 | Supporting: `ErrorDialog`, `FileDialog`, `RecentRepos`, `FileWatcher` | Complete |
| Phase 3.0 | Stubs: `RepositoryView`, `CommitView`, `CommitGraph` (new) | Not started |
---

## UX Consistency Patterns

### Button Hierarchy

| Priority | Style | Examples |
|----------|-------|----------|
| Primary action | Enabled button with text | "Commit" (only when valid) |
| Secondary action | Enabled button | "Refresh", "Browse..." |
| Destructive action | Button with clear label | "Clear" (commit message) |
| Disabled | Greyed-out button, no click | "Commit" when no staged files |

**Rule:** Only one primary action per panel section. No dual-CTA confusion.

### Feedback Patterns

- **Success:** Info bar text, auto-clears on next action. No toast, no popup.
- **Error:** Modal dialog with error message and optional detail toggle. User
  dismisses explicitly.
- **Progress:** None for operations < 100ms. Future: progress bar for clone/push/pull.
- **State change:** Instant visual change (checkmark appears, file moves lists).

### Form Patterns

- **Single-line inputs:** Immediate, no submit button needed. Used for branch filter,
  author overrides, path entry.
- **Multi-line input:** Used for commit description. No character limit.
- **Required field:** Commit summary — validity indicator (red/green) shows status
  in real time. Submit button disabled until valid.
- **Validation:** Real-time, not on-submit. Character counter always visible.

### Empty State Patterns

- **No repo loaded:** Centered heading "Git Gud" with "Open Repository..." button.
- **No changed files:** Empty file lists (no "nothing to commit" message needed —
  empty list is self-evident).
- **No branches:** Empty branch list (edge case — impossible in valid repos).
- **No diff selected:** Empty diff panel ("click a file to view diff" not shown —
  empty space is sufficient signal).

### Search & Filter Patterns

- **Branch filter:** Real-time text filter in branch list. No search button.
  Filtering is immediate as user types.
- **File filter:** (Planned) Filter changed files by name or path pattern.
- **Diff search:** (Planned) Find text within current diff view.

### Modal & Overlay Patterns

- **Error dialog:** Modal window, centered, with "OK" dismiss button. Optional
  "Show Details" toggle for technical error chain.
- **Open Repository:** Modal window with path input, Browse button, recent repos
  list. Cancel and Open buttons.
- **Rule:** Only one modal at a time. No stacked dialogs.

### Desktop-Specific Patterns

- **Resizable panels:** All three panels independently resizable via drag handles.
  Minimum width 150px prevents collapse-to-zero.
- **Menu bar:** Standard desktop menu (File, Edit, Repository, View, Help).
  Keyboard-accessible via Alt+letter.
- **Context menus:** Right-click on branches and files for operations.
- **Recent files:** Recent repos persisted to disk via `dirs` config directory.
---

## Responsive Design & Accessibility

### Window Resize Strategy

git-gud is **desktop-only** — no tablet or mobile targets. The responsive
strategy focuses on window resize behavior:

- **Minimum window:** 800×600 (`ViewportBuilder::with_min_inner_size`)
- **Default window:** 1200×800
- **Panel minimums:** 150px each — panels cannot collapse to zero
- **Resize priority:** As window narrows, the center panel (file lists) absorbs
  the reduction. Left (branches) and right (diff) panels maintain usability
  down to their 150px minimums.
- **Diff view:** Side-by-side mode may become unusable below ~900px window width.
  Unified diff mode recommended for narrow windows.

### Accessibility Strategy

**Target:** WCAG 2.1 Level AA where applicable to native desktop apps.

| Criterion | Implementation |
|-----------|---------------|
| Color contrast | Dark theme meets AA (white on dark backgrounds) |
| Resizable UI | Panels resizeable, minimum window enforced |
| Keyboard navigation | Arrow keys in lists (implemented), Tab for focus (egui built-in) |
| Keyboard shortcuts | Planned: full parity with mouse actions |
| Text alternatives | Status indicators include text labels alongside color |
| No color-only info | "✓ Ready to commit" in green text, not just green dot |
| Font size | Default egui font. Future: configurable font size option |

**Keyboard Navigation (current):**
- Arrow Up/Down: Navigate file lists and branch lists
- Page Up/Down, Home/End: Virtual scroll navigation
- Tab: Standard egui focus traversal between widgets
- Enter/Space: Activate focused button

**Keyboard Shortcuts (planned):**
- Ctrl+O: Open repository
- Ctrl+Enter: Commit
- Ctrl+S: Stage selected file
- Ctrl+U: Unstage selected file
- Ctrl+R: Refresh
- F5: Refresh

### Testing Strategy

| Type | Method | Status |
|------|--------|--------|
| Keyboard-only | Navigate all journeys without mouse | Partially implemented |
| Screen reader | Test with OS accessibility APIs (NVDA on Windows, VoiceOver on macOS) | Not started |
| Color blindness | Deuteranopia/protanopia/tritanopia simulation | Not started |
| High contrast | OS-level high contrast mode compatibility | Not started |
| Resize stress | Window resize to minimum, all panel drag combinations | Manual testing |

### Implementation Guidelines

- **Panel layout:** Side panels declared before CentralPanel for z-order
- **Keyboard events:** egui `ctx.input().consume_key()` for shortcut handling
- **Focus management:** egui auto-manages widget focus via Tab order
- **Font scaling:** Use egui's `style.text_styles` for future font size config
- **Contrast:** Use semantic color constants (not inline RGB) for future theme
  variants (high-contrast, light mode)
---

