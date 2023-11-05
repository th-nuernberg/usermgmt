use crate::{drawing::about, prelude::*};
use drawing::{
    self, configuration, draw_add_state, draw_delete_state, draw_listing_of_users, modify_state,
};
use eframe::egui::RichText;
use std::convert::AsRef;

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();

    ui.label(RichText::new(current_view.as_ref()).size(window.settings.title_font_size));

    match current_view {
        CurrentSelectedView::Configuration => configuration::draw(window, ui),
        CurrentSelectedView::Listing => draw_listing_of_users::draw(window, ui),
        CurrentSelectedView::Adding => draw_add_state::draw(ui, window),
        CurrentSelectedView::Removing => draw_delete_state::draw(ui, window),
        CurrentSelectedView::Modifing => modify_state::draw(ui, window),
        CurrentSelectedView::About => about::draw(window, ui),
    }
}
