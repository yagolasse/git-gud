//! Virtual scrolling component for Git Gud
//!
//! This component provides efficient scrolling for large lists by only
//! rendering visible items.

use eframe::egui;
use std::ops::Range;

/// Virtual scrolling state
#[derive(Debug, Clone)]
pub struct VirtualScrollState {
    /// Total number of items
    pub total_items: usize,

    /// Height of each item (if uniform)
    pub item_height: Option<f32>,

    /// Cached item heights (for variable height items)
    pub item_heights: Vec<f32>,

    /// Current scroll offset
    pub scroll_offset: f32,

    /// Viewport height
    pub viewport_height: f32,

    /// Visible range of items
    pub visible_range: Range<usize>,

    /// Total height of all items
    pub total_height: f32,
}

impl VirtualScrollState {
    /// Create a new virtual scroll state
    pub fn new(total_items: usize) -> Self {
        Self {
            total_items,
            item_height: None,
            item_heights: Vec::new(),
            scroll_offset: 0.0,
            viewport_height: 0.0,
            visible_range: 0..0,
            total_height: 0.0,
        }
    }

    /// Create a new virtual scroll state with uniform item height
    pub fn with_uniform_height(total_items: usize, item_height: f32) -> Self {
        let total_height = total_items as f32 * item_height;
        Self {
            total_items,
            item_height: Some(item_height),
            item_heights: Vec::new(),
            scroll_offset: 0.0,
            viewport_height: 0.0,
            visible_range: 0..0,
            total_height,
        }
    }

    /// Update the viewport and calculate visible range
    pub fn update_viewport(&mut self, viewport_height: f32) {
        self.viewport_height = viewport_height;

        if self.total_items == 0 {
            self.visible_range = 0..0;
            return;
        }

        if let Some(item_height) = self.item_height {
            // Uniform height items
            self.total_height = self.total_items as f32 * item_height;

            let start_idx = (self.scroll_offset / item_height).floor() as usize;
            let end_idx =
                ((self.scroll_offset + viewport_height) / item_height).ceil() as usize + 1;

            self.visible_range = start_idx.max(0)..end_idx.min(self.total_items);
        } else {
            // Variable height items
            self.calculate_variable_visible_range();
        }
    }

    /// Calculate visible range for variable height items
    fn calculate_variable_visible_range(&mut self) {
        if self.item_heights.len() != self.total_items {
            // Initialize with estimated heights if not available
            self.item_heights = vec![20.0; self.total_items];
        }

        // Calculate total height
        self.total_height = self.item_heights.iter().sum();

        // Find start index
        let mut accumulated = 0.0;
        let mut start_idx = 0;
        for (i, &height) in self.item_heights.iter().enumerate() {
            if accumulated + height > self.scroll_offset {
                start_idx = i;
                break;
            }
            accumulated += height;
        }

        // Find end index
        let mut end_idx = start_idx;
        let target_height = self.scroll_offset + self.viewport_height;
        for (i, &height) in self.item_heights.iter().enumerate().skip(start_idx) {
            if accumulated > target_height {
                break;
            }
            accumulated += height;
            end_idx = i + 1;
        }

        self.visible_range = start_idx..end_idx.min(self.total_items);
    }

    /// Set item height for variable height items
    pub fn set_item_height(&mut self, index: usize, height: f32) {
        if index < self.item_heights.len() {
            self.item_heights[index] = height;
        } else if index < self.total_items {
            // Resize and fill with default height
            self.item_heights.resize(self.total_items, 20.0);
            self.item_heights[index] = height;
        }
    }

    /// Scroll to a specific item
    pub fn scroll_to_item(&mut self, index: usize) {
        if index >= self.total_items {
            return;
        }

        if let Some(item_height) = self.item_height {
            // Uniform height
            self.scroll_offset = (index as f32 * item_height).max(0.0);
        } else {
            // Variable height
            let offset: f32 = self.item_heights.iter().take(index).sum();
            self.scroll_offset = offset.max(0.0);
        }
    }

