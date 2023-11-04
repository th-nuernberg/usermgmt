mod prelude;
use prelude::*;

pub mod current_selected_view;
pub mod drawing;
mod which_systems;

pub mod general_utils;
pub mod gui_design;
mod io_resource_manager;
pub mod main_logic;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(
            gui_design::WINDOW_START_WIDTH,
            gui_design::WINDOW_START_HEIGHT,
        )),
        ..Default::default()
    };
    eframe::run_native(
        text_design::WINDOW_TITLE,
        options,
        Box::new(|_| Box::<UsermgmtWindow>::default()),
    )
}
