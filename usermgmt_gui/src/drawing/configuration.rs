use std::path::PathBuf;

use crate::{current_selected_view::ConfigurationState, prelude::*};
use eframe::{egui::RichText, epaint::Color32};
use usermgmt_lib::config::LoadedMgmtConfig;

pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let mut can_reload = true;
    // TODO: use "draw_utils::draw_status_msg()" instead
    match &window.conf_state.io_conf.status() {
        IoTaskStatus::NotStarted => {
            ui.label("No configuration loaded yet");
        }
        IoTaskStatus::Loading => {
            ui.label(RichText::new("Loading configuration").color(Color32::BLUE));
            can_reload = false;
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

    draw_utils::draw_status_msg_w_label(
        ui,
        "Save Progress",
        window.conf_state.io_save_conf.status(),
        // TODO: Supply meaningful messages
        || String::from("a"),
        || String::from("a"),
        || String::from("a"),
        || String::from("a"),
    );

    draw_utils::draw_file_path(ui, window);
    draw_buttons(window, ui, can_reload);
    draw_fields(&mut window.conf_state, ui);
}

fn draw_buttons(window: &mut UsermgmtWindow, ui: &mut egui::Ui, can_reload: bool) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_reload, egui::Button::new("Reload"))
            .clicked()
        {
            let path = window.conf_path.clone();
            general_utils::start_load_config(&mut window.conf_state, Some(path));
        }
        if ui
            .add_enabled(can_reload, egui::Button::new("Save"))
            .clicked()
        {
            let path = window.conf_path.clone();
            save_config(&mut window.conf_state, path);
        }
    });
}

fn save_config(window: &mut ConfigurationState, conf_path: PathBuf) {
    if let IoTaskStatus::Successful(loaded) = window.io_conf.status() {
        let config = loaded.config.clone();
        window.io_save_conf.spawn_task(
            move || config.save(&conf_path),
            String::from("Saving configuration"),
        );
    }
}

