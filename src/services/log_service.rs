//! Logging service for Git Gud application
//!
//! This module provides centralized logging configuration
//! with plain text timestamps format.

use std::path::Path;
use anyhow::Result;

/// Logging service for configuring application-wide logging
pub struct LogService;

impl LogService {
    /// Initialize logging with default configuration
    pub fn init() -> Result<()> {
        Self::init_with_level(log::LevelFilter::Info, None)
    }
    
    /// Initialize logging with custom level and optional log file
    pub fn init_with_level(level: log::LevelFilter, log_file: Option<&Path>) -> Result<()> {
        use env_logger::{Builder, Target};
        use std::fs::OpenOptions;
        
        let mut builder = Builder::new();
        
        builder.filter_level(level);
        
        // Configure plain text format with timestamps
        builder.format_timestamp_secs();
        builder.format_module_path(false);
        builder.format_target(false);
        
        // Set output target
        if let Some(log_file) = log_file {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?;
            builder.target(Target::Pipe(Box::new(file)));
        }
        
        builder.init();
        
        log::info!("Logging initialized with level: {:?}", level);
        if let Some(log_file) = log_file {
            log::info!("Logging to file: {:?}", log_file);
        }
        
        Ok(())
    }
    
    /// Log a Git operation with parameters
    pub fn log_git_operation(operation: &str, params: &[(&str, &str)]) {
        let params_str = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        
        log::info!("Git operation: {} ({})", operation, params_str);
    }
    
    /// Log a UI interaction
    pub fn log_ui_interaction(component: &str, action: &str, details: Option<&str>) {
        if let Some(details) = details {
            log::info!("UI interaction: {}.{} - {}", component, action, details);
        } else {
            log::info!("UI interaction: {}.{}", component, action);
        }
    }
    
    /// Log an error with context
    pub fn log_error(error: &anyhow::Error, context: &str) {
        log::error!("{}: {}", context, error);
        
        // Log chain of errors in debug mode
        if log::log_enabled!(log::Level::Debug) {
            for cause in error.chain().skip(1) {
                log::debug!("Caused by: {}", cause);
            }
        }
    }
}