    /// Scroll by a delta amount
    pub fn scroll_by(&mut self, delta: f32) {
        self.scroll_offset = (self.scroll_offset + delta)
            .max(0.0)
            .min(self.total_height - self.viewport_height);
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0.0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = (self.total_height - self.viewport_height).max(0.0);
    }

    /// Check if at the top
    pub fn at_top(&self) -> bool {
        self.scroll_offset <= 0.0
    }

    /// Check if at the bottom
    pub fn at_bottom(&self) -> bool {
        self.scroll_offset >= self.total_height - self.viewport_height
    }

    /// Get the scroll position as a ratio (0.0 = top, 1.0 = bottom)
    pub fn scroll_ratio(&self) -> f32 {
        if self.total_height <= self.viewport_height {
            0.0
        } else {
            self.scroll_offset / (self.total_height - self.viewport_height)
        }
    }

    /// Set scroll position from ratio
    pub fn set_scroll_ratio(&mut self, ratio: f32) {
        let ratio = ratio.clamp(0.0, 1.0);
        self.scroll_offset = ratio * (self.total_height - self.viewport_height).max(0.0);
    }
}

/// Virtual scrolling widget
pub struct VirtualScroll {
    /// Scroll state
    state: VirtualScrollState,

    /// Widget ID for egui
    id: egui::Id,

    /// Whether to show scrollbar
    show_scrollbar: bool,

    /// Whether to auto-scroll to new items
    auto_scroll: bool,
}

impl VirtualScroll {
    /// Create a new virtual scroll widget
    pub fn new(id_source: impl std::hash::Hash, total_items: usize) -> Self {
        Self {
            state: VirtualScrollState::new(total_items),
            id: egui::Id::new(id_source),
            show_scrollbar: true,
            auto_scroll: false,
        }
    }

    /// Create with uniform item height
    pub fn with_uniform_height(
        id_source: impl std::hash::Hash,
        total_items: usize,
        item_height: f32,
    ) -> Self {
        Self {
            state: VirtualScrollState::with_uniform_height(total_items, item_height),
            id: egui::Id::new(id_source),
            show_scrollbar: true,
            auto_scroll: false,
        }
    }

    /// Show or hide the scrollbar
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Enable or disable auto-scroll
    pub fn auto_scroll(mut self, auto: bool) -> Self {
        self.auto_scroll = auto;
        self
    }

    /// Show the virtual scroll widget
    pub fn show<F>(&mut self, ui: &mut egui::Ui, mut content: F)
    where
        F: FnMut(&mut egui::Ui, Range<usize>),
    {
        // Update viewport height
        let available_height = ui.available_height();
        self.state.update_viewport(available_height);

        // Create scroll area
        let scroll_area = egui::ScrollArea::vertical()
            .id_source(self.id)
            .scroll_offset(egui::Vec2::new(0.0, self.state.scroll_offset))
            .auto_shrink([false, false]);

        let scroll_response = if self.show_scrollbar {
            scroll_area.show(ui, |ui| {
                // Allocate space for all items
                let total_height = self.state.total_height;
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(ui.available_width(), total_height),
                    egui::Sense::hover(),
                );

                // Calculate clip rectangle
                let clip_rect = painter.clip_rect();
                let clip_top = clip_rect.min.y;
                let clip_bottom = clip_rect.max.y;

                // Update scroll offset based on clip rect
                self.state.scroll_offset = clip_top - response.rect.min.y;
                self.state.update_viewport(clip_bottom - clip_top);

                // Only render visible items
                content(ui, self.state.visible_range.clone());

                response
            })
        } else {
            scroll_area.show(ui, |ui| {
                // Allocate space for all items
                let total_height = self.state.total_height;
                let (response, _) = ui.allocate_painter(
                    egui::Vec2::new(ui.available_width(), total_height),
                    egui::Sense::hover(),
                );

                // Only render visible items
                content(ui, self.state.visible_range.clone());

                response
            })
        };

