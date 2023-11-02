use std::{path::PathBuf, time::Duration};

use eframe::egui;
use log::{debug, info};
use usermgmt_lib::{config, ldap::LdapSimpleCredential, ssh::SshGivenCredential};

use crate::{
    current_selected_view::{
        AddState, ConfigurationState, CurrentSelectedView, LdapConnectionState, ListingState,
        RemoveState, SshConnectionState,
    },
    draw_selected_view::{draw_selected_view, util::draw_box_group},
    which_systems::WhichSystem,
};

#[derive(Debug)]
pub struct UsermgmtWindow {
    pub selected_view: CurrentSelectedView,
    pub conf_state: ConfigurationState,
    pub conf_path: PathBuf,
    pub listin_state: ListingState,
    pub ssh_state: SshConnectionState,
    pub ldap_state: LdapConnectionState,
    pub which_sys: WhichSystem,
    pub adding_state: AddState,
    pub remove_state: RemoveState,
}

impl Default for UsermgmtWindow {
    fn default() -> Self {
        let mut conf_state: ConfigurationState = Default::default();
        start_load_config(&mut conf_state, None);

        Self {
            listin_state: Default::default(),
            selected_view: Default::default(),
            ssh_state: Default::default(),
            conf_path: Default::default(),
            which_sys: Default::default(),
            ldap_state: Default::default(),
            adding_state: Default::default(),
            remove_state: Default::default(),
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

    pub fn conf_path_owned(&self) -> String {
        self.conf_path.to_string_lossy().to_string()
    }
    pub fn set_conf_path(&mut self, new: impl Into<PathBuf>) {
        self.conf_path = new.into();
    }

    pub fn is_ssh_cred_needed(&self, supporsts_dir: bool) -> bool {
        self.which_sys.is_ssh_cred_needed(supporsts_dir)
    }
    pub fn is_ldap_needed(&self) -> bool {
        self.which_sys.is_ldap_needed()
    }

    pub fn create_ssh_credentials(&self) -> Option<SshGivenCredential> {
        let ssh_state = &self.ssh_state;
        let (username, password) = (ssh_state.username.as_ref(), ssh_state.password.as_ref());
        let cred = SshGivenCredential::new(username?, password?);
        Some(cred)
    }
    pub fn create_ldap_credentials(&self) -> Option<LdapSimpleCredential> {
        let ldap_state = &self.ldap_state;
        let (username, password) = (ldap_state.username.as_ref(), ldap_state.password.as_ref());

        let cred = LdapSimpleCredential::new(username?.to_owned(), password?.to_owned());
        Some(cred)
    }
}

impl eframe::App for UsermgmtWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            query_pending_io_taks(self);
            ui.horizontal(|ui| {
                ui.vertical(|ui| draw_box_group(ui, "Actions", |ui| ui_action_menu(self, ui)));
                ui.vertical(|ui| draw_selected_view(self, ui));
            });
        });
    }
}

fn query_pending_io_taks(window: &mut UsermgmtWindow) {
    if let Some(conf) = window.conf_state.io_conf.query_task() {
        let listing_state = &mut window.listin_state;
        let ssh_state = &mut window.ssh_state;
        let path = &mut window.conf_path;
        *path = conf.path.to_owned();
        let config = &conf.config;
        if listing_state.rw_user_name.is_none() {
            if let Some(rw_user) = config.ldap_readonly_user.as_deref() {
                listing_state.rw_user_name = Some(rw_user.to_owned());
            }
        }
        if listing_state.rw_pw.is_none() {
            if let Some(rw_password) = config.ldap_readonly_pw.as_deref() {
                listing_state.rw_pw = Some(rw_password.to_owned());
            }
        }
        if ssh_state.username.is_none() && !config.default_ssh_user.is_empty() {
            debug!("GUI: Ssh user name taken from default ssh user in loaded config");
            ssh_state.username = Some(config.default_ssh_user.to_owned());
        }
    }
    let _ = window.listin_state.list_ldap_res.query_task();
    let _ = window.listin_state.list_slurm_user_res.query_task();
    let _ = window.adding_state.adding_res_io.query_task();
    let _ = window.remove_state.remove_res_io.query_task();
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
        if ui.button(view.create_str()).clicked() {
            let previous_view = window.selected_view();
            info!("Changed from ({:?}) to ({:?}) view", previous_view, view);
            window.set_selected_view(view);
            ui.close_menu();
        }
    }
}

pub fn start_load_config(conf_state: &mut ConfigurationState, path: Option<PathBuf>) {
    conf_state.io_conf.spawn_task(
        || {
            std::thread::sleep(Duration::from_secs(2));
            let loaded = config::load_config(path)?;
            Ok(loaded)
        },
        "Loading configuration".to_string(),
    );
}
