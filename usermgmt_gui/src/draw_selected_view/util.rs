use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};

use crate::{
    current_selected_view::{LdapConnectionState, SshConnectionState},
    io_resource_manager::IoTaskStatus,
    usermgmt_window::UsermgmtWindow,
    which_systems,
};

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

pub fn draw_ldap_credentials(ui: &mut egui::Ui, ldap_state: &mut LdapConnectionState) {
    let mut username = ldap_state
        .username
        .as_deref()
        .unwrap_or_default()
        .to_owned();
    let mut password = ldap_state
        .password
        .as_deref()
        .unwrap_or_default()
        .to_owned();
    user_password_box(
        ui,
        "LDAP credentails",
        &mut username,
        &mut password,
        |new| ldap_state.username = Some(new.to_string()),
        |new| {
            ldap_state.password = Some(new.to_string());
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
pub fn draw_status_msg<T>(
    ui: &mut egui::Ui,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce() -> String,
    error_msg: impl FnOnce() -> String,
) {
    draw_box_group(ui, "Status", |ui| {
        let (color, raw_text) = match status {
            IoTaskStatus::NotStarted => (Color32::BLUE, msg_init()),
            IoTaskStatus::Loading => (Color32::GRAY, msg_loading()),
            IoTaskStatus::Successful(_) => (Color32::GREEN, msg_success()),
            IoTaskStatus::Failed(error) => (
                Color32::RED,
                format!("{}. Details: \n{}", error_msg(), error),
            ),
        };
        let text = RichText::new(raw_text).color(color).strong();
        ui.label(text);
    });
}
pub fn draw_credentails(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    which_systems::draw_which_system(ui, &mut window.which_sys);
    if window.is_ssh_cred_needed() {
        draw_ssh_credentials(ui, &mut window.ssh_state);
    }
    if window.is_ldap_needed() {
        draw_ldap_credentials(ui, &mut window.ldap_state)
    }
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
