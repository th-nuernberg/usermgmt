use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};

use crate::{
    current_selected_view::CurrentSelectedView, gui_design::WHICH_GUI_VIEW_SIZE,
    io_resource_manager::IoTaskStatus, usermgmt_window::UsermgmtWindow,
};

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();
    ui.label(RichText::new(current_view.to_str()).size(WHICH_GUI_VIEW_SIZE));
    match current_view {
        CurrentSelectedView::Configuration => match &window.conf_state.io_conf.status() {
            IoTaskStatus::NotStarted => {
                ui.label("No configuration loaded yet");
            }
            IoTaskStatus::Loading => {
                ui.label(RichText::new("Loading configuration").color(Color32::BLUE));
                match window.conf_state.io_conf.query_task() {
                    Some(conf) => window.conf_state.conf = Some(conf),
                    None => (),
                }
            }
            IoTaskStatus::Successful => {
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
        },
        _ => not_implemented_yet(current_view.to_str(), ui),
    }

    fn not_implemented_yet(action_name: &str, ui: &mut egui::Ui) {
        ui.label(format!("The action {} is not implemented yet", action_name));
    }
}
