use eframe::egui;
mod current_selected_view;
mod draw_selected_view;
mod usermgmt_window;
use usermgmt_window::UsermgmtWindow;

mod gui_design;
mod io_background_worker;
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Usermgmt",
        options,
        Box::new(|_| Box::<UsermgmtWindow>::default()),
    )
}
