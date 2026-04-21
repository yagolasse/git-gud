//! Integration tests for diff viewing features

use crate::models::diff::{DiffConfig, DiffDisplayMode};
use crate::services::{DiffParser, SyntaxService};
use crate::ui::components::EnhancedDiffViewer;
use std::sync::Arc;

/// Test that syntax highlighting service works with diff parser
#[test]
fn test_syntax_highlighting_with_diff() {
    let syntax_service = Arc::new(SyntaxService::new());
    let diff_parser = DiffParser::new();
    
    // Create a sample diff
    let diff_text = "\
--- a/test.rs
+++ b/test.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
 }
";
    
    // Parse the diff
    let unified = diff_parser.parse_unified(diff_text);
    assert_eq!(unified.lines.len(), 7);
    assert_eq!(unified.lines_added, 1);
    assert_eq!(unified.lines_removed, 0);
    
    // Convert to side-by-side
    let side_by_side = diff_parser.unified_to_side_by_side(&unified);
    assert_eq!(side_by_side.left_lines.len(), side_by_side.right_lines.len());
    
    // Test syntax detection
    let rust_syntax = syntax_service.detect_syntax_from_name("test.rs");
    assert!(rust_syntax.is_some());
    
    // Test highlighting
    let line = &unified.lines[3]; // " fn main() {"
    let job = syntax_service.highlight_diff_line(line, rust_syntax);
    assert!(!job.sections.is_empty());
}

/// Test enhanced diff viewer creation and configuration
#[test]
fn test_enhanced_diff_viewer() {
    let syntax_service = Arc::new(SyntaxService::new());
    let viewer = EnhancedDiffViewer::with_syntax_service(syntax_service);
    
    // Check default configuration
    let config = viewer.config();
    assert_eq!(config.mode, DiffDisplayMode::Unified);
    assert!(config.show_line_numbers);
    assert!(!config.wrap_lines);
    assert!(config.syntax_highlighting);
    
    // Test available themes
    let themes = viewer.syntax_service().available_themes();
    assert!(!themes.is_empty());
}

/// Test virtual scrolling state management
#[test]
fn test_virtual_scroll_integration() {
    use crate::ui::components::VirtualScroll;
    
    // Create virtual scroll with 100 items
    let mut scroll = VirtualScroll::with_uniform_height("test", 100, 20.0);
    
    // Test initial state
    let state = scroll.state();
    assert_eq!(state.total_items, 100);
    assert_eq!(state.total_height, 2000.0);
    
    // Test scrolling
    scroll.scroll_by(100.0);
    assert!(!scroll.state().at_top());
    
    scroll.scroll_to_top();
    assert!(scroll.state().at_top());
    
    scroll.scroll_to_bottom();
    assert!(scroll.state().at_bottom());
}

/// Test diff line parsing and classification
#[test]
fn test_diff_line_parsing() {
    use crate::models::diff::{DiffLine, LineChangeType};
    
    // Test added line
    let added_line = DiffLine::from_unified("+    println!(\"Hello\");", 1);
    assert!(added_line.is_some());
    let added_line = added_line.unwrap();
    assert_eq!(added_line.change_type, LineChangeType::Added);
    assert_eq!(added_line.prefix, '+');
    assert_eq!(added_line.content, "    println!(\"Hello\");");
    
    // Test removed line
    let removed_line = DiffLine::from_unified("-    old_code();", 2);
    assert!(removed_line.is_some());
    let removed_line = removed_line.unwrap();
    assert_eq!(removed_line.change_type, LineChangeType::Removed);
    assert_eq!(removed_line.prefix, '-');
    
    // Test unchanged line
    let unchanged_line = DiffLine::from_unified(" fn main() {", 3);
    assert!(unchanged_line.is_some());
    let unchanged_line = unchanged_line.unwrap();
    assert_eq!(unchanged_line.change_type, LineChangeType::Unchanged);
    assert_eq!(unchanged_line.prefix, ' ');
    
    // Test hunk header
    let hunk_line = DiffLine::from_unified("@@ -1,3 +1,4 @@", 4);
    assert!(hunk_line.is_some());
    let hunk_line = hunk_line.unwrap();
    assert_eq!(hunk_line.change_type, LineChangeType::HunkHeader);
    assert_eq!(hunk_line.prefix, '@');
    
    // Test file header
    let file_line = DiffLine::from_unified("--- a/test.rs", 5);
    assert!(file_line.is_some());
    let file_line = file_line.unwrap();
    assert_eq!(file_line.change_type, LineChangeType::FileHeader);
    
    // Test empty line (should return None)
    let empty_line = DiffLine::from_unified("", 6);
    assert!(empty_line.is_none());
}

/// Test theme switching
#[test]
fn test_theme_switching_integration() {
    let syntax_service = Arc::new(SyntaxService::new());
    let themes = syntax_service.available_themes();
    
    if themes.len() > 1 {
        let first_theme = &themes[0];
        let second_theme = &themes[1];
        
        // Switch to second theme
        assert!(syntax_service.set_theme(second_theme));
        assert_eq!(syntax_service.current_theme_name(), *second_theme);
        
        // Switch back to first theme
        assert!(syntax_service.set_theme(first_theme));
        assert_eq!(syntax_service.current_theme_name(), *first_theme);
        
        // Try to switch to non-existent theme (should fail)
        assert!(!syntax_service.set_theme("non-existent-theme"));
        assert_eq!(syntax_service.current_theme_name(), *first_theme);
    }
}

/// Test cache functionality
#[test]
fn test_cache_integration() {
    let syntax_service = Arc::new(SyntaxService::new());
    
    // First call should populate cache
    let line = "fn main() { println!(\"test\"); }";
    let syntax = syntax_service.detect_syntax_from_name("test.rs");
    let _job1 = syntax_service.highlight_line(line, syntax);
    let (len1, _) = syntax_service.cache_stats();
    assert_eq!(len1, 1);
    
    // Same call should use cache
    let _job2 = syntax_service.highlight_line(line, syntax);
    let (len2, _) = syntax_service.cache_stats();
    assert_eq!(len2, 1); // Cache hit, size unchanged
    
    // Different line should add to cache
    let line2 = "let x = 5;";
    let _job3 = syntax_service.highlight_line(line2, syntax);
    let (len3, _) = syntax_service.cache_stats();
    assert_eq!(len3, 2); // New entry added
    
    // Clear cache
    syntax_service.clear_cache();
    let (len4, _) = syntax_service.cache_stats();
    assert_eq!(len4, 0);
}