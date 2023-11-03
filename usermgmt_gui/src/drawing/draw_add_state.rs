use crate::prelude::*;

use super::draw_utils::{draw_box_group, no_password_enty_field};

pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    {
        let adding_fields = &mut window.adding_state;
        draw_box_group(ui, text_design::group::REQUIRED, |ui| {
            no_password_enty_field(
                ui,
                text_design::label::USERNAME,
                &mut adding_fields.username,
                |_| {},
            );
            no_password_enty_field(
                ui,
                text_design::label::FIRSTNAME,
                &mut adding_fields.firstname,
                |_| {},
            );
            no_password_enty_field(
                ui,
                text_design::label::LASTNAME,
                &mut adding_fields.lastname,
                |_| {},
            );
        });
        draw_box_group(ui, text_design::group::OPTIONAL, |ui| {
            no_password_enty_field(
                ui,
                text_design::label::MAIL,
                &mut adding_fields.mail,
                |_| {},
            );
            no_password_enty_field(
                ui,
                text_design::label::DEFAULT_QOS,
                &mut adding_fields.default_qos,
                |_| {},
            );
            no_password_enty_field(
                ui,
                text_design::label::PUBLIC_KEY,
                &mut adding_fields.publickey,
                |_| {},
            );
            no_password_enty_field(
                ui,
                text_design::label::GROUP,
                &mut adding_fields.group,
                |_| {},
            );
            draw_utils::list_view(ui, &mut adding_fields.qos, text_design::label::QOS);
        });
    }

    draw_utils::draw_credentails(ui, window, true);
    let adding_fields = &mut window.adding_state;
    let last_username = &adding_fields.last_added_username;
    draw_utils::draw_status_msg(
        ui,
        adding_fields.adding_res_io.status(),
        || "No user added yet".to_string(),
        || format!("User ({}) is being added", last_username),
        || format!("User ({}) was added", last_username),
        || format!("Failed to add user ({})", last_username),
    );
    let allow_adding_user = adding_fields.all_needed_fields_filled();
    ui.add_enabled_ui(allow_adding_user, |ui| {
        if ui.button(text_design::button::ACTION_ADD).clicked() {
            if let Err(error) = request_addition_of_user(window) {
                window.adding_state.adding_res_io.set_error(error);
            }
        }
    });

    fn request_addition_of_user(window: &mut UsermgmtWindow) -> AppResult {
        window.adding_state.last_added_username = window.adding_state.username.clone();
        if let Ok(prep) =
            general_utils::prep_conf_creds(window, |app| &mut app.adding_state.adding_res_io, true)
        {
            let adding_state = &mut window.adding_state;
            let to_add = adding_state.create_user_to_add()?;
            let _ = adding_state.adding_res_io.spawn_task(
                move || {
                    usermgmt_lib::add_user(
                        to_add,
                        &prep.on_which_sys,
                        &prep.config,
                        prep.ldap_cred,
                        prep.ssh_cred,
                    )
                },
                String::from("Adding user"),
            );
        }
        Ok(())
    }
}
