use eframe::egui::{self, RichText};

use crate::{
    current_selected_view::CurrentSelectedView, gui_design::WHICH_GUI_VIEW_SIZE,
    usermgmt_window::UsermgmtWindow,
};

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();
    ui.label(RichText::new(current_view.to_str()).size(WHICH_GUI_VIEW_SIZE));
    match current_view {
        CurrentSelectedView::Nothing => {
            ui.label("Please select one of the actions via the menu button \"Actions\"");
        }
        _ => not_implemented_yet(current_view.to_str(), ui),
    }

    fn not_implemented_yet(action_name: &str, ui: &mut egui::Ui) {
        ui.label(format!("The action {} is not implemented yet", action_name));
    }
}
