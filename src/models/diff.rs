//! Diff data structures for Git Gud
//!
//! This module contains data structures for representing diffs in various formats.

/// Type of change for a line in a diff
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineChangeType {
    /// Line unchanged (appears in both left and right)
    Unchanged,

    /// Line added (only in right file)
    Added,

    /// Line removed (only in left file)
    Removed,

    /// Line modified (similar content but changed)
    Modified,

    /// Hunk header (e.g., @@ -1,3 +1,4 @@)
    HunkHeader,

    /// File header (e.g., --- a/file.txt)
    FileHeader,

    /// Binary file indicator
    Binary,

    /// No newline at end of file indicator
    NoNewline,
}

/// Word-level change within a line
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordChange {
    /// Start position in the line (character index)
    pub start: usize,

    /// End position in the line (character index, exclusive)
    pub end: usize,

    /// Type of word change
    pub change_type: WordChangeType,
}

/// Type of word-level change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordChangeType {
    Added,
    Removed,
    Modified,
}

/// Represents a single line in a diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// Line number in the left (old) file, None for added lines
    pub left_line_num: Option<usize>,

    /// Line number in the right (new) file, None for removed lines
    pub right_line_num: Option<usize>,

    /// The actual line content (without +/- prefix for unified diffs)
    pub content: String,

    /// Type of change for this line
    pub change_type: LineChangeType,

    /// Optional word-level changes within the line
    pub word_changes: Vec<WordChange>,

    /// Whether this line should be highlighted with syntax highlighting
    pub should_highlight: bool,

    /// Original line prefix in unified diff format (+, -, space, @, etc.)
    pub prefix: char,
}

impl DiffLine {
    /// Create a new diff line
    pub fn new(
        left_line_num: Option<usize>,
        right_line_num: Option<usize>,
        content: String,
        change_type: LineChangeType,
        prefix: char,
    ) -> Self {
        Self {
            left_line_num,
            right_line_num,
            content,
            change_type,
            word_changes: Vec::new(),
            should_highlight: matches!(
                change_type,
                LineChangeType::Unchanged | LineChangeType::Modified | LineChangeType::Added
            ),
            prefix,
        }
    }

    /// Create a line from unified diff format
    pub fn from_unified(line: &str, line_num: usize) -> Option<Self> {
        if line.is_empty() {
            return None;
        }

        let prefix = line.chars().next().unwrap_or(' ');
        let content = if line.len() > 1 { &line[1..] } else { "" }.to_string();

        let (change_type, left_line, right_line) = {
            // Check for file headers first (they start with --- or +++ but are special)
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                (LineChangeType::FileHeader, Some(line_num), Some(line_num))
            } else if line.starts_with("Binary files ") {
                (LineChangeType::Binary, Some(line_num), Some(line_num))
            } else {
                match prefix {
                    '+' => (LineChangeType::Added, None, Some(line_num)),
                    '-' => (LineChangeType::Removed, Some(line_num), None),
                    '@' => (LineChangeType::HunkHeader, Some(line_num), Some(line_num)),
                    ' ' => (LineChangeType::Unchanged, Some(line_num), Some(line_num)),
                    '\\' => (LineChangeType::NoNewline, Some(line_num), Some(line_num)),
                    _ => {
                        // Unknown line type, treat as context
                        (LineChangeType::Unchanged, Some(line_num), Some(line_num))
                    }
                }
            }
        };

        Some(Self::new(
            left_line,
            right_line,
            content,
            change_type,
            prefix,
        ))
    }

    /// Check if this line represents actual content (not metadata)
    pub fn is_content(&self) -> bool {
        matches!(
            self.change_type,
            LineChangeType::Unchanged
                | LineChangeType::Added
                | LineChangeType::Removed
                | LineChangeType::Modified
        )
    }

    /// Get the display line number for the left side
    pub fn left_display_num(&self) -> Option<String> {
        self.left_line_num.map(|n| n.to_string())
    }

    /// Get the display line number for the right side
    pub fn right_display_num(&self) -> Option<String> {
        self.right_line_num.map(|n| n.to_string())
    }

    /// Add word-level changes to this line
    pub fn with_word_changes(mut self, changes: Vec<WordChange>) -> Self {
        self.word_changes = changes;
        self
    }
}

