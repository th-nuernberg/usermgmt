use crate::prelude::*;

use super::draw_utils::{draw_box_group, no_password_enty_field};

pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    {
        let texts = window.settings.texts();
        let adding_fields = &mut window.adding_state;
        draw_box_group(ui, texts.required(), |ui| {
            no_password_enty_field(ui, texts.username(), &mut adding_fields.username, |_| {});
            no_password_enty_field(ui, texts.firstname(), &mut adding_fields.firstname, |_| {});
            no_password_enty_field(ui, texts.lastname(), &mut adding_fields.lastname, |_| {});
        });
        draw_box_group(ui, texts.optional(), |ui| {
            no_password_enty_field(ui, texts.mail(), &mut adding_fields.mail, |_| {});
            no_password_enty_field(
                ui,
                texts.default_qos(),
                &mut adding_fields.default_qos,
                |_| {},
            );
            no_password_enty_field(ui, texts.public_key(), &mut adding_fields.publickey, |_| {});
            no_password_enty_field(ui, texts.group(), &mut adding_fields.group, |_| {});
            draw_utils::list_view(ui, &window.settings, &mut adding_fields.qos, texts.qos());
        });
    }

    draw_utils::draw_credentails(ui, window, true);
    let adding_fields = &mut window.adding_state;
    let last_username = &adding_fields.last_added_username;
    draw_utils::draw_status_msg(
        ui,
        &window.settings,
        adding_fields.adding_res_io.status(),
        || "No user added yet".to_string(),
        || format!("User ({}) is being added", last_username),
        |username| format!("User ({}) was added", username),
        || format!("Failed to add user ({})", last_username),
    );
    let allow_adding_user = adding_fields.all_needed_fields_filled();

    ui.add_enabled_ui(allow_adding_user, |ui| {
        let texts = window.settings.texts();
        if ui.button(texts.btn_action_add()).clicked() {
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
            let username = to_add.common_user_fields.username.to_string();
            let _ = adding_state.adding_res_io.spawn_task(
                move || {
                    usermgmt_lib::add_user(
                        to_add,
                        &prep.on_which_sys,
                        &prep.config,
                        prep.ldap_cred,
                        prep.ssh_cred,
                    )?;
                    Ok(username)
                },
                String::from("Adding user"),
            );
        }
        Ok(())
    }
}
