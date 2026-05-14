use crate::services::askpass;
use eframe::egui;

pub struct PassphraseDialog;

impl PassphraseDialog {
    pub fn new() -> Self {
        Self
    }

    /// Poll the askpass state for pending requests. If one exists and the dialog
    /// isn't already showing, open it. Returns true if the dialog is currently visible.
    pub fn poll_and_show(&mut self, ctx: &egui::Context, ui_state: &mut crate::state::UIState) {
        let askpass_requests = askpass::state().lock().unwrap();

        if !ui_state.show_passphrase_dialog {
            if let Some(req) = askpass_requests.pending.last() {
                ui_state.show_passphrase_dialog = true;
                ui_state.passphrase_prompt = req.prompt.clone();
                ui_state.passphrase_input.clear();
            }
        }

        drop(askpass_requests);

        if ui_state.show_passphrase_dialog {
            let mut do_submit = false;
            let mut do_cancel = false;
            let prompt = ui_state.passphrase_prompt.clone();
            let mut pass = std::mem::take(&mut ui_state.passphrase_input);

            egui::Window::new("Authentication Required")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(&prompt);
                    ui.add_space(4.0);
                    let is_username = prompt.to_lowercase().contains("username");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut pass)
                            .password(!is_username)
                            .hint_text(if is_username { "Username" } else { "Password" })
                            .desired_width(260.0),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_submit = true;
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("Submit").clicked() {
                            do_submit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_submit {
                let mut askpass_requests = askpass::state().lock().unwrap();
                askpass_requests.response = Some(pass);
                askpass_requests.pending.clear();
                ui_state.show_passphrase_dialog = false;
                ui_state.passphrase_input.clear();
            } else if do_cancel {
                let mut askpass_requests = askpass::state().lock().unwrap();
                askpass_requests.response = Some(String::new());
                askpass_requests.pending.clear();
                ui_state.show_passphrase_dialog = false;
                ui_state.passphrase_input.clear();
            } else {
                ui_state.passphrase_input = pass;
            }
        }
    }
}

impl Default for PassphraseDialog {
    fn default() -> Self {
        Self::new()
    }
}
