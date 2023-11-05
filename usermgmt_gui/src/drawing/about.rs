use crate::prelude::*;
pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let settings = &window.settings;
    draw_utils::box_centered_single_line(ui, settings, "Version", env!("CARGO_PKG_VERSION"));
    draw_utils::link_box(ui, "Readme", usermgmt_lib::constants::README_LINK, None);
    draw_utils::link_box(
        ui,
        "License",
        usermgmt_lib::constants::MIT_LINK,
        Some("MIT"),
    );
    draw_utils::link_box(
        ui,
        "Where to report bugs",
        usermgmt_lib::constants::ISSUE_LINK,
        None,
    );
    draw_utils::link_box(
        ui,
        "Link to source code",
        usermgmt_lib::constants::REPOSITORY_LINK,
        None,
    );
}
