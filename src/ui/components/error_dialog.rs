//! Error dialog component for Git Gud
//!
//! This component displays error messages in a modal dialog.

use eframe::egui;

/// Error dialog UI component
pub struct ErrorDialog {
    /// Whether the dialog is currently visible
    visible: bool,

    /// Error message to display
    error_message: String,

    /// Whether to show detailed error information
    show_details: bool,
}

impl ErrorDialog {
    /// Create a new error dialog
    pub fn new() -> Self {
        Self {
            visible: false,
            error_message: String::new(),
            show_details: false,
        }
    }

    /// Show an error message
    pub fn show_error(&mut self, error_message: String) {
        self.visible = true;
        self.error_message = error_message;
        self.show_details = false;
    }

    /// Hide the error dialog
    pub fn hide(&mut self) {
        self.visible = false;
        self.error_message.clear();
        self.show_details = false;
    }

    /// Check if the dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the error dialog
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("Error")
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("An error occurred:");
                ui.separator();

                // Show error message
                ui.label(&self.error_message);

                // Show details toggle if error is long
                if self.error_message.len() > 100 {
                    ui.checkbox(&mut self.show_details, "Show details");

                    if self.show_details {
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.monospace(&self.error_message);
                            });
                    }
                }

                ui.add_space(10.0);

                // OK button
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() - 50.0);
                    if ui.button("OK").clicked() {
                        self.hide();
                    }
                });
            });
    }
}

impl Default for ErrorDialog {
    fn default() -> Self {
        Self::new()
    }
}
