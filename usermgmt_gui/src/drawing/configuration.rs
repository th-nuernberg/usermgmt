use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

type LabelTyp = Rc<str>;
pub type CacheForConfFiels = Rc<RefCell<HashMap<&'static str, LabelTyp>>>;
use crate::{current_selected_view::ConfigurationState, prelude::*};
use usermgmt_lib::config::{LoadedMgmtConfig, MgmtConfig};

pub fn draw(window: &mut UsermgmtWindow, ui: &mut egui::Ui) {
    let mut can_reload = true;
    // TODO: use "draw_utils::draw_status_msg()" instead
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
        || texts.conf_load_success_msg().to_owned(),
        || texts.conf_load_err_msg().to_owned(),
    );

    draw_utils::draw_status_msg_w_label(
        ui,
        settings,
        texts.conf_save_group(),
        window.conf_state.io_save_conf.status(),
        // TODO: Supply meaningful messages
        || texts.conf_save_init_msg().to_owned(),
        || texts.conf_save_loading_msg().to_owned(),
        || texts.conf_save_success_msg().to_owned(),
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

fn save_config(config_state: &mut ConfigurationState, conf_path: PathBuf) {
    if let IoTaskStatus::Successful(loaded) = config_state.io_conf.status() {
        let config = loaded.config.clone();
        config_state.io_save_conf.spawn_task(
            move || config.save(&conf_path),
            String::from("Saving configuration"),
        );
    }
}

fn draw_fields(window: &mut ConfigurationState, settings: &Settings, ui: &mut egui::Ui) {
    if let IoTaskStatus::Successful(LoadedMgmtConfig { config, .. }) = window.io_conf.status_mut() {
        egui::ScrollArea::vertical()
            .min_scrolled_height(400.0)
            .show(ui, |ui| {
                let map = window.gui_field_cache.clone();
                let fields = construct_fields(config, map);
                for next in fields {
                    let mut draw_sep = true;
                    match next {
                        ConfiField::SingleOpt { val, label } => {
                            draw_utils::no_password_opt_enty_field(ui, &label, val);
                        }
                        ConfiField::Single { val, label } => {
                            draw_utils::no_password_enty_field(ui, &label, val, |_| {});
                        }
                        ConfiField::List { val, label } => {
                            draw_utils::list_view(ui, settings, val, &label);
                            draw_sep = false;
                        }
                        ConfiField::Checkbox { val, label } => _ = ui.checkbox(val, &*label),
                        ConfiField::Number { val, label } => {
                            draw_utils::whole_pos_number_fields(ui, &label, val);
                        }
                        ConfiField::NegNumber { val, label } => {
                            draw_utils::whole_neg_number_fields(ui, &label, val);
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

fn construct_fields(config: &mut MgmtConfig, map: CacheForConfFiels) -> Vec<ConfiField> {
    macro_rules! create_conf_field {
        ($field:ident) => {
            (
                &mut config.$field,
                snake_to_label(stringify!($field), map.clone()),
            )
                .into()
        };
    }
    let mut fields: Vec<ConfiField> = vec![
        create_conf_field!(student_default_qos),
        create_conf_field!(staff_default_qos),
        create_conf_field!(default_ssh_user),
        create_conf_field!(head_node),
        create_conf_field!(nfs_host),
        create_conf_field!(nfs_root_dir),
        create_conf_field!(valid_qos),
        create_conf_field!(student_qos),
        create_conf_field!(staff_qos),
        create_conf_field!(valid_slurm_groups),
        create_conf_field!(compute_nodes),
        create_conf_field!(ldap_domain_components),
        create_conf_field!(ldap_org_unit),
        create_conf_field!(ldap_bind_org_unit),
        create_conf_field!(ldap_bind_prefix),
        create_conf_field!(ldap_readonly_user_prefix),
        create_conf_field!(ldap_readonly_bind),
        create_conf_field!(ldap_server),
        create_conf_field!(ldap_readonly_user),
        create_conf_field!(ldap_readonly_pw),
        create_conf_field!(include_ldap),
        create_conf_field!(include_slurm),
        create_conf_field!(include_dir_mgmt),
        create_conf_field!(use_homedir_helper),
        create_conf_field!(run_slurm_remote),
        create_conf_field!(ssh_agent),
        create_conf_field!(ssh_port),
        create_conf_field!(compute_node_root_dir),
        create_conf_field!(filesystem),
        create_conf_field!(home_filesystem),
        create_conf_field!(nfs_filesystem),
        create_conf_field!(quota_softlimit),
        create_conf_field!(quota_hardlimit),
        create_conf_field!(quota_nfs_softlimit),
        create_conf_field!(quota_nfs_hardlimit),
        create_conf_field!(quota_home_softlimit),
        create_conf_field!(quota_home_hardlimit),
        create_conf_field!(login_shell),
        create_conf_field!(student_gid),
        create_conf_field!(staff_gid),
        create_conf_field!(faculty_gid),
        create_conf_field!(sacctmgr_path),
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
    },
    Single {
        val: &'a mut String,
        label: LabelTyp,
    },
    List {
        val: &'a mut Vec<String>,
        label: LabelTyp,
    },
    Checkbox {
        val: &'a mut bool,
        label: LabelTyp,
    },
    Number {
        val: &'a mut u32,
        label: LabelTyp,
    },
    NegNumber {
        val: &'a mut i32,
        label: LabelTyp,
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
        impl<'a> From<(&'a mut $type, LabelTyp)> for ConfiField<'a> {
            fn from((val, label): (&'a mut $type, LabelTyp)) -> Self {
                Self::$variant { val, label }
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
