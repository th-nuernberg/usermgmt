use eframe::egui;
use log::info;
use usermgmt_lib::config::load_config;

use crate::{
    current_selected_view::{ConfigurationState, CurrentSelectedView},
    draw_selected_view::draw_selected_view,
    io_task_status::IoTaskStatus,
};

#[derive(Debug)]
pub struct UsermgmtWindow {
    pub selected_view: CurrentSelectedView,
    pub conf_state: ConfigurationState,
}

impl Default for UsermgmtWindow {
    fn default() -> Self {
        let mut conf_state: ConfigurationState = Default::default();
        conf_state.io_status_conf = IoTaskStatus::Loading;
        conf_state
            .io_load_conf
            .spawn(|| load_config(), "Loading configuration".to_string())
            .unwrap();

        Self {
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
            draw_selected_view(self, ui);
        });
    }
}

fn ui_top_general(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    ui.menu_button("Actions", |ui| ui_action_menu(window, ui));
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
