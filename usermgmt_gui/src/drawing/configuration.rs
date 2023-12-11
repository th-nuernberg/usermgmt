use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

type LabelTyp = Rc<str>;
pub type CacheForConfFiels = Rc<RefCell<HashMap<&'static str, LabelTyp>>>;
use crate::{current_selected_view::ConfigurationState, prelude::*};
use usermgmt_lib::config::{LoadedMgmtConfig, MgmtConfig};

use super::draw_utils::{GroupDrawing, TextFieldEntry};

pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let mut can_reload = true;
    let settings = &window.settings;
    let texts = settings.texts();
    draw_utils::draw_status_msg_w_label(
        ui,
        settings,
        texts.conf_load_group(),
        window.conf_state.io_conf.status(),
        || texts.conf_load_init_msg().to_owned(),
        || {
            can_reload = false;
            texts.conf_load_loading_msg().to_owned()
        },
        |loaded| {
            format!(
                "{}:\n{:?}",
                texts.conf_load_success_msg().to_owned(),
                &loaded.path
            )
        },
        || texts.conf_load_err_msg().to_owned(),
    );

    draw_utils::draw_status_msg_w_label(
        ui,
        settings,
        texts.conf_save_group(),
        window.conf_state.io_save_conf.status(),
        || texts.conf_save_init_msg().to_owned(),
        || texts.conf_save_loading_msg().to_owned(),
        |path| format!("{}:\n{:?}", texts.conf_save_success_msg().to_owned(), path),
        || texts.conf_save_err_msg().to_owned(),
    );

    draw_utils::draw_file_path(ui, window);
    ui.separator();
    draw_buttons(window, ui, can_reload);
    ui.separator();
    ui.add_space(20.0);
    draw_fields(&mut window.conf_state, &window.settings, ui);
}

fn draw_buttons(window: &mut UsermgmtWindow, ui: &mut egui::Ui, can_reload: bool) {
    let text = &window.settings.texts();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_reload, egui::Button::new(text.btn_action_conf_load()))
            .clicked()
        {
            let path = window.conf_path.clone();
            general_utils::start_load_config(&mut window.conf_state, Some(path));
        }

        let can_save = can_reload && !window.conf_state.io_save_conf.is_loading();
        if ui
            .add_enabled(can_save, egui::Button::new(text.btn_action_conf_save()))
            .clicked()
        {
            let path = window.conf_path.clone();
            save_config(&mut window.conf_state, path);
        }
        if ui
            .add_enabled(can_save, egui::Button::new(text.btn_action_conf_default()))
            .clicked()
        {
            let default = MgmtConfig::default();
            let loaded_conf = LoadedMgmtConfig {
                path: Default::default(),
                config: default,
            };
            window.conf_state.io_conf.set_success(loaded_conf);
        }
    });
}

fn save_config(config_state: &mut ConfigurationState, conf_path: PathBuf) {
    if let IoTaskStatus::Successful(loaded) = config_state.io_conf.status() {
        let config = loaded.config.clone();
        config_state.io_save_conf.spawn_task(
            move || config.save(&conf_path),
            String::from("Saving configuration"),
        );
    } else {
        config_state
            .io_save_conf
            .set_error(anyhow!("There is no loaded configuration to be saved"));
    }
}

