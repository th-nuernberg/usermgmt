use log::info;

slint::include_modules!();

fn main() {
    env_logger::init();
    let app = AppWindow::new().expect("Could not initialize the gui front end of the app.");

    info!("Starting the app");
    app.run().expect("Could not start the gui main window.");
}