/// Side-by-side diff representation
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct SideBySideDiff {
    /// Left side lines (old file)
    pub left_lines: Vec<DiffLine>,

    /// Right side lines (new file)
    pub right_lines: Vec<DiffLine>,

    /// File paths
    pub old_file_path: Option<String>,
    pub new_file_path: Option<String>,

    /// Whether the diff is for a binary file
    pub is_binary: bool,

    /// Total lines added
    pub lines_added: usize,

    /// Total lines removed
    pub lines_removed: usize,
}

impl SideBySideDiff {
    /// Create a new empty side-by-side diff
    pub fn new() -> Self {
        Self {
            left_lines: Vec::new(),
            right_lines: Vec::new(),
            old_file_path: None,
            new_file_path: None,
            is_binary: false,
            lines_added: 0,
            lines_removed: 0,
        }
    }

    /// Check if the diff is empty
    pub fn is_empty(&self) -> bool {
        self.left_lines.is_empty() && self.right_lines.is_empty()
    }

    /// Get the total number of lines (max of left and right)
    pub fn total_lines(&self) -> usize {
        self.left_lines.len().max(self.right_lines.len())
    }
}

/// Unified diff representation (traditional Git format)
#[derive(Debug, Clone)]
pub struct UnifiedDiff {
    /// All lines in the diff
    pub lines: Vec<DiffLine>,

    /// File paths
    pub old_file_path: Option<String>,
    pub new_file_path: Option<String>,

    /// Whether the diff is for a binary file
    pub is_binary: bool,

    /// Total lines added
    pub lines_added: usize,

    /// Total lines removed
    pub lines_removed: usize,
}

impl UnifiedDiff {
    /// Create a unified diff from raw diff text
    pub fn from_raw(diff_text: &str) -> Self {
        let mut lines = Vec::new();
        let mut lines_added = 0;
        let mut lines_removed = 0;
        let mut is_binary = false;
        let mut old_file_path = None;
        let mut new_file_path = None;

        for (i, line) in diff_text.lines().enumerate() {
            if let Some(diff_line) = DiffLine::from_unified(line, i + 1) {
                match diff_line.change_type {
                    LineChangeType::Added => lines_added += 1,
                    LineChangeType::Removed => lines_removed += 1,
                    LineChangeType::Binary => is_binary = true,
                    LineChangeType::FileHeader => {
                        // Extract file paths from file headers
                        if let Some(rest) = line.strip_prefix("--- ") {
                            old_file_path = Some(rest.trim().to_string());
                        } else if let Some(rest) = line.strip_prefix("+++ ") {
                            new_file_path = Some(rest.trim().to_string());
                        }
                    }
                    _ => {}
                }
                lines.push(diff_line);
            }
        }

        Self {
            lines,
            old_file_path,
            new_file_path,
            is_binary,
            lines_added,
            lines_removed,
        }
    }