fn draw_fields(window: &mut ConfigurationState, settings: &Settings, ui: &mut egui::Ui) {
    if let IoTaskStatus::Successful(LoadedMgmtConfig { config, .. }) = window.io_conf.status_mut() {
        egui::ScrollArea::vertical()
            .min_scrolled_height(400.0)
            .show(ui, |ui| {
                let map = window.gui_field_cache.clone();
                let fields = construct_fields(settings, config, map);
                for next in fields {
                    let mut draw_sep = true;
                    match next {
                        ConfiField::SingleOpt {
                            val,
                            label,
                            tool_tip,
                        } => {
                            draw_utils::entry_field(
                                ui,
                                settings,
                                &mut TextFieldEntry::new_opt(&label, val).tool_tip(tool_tip),
                            );
                        }
                        ConfiField::Single {
                            val,
                            label,
                            tool_tip,
                        } => {
                            draw_utils::entry_field(
                                ui,
                                settings,
                                &mut TextFieldEntry::new(&label, val).tool_tip(tool_tip),
                            );
                        }
                        ConfiField::List {
                            val,
                            label,
                            tool_tip,
                        } => {
                            draw_utils::list_view(
                                ui,
                                settings,
                                val,
                                &GroupDrawing::new(&label).tooltip(tool_tip),
                            );
                            draw_sep = false;
                        }
                        ConfiField::Checkbox {
                            val,
                            label,
                            tool_tip,
                        } => {
                            ui.horizontal(|ui| {
                                _ = ui.checkbox(val, &*label);
                                if let Some(tool_tip) = tool_tip {
                                    draw_utils::tooltip_widget(ui, settings, tool_tip);
                                }
                            });
                        }
                        ConfiField::Number {
                            val,
                            label,
                            tool_tip,
                        } => {
                            draw_utils::whole_pos_number_fields(
                                ui, settings, &label, val, tool_tip,
                            );
                        }
                        ConfiField::NegNumber {
                            val,
                            label,
                            tool_tip,
                        } => {
                            draw_utils::whole_neg_number_fields(
                                ui, settings, &label, val, tool_tip,
                            );
                        }
                    }

                    if draw_sep {
                        ui.separator();
                    }
                }
            });
    }
}

fn snake_to_label(input: &'static str, repos: CacheForConfFiels) -> Rc<str> {
    const SPLIT_BY: char = '_';
    const JOIN_BY: &str = " ";
    let mut repos = repos.borrow_mut();
    Rc::clone(repos.entry(input).or_insert_with(|| {
        Rc::from(
            input
                .split(SPLIT_BY)
                .map(|word| {
                    let first = word.chars().next().unwrap().to_uppercase();
                    let list: String = first.chain(word.chars().skip(1)).collect();
                    list
                })
                .collect::<Vec<String>>()
                .join(JOIN_BY),
        )
    }))
}

