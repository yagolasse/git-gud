//! Application state management for Git Gud
//!
//! This module manages the global application state, including
//! repository data, UI state, and user preferences.

mod app_state;
mod repository_state;
mod ui_state;

pub use app_state::{AppState, SharedAppState};
pub use repository_state::RepositoryState;
pub use ui_state::{UIState, PendingAction};