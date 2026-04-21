//! Diff parser service for Git Gud
//!
//! This service provides parsing and conversion of diffs between different formats.

use crate::models::diff::{SideBySideDiff, UnifiedDiff};
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
                "swift", "kt", "scala", "m", "mm", "r", "pl", "pm", "t", "lua", "sql", "sh", "bash",
                "zsh", "fish", "ps1", "bat", "cmd",
                // Markup
                "md", "markdown", "txt", "text", "rst", "asciidoc", "adoc", "org", "tex", "latex",
                "html", "htm", "xml", "json", "yaml", "yml", "toml", "ini", "cfg", "conf", "csv",
                "tsv", "log",
                // Config
                "toml", "yaml", "yml", "json", "xml", "ini", "cfg", "conf",
            ];
            
            return text_extensions.contains(&ext_str.as_str());
        }
        
        // If no extension, assume it might be text
        true
    }
}

impl Default for DiffParser {
    fn default() -> Self {
        Self::new()
    }
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
        
        assert_eq!(side_by_side.left_lines.len(), side_by_side.right_lines.len());
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