fn construct_fields<'a>(
    settings: &'a Settings,
    config: &'a mut MgmtConfig,
    map: CacheForConfFiels,
) -> Vec<ConfiField<'a>> {
    macro_rules! create_conf_field {
        ($field:ident, $too_tip:expr) => {
            (
                &mut config.$field,
                snake_to_label(stringify!($field), map.clone()),
                Some($too_tip.as_ref()),
            )
                .into()
        };
        ($field:ident) => {
            (
                &mut config.$field,
                snake_to_label(stringify!($field), map.clone()),
                None,
            )
                .into()
        };
    }
    let tool_tip_text = settings.tooltiptexts();

    let mut fields: Vec<ConfiField> = vec![
        create_conf_field!(student_default_qos, tool_tip_text.conf_student_qos()),
        create_conf_field!(staff_default_qos, tool_tip_text.conf_staff_default_qos()),
        create_conf_field!(default_ssh_user, tool_tip_text.conf_default_ssh_user()),
        create_conf_field!(head_node, tool_tip_text.conf_head_node()),
        create_conf_field!(nfs_host, tool_tip_text.conf_nfs_host()),
        create_conf_field!(nfs_root_dir, tool_tip_text.conf_nfs_root_dir()),
        create_conf_field!(valid_qos, tool_tip_text.conf_valid_qos()),
        create_conf_field!(student_qos, tool_tip_text.conf_student_qos()),
        create_conf_field!(staff_qos, tool_tip_text.conf_staff_qos()),
        create_conf_field!(valid_slurm_groups, tool_tip_text.conf_valid_slurm_groups()),
        create_conf_field!(compute_nodes, tool_tip_text.conf_compute_nodes()),
        create_conf_field!(
            ldap_domain_components,
            tool_tip_text.conf_ldap_domain_components()
        ),
        create_conf_field!(ldap_org_unit, tool_tip_text.conf_ldap_org_unit()),
        create_conf_field!(ldap_bind_org_unit, tool_tip_text.conf_ldap_bind_org_unit()),
        create_conf_field!(ldap_bind_prefix, tool_tip_text.conf_ldap_bind_prefix()),
        create_conf_field!(
            ldap_readonly_user_prefix,
            tool_tip_text.conf_ldap_readonly_user_prefix()
        ),
        create_conf_field!(ldap_readonly_bind, tool_tip_text.conf_ldap_readonly_bind()),
        create_conf_field!(ldap_server, tool_tip_text.conf_ldap_server()),
        create_conf_field!(ldap_readonly_user, tool_tip_text.conf_ldap_readonly_user()),
        create_conf_field!(ldap_readonly_pw, tool_tip_text.conf_ldap_readonly_pw()),
        create_conf_field!(include_ldap, tool_tip_text.conf_include_ldap()),
        create_conf_field!(include_slurm, tool_tip_text.conf_include_slurm()),
        create_conf_field!(include_dir_mgmt, tool_tip_text.conf_include_dir_mgmt()),
        create_conf_field!(use_homedir_helper, tool_tip_text.conf_use_homedir_helper()),
        create_conf_field!(run_slurm_remote, tool_tip_text.conf_run_slurm_remote()),
        create_conf_field!(ssh_agent, tool_tip_text.conf_ssh_agent()),
        create_conf_field!(ssh_port, tool_tip_text.conf_ssh_port()),
        create_conf_field!(
            compute_node_root_dir,
            tool_tip_text.conf_compute_node_root_dir()
        ),
        create_conf_field!(filesystem, tool_tip_text.conf_filesystem()),
        create_conf_field!(home_filesystem, tool_tip_text.conf_home_filesystem()),
        create_conf_field!(nfs_filesystem, tool_tip_text.conf_nfs_filesystem()),
        create_conf_field!(quota_softlimit, tool_tip_text.conf_quota_softlimit()),
        create_conf_field!(quota_hardlimit, tool_tip_text.conf_quota_nfs_hardlimit()),
        create_conf_field!(
            quota_nfs_softlimit,
            tool_tip_text.conf_quota_nfs_softlimit()
        ),
        create_conf_field!(
            quota_nfs_hardlimit,
            tool_tip_text.conf_quota_nfs_hardlimit()
        ),
        create_conf_field!(
            quota_home_softlimit,
            tool_tip_text.conf_quota_home_softlimit()
        ),
        create_conf_field!(
            quota_home_hardlimit,
            tool_tip_text.conf_quota_home_hardlimit()
        ),
        create_conf_field!(login_shell, tool_tip_text.conf_login_shell()),
        create_conf_field!(student_gid, tool_tip_text.conf_student_gid()),
        create_conf_field!(staff_gid, tool_tip_text.conf_staff_gid()),
        create_conf_field!(faculty_gid, tool_tip_text.conf_faculty_gid()),
        create_conf_field!(sacctmgr_path, tool_tip_text.conf_sacctmgr_path()),
    ];
    fields.sort();
    fields
}

/// Encapsulate a value from the app configuration.
/// Every variant is mapped to a draw function which lets the user edit the respective value.
/// The drawing of the values is alphabetically ordered by the field called label in every variant.
#[derive(Debug)]
enum ConfiField<'a> {
    SingleOpt {
        val: &'a mut Option<String>,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
    },
    Single {
        val: &'a mut String,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
    },
    List {
        val: &'a mut Vec<String>,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
    },
    Checkbox {
        val: &'a mut bool,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
    },
    Number {
        val: &'a mut u32,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
    },
    NegNumber {
        val: &'a mut i32,
        label: LabelTyp,
        tool_tip: Option<&'a str>,
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

type ToolTippLabel<'a> = Option<&'a str>;

/// Generates conversion from a rust value into ConfiField which
/// is used to render a value in egui.
/// # Usage
/// ```text
/// // 1. Type to convert from
/// // 2. Variant as a result after conversion
/// impl_from_conf_field!(bool, Checkbox);
/// ```
macro_rules! impl_from_conf_field {
    ($type:ty, $variant:ident) => {
        impl<'a> From<(&'a mut $type, LabelTyp, ToolTippLabel<'a>)> for ConfiField<'a> {
            fn from((val, label, tool_tip): (&'a mut $type, LabelTyp, ToolTippLabel<'a>)) -> Self {
                Self::$variant {
                    val,
                    label,
                    tool_tip,
                }
            }
        }
    };
}

impl_from_conf_field!(u32, Number);
impl_from_conf_field!(i32, NegNumber);
impl_from_conf_field!(String, Single);
impl_from_conf_field!(Vec<String>, List);
impl_from_conf_field!(Option<String>, SingleOpt);
impl_from_conf_field!(bool, Checkbox);
