use crate::prelude::*;

pub mod draw_add_state;
pub mod draw_delete_state;
pub mod draw_listing_of_users;
pub mod modify_state;

pub mod about;
pub mod configuration;
pub mod draw_utils;

pub fn draw_ssh_connection(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    draw_utils::draw_ssh_credentials(ui, &window.settings, &mut window.ssh_state);
}
