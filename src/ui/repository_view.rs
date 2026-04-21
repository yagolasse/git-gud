//! Repository view for Git Gud application
//!
//! This module contains the repository browser UI.

use eframe::egui;

/// Repository browser view
pub struct RepositoryView;

impl RepositoryView {
    /// Create a new repository view
    pub fn new() -> Self {
        Self
    }
    
    /// Show the repository view
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("Repository View");
    }
}