    /// Convert unified diff to side-by-side format
    pub fn to_side_by_side(&self) -> SideBySideDiff {
        let mut side_by_side = SideBySideDiff::new();
        side_by_side.old_file_path = self.old_file_path.clone();
        side_by_side.new_file_path = self.new_file_path.clone();
        side_by_side.is_binary = self.is_binary;
        side_by_side.lines_added = self.lines_added;
        side_by_side.lines_removed = self.lines_removed;

        // Simple conversion: for now, just duplicate lines
        // A proper algorithm would align added/removed lines
        for line in &self.lines {
            match line.change_type {
                LineChangeType::Added => {
                    // Added line only appears on right side
                    side_by_side.right_lines.push(line.clone());
                    // Leave left side empty for this position
                    side_by_side.left_lines.push(DiffLine::new(
                        None,
                        None,
                        String::new(),
                        LineChangeType::Unchanged,
                        ' ',
                    ));
                }
                LineChangeType::Removed => {
                    // Removed line only appears on left side
                    side_by_side.left_lines.push(line.clone());
                    // Leave right side empty for this position
                    side_by_side.right_lines.push(DiffLine::new(
                        None,
                        None,
                        String::new(),
                        LineChangeType::Unchanged,
                        ' ',
                    ));
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
}

/// Display mode for diffs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffDisplayMode {
    /// Traditional unified diff
    Unified,

    /// Side-by-side diff
    SideBySide,

    /// Word-level diff highlighting
    WordLevel,
}

/// Configuration for diff display
#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// Display mode
    pub mode: DiffDisplayMode,

    /// Number of context lines to show around changes
    pub context_lines: usize,

    /// Whether to ignore whitespace changes
    pub ignore_whitespace: bool,

    /// Whether to show line numbers
    pub show_line_numbers: bool,

    /// Whether to wrap lines
    pub wrap_lines: bool,

    /// Whether to highlight syntax
    pub syntax_highlighting: bool,

    /// Theme for syntax highlighting
    pub theme: String,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            mode: DiffDisplayMode::Unified,
            context_lines: 3,
            ignore_whitespace: false,
            show_line_numbers: true,
            wrap_lines: false,
            syntax_highlighting: true,
            theme: "base16-ocean.dark".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_line_creation() {
        let line = DiffLine::new(
            Some(1),
            Some(2),
            "Hello, world!".to_string(),
            LineChangeType::Unchanged,
            ' ',
        );

        assert_eq!(line.left_line_num, Some(1));
        assert_eq!(line.right_line_num, Some(2));
        assert_eq!(line.content, "Hello, world!");
        assert_eq!(line.change_type, LineChangeType::Unchanged);
        assert_eq!(line.prefix, ' ');
        assert!(line.word_changes.is_empty());
        assert!(line.should_highlight);
    }

    #[test]
    fn test_diff_line_from_unified() {
        // Test added line
        let added = DiffLine::from_unified("+    added line", 1).unwrap();
        assert_eq!(added.change_type, LineChangeType::Added);
        assert_eq!(added.prefix, '+');
        assert_eq!(added.content, "    added line");
        assert_eq!(added.left_line_num, None);
        assert_eq!(added.right_line_num, Some(1));

        // Test removed line
        let removed = DiffLine::from_unified("-    removed line", 2).unwrap();
        assert_eq!(removed.change_type, LineChangeType::Removed);
        assert_eq!(removed.prefix, '-');
        assert_eq!(removed.content, "    removed line");
        assert_eq!(removed.left_line_num, Some(2));
        assert_eq!(removed.right_line_num, None);

        // Test unchanged line
        let unchanged = DiffLine::from_unified(" unchanged line", 3).unwrap();
        assert_eq!(unchanged.change_type, LineChangeType::Unchanged);
        assert_eq!(unchanged.prefix, ' ');
        assert_eq!(unchanged.content, "unchanged line");
        assert_eq!(unchanged.left_line_num, Some(3));
        assert_eq!(unchanged.right_line_num, Some(3));

        // Test hunk header
        let hunk = DiffLine::from_unified("@@ -1,3 +1,4 @@", 4).unwrap();
        assert_eq!(hunk.change_type, LineChangeType::HunkHeader);
        assert_eq!(hunk.prefix, '@');
        assert_eq!(hunk.content, "@ -1,3 +1,4 @@");

        // Test file header
        let file_header = DiffLine::from_unified("--- a/file.txt", 5).unwrap();
        assert_eq!(file_header.change_type, LineChangeType::FileHeader);
        assert_eq!(file_header.prefix, '-');
        assert_eq!(file_header.content, "-- a/file.txt");

        // Test empty line returns None
        assert!(DiffLine::from_unified("", 6).is_none());
    }

    #[test]
    fn test_diff_line_is_content() {
        assert!(
            DiffLine::new(None, None, "".to_string(), LineChangeType::Unchanged, ' ').is_content()
        );
        assert!(DiffLine::new(None, None, "".to_string(), LineChangeType::Added, '+').is_content());
        assert!(
            DiffLine::new(None, None, "".to_string(), LineChangeType::Removed, '-').is_content()
        );
        assert!(
            DiffLine::new(None, None, "".to_string(), LineChangeType::Modified, ' ').is_content()
        );
        assert!(
            !DiffLine::new(None, None, "".to_string(), LineChangeType::HunkHeader, '@')
                .is_content()
        );
        assert!(
            !DiffLine::new(None, None, "".to_string(), LineChangeType::FileHeader, '-')
                .is_content()
        );
        assert!(
            !DiffLine::new(None, None, "".to_string(), LineChangeType::Binary, ' ').is_content()
        );
        assert!(
            !DiffLine::new(None, None, "".to_string(), LineChangeType::NoNewline, '\\')
                .is_content()
        );
    }

    #[test]
    fn test_side_by_side_diff() {
        let mut diff = SideBySideDiff::new();
        assert!(diff.is_empty());
        assert_eq!(diff.total_lines(), 0);
        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 0);
        assert!(!diff.is_binary);

        // Add some lines
        diff.left_lines.push(DiffLine::new(
            Some(1),
            Some(1),
            "left".to_string(),
            LineChangeType::Unchanged,
            ' ',
        ));
        diff.right_lines.push(DiffLine::new(
            Some(1),
            Some(1),
            "right".to_string(),
            LineChangeType::Unchanged,
            ' ',
        ));

        assert!(!diff.is_empty());
        assert_eq!(diff.total_lines(), 1);
    }

    #[test]
    fn test_unified_diff_from_raw() {
        let diff_text = "\
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
 }";

        let unified = UnifiedDiff::from_raw(diff_text);
        assert_eq!(unified.lines.len(), 7); // 2 file headers, 1 hunk header, 4 content lines
        assert_eq!(unified.lines_added, 1);
        assert_eq!(unified.lines_removed, 0);
        assert!(!unified.is_binary);
        assert_eq!(
            unified.old_file_path.as_ref().map(|s| s.as_str()),
            Some("a/test.rs")
        );
        assert_eq!(
            unified.new_file_path.as_ref().map(|s| s.as_str()),
            Some("b/test.rs")
        );
    }

