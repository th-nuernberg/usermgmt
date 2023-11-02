use eframe::egui;

use crate::{general_utils, usermgmt_window::UsermgmtWindow};

use super::util;

pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    let allow_deletion = {
        let remove_state = &mut window.remove_state;
        util::draw_box_group(ui, "Required", |ui| {
            util::no_password_enty_field(ui, "Username", &mut remove_state.username, |_| {});
        });
        !remove_state.username.trim().is_empty()
    };
    util::draw_credentails(ui, window, false);
    ui.add_enabled_ui(allow_deletion, |ui| {
        if ui.button("Delete").clicked() {
            delte_user(window)
        }
    });
    let remove_state = &mut window.remove_state;
    let last_username = &remove_state.last_username;
    util::draw_status_msg(
        ui,
        remove_state.remove_res_io.status(),
        || "No user remove yet".to_owned(),
        || format!("In the process of removing user ({}).", last_username),
        || format!("Removed user ({}) !", last_username),
        || format!("Failed to remove user ({}).", last_username),
    );
}

fn delte_user(window: &mut UsermgmtWindow) {
    window.remove_state.last_username = window.remove_state.username.clone();
    if let Ok(prep) =
        general_utils::prep_conf_creds(window, |app| &mut app.remove_state.remove_res_io, false)
    {
        let username = window.remove_state.username.clone();
        let _ = window.remove_state.remove_res_io.spawn_task(
            move || {
                usermgmt_lib::delete_user(
                    &username,
                    &prep.on_which_sys,
                    &prep.config,
                    prep.ldap_cred,
                    prep.ssh_cred,
                )
            },
            String::from("Deleting user"),
        );
    }
}
