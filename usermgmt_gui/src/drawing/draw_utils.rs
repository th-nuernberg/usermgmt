use crate::prelude::*;

use eframe::egui::{self, RichText};

use crate::{
    current_selected_view::{LdapConnectionState, SshConnectionState},
    which_systems,
};

pub fn list_view(ui: &mut egui::Ui, list_field: &mut Vec<String>, group_label: &str) {
    draw_box_group(ui, group_label, |ui| {
        if ui.button(text_design::button::NEW_ITEM).clicked() {
            list_field.push(Default::default());
        }
        let mut to_delete: Vec<usize> = Default::default();
        for (index, next_field) in &mut list_field.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(next_field);
                if ui.button(text_design::button::LIST_REMOVE).clicked() {
                    to_delete.push(index);
                }
            });
        }
        if !to_delete.is_empty() {
            let taken = std::mem::take(list_field);
            *list_field = taken
                .into_iter()
                .enumerate()
                .filter_map(|(index, element)| {
                    if !to_delete.contains(&index) {
                        Some(element)
                    } else {
                        None
                    }
                })
                .collect();
        }
    });
}

pub fn draw_file_path(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    let conf_state = &window.conf_state;
    let mut path = window.conf_path_owned();

    if conf_state.io_conf.status().is_loading() {
        draw_box_group(ui, text_design::group::DIR_CONF_PATH, |ui| ui.label(&path));
    } else {
        draw_box_group(ui, text_design::group::DIR_CONF_PATH, |ui| {
            ui.text_edit_singleline(&mut path);
            window.set_conf_path(path);
        });
    }
}
pub fn draw_ssh_credentials(
    ui: &mut egui::Ui,
    settings: &Settings,
    ssh_state: &mut SshConnectionState,
) {
    let mut username = ssh_state.username.as_deref().unwrap_or_default().to_owned();
    let mut password = ssh_state.password.as_deref().unwrap_or_default().to_owned();
    user_password_box(
        ui,
        settings,
        text_design::group::SSH_CRED,
        &mut username,
        &mut password,
        |new| ssh_state.username = Some(new.to_string()),
        |new| {
            ssh_state.password = Some(new.to_string());
        },
    );
}

pub fn draw_ldap_credentials(
    ui: &mut egui::Ui,
    settings: &Settings,
    ldap_state: &mut LdapConnectionState,
) {
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
        settings,
        text_design::group::LDAP_CRED,
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
    settings: &Settings,
    group_name: &str,
    username_content: &mut String,
    password_content: &mut String,
    on_change_username: impl FnOnce(&mut String),
    on_change_password: impl FnOnce(&mut String),
) {
    draw_box_group(ui, group_name, |ui| {
        no_password_enty_field(
            ui,
            &settings.username_label,
            username_content,
            on_change_username,
        );
        password_enty_field(
            ui,
            text_design::label::PASSWORD,
            password_content,
            on_change_password,
        );
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
pub fn number_field(ui: &mut egui::Ui, label: &str, content: &mut u32) {
    ui.horizontal(|ui| {
        ui.label(label);

        let mut float = *content as f32;
        ui.add(egui::DragValue::new(&mut float).speed(0.1));
        *content = float.round() as u32;
    });
}
pub fn neg_number_field(ui: &mut egui::Ui, label: &str, content: &mut i32) {
    ui.horizontal(|ui| {
        ui.label(label);

        let mut float = *content as f32;
        ui.add(egui::DragValue::new(&mut float).speed(0.1));
        *content = float.round() as i32;
    });
}

pub fn no_password_opt_enty_field(ui: &mut egui::Ui, label: &str, content: &mut Option<String>) {
    let mut text = content.to_owned().unwrap_or_default();
    draw_enty_field(ui, label, &mut text, false, |text| {
        *content = general_utils::some_if_not_blank_str(text).map(|trimmed| trimmed.into())
    })
}
pub fn password_enty_field(
    ui: &mut egui::Ui,
    label: &str,
    content: &mut String,
    on_change: impl FnOnce(&mut String),
) {
    draw_enty_field(ui, label, content, true, on_change)
}
pub fn draw_status_msg_w_label<T>(
    ui: &mut egui::Ui,
    label: &str,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce() -> String,
    error_msg: impl FnOnce() -> String,
) {
    status_msg(
        ui,
        label,
        status,
        msg_init,
        msg_loading,
        msg_success,
        error_msg,
    )
}

pub fn draw_status_msg<T>(
    ui: &mut egui::Ui,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce() -> String,
    error_msg: impl FnOnce() -> String,
) {
    status_msg(
        ui,
        text_design::group::GENERAL_STATUS,
        status,
        msg_init,
        msg_loading,
        msg_success,
        error_msg,
    )
}

pub fn draw_credentails(ui: &mut egui::Ui, window: &mut UsermgmtWindow, supports_dir: bool) {
    which_systems::draw_which_system(ui, &mut window.which_sys, supports_dir);
    if window.is_ssh_cred_needed(supports_dir) {
        draw_ssh_credentials(ui, &window.settings, &mut window.ssh_state);
    }
    if window.is_ldap_needed() {
        draw_ldap_credentials(ui, &window.settings, &mut window.ldap_state)
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

fn status_msg<T>(
    ui: &mut egui::Ui,
    label: &str,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce() -> String,
    error_msg: impl FnOnce() -> String,
) {
    draw_box_group(ui, label, |ui| {
        let (color, raw_text) = match status {
            IoTaskStatus::NotStarted => (gui_design::colors::INIT_MSG, msg_init()),
            IoTaskStatus::Loading => (gui_design::colors::LOADING_MSG, msg_loading()),
            IoTaskStatus::Successful(_) => (gui_design::colors::SUCCESS_MSG, msg_success()),
            IoTaskStatus::Failed(error) => (
                gui_design::colors::ERROR_MSG,
                text_design::create_msg::error_status(&error_msg(), error),
            ),
        };
        let text = RichText::new(raw_text).color(color).strong();
        ui.label(text);
    });
}
