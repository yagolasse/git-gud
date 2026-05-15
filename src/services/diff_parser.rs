//! Diff parser service for Git Gud
//!
//! This service provides parsing and conversion of diffs between different formats.

use crate::models::diff::{
    DiffLine, LineChangeType, SideBySideDiff, UnifiedDiff, WordChange, WordChangeType,
};
use std::collections::HashSet;
use std::path::Path;

/// Diff parser service
pub struct DiffParser;

impl DiffParser {
    /// Create a new diff parser
    pub fn new() -> Self {
        Self
    }

    /// Parse unified diff text into structured format
    pub fn parse_unified(&self, diff_text: &str) -> UnifiedDiff {
        UnifiedDiff::from_raw(diff_text)
    }

    /// Convert unified diff to side-by-side format
    pub fn unified_to_side_by_side(&self, unified_diff: &UnifiedDiff) -> SideBySideDiff {
        unified_diff.to_side_by_side()
    }

    /// Parse unified diff text and convert to side-by-side format
    pub fn parse_to_side_by_side(&self, diff_text: &str) -> SideBySideDiff {
        let unified = self.parse_unified(diff_text);
        self.unified_to_side_by_side(&unified)
    }

    /// Detect if a file is likely to be a text file (for syntax highlighting)
    pub fn is_likely_text_file(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            // Common text file extensions
            let text_extensions = [
                // Source code
                "rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "cs", "go", "rb", "php",
                "swift", "kt", "scala", "m", "mm", "r", "pl", "pm", "t", "lua", "sql", "sh",
                "bash", "zsh", "fish", "ps1", "bat", "cmd", // Markup
                "md", "markdown", "txt", "text", "rst", "asciidoc", "adoc", "org", "tex", "latex",
                "html", "htm", "xml", "json", "yaml", "yml", "toml", "ini", "cfg", "conf", "csv",
                "tsv", "log", // Config
                "toml", "yaml", "yml", "json", "xml", "ini", "cfg", "conf",
            ];

            return text_extensions.contains(&ext_str.as_str());
        }

        // If no extension, assume it might be text
        true
    }

    /// Post-process parsed diff lines to populate word-level changes on paired Removed/Added runs.
    pub fn apply_word_diffs(&self, lines: &mut [DiffLine]) {
        let mut i = 0;
        while i < lines.len() {
            let rem_start = i;
            while i < lines.len() && lines[i].change_type == LineChangeType::Removed {
                i += 1;
            }
            let rem_end = i;

            let add_start = i;
            while i < lines.len() && lines[i].change_type == LineChangeType::Added {
                i += 1;
            }
            let add_end = i;

            let n_pairs = (rem_end - rem_start).min(add_end - add_start);
            if n_pairs > 0 {
                let pairs: Vec<(String, String)> = (0..n_pairs)
                    .map(|k| {
                        (
                            lines[rem_start + k].content.clone(),
                            lines[add_start + k].content.clone(),
                        )
                    })
                    .collect();
                for (k, (rem_content, add_content)) in pairs.iter().enumerate() {
                    let (rw, aw) = compute_word_changes(rem_content, add_content);
                    lines[rem_start + k].word_changes = rw;
                    lines[add_start + k].word_changes = aw;
                }
            }

            if rem_end == rem_start && add_end == add_start {
                i += 1;
            }
        }
    }
}

impl Default for DiffParser {
    fn default() -> Self {
        Self::new()
    }
}

fn tokenize(s: &str) -> Vec<(usize, usize)> {
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let (byte_start, c) = chars[i];
        if c.is_alphanumeric() || c == '_' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].1.is_alphanumeric() || chars[j].1 == '_') {
                j += 1;
            }
            let byte_end = if j < chars.len() { chars[j].0 } else { s.len() };
            tokens.push((byte_start, byte_end));
            i = j;
        } else if c.is_whitespace() {
            let mut j = i + 1;
            while j < chars.len() && chars[j].1.is_whitespace() {
                j += 1;
            }
            let byte_end = if j < chars.len() { chars[j].0 } else { s.len() };
            tokens.push((byte_start, byte_end));
            i = j;
        } else {
            let byte_end = if i + 1 < chars.len() { chars[i + 1].0 } else { s.len() };
            tokens.push((byte_start, byte_end));
            i += 1;
        }
    }
    tokens
}

fn lcs_matched(a: &[&str], b: &[&str]) -> (HashSet<usize>, HashSet<usize>) {
    let m = a.len();
    let n = b.len();
    if m > 200 || n > 200 {
        return (HashSet::new(), HashSet::new());
    }
    let mut dp = vec![vec![0u16; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }
    let mut a_matched = HashSet::new();
    let mut b_matched = HashSet::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            a_matched.insert(i - 1);
            b_matched.insert(j - 1);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    (a_matched, b_matched)
}

pub fn compute_word_changes(removed: &str, added: &str) -> (Vec<WordChange>, Vec<WordChange>) {
    let rem_tok = tokenize(removed);
    let add_tok = tokenize(added);
    let rem_strs: Vec<&str> = rem_tok.iter().map(|&(s, e)| &removed[s..e]).collect();
    let add_strs: Vec<&str> = add_tok.iter().map(|&(s, e)| &added[s..e]).collect();
    let (rem_matched, add_matched) = lcs_matched(&rem_strs, &add_strs);
    let rem_changes = rem_tok
        .iter()
        .enumerate()
        .filter(|(i, _)| !rem_matched.contains(i))
        .map(|(_, &(s, e))| WordChange { start: s, end: e, change_type: WordChangeType::Removed })
        .collect();
    let add_changes = add_tok
        .iter()
        .enumerate()
        .filter(|(j, _)| !add_matched.contains(j))
        .map(|(_, &(s, e))| WordChange { start: s, end: e, change_type: WordChangeType::Added })
        .collect();
    (rem_changes, add_changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unified_diff() {
        let parser = DiffParser::new();

        let diff_text = "\
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
 }
";

        let unified = parser.parse_unified(diff_text);
        // Should have: 2 file headers, 1 hunk header, 4 content lines = 7 total
        // But file headers are now correctly identified as FileHeader type
        assert_eq!(unified.lines.len(), 7);
        assert_eq!(unified.lines_added, 1);
        assert_eq!(unified.lines_removed, 0);
        assert!(!unified.is_binary);
    }

    #[test]
    fn test_unified_to_side_by_side() {
        let parser = DiffParser::new();

        let diff_text = "\
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
 }
";

        let unified = parser.parse_unified(diff_text);
        let side_by_side = parser.unified_to_side_by_side(&unified);

        assert_eq!(
            side_by_side.left_lines.len(),
            side_by_side.right_lines.len()
        );
        assert_eq!(side_by_side.lines_added, 1);
        assert_eq!(side_by_side.lines_removed, 0);
    }

    #[test]
    fn test_is_likely_text_file() {
        let parser = DiffParser::new();

        // Text files
        assert!(parser.is_likely_text_file(Path::new("test.rs")));
        assert!(parser.is_likely_text_file(Path::new("test.py")));
        assert!(parser.is_likely_text_file(Path::new("test.js")));
        assert!(parser.is_likely_text_file(Path::new("README.md")));
        assert!(parser.is_likely_text_file(Path::new("config.toml")));

        // Unknown extension (not in our list, assume not text for syntax highlighting)
        assert!(!parser.is_likely_text_file(Path::new("test.unknown")));

        // No extension (assume text)
        assert!(parser.is_likely_text_file(Path::new("Makefile")));
    }
}
