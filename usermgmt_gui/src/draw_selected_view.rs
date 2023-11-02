use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};

use crate::{
    current_selected_view::CurrentSelectedView,
    draw_selected_view::draw_listing_of_users::draw_listing_view, gui_design::WHICH_GUI_VIEW_SIZE,
    io_resource_manager::IoTaskStatus, usermgmt_window::UsermgmtWindow,
};

use self::util::draw_box_group;

mod draw_add_state;
mod draw_delete_state;
mod draw_listing_of_users;
pub mod util;

pub fn draw_ssh_connection(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    util::draw_ssh_credentials(ui, &mut window.ssh_state);
}

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();
    ui.label(RichText::new(current_view.create_str()).size(WHICH_GUI_VIEW_SIZE));
    match current_view {
        CurrentSelectedView::Configuration => draw_configuration_view(window, ui),
        CurrentSelectedView::Listing => draw_listing_view(window, ui),
        CurrentSelectedView::SshConnection => draw_ssh_connection(window, ui),
        CurrentSelectedView::Adding => draw_add_state::draw(ui, window),
        CurrentSelectedView::Removing => draw_delete_state::draw(ui, window),
        _ => not_implemented_yet(current_view.create_str(), ui),
    }

    fn not_implemented_yet(action_name: &str, ui: &mut egui::Ui) {
        ui.label(format!("The action {} is not implemented yet", action_name));
    }
}

fn draw_configuration_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let mut can_reload = true;
    match &window.conf_state.io_conf.status() {
        IoTaskStatus::NotStarted => {
            ui.label("No configuration loaded yet");
        }
        IoTaskStatus::Loading => {
            ui.label(RichText::new("Loading configuration").color(Color32::BLUE));
            can_reload = false;
        }
        IoTaskStatus::Successful(_) => {
            ui.label(RichText::new("Configuration loaded").color(Color32::GREEN));
        }
        IoTaskStatus::Failed(error) => {
            ui.label(
                RichText::new("Error in loadiding configuration.")
                    .strong()
                    .color(Color32::RED),
            );
            ui.label(RichText::new(format!("Error details. {}", error)).color(Color32::RED));
        }
    }

    draw_file_path(ui, window);
    if ui
        .add_enabled(can_reload, egui::Button::new("Reload"))
        .clicked()
    {
        let path = window.conf_path.clone();
        crate::usermgmt_window::start_load_config(&mut window.conf_state, Some(path));
    }
}

fn draw_file_path(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    let conf_state = &window.conf_state;
    let mut path = window.conf_path_owned();

    if conf_state.io_conf.status().is_loading() {
        draw_box_group(ui, "Path", |ui| ui.label(&path));
    } else {
        draw_box_group(ui, "Path", |ui| {
            ui.text_edit_singleline(&mut path);
            window.set_conf_path(path);
        });
    }
}
