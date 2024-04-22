pub use group_drawing::GroupDrawing;
pub use text_field_entry::TextFieldEntry;

mod group_drawing;
mod text_field_entry;

use crate::prelude::*;

use eframe::egui::{self, RichText};
use num::{Bounded, FromPrimitive, Signed, ToPrimitive};

use crate::{
    current_selected_view::{LdapConnectionState, SshConnectionState},
    which_systems,
};

use super::ProduceIoStatusMessages;

pub fn tooltip_widget(ui: &mut egui::Ui, settings: &Settings, text: &str) {
    ui.label(
        RichText::new(settings.tooltip_symbol())
            .size(settings.tooltip_size())
            .color(settings.colors().tool_tip()),
    )
    .on_hover_text(text);
}

pub fn list_view(
    ui: &mut egui::Ui,
    settings: &Settings,
    list_field: &mut Vec<String>,
    group_drawing: &GroupDrawing,
) {
    let text = settings.texts();
    draw_box_group(ui, settings, group_drawing, |ui| {
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

    let settings = &window.settings;
    let texts = settings.texts();
    if conf_state.io_conf.status().is_loading() {
        draw_box_group(
            ui,
            settings,
            &GroupDrawing::new(texts.dir_conf_path()),
            |ui| ui.label(&path),
        );
    } else {
        draw_box_group(
            ui,
            settings,
            &GroupDrawing::new(texts.dir_conf_path()),
            |ui| {
                ui.text_edit_singleline(&mut path);
            },
        );
        window.set_conf_path(path);
    }
}
pub fn draw_ssh_credentials(
    ui: &mut egui::Ui,
    settings: &Settings,
    ssh_state: &mut SshConnectionState,
) {
    let username = &mut ssh_state.username;
    let password = &mut ssh_state.password;
    user_password_box(
        ui,
        settings,
        &GroupDrawing::new(settings.texts().ssh_cred())
            .add_tooltip(settings.tooltiptexts().ssh_creds()),
        username,
        password,
    );
}

pub fn draw_ldap_credentials(
    ui: &mut egui::Ui,
    settings: &Settings,
    ldap_state: &mut LdapConnectionState,
) {
    let username = &mut ldap_state.username;
    let password = &mut ldap_state.password;
    user_password_box(
        ui,
        settings,
        &GroupDrawing::new(settings.texts().ldap_cred())
            .add_tooltip(settings.tooltiptexts().ldap_creds()),
        username,
        password,
    );
}

pub fn user_password_box(
    ui: &mut egui::Ui,
    settings: &Settings,
    group_draw: &GroupDrawing,
    username_content: &mut Option<String>,
    password_content: &mut Option<String>,
) {
    draw_box_group(ui, settings, group_draw, |ui| {
        entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new_opt(settings.texts().username(), username_content),
        );
        entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new_opt(settings.texts().password(), password_content)
                .with_as_password(),
        );
    });
}

pub fn draw_box_group<R>(
    ui: &mut egui::Ui,
    settings: &Settings,
    group: &GroupDrawing,
    on_draw: impl FnOnce(&mut egui::Ui) -> R,
) {
    if let Some(tool_tip_name) = group.tooltip() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(group.name()).strong());
            tooltip_widget(ui, settings, tool_tip_name);
        });
    } else {
        ui.label(RichText::new(group.name()).strong());
    }
    ui.group(on_draw);
}

pub fn box_centered_single_line(
    ui: &mut egui::Ui,
    settings: &Settings,
    box_name: &str,
    label: &str,
) {
    draw_box_group(ui, settings, &GroupDrawing::new(box_name), |ui| {
        ui.label(
            RichText::new(label)
                .strong()
                .size(settings.box_label_font_size),
        );
    });
}
pub fn link_box(
    ui: &mut egui::Ui,
    settings: &Settings,
    box_name: &str,
    link: &str,
    opt_label: Option<&str>,
) {
    draw_box_group(ui, settings, &GroupDrawing::new(box_name), |ui| {
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
pub fn entry_field(ui: &mut egui::Ui, settings: &Settings, entry_field: &mut TextFieldEntry) {
    draw_entry_field(ui, settings, entry_field)
}
pub fn whole_pos_number_fields<T>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    content: &mut T,
    tooltip: Option<&str>,
) where
    T: ToPrimitive + FromPrimitive + Bounded + Copy,
{
    let mut float: f32 = content.to_f32().unwrap_or_else(|| {
        warn!("Integer value could not be casted to f32 for gui.");
        warn!("Using the biggest possible f32 value instead.");
        f32::MAX
    });
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(&mut float).speed(0.1));
        if let Some(tooltip) = tooltip {
            tooltip_widget(ui, settings, tooltip)
        }

        let rounded = float.round().max(0.0);
        let new_value = <T as FromPrimitive>::from_f32(rounded).unwrap_or_else(|| {
            let max = <T as Bounded>::max_value();

            warn!("Floating value from gui could not be casted to integer value.");
            warn!("Using biggest integer number as new value instead.");
            max
        });
        *content = new_value;
    });
}

