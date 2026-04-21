//! Git Gud - A modular Git GUI application
//!
//! This library provides the core functionality for the Git Gud application,
//! including Git operations, repository management, and logging utilities.

pub mod models;
pub mod services;
pub mod state;
pub mod ui;

#[cfg(test)]
mod tests;

/// Re-exports for common types
pub use anyhow::Result;

/// Initialize the application with default settings
pub fn init() -> Result<()> {
    log::info!("Initializing Git Gud library");
    Ok(())
}

/// Cleanup resources before application shutdown
pub fn shutdown() -> Result<()> {
    log::info!("Shutting down Git Gud library");
    Ok(())
}