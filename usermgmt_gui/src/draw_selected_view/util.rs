use eframe::egui;

use crate::current_selected_view::SshConnectionState;

pub fn draw_ssh_credentials(ui: &mut egui::Ui, ssh_state: &mut SshConnectionState) {
    ui.label("Ssh credentails");
    ui.horizontal(|ui| {
        ui.label("Username: ");
        if let Some(username) = ssh_state.username.as_mut() {
            ui.text_edit_singleline(username);
        } else {
            let mut new = String::default();
            if ui.text_edit_singleline(&mut new).changed() {
                ssh_state.username = Some(new);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Password: ");
        if let Some(password) = ssh_state.password.as_mut() {
            ui.add(egui::TextEdit::singleline(password).password(true));
        } else {
            let mut new = String::default();
            if ui
                .add(egui::TextEdit::singleline(&mut new).password(true))
                .changed()
            {
                ssh_state.password = Some(new);
            }
        }
    });
}
