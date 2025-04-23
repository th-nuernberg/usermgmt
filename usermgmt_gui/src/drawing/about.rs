use crate::prelude::*;
pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let settings = &window.settings;
    draw_utils::box_centered_single_line(ui, settings, "Version", env!("CARGO_PKG_VERSION"));
    draw_utils::link_box(
        ui,
        settings,
        "Readme",
        usermgmt_lib::constants::README_LINK,
        None,
    );
    draw_utils::link_box(
        ui,
        settings,
        "License",
        usermgmt_lib::constants::MIT_LINK,
        None,
    );
    draw_utils::link_box(
        ui,
        settings,
        "Report Issues",
        usermgmt_lib::constants::ISSUE_LINK,
        None,
    );
    draw_utils::link_box(
        ui,
        settings,
        "Source Code",
        usermgmt_lib::constants::REPOSITORY_LINK,
        None,
    );
}
