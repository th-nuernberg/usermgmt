use eframe::egui::RichText;

use crate::prelude::*;
use drawing::{
    self, configuration, draw_add_state, draw_delete_state, draw_listing_of_users, modify_state,
};

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();

    ui.label(RichText::new(current_view.create_str()).size(gui_design::WHICH_GUI_VIEW_SIZE));

    match current_view {
        CurrentSelectedView::Configuration => configuration::draw(window, ui),
        CurrentSelectedView::Listing => draw_listing_of_users::draw(window, ui),
        CurrentSelectedView::SshConnection => drawing::draw_ssh_connection(window, ui),
        CurrentSelectedView::Adding => draw_add_state::draw(ui, window),
        CurrentSelectedView::Removing => draw_delete_state::draw(ui, window),
        CurrentSelectedView::Modifing => modify_state::draw(ui, window),
        _ => not_implemented_yet(current_view.create_str(), ui),
    }

    fn not_implemented_yet(action_name: &str, ui: &mut egui::Ui) {
        let msg = text_design::create_msg::not_implemented_action(action_name);
        ui.label(msg);
    }
}
