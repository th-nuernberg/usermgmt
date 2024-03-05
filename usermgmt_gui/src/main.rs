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
    // Logger handler in variable so background thread for file logging is not stopped until the
    // end of application.
    let _keep_logger_handler = usermgmt_lib::logging::set_up_logging(env!("CARGO_PKG_NAME"))
        .expect("Failed to initialize logger");

    // Construct application state before starting the main window for egui frontend.
    // This default impl for app state panics if set up failed due to invalid setting files
    // `Settings` or `Init`.
    let app_state = UsermgmtWindow::default();
    let (options, title) = {
        let init = &app_state.init;
        let window_title = init.window_title();
        let (height, width) = (init.window_start_height(), init.window_start_width());
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([height, width]),
            ..Default::default()
        };
        (options, window_title.clone())
    };
    // Look at the main_logic module for the high level control flow of this application.
    eframe::run_native(&title, options, Box::new(|_| Box::new(app_state)))
}
