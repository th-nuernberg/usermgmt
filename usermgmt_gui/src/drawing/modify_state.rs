use usermgmt_lib::operations;

use crate::{
    current_selected_view::ModifyState, general_utils::PreparationBeforeIoTask, prelude::*,
};

use super::draw_utils::{GroupDrawing, TextFieldEntry};
pub fn draw(ui: &mut egui::Ui, window: &mut UsermgmtWindow) {
    draw_typing_fields(ui, &window.settings, &mut window.modify_state);
    draw_utils::draw_credentials(ui, window, false);
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
        |username| format!("{} {}", text.modify_success(), username),
        || format!("{} {}", text.modify_failure(), &last_username),
    );
}

fn handle_modify_req(window: &mut UsermgmtWindow) {
    window.modify_state.last_added_username = window.modify_state.username.clone();
    if let Ok(PreparationBeforeIoTask {
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
                        let username = changes.username.to_string();
                        operations::modify_user(
                            changes,
                            &on_which_sys,
                            &config,
                            ldap_cred,
                            ssh_cred,
                        )?;
                        Ok(username)
                    },
                    String::from("Modifying User"),
                );
            }
            Err(error) => window.modify_state.res_io.set_error(error),
        }
    }
}

fn draw_typing_fields(ui: &mut egui::Ui, settings: &Settings, modify_state: &mut ModifyState) {
    let texts = settings.texts();
    let tooltips = settings.tooltiptexts();
    draw_utils::draw_box_group(ui, settings, &GroupDrawing::new(texts.required()), |ui| {
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.username(), &mut modify_state.username)
                .with_tooltip(tooltips.username()),
        );
    });
    ui.separator();
    draw_utils::draw_box_group(ui, settings, &GroupDrawing::new(texts.optional()), |ui| {
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.firstname(), &mut modify_state.firstname)
                .with_tooltip(tooltips.firstname()),
        );
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.lastname(), &mut modify_state.lastname)
                .with_tooltip(tooltips.lastname()),
        );
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.mail(), &mut modify_state.mail)
                .with_tooltip(tooltips.email()),
        );
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.group(), &mut modify_state.group)
                .with_tooltip(tooltips.group()),
        );
        draw_utils::entry_field(
            ui,
            settings,
            &mut TextFieldEntry::new(texts.default_qos(), &mut modify_state.default_qos)
                .with_tooltip(tooltips.default_qos()),
        );
        draw_utils::list_view(
            ui,
            settings,
            &mut modify_state.qos,
            &GroupDrawing::new(texts.qos()).with_tooltip(tooltips.qos()),
        );
    });
}
