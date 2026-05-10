use crate::state::{LogEntry, LogLevel};
use eframe::egui;

const TEXT_TERTIARY: egui::Color32 = egui::Color32::from_rgb(95, 95, 100);
const COLOR_INFO: egui::Color32 = egui::Color32::from_rgb(115, 201, 145);
const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(241, 76, 76);

pub struct CommandLog {
    visible: bool,
}

impl CommandLog {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Render the log window. Returns `true` if the user clicked "Clear".
    pub fn show(&mut self, ctx: &egui::Context, entries: &[LogEntry]) -> bool {
        if !self.visible {
            return false;
        }

        let mut clear_requested = false;

        egui::Window::new("Command Log")
            .default_width(520.0)
            .default_height(280.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Header row: count + Clear button
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} entries", entries.len()))
                            .color(TEXT_TERTIARY)
                            .small(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.small_button("Clear").clicked() {
                            clear_requested = true;
                        }
                    });
                });

                ui.separator();

                // Scrollable log list, pinned to bottom so newest entries are visible
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in entries {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&entry.timestamp)
                                        .color(TEXT_TERTIARY)
                                        .monospace()
                                        .small(),
                                );
                                ui.add_space(4.0);
                                let color = match entry.level {
                                    LogLevel::Info => COLOR_INFO,
                                    LogLevel::Error => COLOR_ERROR,
                                };
                                ui.label(
                                    egui::RichText::new(&entry.message).color(color).small(),
                                );
                            });
                        }
                    });

                ui.separator();

                // Footer
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("Close").clicked() {
                        self.visible = false;
                    }
                });
            });

        clear_requested
    }
}

impl Default for CommandLog {
    fn default() -> Self {
        Self::new()
    }
}
