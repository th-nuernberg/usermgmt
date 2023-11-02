use eframe::egui;
use usermgmt_lib::{add_user, prelude::AppResult};

use crate::{
    draw_selected_view::util::{self},
    general_utils,
    usermgmt_window::UsermgmtWindow,
};

use super::util::{draw_box_group, no_password_enty_field};

pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    {
        let adding_fields = &mut window.adding_state;
        draw_box_group(ui, "Required", |ui| {
            no_password_enty_field(ui, "Username", &mut adding_fields.username, |_| {});
            no_password_enty_field(ui, "Firstname: ", &mut adding_fields.firstname, |_| {});
            no_password_enty_field(ui, "Lastname: ", &mut adding_fields.lastname, |_| {});
        });
        draw_box_group(ui, "Optional", |ui| {
            no_password_enty_field(ui, "Mail: ", &mut adding_fields.mail, |_| {});
            no_password_enty_field(ui, "Default Qos: ", &mut adding_fields.default_qos, |_| {});
            no_password_enty_field(ui, "Public key: ", &mut adding_fields.publickey, |_| {});
            draw_box_group(ui, "Quality of services", |ui| {
                if ui.button("Add new qos").clicked() {
                    adding_fields.qos.push(Default::default());
                }
                let mut to_delete: Vec<usize> = Default::default();
                for (index, next_field) in &mut adding_fields.qos.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(next_field);
                        if ui.button("Remove").clicked() {
                            to_delete.push(index);
                        }
                    });
                }
                if !to_delete.is_empty() {
                    let taken = std::mem::take(&mut adding_fields.qos);
                    adding_fields.qos = taken
                        .into_iter()
                        .enumerate()
                        .filter_map(|(index, element)| {
                            if !to_delete.contains(&index) {
                                Some(element)
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            });
        });
    }

    util::draw_credentails(ui, window);
    let adding_fields = &mut window.adding_state;
    let last_username = &adding_fields.last_added_username;
    util::draw_status_msg(
        ui,
        adding_fields.adding_res_io.status(),
        || "No user added yet".to_string(),
        || format!("User ({}) is being added", last_username),
        || format!("User ({}) was added", last_username),
        || format!("Failed to add user ({})", last_username),
    );
    let allow_adding_user = adding_fields.all_needed_fields_filled();
    ui.add_enabled_ui(allow_adding_user, |ui| {
        if ui.button("Add user").clicked() {
            if let Err(error) = request_addition_of_user(window) {
                window.adding_state.adding_res_io.set_error(error);
            }
        }
    });

    fn request_addition_of_user(window: &mut UsermgmtWindow) -> AppResult {
        window.adding_state.last_added_username = window.adding_state.username.clone();
        if let Ok(prep) =
            general_utils::prep_conf_creds(window, |app| &mut app.adding_state.adding_res_io)
        {
            let adding_state = &mut window.adding_state;
            let to_add = adding_state.create_user_to_add()?;
            let _ = adding_state.adding_res_io.spawn_task(
                move || {
                    add_user(
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
