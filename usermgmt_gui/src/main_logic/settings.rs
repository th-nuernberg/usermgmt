use crate::prelude::*;
use eframe::egui::Color32;
use serde::Deserialize;

#[cfg(debug_assertions)]
mod development;
#[cfg(debug_assertions)]
pub use development::DebugSettingWatcher;
use getset::{CopyGetters, Getters};

type ReadonlyText = String;

#[derive(Debug, Deserialize, Default, Getters, CopyGetters)]
/// Values which influence the look of the GUI.
/// These values are updated if the content of the Settings.toml changes
/// during Development.
pub struct Settings {
    #[getset(get_copy = "pub")]
    pub title_font_size: f32,
    #[getset(get_copy = "pub")]
    pub max_height_listing_table: f32,
    #[getset(get_copy = "pub")]
    pub header_table_height: f32,
    #[getset(get = "pub")]
    pub ldap_multi_field_sep: ReadonlyText,
    #[getset(get_copy = "pub")]
    pub box_label_font_size: f32,
    #[getset(get = "pub")]
    colors: Colors,
    #[getset(get = "pub")]
    texts: Texts,
}

#[derive(Debug, Deserialize, Default, Getters, CopyGetters)]
/// This fields are only applied at the start of the application
pub struct Init {
    #[getset(get = "pub")]
    window_title: ReadonlyText,
    #[getset(get_copy = "pub")]
    window_start_height: f32,
    #[getset(get_copy = "pub")]
    window_start_width: f32,
}

#[derive(Debug, Deserialize, Default, Getters)]
#[getset(get = "pub")]
/// Contains text which is shown in the GUI.
/// For example labels for entry fields or headings.
pub struct Texts {
    conf_load_init_msg: ReadonlyText,
    conf_load_success_msg: ReadonlyText,
    conf_load_err_msg: ReadonlyText,
    conf_load_loading_msg: ReadonlyText,
    conf_save_init_msg: ReadonlyText,
    conf_save_success_msg: ReadonlyText,
    conf_save_err_msg: ReadonlyText,
    conf_save_loading_msg: ReadonlyText,
    conf_load_group: ReadonlyText,
    conf_save_group: ReadonlyText,
    ssh_cred: ReadonlyText,
    ldap_cred: ReadonlyText,
    dir_conf_path: ReadonlyText,
    general_status: ReadonlyText,
    required: ReadonlyText,
    optional: ReadonlyText,
    readonly_ldap_cred: ReadonlyText,
    status_list_slurm: ReadonlyText,
    status_list_ldap: ReadonlyText,
    username: ReadonlyText,
    password: ReadonlyText,
    firstname: ReadonlyText,
    lastname: ReadonlyText,
    mail: ReadonlyText,
    qos: ReadonlyText,
    default_qos: ReadonlyText,
    public_key: ReadonlyText,
    group: ReadonlyText,
    btn_action_conf_load: ReadonlyText,
    btn_action_conf_save: ReadonlyText,
    btn_action_conf_default: ReadonlyText,
    btn_action_add: ReadonlyText,
    btn_action_remove: ReadonlyText,
    btn_action_modify: ReadonlyText,
    btn_list_ldap_users: ReadonlyText,
    btn_list_slurm_users: ReadonlyText,
    btn_list_remove: ReadonlyText,
    btn_new_item: ReadonlyText,

    listing_slurm_init: ReadonlyText,
    listing_slurm_loading: ReadonlyText,
    listing_slurm_success: ReadonlyText,
    listing_slurm_failure: ReadonlyText,
    listing_ldap_init: ReadonlyText,
    listing_ldap_loading: ReadonlyText,
    listing_ldap_success: ReadonlyText,
    listing_ldap_failure: ReadonlyText,
    modify_init: ReadonlyText,
    modify_loading: ReadonlyText,
    modify_success: ReadonlyText,
    modify_failure: ReadonlyText,
    mode_main_title: ReadonlyText,
    mode_ldap: ReadonlyText,
    mode_slurm: ReadonlyText,
    mode_directory: ReadonlyText,
    failed_parsing_slurm: ReadonlyText,
    ldap_cred_missing: ReadonlyText,
    ssh_cred_missing: ReadonlyText,
}

#[derive(Debug, Deserialize, Default, CopyGetters)]
#[getset(get_copy = "pub")]
/// Colors of certain elements in the GUI.
pub struct Colors {
    err_msg: Color32,
    init_msg: Color32,
    success_msg: Color32,
    loading_msg: Color32,
}
