use crate::prelude::*;

use eframe::egui::{self, RichText};
use num::{Bounded, FromPrimitive, Signed, ToPrimitive};

use crate::{
    current_selected_view::{LdapConnectionState, SshConnectionState},
    which_systems,
};

pub fn list_view(
    ui: &mut egui::Ui,
    settings: &Settings,
    list_field: &mut Vec<String>,
    group_label: &str,
) {
    let text = settings.texts();
    draw_box_group(ui, group_label, |ui| {
        if ui.button(text.btn_new_item()).clicked() {
            list_field.push(Default::default());
        }
        let mut to_delete: Vec<usize> = Default::default();
        for (index, next_field) in &mut list_field.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(next_field);
                if ui.button(text.btn_list_remove()).clicked() {
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

    let texts = window.settings.texts();
    if conf_state.io_conf.status().is_loading() {
        draw_box_group(ui, texts.dir_conf_path(), |ui| ui.label(&path));
    } else {
        draw_box_group(ui, texts.dir_conf_path(), |ui| {
            ui.text_edit_singleline(&mut path);
        });
        window.set_conf_path(path);
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
        settings.texts().ssh_cred(),
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
        settings.texts().ldap_cred(),
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
            settings.texts().username(),
            username_content,
            on_change_username,
        );
        password_enty_field(
            ui,
            settings.texts().password(),
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

pub fn box_centered_single_line(
    ui: &mut egui::Ui,
    settings: &Settings,
    box_name: &str,
    label: &str,
) {
    draw_box_group(ui, box_name, |ui| {
        ui.label(
            RichText::new(label)
                .strong()
                .size(settings.box_label_font_size),
        );
    });
}
pub fn link_box(ui: &mut egui::Ui, box_name: &str, link: &str, opt_label: Option<&str>) {
    draw_box_group(ui, box_name, |ui| {
        if let Some(label) = opt_label {
            ui.vertical_centered(|ui| {
                ui.label(label);
                _ = ui.hyperlink_to(link, link);
            });
        } else {
            _ = ui.hyperlink_to(link, link);
        };
    })
}
pub fn no_password_enty_field(
    ui: &mut egui::Ui,
    label: &str,
    content: &mut String,
    on_change: impl FnOnce(&mut String),
) {
    draw_enty_field(ui, label, content, false, on_change)
}
pub fn whole_pos_number_fields<T>(ui: &mut egui::Ui, label: &str, content: &mut T)
where
    T: ToPrimitive + FromPrimitive + Bounded + Copy,
{
    let mut float: f32 = content.to_f32().unwrap_or_else(|| {
        warn!("Interger value could not be casted to f32 for gui.");
        warn!("Using the biggest possible f32 value instead.");
        f32::MAX
    });
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(&mut float).speed(0.1));
        let rounded = float.round().max(0.0);
        let new_value = <T as FromPrimitive>::from_f32(rounded).unwrap_or_else(|| {
            let max = <T as Bounded>::max_value();

            warn!("Floating value from gui could not be casted to interger value.");
            warn!("Using biggest interger number as new value instead.");
            max
        });
        *content = new_value;
    });
}

pub fn whole_neg_number_fields<T>(ui: &mut egui::Ui, label: &str, content: &mut T)
where
    T: ToPrimitive + FromPrimitive + Signed + Bounded + Copy,
{
    let mut float: f32 = content.to_f32().unwrap_or_else(|| {
        warn!("Interger value could not be casted to f32 for gui.");
        if content.is_negative() {
            warn!("Using the smallest possible f32 value instead.");
            f32::MIN
        } else {
            warn!("Using the biggest possible f32 value instead.");
            f32::MAX
        }
    });
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(&mut float).speed(0.1));
        let rounded = float.round();
        let new_value = <T as FromPrimitive>::from_f32(rounded).unwrap_or_else(|| {
            let (min, max) = (<T as Bounded>::min_value(), <T as Bounded>::max_value());
            warn!("Floating value from gui could not be casted to interger value.");
            if rounded.is_negative() {
                warn!("Using smallest interger number as new value instead.");
                min
            } else {
                warn!("Using biggest interger number as new value instead.");
                max
            }
        });
        *content = new_value;
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

#[allow(clippy::too_many_arguments)]
pub fn draw_status_msg_w_label<T>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce(&T) -> String,
    error_msg: impl FnOnce() -> String,
) {
    status_msg(
        ui,
        settings,
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
    settings: &Settings,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce(&T) -> String,
    error_msg: impl FnOnce() -> String,
) {
    status_msg(
        ui,
        settings,
        settings.texts().general_status(),
        status,
        msg_init,
        msg_loading,
        msg_success,
        error_msg,
    )
}

pub fn draw_credentails(ui: &mut egui::Ui, window: &mut UsermgmtWindow, supports_dir: bool) {
    which_systems::draw_which_system(ui, &window.settings, &mut window.which_sys, supports_dir);
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

#[allow(clippy::too_many_arguments)]
fn status_msg<T>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    status: &IoTaskStatus<T>,
    msg_init: impl FnOnce() -> String,
    msg_loading: impl FnOnce() -> String,
    msg_success: impl FnOnce(&T) -> String,
    error_msg: impl FnOnce() -> String,
) {
    draw_box_group(ui, label, |ui| {
        let colors = settings.colors();
        let (color, raw_text) = match status {
            IoTaskStatus::NotStarted => (colors.init_msg(), msg_init()),
            IoTaskStatus::Loading => (colors.loading_msg(), msg_loading()),
            IoTaskStatus::Successful(val) => (colors.success_msg(), msg_success(val)),
            IoTaskStatus::Failed(error) => (
                colors.err_msg(),
                general_utils::error_status(&error_msg(), error),
            ),
        };
        let text = RichText::new(raw_text).color(color).strong();
        ui.label(text);
    });
}