fn draw_fields(window: &mut ConfigurationState, ui: &mut egui::Ui) {
    if let IoTaskStatus::Successful(LoadedMgmtConfig { config, .. }) = window.io_conf.status_mut() {
        egui::ScrollArea::vertical()
            .min_scrolled_height(400.0)
            .show(ui, |ui| {
                let mut fields: Vec<ConfiField> = vec![
                    ConfiField::Single {
                        val: &mut config.student_default_qos,
                        label: "Student Default Qos",
                    },
                    ConfiField::Single {
                        val: &mut config.staff_default_qos,
                        label: "Staff Default QOS",
                    },
                    ConfiField::Single {
                        label: "Default SSH Username",
                        val: &mut config.default_ssh_user,
                    },
                    ConfiField::Single {
                        label: "Head/Slurm node address",
                        val: &mut config.head_node,
                    },
                    ConfiField::Single {
                        label: "NFS Host",
                        val: &mut config.nfs_host,
                    },
                    ConfiField::Single {
                        label: "NFS Root Directory",
                        val: &mut config.nfs_root_dir,
                    },
                    ConfiField::List {
                        val: &mut config.valid_qos,
                        label: "Valid Qos",
                    },
                    ConfiField::List {
                        val: &mut config.student_qos,
                        label: "Student Qos",
                    },
                    ConfiField::List {
                        val: &mut config.staff_qos,
                        label: "Staff Qos",
                    },
                    ConfiField::List {
                        val: &mut config.valid_slurm_groups,
                        label: "Valid slurm Groups",
                    },
                    ConfiField::List {
                        val: &mut config.compute_nodes,
                        label: "Computer nodes",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_domain_components,
                        label: "LDAP Domain Components",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_org_unit,
                        label: "LDAP Org Unit",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_bind_org_unit,
                        label: "LDAP Binding Org Unit",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_bind_prefix,
                        label: "LDAP Binding Prefix",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_readonly_user_prefix,
                        label: "LDAP Readonly User Prefix",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_readonly_bind,
                        label: "LDAP Readonly Binding",
                    },
                    ConfiField::Single {
                        val: &mut config.ldap_server,
                        label: "LDAP Server",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_readonly_user,
                        label: "LDAP Readonly User",
                    },
                    ConfiField::SingleOpt {
                        val: &mut config.ldap_readonly_pw,
                        label: "LDAP Readonly Password",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.include_ldap,
                        label: "Include LDAP",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.include_slurm,
                        label: "Incldue Slurm Readonly Password",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.include_dir_mgmt,
                        label: "Include Directory Management",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.use_homedir_helper,
                        label: "Use homedir Helper",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.run_slurm_remote,
                        label: "Runs slurm commands on remote machines",
                    },
                    ConfiField::Checkbox {
                        val: &mut config.ssh_agent,
                        label: "Use ssh agent",
                    },
                    ConfiField::Number {
                        val: &mut config.ssh_port,
                        label: "Ssh port",
                    },
                    ConfiField::Single {
                        val: &mut config.compute_node_root_dir,
                        label: "Computer Node Root Directory",
                    },
                    ConfiField::Single {
                        val: &mut config.filesystem,
                        label: "File System",
                    },
                    ConfiField::Single {
                        val: &mut config.home_filesystem,
                        label: "Home Filesystem",
                    },
                    ConfiField::Single {
                        val: &mut config.nfs_filesystem,
                        label: "NFS Filesystem",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_softlimit,
                        label: "Quota Softlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_hardlimit,
                        label: "Quota Hardlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_nfs_softlimit,
                        label: "Quota Nfs Softlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_nfs_hardlimit,
                        label: "Quota Nfs Hardlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_home_softlimit,
                        label: "Quota Home Softlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.quota_home_hardlimit,
                        label: "Quota Home Hardlimit",
                    },
                    ConfiField::Single {
                        val: &mut config.login_shell,
                        label: "Login Shell",
                    },
                    ConfiField::NegNumber {
                        val: &mut config.student_gid,
                        label: "Sudent GID",
                    },
                    ConfiField::NegNumber {
                        val: &mut config.staff_gid,
                        label: "Staff GID",
                    },
                    ConfiField::NegNumber {
                        val: &mut config.faculty_gid,
                        label: "Faculty GID",
                    },
                    ConfiField::Single {
                        val: &mut config.sacctmgr_path,
                        label: "Sacctmgr Path",
                    },
                ];
                fields.sort();
                for next in fields {
                    let mut draw_sep = true;
                    match next {
                        ConfiField::SingleOpt { val, label } => {
                            draw_utils::no_password_opt_enty_field(ui, label, val);
                        }
                        ConfiField::Single { val, label } => {
                            draw_utils::no_password_enty_field(ui, label, val, |_| {});
                        }
                        ConfiField::List { val, label } => {
                            draw_utils::list_view(ui, val, label);
                            draw_sep = false;
                        }
                        ConfiField::Checkbox { val, label } => _ = ui.checkbox(val, label),
                        ConfiField::Number { val, label } => {
                            draw_utils::number_field(ui, label, val)
                        }
                        ConfiField::NegNumber { val, label } => {
                            draw_utils::neg_number_field(ui, label, val)
                        }
                    }

                    if draw_sep {
                        ui.separator();
                    }
                }
            });
    }
}

#[derive(Debug)]
enum ConfiField<'a> {
    SingleOpt {
        val: &'a mut Option<String>,
        label: &'a str,
    },
    Single {
        val: &'a mut String,
        label: &'a str,
    },
    List {
        val: &'a mut Vec<String>,
        label: &'a str,
    },
    Checkbox {
        val: &'a mut bool,
        label: &'a str,
    },
    Number {
        val: &'a mut u32,
        label: &'a str,
    },
    NegNumber {
        val: &'a mut i32,
        label: &'a str,
    },
}

impl ConfiField<'_> {
    pub fn label(&self) -> &str {
        match self {
            ConfiField::SingleOpt { label, .. }
            | ConfiField::Single { label, .. }
            | ConfiField::List { label, .. }
            | ConfiField::Checkbox { label, .. }
            | ConfiField::Number { label, .. }
            | ConfiField::NegNumber { label, .. } => label,
        }
    }
}

impl PartialEq for ConfiField<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.label() == other.label()
    }
}
impl Eq for ConfiField<'_> {}
impl Ord for ConfiField<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.label().cmp(other.label())
    }
}
impl PartialOrd for ConfiField<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label().cmp(other.label()))
    }
}
