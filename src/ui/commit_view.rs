//! Commit view for Git Gud application
//!
//! This module contains the commit history viewer UI.

use eframe::egui;

/// Commit history viewer
pub struct CommitView;

impl CommitView {
    /// Create a new commit view
    pub fn new() -> Self {
        Self
    }
    
    /// Show the commit view
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Commit View");
    }
}