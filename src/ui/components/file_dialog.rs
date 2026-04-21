//! File dialog component for Git Gud
//!
//! This component provides native file dialogs for opening repositories.

use std::path::PathBuf;

/// File dialog utilities
pub struct FileDialog;

impl FileDialog {
    /// Open a native file dialog to select a directory (for Git repositories)
    pub fn open_directory() -> Option<PathBuf> {
        let dialog = rfd::FileDialog::new()
            .set_title("Select Git Repository")
            .pick_folder();
        
        dialog
    }
    
    /// Open a native file dialog to select a file
    pub fn open_file() -> Option<PathBuf> {
        let dialog = rfd::FileDialog::new()
            .set_title("Select File")
            .pick_file();
        
        dialog
    }
    
    /// Save file dialog
    pub fn save_file() -> Option<PathBuf> {
        let dialog = rfd::FileDialog::new()
            .set_title("Save File")
            .save_file();
        
        dialog
    }
}