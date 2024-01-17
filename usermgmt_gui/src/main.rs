use eframe::egui;
use prelude::UsermgmtWindow;

mod prelude;

pub mod current_selected_view;
pub mod drawing;
mod which_systems;

mod constants;
pub mod general_utils;
mod io_resource_manager;
pub mod main_logic;

fn main() -> Result<(), eframe::Error> {
    // Set up logging and panic messages with link to issue page
    usermgmt_lib::app_panic_hook::set_app_panic_hook();
    let _logger = usermgmt_lib::logging::set_up_logging(env!("CARGO_PKG_NAME")).unwrap();

    // Construct application state before starting the main window for egui frontend.
    // This default impl for app state panics if set up failed due to invalid setting files
    // `Settings` or `Init`.
    let app_state = UsermgmtWindow::default();
    let (options, title) = {
        let init = &app_state.init;
        let window_title = init.window_title();
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(
                init.window_start_width(),
                init.window_start_height(),
            )),
            ..Default::default()
        };
        (options, window_title.clone())
    };
    // Look at the main_logic module for the high level control flow of this application.
    eframe::run_native(&title, options, Box::new(|_| Box::new(app_state)))
}