pub fn whole_neg_number_fields<T>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    content: &mut T,
    tooltip: Option<&str>,
) where
    T: ToPrimitive + FromPrimitive + Signed + Bounded + Copy,
{
    let mut float: f32 = content.to_f32().unwrap_or_else(|| {
        warn!("Integer value could not be casted to f32 for gui.");
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
        if let Some(tooltip) = tooltip {
            tooltip_widget(ui, settings, tooltip)
        }
        let rounded = float.round();
        let new_value = <T as FromPrimitive>::from_f32(rounded).unwrap_or_else(|| {
            let (min, max) = (<T as Bounded>::min_value(), <T as Bounded>::max_value());
            warn!("Floating value from gui could not be casted to integer value.");
            if rounded.is_negative() {
                warn!("Using smallest integer number as new value instead.");
                min
            } else {
                warn!("Using biggest integer number as new value instead.");
                max
            }
        });
        *content = new_value;
    });
}

pub fn draw_status_msg_w_label<T, C>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    status: &IoTaskStatus<T>,
    msg: C,
) where
    C: ProduceIoStatusMessages<T>,
{
    status_msg(ui, settings, label, status, msg)
}

pub fn draw_status_msg<T, C>(
    ui: &mut egui::Ui,
    settings: &Settings,
    status: &IoTaskStatus<T>,
    msg: C,
) where
    C: ProduceIoStatusMessages<T>,
{
    status_msg(ui, settings, settings.texts().general_status(), status, msg)
}

pub fn draw_credentials(ui: &mut egui::Ui, window: &mut UsermgmtWindow, supports_dir: bool) {
    which_systems::draw_which_system(ui, &window.settings, &mut window.which_sys, supports_dir);
    if window.is_ssh_cred_needed(supports_dir) {
        draw_ssh_credentials(ui, &window.settings, &mut window.ssh_state);
    }
    if window.is_ldap_needed() {
        draw_ldap_credentials(ui, &window.settings, &mut window.ldap_state)
    }
}

fn draw_entry_field(ui: &mut egui::Ui, settings: &Settings, entry_field: &mut TextFieldEntry) {
    ui.horizontal(|ui| {
        ui.label(entry_field.label());

        let password = entry_field.as_password();
        let content = entry_field.content();
        let mut buffer = content.to_owned();
        if ui
            .add(egui::TextEdit::singleline(&mut buffer).password(password))
            .changed()
        {
            entry_field.set_content(buffer);
        }
        if let Some(tool_tip_text) = entry_field.tool_tip() {
            tooltip_widget(ui, settings, tool_tip_text)
        }
    });
}

fn status_msg<T, C>(
    ui: &mut egui::Ui,
    settings: &Settings,
    label: &str,
    status: &IoTaskStatus<T>,
    mut msg: C,
) where
    C: ProduceIoStatusMessages<T>,
{
    draw_box_group(ui, settings, &GroupDrawing::new(label), |ui| {
        let colors = settings.colors();
        let (color, raw_text) = match status {
            IoTaskStatus::NotStarted => (colors.init_msg(), msg.msg_init()),
            IoTaskStatus::Loading => (colors.loading_msg(), msg.msg_loading()),
            IoTaskStatus::Successful(val) => (colors.success_msg(), msg.msg_success(val)),
            IoTaskStatus::Failed(error) => (
                colors.err_msg(),
                general_utils::error_status(&msg.msg_error(), error),
            ),
        };
        let text = RichText::new(raw_text).color(color).strong();
        ui.label(text);
    });
}
