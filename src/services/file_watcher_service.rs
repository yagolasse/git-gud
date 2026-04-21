//! File watcher service for Git Gud
//!
//! This service monitors repository directories for changes
//! and triggers refreshes when files are modified.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// File watcher service for monitoring repository changes
pub struct FileWatcherService {
    /// Channel sender for file change events
    event_sender: Sender<notify::Result<Event>>,
    
    /// Channel receiver for file change events
    event_receiver: Receiver<notify::Result<Event>>,
    
    /// The file system watcher
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    
    /// Whether the watcher is currently active
    is_watching: Arc<Mutex<bool>>,
    
    /// Debounce timer to avoid excessive refreshes
    last_event_time: Arc<Mutex<std::time::Instant>>,
    
    /// Minimum time between refresh triggers (debounce)
    debounce_interval: Duration,
}

impl FileWatcherService {
    /// Create a new file watcher service
    pub fn new() -> Self {
        let (event_sender, event_receiver) = channel();
        
        Self {
            event_sender,
            event_receiver,
            watcher: Arc::new(Mutex::new(None)),
            is_watching: Arc::new(Mutex::new(false)),
            last_event_time: Arc::new(Mutex::new(std::time::Instant::now())),
            debounce_interval: Duration::from_millis(500), // 500ms debounce
        }
    }
    
    /// Start watching a repository directory
    pub fn start_watching(&mut self, path: &Path) -> anyhow::Result<()> {
        log::info!("Starting file watcher for: {:?}", path);
        
        // Stop any existing watcher
        self.stop_watching();
        
        // Create new watcher
        let mut watcher = RecommendedWatcher::new(
            self.event_sender.clone(),
            Config::default()
                .with_poll_interval(Duration::from_secs(1)) // Poll every second
                .with_compare_contents(true), // Compare file contents
        )?;
        
        // Watch the repository directory recursively
        watcher.watch(path, RecursiveMode::Recursive)?;
        
        // Store the watcher
        *self.watcher.lock().unwrap() = Some(watcher);
        *self.is_watching.lock().unwrap() = true;
        
        log::debug!("File watcher started successfully");
        Ok(())
    }
    
    /// Stop watching the current directory
    pub fn stop_watching(&mut self) {
        log::debug!("Stopping file watcher");
        
        if let Some(watcher) = self.watcher.lock().unwrap().take() {
            // Watcher will be dropped automatically, which stops it
            drop(watcher);
        }
        
        *self.is_watching.lock().unwrap() = false;
    }
    
    /// Check if there are file change events that should trigger a refresh
    pub fn should_refresh(&mut self) -> bool {
        let mut should_refresh = false;
        let now = std::time::Instant::now();
        
        // Check for any file change events
        while let Ok(event_result) = self.event_receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    // Check if this is a relevant file change event
                    if Self::is_relevant_event(&event) {
                        log::debug!("File change detected: {:?}", event.paths);
                        
                        // Check debounce interval
                        let mut last_event_time = self.last_event_time.lock().unwrap();
                        if now.duration_since(*last_event_time) > self.debounce_interval {
                            should_refresh = true;
                            *last_event_time = now;
                        }
                    }
                }
                Err(e) => {
                    log::error!("File watcher error: {}", e);
                }
            }
        }
        
        should_refresh
    }
    
    /// Check if an event is relevant for triggering a refresh
    fn is_relevant_event(event: &Event) -> bool {
        // We're interested in file modifications, creations, deletions, and renames
        match event.kind {
            EventKind::Create(_) |
            EventKind::Modify(_) |
            EventKind::Remove(_) => true,
            _ => false,
        }
    }
    
    /// Check if the watcher is currently active
    pub fn is_watching(&self) -> bool {
        *self.is_watching.lock().unwrap()
    }
    
    /// Set the debounce interval (minimum time between refresh triggers)
    pub fn set_debounce_interval(&mut self, interval: Duration) {
        self.debounce_interval = interval;
    }
}

impl Default for FileWatcherService {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe file watcher that can be shared between threads
pub struct SharedFileWatcher {
    inner: Arc<Mutex<FileWatcherService>>,
}

impl SharedFileWatcher {
    /// Create a new shared file watcher
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(FileWatcherService::new())),
        }
    }
    
    /// Start watching a repository directory
    pub fn start_watching(&self, path: &Path) -> anyhow::Result<()> {
        self.inner.lock().unwrap().start_watching(path)
    }
    
    /// Stop watching the current directory
    pub fn stop_watching(&self) {
        self.inner.lock().unwrap().stop_watching()
    }
    
    /// Check if there are file change events that should trigger a refresh
    pub fn should_refresh(&self) -> bool {
        self.inner.lock().unwrap().should_refresh()
    }
    
    /// Check if the watcher is currently active
    pub fn is_watching(&self) -> bool {
        self.inner.lock().unwrap().is_watching()
    }
}

impl Default for SharedFileWatcher {
    fn default() -> Self {
        Self::new()
    }
}