    #[test]
    fn test_unified_to_side_by_side() {
        let diff_text = "\
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
 }";

        let unified = UnifiedDiff::from_raw(diff_text);
        let side_by_side = unified.to_side_by_side();

        assert_eq!(
            side_by_side.left_lines.len(),
            side_by_side.right_lines.len()
        );
        assert_eq!(side_by_side.lines_added, 1);
        assert_eq!(side_by_side.lines_removed, 0);
        assert!(!side_by_side.is_binary);
    }

    #[test]
    fn test_diff_config_default() {
        let config = DiffConfig::default();
        assert_eq!(config.mode, DiffDisplayMode::Unified);
        assert_eq!(config.context_lines, 3);
        assert!(!config.ignore_whitespace);
        assert!(config.show_line_numbers);
        assert!(!config.wrap_lines);
        assert!(config.syntax_highlighting);
        assert_eq!(config.theme, "base16-ocean.dark");
    }

    #[test]
    fn test_diff_display_mode_equality() {
        assert_eq!(DiffDisplayMode::Unified, DiffDisplayMode::Unified);
        assert_ne!(DiffDisplayMode::Unified, DiffDisplayMode::SideBySide);
        assert_ne!(DiffDisplayMode::Unified, DiffDisplayMode::WordLevel);
        assert_ne!(DiffDisplayMode::SideBySide, DiffDisplayMode::WordLevel);
    }
}
