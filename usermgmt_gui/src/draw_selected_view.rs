use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use usermgmt_lib::ldap::{list_ldap_users, text_list_output, LDAPConfig, LdapSimpleCredential};

use crate::{
    current_selected_view::CurrentSelectedView, gui_design::WHICH_GUI_VIEW_SIZE,
    io_resource_manager::IoTaskStatus, usermgmt_window::UsermgmtWindow,
};

pub fn draw_selected_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let current_view = window.selected_view();
    ui.label(RichText::new(current_view.to_str()).size(WHICH_GUI_VIEW_SIZE));
    match current_view {
        CurrentSelectedView::Configuration => draw_configuration_view(window, ui),
        CurrentSelectedView::Listing => draw_listing_view(window, ui),
        _ => not_implemented_yet(current_view.to_str(), ui),
    }

    fn not_implemented_yet(action_name: &str, ui: &mut egui::Ui) {
        ui.label(format!("The action {} is not implemented yet", action_name));
    }
}
fn draw_listing_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let (conf_user_name, conf_pw) =
        if let IoTaskStatus::Successful(configuration) = &window.conf_state.io_conf.status() {
            (
                configuration.ldap_readonly_user.as_deref(),
                configuration.ldap_readonly_pw.as_deref(),
            )
        } else {
            Default::default()
        };
    ui.horizontal(|ui| {
        let mut rw_user =
            field_conf_or_state(window.listin_state.rw_user_name.as_deref(), conf_user_name);

        ui.label("Readonly Username: ");
        if ui.text_edit_singleline(&mut rw_user).changed() {
            window.listin_state.rw_user_name = Some(rw_user);
        }
    });
    ui.horizontal(|ui| {
        let mut rw_password = field_conf_or_state(window.listin_state.rw_pw.as_deref(), conf_pw);
        ui.label("Readonly Password: ");

        if ui
            .add(egui::TextEdit::singleline(&mut rw_password).password(true))
            .changed()
        {
            window.listin_state.rw_pw = Some(rw_password);
        }
    });

    let list_ldap_btn_enabled = {
        let list_state = &window.listin_state;
        let conf_is_there = !list_state.list_ldap_res.is_there();
        let enabled =
            list_state.rw_user_name.is_some() && list_state.rw_pw.is_some() && conf_is_there;
        enabled
    };

    if ui
        .add_enabled(list_ldap_btn_enabled, egui::Button::new("List ldap users"))
        .clicked()
    {
        if let IoTaskStatus::Successful(mgmt_conf) = &window.conf_state.io_conf.status() {
            let lising_state = &window.listin_state;
            let (username, password) = (
                lising_state.rw_user_name.clone().unwrap(),
                lising_state.rw_pw.clone().unwrap(),
            );
            let mgmt_conf = mgmt_conf.clone();
            window.listin_state.list_ldap_res.spawn_task(
                move || {
                    let config =
                        LDAPConfig::new(&mgmt_conf, LdapSimpleCredential::new(username, password))
                            .unwrap();
                    list_ldap_users(config)
                        .map(|to_map| text_list_output::ldap_search_to_pretty_table(&to_map))
                },
                "Listing ldap user".to_owned(),
            );
        } else {
            unreachable!();
        }
    };

    let listing_state = &window.listin_state;

    match listing_state.list_ldap_res.status() {
        IoTaskStatus::NotStarted => ui.label("No ldap user listed yet."),
        IoTaskStatus::Loading => ui.label("Fetching ldap users"),
        IoTaskStatus::Successful(listed_ldap_user) => ui.label(listed_ldap_user),
        IoTaskStatus::Failed(error) => ui.label(format!("Failed to fetch ldpa users:\n{}", error)),
    };

    fn field_conf_or_state(from_window: Option<&str>, from_conf: Option<&str>) -> String {
        from_window
            .unwrap_or(from_conf.unwrap_or_default())
            .to_owned()
    }
}

fn draw_configuration_view(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    match &window.conf_state.io_conf.status() {
        IoTaskStatus::NotStarted => {
            ui.label("No configuration loaded yet");
        }
        IoTaskStatus::Loading => {
            ui.label(RichText::new("Loading configuration").color(Color32::BLUE));
        }
        IoTaskStatus::Successful(_) => {
            ui.label(RichText::new("Configuration loaded").color(Color32::GREEN));
        }
        IoTaskStatus::Failed(error) => {
            ui.label(
                RichText::new("Error in loadiding configuration.")
                    .strong()
                    .color(Color32::RED),
            );
            ui.label(RichText::new(format!("Error details. {}", error)).color(Color32::RED));
        }
    }
}
