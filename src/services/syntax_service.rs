//! Syntax highlighting service for Git Gud
//!
//! This service provides syntax highlighting for various programming languages
//! using the Syntect library.

use crate::models::diff::{DiffLine, LineChangeType};
use eframe::egui;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Syntax highlighting service
pub struct SyntaxService {
    /// Syntax definitions for various languages
    syntax_set: SyntaxSet,
    
    /// Color themes
    theme_set: ThemeSet,
    
    /// Current theme (protected by RwLock for thread-safe updates)
    current_theme: RwLock<String>,
    
    /// Cache for highlighted lines to improve performance
    highlight_cache: Mutex<LruCache<String, egui::text::LayoutJob>>,
}

impl SyntaxService {
    /// Create a new syntax highlighting service
    pub fn new() -> Self {
        // Load syntax definitions and themes
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        Self {
            syntax_set,
            theme_set,
            current_theme: RwLock::new("base16-ocean.dark".to_string()),
            highlight_cache: Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
        }
    }
    
    /// Get the current theme
    pub fn current_theme(&self) -> Theme {
        let theme_name = self.current_theme.read().unwrap().clone();
        self.theme_set
            .themes
            .get(&theme_name)
            .unwrap_or_else(|| self.theme_set.themes.values().next().unwrap())
            .clone()
    }
    
    /// Set the current theme
    pub fn set_theme(&self, theme_name: &str) -> bool {
        if self.theme_set.themes.contains_key(theme_name) {
            *self.current_theme.write().unwrap() = theme_name.to_string();
            // Clear cache when theme changes
            self.highlight_cache.lock().unwrap().clear();
            true
        } else {
            false
        }
    }
    
    /// Get available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }
    
    /// Get current theme name
    pub fn current_theme_name(&self) -> String {
        self.current_theme.read().unwrap().clone()
    }
    
    /// Detect syntax from file extension
    pub fn detect_syntax(&self, path: &Path) -> Option<&SyntaxReference> {
        let extension = path.extension()?.to_str()?;
        
        // Try to find syntax by extension
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(extension) {
            return Some(syntax);
        }
        
        // Fallback: try to find by file name
        let file_name = path.file_name()?.to_str()?;
        self.syntax_set.find_syntax_by_token(file_name)
    }
    
    /// Detect syntax from file name (for diff headers)
    pub fn detect_syntax_from_name(&self, file_name: &str) -> Option<&SyntaxReference> {
        // Extract extension from file name
        if let Some(dot_pos) = file_name.rfind('.') {
            let extension = &file_name[dot_pos + 1..];
            if let Some(syntax) = self.syntax_set.find_syntax_by_extension(extension) {
                return Some(syntax);
            }
        }
        
        // Try to find by file name
        self.syntax_set.find_syntax_by_token(file_name)
    }
    
    /// Convert Syntect style to egui color
    fn syntect_to_egui_color(&self, color: syntect::highlighting::Color) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
    }
    
    /// Highlight a single line of code
    pub fn highlight_line(
        &self,
        line: &str,
        syntax: Option<&SyntaxReference>,
    ) -> egui::text::LayoutJob {
        // Check cache first
        let cache_key = format!("{}:{:?}", line, syntax.map(|s| s.name.as_str()));
        {
            let mut cache = self.highlight_cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }
        
        let theme = self.current_theme();
        let mut job = egui::text::LayoutJob::default();
        
        if let Some(syntax) = syntax {
            // Use syntax highlighting
            let mut highlighter = HighlightLines::new(syntax, &theme);
            
            // Syntect can highlight lines without newlines
            if let Ok(regions) = highlighter.highlight_line(line, &self.syntax_set) {
                for (style, text) in regions {
                    let color = self.syntect_to_egui_color(style.foreground);
                    let format = egui::TextFormat::simple(
                        egui::FontId::monospace(12.0),
                        color,
                    );
                    job.append(text, 0.0, format);
                }
            } else {
                // Fallback: plain text
                job.append(line, 0.0, egui::TextFormat::simple(
                    egui::FontId::monospace(12.0),
                    egui::Color32::WHITE,
                ));
            }
        } else {
            // No syntax highlighting, use plain text
            job.append(line, 0.0, egui::TextFormat::simple(
                egui::FontId::monospace(12.0),
                egui::Color32::WHITE,
            ));
        }
        
        // Cache the result
        {
            let mut cache = self.highlight_cache.lock().unwrap();
            cache.put(cache_key, job.clone());
        }
        
        job
    }
    
    /// Highlight a diff line with appropriate colors
    pub fn highlight_diff_line(&self, diff_line: &DiffLine, syntax: Option<&SyntaxReference>) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        
        // Determine base color based on diff type
        let base_color = match diff_line.change_type {
            LineChangeType::Added => egui::Color32::DARK_GREEN,
            LineChangeType::Removed => egui::Color32::DARK_RED,
            LineChangeType::HunkHeader => egui::Color32::BLUE,
            LineChangeType::FileHeader => egui::Color32::GRAY,
            LineChangeType::Binary => egui::Color32::YELLOW,
            LineChangeType::NoNewline => egui::Color32::LIGHT_GRAY,
            _ => egui::Color32::WHITE, // Unchanged, Modified
        };
        
        // Add prefix if present (for unified diffs)
        if diff_line.prefix != ' ' {
            job.append(
                &diff_line.prefix.to_string(),
                0.0,
                egui::TextFormat::simple(
                    egui::FontId::monospace(12.0),
                    base_color,
                ),
            );
        }
        
        if diff_line.should_highlight && syntax.is_some() {
            // Use syntax highlighting for content
            let highlighted = self.highlight_line(&diff_line.content, syntax);
            // Merge the highlighted job into our job
            for section in highlighted.sections {
                // Extract text from the original line content
                // Handle potential byte range issues
                let byte_range = section.byte_range.clone();
                if byte_range.end <= diff_line.content.len() {
                    let text = &diff_line.content[byte_range];
                    job.append(text, section.leading_space, section.format.clone());
                } else {
                    // Fallback: use the full line
                    job.append(&diff_line.content, section.leading_space, section.format.clone());
                }
            }
        } else {
            // Use base color for content
            job.append(
                &diff_line.content,
                0.0,
                egui::TextFormat::simple(
                    egui::FontId::monospace(12.0),
                    base_color,
                ),
            );
        }
        
        job
    }
    
    /// Highlight multiple lines (batch operation for performance)
    pub fn highlight_lines(
        &self,
        lines: &[String],
        syntax: Option<&SyntaxReference>,
    ) -> Vec<egui::text::LayoutJob> {
        lines
            .iter()
            .map(|line| self.highlight_line(line, syntax))
            .collect()
    }
    
    /// Highlight diff lines with syntax detection from file path
    pub fn highlight_diff_lines_with_path(
        &self,
        diff_lines: &[DiffLine],
        file_path: Option<&Path>,
    ) -> Vec<egui::text::LayoutJob> {
        let syntax = file_path.and_then(|p| self.detect_syntax(p));
        
        diff_lines
            .iter()
            .map(|line| self.highlight_diff_line(line, syntax))
            .collect()
    }
    
    /// Highlight diff lines with syntax detection from file name
    pub fn highlight_diff_lines_with_name(
        &self,
        diff_lines: &[DiffLine],
        file_name: Option<&str>,
    ) -> Vec<egui::text::LayoutJob> {
        let syntax = file_name.and_then(|name| self.detect_syntax_from_name(name));
        
        diff_lines
            .iter()
            .map(|line| self.highlight_diff_line(line, syntax))
            .collect()
    }
    
    /// Clear the highlight cache
    pub fn clear_cache(&self) {
        self.highlight_cache.lock().unwrap().clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.highlight_cache.lock().unwrap();
        (cache.len(), cache.cap().get())
    }
}