        // Handle scroll events
        let mut scroll_events = scroll_response.inner.ctx.input(|i| i.events.clone());
        for event in scroll_events.drain(..) {
            match event {
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta,
                    ..
                } => {
                    self.state.scroll_by(-delta.y);
                }
                egui::Event::Key {
                    key: egui::Key::ArrowUp,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_by(-20.0);
                }
                egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_by(20.0);
                }
                egui::Event::Key {
                    key: egui::Key::PageUp,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_by(-self.state.viewport_height);
                }
                egui::Event::Key {
                    key: egui::Key::PageDown,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_by(self.state.viewport_height);
                }
                egui::Event::Key {
                    key: egui::Key::Home,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_to_top();
                }
                egui::Event::Key {
                    key: egui::Key::End,
                    pressed: true,
                    ..
                } => {
                    self.state.scroll_to_bottom();
                }
                _ => {}
            }
        }

        // Auto-scroll to bottom if enabled and new items added
        if self.auto_scroll && scroll_response.inner.changed() {
            self.state.scroll_to_bottom();
        }
    }

    /// Get the current scroll state
    pub fn state(&self) -> &VirtualScrollState {
        &self.state
    }

    /// Get mutable reference to scroll state
    pub fn state_mut(&mut self) -> &mut VirtualScrollState {
        &mut self.state
    }

    /// Scroll to a specific item
    pub fn scroll_to_item(&mut self, index: usize) {
        self.state.scroll_to_item(index);
    }

    /// Scroll by a delta amount
    pub fn scroll_by(&mut self, delta: f32) {
        self.state.scroll_by(delta);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.state.scroll_to_top();
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.state.scroll_to_bottom();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_scroll_state_uniform() {
        let mut state = VirtualScrollState::with_uniform_height(100, 20.0);

        // Test initial state
        assert_eq!(state.total_items, 100);
        assert_eq!(state.total_height, 2000.0);

        // Test viewport update
        state.update_viewport(400.0);
        assert_eq!(state.viewport_height, 400.0);

        // Test visible range calculation
        assert_eq!(state.visible_range, 0..21); // 400/20 = 20 items + 1 buffer

        // Test scrolling
        state.scroll_by(100.0);
        state.update_viewport(400.0);
        assert_eq!(state.visible_range, 5..26); // 100/20 = 5 offset

        // Test scroll to item
        state.scroll_to_item(50);
        state.update_viewport(400.0);
        assert_eq!(state.visible_range.start, 50);

        // Test bounds
        state.scroll_to_top();
        assert!(state.at_top());

        state.scroll_to_bottom();
        assert!(state.at_bottom());
    }

    #[test]
    fn test_virtual_scroll_state_variable() {
        let mut state = VirtualScrollState::new(5);

        // Set variable heights
        state.set_item_height(0, 30.0);
        state.set_item_height(1, 40.0);
        state.set_item_height(2, 50.0);
        state.set_item_height(3, 60.0);
        state.set_item_height(4, 70.0);

        // Test viewport update
        state.update_viewport(100.0);
        assert_eq!(state.total_height, 250.0);

        // Test visible range (should show first 2-3 items)
        assert!(state.visible_range.start <= 2);
        assert!(state.visible_range.end <= 4);

        // Test scroll to item
        state.scroll_to_item(3);
        state.update_viewport(100.0);
        assert!(state.visible_range.start >= 3);
    }

    #[test]
    fn test_scroll_ratio() {
        let mut state = VirtualScrollState::with_uniform_height(100, 20.0);
        state.update_viewport(400.0);

        // Test at top
        assert_eq!(state.scroll_ratio(), 0.0);

        // Test at bottom
        state.scroll_to_bottom();
        state.update_viewport(400.0);
        assert_eq!(state.scroll_ratio(), 1.0);

        // Test middle
        state.set_scroll_ratio(0.5);
        state.update_viewport(400.0);
        assert!((state.scroll_ratio() - 0.5).abs() < 0.01);
    }
}
