use eframe::egui;
use log::info;
use usermgmt_lib::config::load_config;

use crate::{
    current_selected_view::{ConfigurationState, CurrentSelectedView, ListingState},
    draw_selected_view::draw_selected_view,
};

#[derive(Debug)]
pub struct UsermgmtWindow {
    pub selected_view: CurrentSelectedView,
    pub conf_state: ConfigurationState,
    pub listin_state: ListingState,
}

impl Default for UsermgmtWindow {
    fn default() -> Self {
        let mut conf_state: ConfigurationState = Default::default();
        conf_state
            .io_conf
            .spawn_task(|| load_config(), "Loading configuration".to_string());

        Self {
            listin_state: Default::default(),
            selected_view: Default::default(),
            conf_state,
        }
    }
}

impl UsermgmtWindow {
    pub fn selected_view(&self) -> CurrentSelectedView {
        self.selected_view
    }

    pub fn set_selected_view(&mut self, selected_view: CurrentSelectedView) {
        self.selected_view = selected_view;
    }
}

impl eframe::App for UsermgmtWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_top_general(self, ui);
            ui.separator();
            query_pending_io_taks(self);
            draw_selected_view(self, ui);
        });
    }
}

fn ui_top_general(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    ui.menu_button("Actions", |ui| ui_action_menu(window, ui));
}
fn query_pending_io_taks(window: &mut UsermgmtWindow) {
    if let Some(conf) = window.conf_state.io_conf.query_task() {
        let listing_state = &mut window.listin_state;
        if listing_state.rw_user_name.is_none() {
            if let Some(rw_user) = conf.ldap_readonly_user.as_deref() {
                listing_state.rw_user_name = Some(rw_user.to_owned());
            }
        }
        if listing_state.rw_pw.is_none() {
            if let Some(rw_password) = conf.ldap_readonly_pw.as_deref() {
                listing_state.rw_pw = Some(rw_password.to_owned());
            }
        }
    }
    let _ = window.listin_state.list_ldap_res.query_task();
}
fn ui_action_menu(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    change_to_if_clicked(window, ui, CurrentSelectedView::LdapConnection);
    change_to_if_clicked(window, ui, CurrentSelectedView::SshConnection);
    change_to_if_clicked(window, ui, CurrentSelectedView::Configuration);
    change_to_if_clicked(window, ui, CurrentSelectedView::Adding);
    change_to_if_clicked(window, ui, CurrentSelectedView::Removing);
    change_to_if_clicked(window, ui, CurrentSelectedView::Modifing);
    change_to_if_clicked(window, ui, CurrentSelectedView::Listing);

    fn change_to_if_clicked(
        window: &mut UsermgmtWindow,
        ui: &mut egui::Ui,
        view: CurrentSelectedView,
    ) {
        if ui.button(view.to_str()).clicked() {
            let previous_view = window.selected_view();
            info!("Changed from ({:?}) to ({:?}) view", previous_view, view);
            window.set_selected_view(view);
            ui.close_menu();
        }
    }
}
