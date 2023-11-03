use std::path::PathBuf;

use crate::{current_selected_view::ConfigurationState, prelude::*};
use eframe::{egui::RichText, epaint::Color32};
use usermgmt_lib::config::LoadedMgmtConfig;

pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let mut can_reload = true;
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
    let file_path = conf_path.join("conf.toml");
    if let IoTaskStatus::Successful(loaded) = window.io_conf.status() {
        let config = loaded.config.clone();
        window.io_save_conf.spawn_task(
            move || config.save(&file_path),
            String::from("Saving configuration"),
        );
    }
}

fn draw_fields(window: &mut ConfigurationState, ui: &mut egui::Ui) {
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
    }

    impl ConfiField<'_> {
        pub fn label(&self) -> &str {
            match self {
                ConfiField::SingleOpt { label, .. } => &label,
                ConfiField::Single { label, .. } => &label,
                ConfiField::List { label, .. } => &label,
                ConfiField::Checkbox { label, .. } => &label,
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
            self.label().partial_cmp(other.label())
        }
    }

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
                    }

                    if draw_sep {
                        ui.separator();
                    }
                }
            });
    }
}

// # this configuration is set up for development
// # via docker container serving as end points for ldap and slurm nodes
//
// # ldap bind for user is cn=admin,dc=example,dc=org in docker set up
//
// x student_default_qos = 'basic'
// x staff_default_qos = 'advanced'
// x student_qos = ['interactive', 'basic']
// x staff_qos = ['interactive', 'advanced']
// x valid_qos = ['interactive', 'basic', 'advanced']
// x valid_slurm_groups = ['staff', 'student']
// objectclass_common = [
//     'inetOrgPerson',
//     'ldapPublicKey',
//     'organizationalPerson',
//     'person',
//     'posixAccount',
//     'shadowAccount',
//     'slurmRole',
//     'top',
// ]
// x compute_nodes = ['m0.host.de', 'ml1.host.de']
// # Static ip address of docker container as head node for development
// x head_node = '172.25.0.13'
// # Name of user with slurm admin rights in the slurmdb of the development docker set up.
// x default_ssh_user = 'dev_user'
// # Static ip address of docker container as head node for development
// x home_host = '172.25.0.13'
// x nfs_host = 'nfs.server.de'
// x nfs_root_dir = '/mnt/md0/scratch'
// compute_node_root_dir = '/mnt/md0/user'
// filesystem = '/mnt/md0'
// home_filesystem = '/dev/sdb4'
// nfs_filesystem = '/dev/sda1'
// quota_softlimit = '200G'
// quota_hardlimit = '220G'
// quota_nfs_softlimit = '200G'
// quota_nfs_hardlimit = '220G'
// quota_home_softlimit = '20G'
// quota_home_hardlimit = '22G'
// login_shell = '/bin/bash'
// student_gid = 1002
// staff_gid = 1001
// faculty_gid = 1000
// # In development the access to slurm is remote via a docker container
// sacctmgr_path = 'sacctmgr'
//x ldap_domain_components = 'dc=example,dc=org'
//x ldap_org_unit = 'ou=people'
//x # ldap_bind_org_unit = 'ou=people'
//x # ldap_bind_prefix = ''
//x # ldap_readonly_user_prefix = "read_only_uid"
//x # ldap_readonly_bind = "ou=readonly,ou=realm"
//x ldap_server = 'ldap://localhost:389'
//x ldap_readonly_user = 'admin'
//x ldap_readonly_pw = 'admin'
//x include_slurm = true
//x include_ldap = true
//x include_dir_mgmt = false
// # We do not support directory management in the docker development set up
// use_homedir_helper = false
// run_slurm_remote = true
// ssh_agent = true
// ssh_port = 22
