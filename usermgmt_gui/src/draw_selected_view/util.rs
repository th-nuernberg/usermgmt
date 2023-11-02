use eframe::egui::{self, RichText};

use crate::current_selected_view::SshConnectionState;

pub fn draw_ssh_credentials(ui: &mut egui::Ui, ssh_state: &mut SshConnectionState) {
    let mut username = ssh_state.username.as_deref().unwrap_or_default().to_owned();
    let mut password = ssh_state.password.as_deref().unwrap_or_default().to_owned();
    user_password_box(
        ui,
        "Ssh credentails",
        &mut username,
        &mut password,
        |new| ssh_state.username = Some(new.to_string()),
        |new| {
            ssh_state.password = Some(new.to_string());
        },
    );
}

pub fn user_password_box(
    ui: &mut egui::Ui,
    group_name: &str,
    username_content: &mut String,
    password_content: &mut String,
    on_change_username: impl FnOnce(&mut String),
    on_change_password: impl FnOnce(&mut String),
) {
    draw_box_group(ui, group_name, |ui| {
        no_password_enty_field(ui, "Username: ", username_content, on_change_username);
        password_enty_field(ui, "Password: ", password_content, on_change_password);
    });
}

pub fn draw_box_group<R>(
    ui: &mut egui::Ui,
    group_name: &str,
    on_draw: impl FnOnce(&mut egui::Ui) -> R,
) {
    ui.label(RichText::new(group_name).strong());
    ui.group(on_draw);
}

pub fn no_password_enty_field(
    ui: &mut egui::Ui,
    label: &str,
    content: &mut String,
    on_change: impl FnOnce(&mut String),
) {
    draw_enty_field(ui, label, content, false, on_change)
}
pub fn password_enty_field(
    ui: &mut egui::Ui,
    label: &str,
    content: &mut String,
    on_change: impl FnOnce(&mut String),
) {
    draw_enty_field(ui, label, content, true, on_change)
}
fn draw_enty_field(
    ui: &mut egui::Ui,
    label: &str,
    content: &mut String,
    password: bool,
    on_change: impl FnOnce(&mut String),
) {
    ui.horizontal(|ui| {
        ui.label(label);

        if ui
            .add(egui::TextEdit::singleline(content).password(password))
            .changed()
        {
            on_change(content);
        }
    });
}
