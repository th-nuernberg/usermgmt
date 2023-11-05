use crate::{
    current_selected_view::ModifyState, general_utils::PreparationBeforIoTask, prelude::*,
};
pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    draw_typing_fields(ui, &window.settings, &mut window.modify_state);
    draw_utils::draw_credentails(ui, window, false);
    ui.separator();
    if ui.button("Modify User").clicked() {
        handle_modify_req(window);
    }
    ui.separator();
    let last_username = &window.modify_state.last_added_username;
    let text = window.settings.texts();
    draw_utils::draw_status_msg(
        ui,
        &window.settings,
        window.modify_state.res_io.status(),
        || text.modify_init().to_string(),
        || format!("{} {}", text.modify_loading(), &last_username),
        || format!("{} {}", text.modify_success(), &last_username),
        || format!("{} {}", text.modify_failure(), &last_username),
    );
}

fn handle_modify_req(window: &mut UsermgmtWindow) {
    window.modify_state.last_added_username = window.modify_state.username.clone();
    if let Ok(PreparationBeforIoTask {
        ldap_cred,
        ssh_cred,
        config,
        on_which_sys,
    }) = general_utils::prep_conf_creds(window, |app| &mut app.adding_state.adding_res_io, true)
    {
        match window.modify_state.create_changes_to_user(&config) {
            Ok(changes) => {
                window.modify_state.res_io.spawn_task(
                    move || {
                        usermgmt_lib::modify_user(
                            changes,
                            &on_which_sys,
                            &config,
                            ldap_cred,
                            ssh_cred,
                        )
                    },
                    String::from("Modifing User"),
                );
            }
            Err(error) => window.modify_state.res_io.set_error(error),
        }
    }
}

fn draw_typing_fields(ui: &mut egui::Ui, settings: &Settings, modify_state: &mut ModifyState) {
    let texts = settings.texts();
    draw_utils::draw_box_group(ui, texts.required(), |ui| {
        draw_utils::no_password_enty_field(
            ui,
            texts.username(),
            &mut modify_state.username,
            |_| {},
        );
    });
    ui.separator();
    draw_utils::draw_box_group(ui, texts.optional(), |ui| {
        draw_utils::no_password_enty_field(
            ui,
            texts.firstname(),
            &mut modify_state.firstname,
            |_| {},
        );
        draw_utils::no_password_enty_field(
            ui,
            texts.lastname(),
            &mut modify_state.lastname,
            |_| {},
        );
        draw_utils::no_password_enty_field(ui, texts.mail(), &mut modify_state.mail, |_| {});
        draw_utils::no_password_enty_field(ui, texts.group(), &mut modify_state.group, |_| {});
        draw_utils::no_password_enty_field(
            ui,
            texts.default_qos(),
            &mut modify_state.default_qos,
            |_| {},
        );
        draw_utils::list_view(ui, settings, &mut modify_state.qos, texts.qos());
    });
}
