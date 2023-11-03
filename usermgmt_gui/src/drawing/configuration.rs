use eframe::{egui::RichText, epaint::Color32};

use crate::prelude::*;

pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
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

    draw_utils::draw_file_path(ui, window);
    if ui
        .add_enabled(can_reload, egui::Button::new("Reload"))
        .clicked()
    {
        let path = window.conf_path.clone();
        general_utils::start_load_config(&mut window.conf_state, Some(path));
    }
}