impl Default for SyntaxService {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared syntax service for thread-safe access
pub type SharedSyntaxService = Arc<SyntaxService>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_syntax_service_creation() {
        let service = SyntaxService::new();
        assert!(!service.available_themes().is_empty());
        assert!(!service.current_theme_name().is_empty());
    }
    
    #[test]
    fn test_theme_switching() {
        let service = SyntaxService::new();
        let themes = service.available_themes();
        
        if themes.len() > 1 {
            let new_theme = &themes[1];
            assert!(service.set_theme(new_theme));
            assert_eq!(service.current_theme_name(), *new_theme);
        }
    }
    
    #[test]
    fn test_syntax_detection() {
        let service = SyntaxService::new();
        
        // Test Rust file
        let rust_path = PathBuf::from("test.rs");
        assert!(service.detect_syntax(&rust_path).is_some());
        
        // Test Python file
        let python_path = PathBuf::from("test.py");
        assert!(service.detect_syntax(&python_path).is_some());
        
        // Test JavaScript file
        let js_path = PathBuf::from("test.js");
        assert!(service.detect_syntax(&js_path).is_some());
        
        // Test unknown extension
        let unknown_path = PathBuf::from("test.unknown");
        assert!(service.detect_syntax(&unknown_path).is_none());
    }
    
    #[test]
    fn test_highlight_line() {
        let service = SyntaxService::new();
        
        // Test Rust code highlighting
        let rust_code = "fn main() { println!(\"Hello\"); }";
        let rust_syntax = service.detect_syntax_from_name("test.rs");
        assert!(rust_syntax.is_some());
        
        let job = service.highlight_line(rust_code, rust_syntax);
        assert!(!job.sections.is_empty());
        
        // Test plain text (no syntax)
        let plain_text = "This is plain text";
        let job = service.highlight_line(plain_text, None);
        assert_eq!(job.sections.len(), 1);
    }
    
    #[test]
    fn test_cache_functionality() {
        let service = SyntaxService::new();
        
        // First call should populate cache
        let line = "test line";
        let _job1 = service.highlight_line(line, None);
        let (len1, _cap) = service.cache_stats();
        assert_eq!(len1, 1);
        
        // Second call should use cache
        let _job2 = service.highlight_line(line, None);
        let (len2, _) = service.cache_stats();
        assert_eq!(len2, 1); // Same cache entry
        
        // Clear cache
        service.clear_cache();
        let (len3, _) = service.cache_stats();
        assert_eq!(len3, 0);
    }
    
    #[test]
    fn test_diff_line_highlighting() {
        let service = SyntaxService::new();
        
        // Test added line
        let added_line = DiffLine::new(
            None,
            Some(1),
            "let x = 5;".to_string(),
            LineChangeType::Added,
            '+',
        );
        
        let job = service.highlight_diff_line(&added_line, None);
        assert!(!job.sections.is_empty());
        
        // Test removed line
        let removed_line = DiffLine::new(
            Some(1),
            None,
            "let y = 10;".to_string(),
            LineChangeType::Removed,
            '-',
        );
        
        let job = service.highlight_diff_line(&removed_line, None);
        assert!(!job.sections.is_empty());
        
        // Test hunk header
        let hunk_line = DiffLine::new(
            Some(1),
            Some(1),
            "@@ -1,3 +1,4 @@".to_string(),
            LineChangeType::HunkHeader,
            '@',
        );
        
        let job = service.highlight_diff_line(&hunk_line, None);
        assert!(!job.sections.is_empty());
    }
}