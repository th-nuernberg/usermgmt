use crate::prelude::*;

use eframe::egui::{self, RichText};
use num::{Bounded, FromPrimitive, Signed, ToPrimitive};

use crate::{
    current_selected_view::{LdapConnectionState, SshConnectionState},
    which_systems,
};

#[derive(Debug)]
pub struct GroupDrawing<'a, 'b> {
    name: &'a str,
    tooltip: Option<&'b str>,
}

impl<'a, 'b> GroupDrawing<'a, 'b> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            tooltip: None,
        }
    }
    pub fn with_tooltip(self, text: &'b str) -> GroupDrawing<'a, 'b> {
        Self {
            name: self.name,
            tooltip: Some(text),
        }
    }
    pub fn tooltip(self, text: Option<&'b str>) -> Self {
        Self {
            name: self.name,
            tooltip: text,
        }
    }
}

#[derive(Debug)]
enum ContentField<'a> {
    Required(&'a mut String),
    Optional(&'a mut Option<String>),
}
#[derive(Debug)]
pub struct TextFieldEntry<'a, 'b> {
    label: &'a str,
    content: ContentField<'a>,
    tool_tip: Option<&'b str>,
    as_password: bool,
}

impl<'a, 'b> TextFieldEntry<'a, 'b> {
    pub fn new(label: &'a str, content: &'a mut String) -> Self {
        Self {
            label,
            content: ContentField::Required(content),
            as_password: false,
            tool_tip: None,
        }
    }
    pub fn new_opt(label: &'a str, content: &'a mut Option<String>) -> Self {
        Self {
            label,
            content: ContentField::Optional(content),
            as_password: false,
            tool_tip: None,
        }
    }
    pub fn as_password(mut self) -> Self {
        self.as_password = true;
        self
    }
    pub fn tool_tip(self, too_tip: Option<&'b str>) -> Self {
        Self {
            label: self.label,
            content: self.content,
            as_password: self.as_password,
            tool_tip: too_tip,
        }
    }
    pub fn with_tool_tip(self, tooltip: &'b str) -> Self {
        Self {
            label: self.label,
            content: self.content,
            as_password: self.as_password,
            tool_tip: Some(tooltip),
        }
    }
}

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
            .with_tooltip(settings.tooltiptexts().ssh_creds()),
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
            .with_tooltip(settings.tooltiptexts().ldap_creds()),
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
                .as_password(),
        );
    });
}

pub fn draw_box_group<R>(
    ui: &mut egui::Ui,
    settings: &Settings,
    group: &GroupDrawing,
    on_draw: impl FnOnce(&mut egui::Ui) -> R,
) {
    if let Some(tool_tip_name) = group.tooltip {
        ui.horizontal(|ui| {
            ui.label(RichText::new(group.name).strong());
            tooltip_widget(ui, settings, tool_tip_name);
        });
    } else {
        ui.label(RichText::new(group.name).strong());
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
    draw_enty_field(ui, settings, entry_field)
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
        warn!("Interger value could not be casted to f32 for gui.");
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

            warn!("Floating value from gui could not be casted to interger value.");
            warn!("Using biggest interger number as new value instead.");
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
        if let Some(tooltip) = tooltip {
            tooltip_widget(ui, settings, tooltip)
        }
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

fn draw_enty_field(ui: &mut egui::Ui, settings: &Settings, entry_field: &mut TextFieldEntry) {
    ui.horizontal(|ui| {
        ui.label(entry_field.label);

        let mut empty = String::default();
        let mut opt = false;
        let content: &mut String = match &mut entry_field.content {
            ContentField::Required(content) => content,
            ContentField::Optional(optional) => {
                opt = true;
                optional.as_mut().unwrap_or(&mut empty)
            }
        };
        if ui
            .add(egui::TextEdit::singleline(content).password(entry_field.as_password))
            .changed()
            && opt
            && !content.as_str().trim().is_empty()
        {
            let content = content.to_owned();
            if let ContentField::Optional(to_change) = &mut entry_field.content {
                **to_change = Some(content)
            } else {
                unreachable!();
            }
        }
        if let Some(tool_tip_text) = entry_field.tool_tip {
            tooltip_widget(ui, settings, tool_tip_text)
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
    draw_box_group(ui, settings, &GroupDrawing::new(label), |ui| {
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
