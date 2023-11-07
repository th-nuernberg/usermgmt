mod path_sources;
use std::path::{Path, PathBuf};

use anyhow::Context;
use log::info;
pub use path_sources::get_path_to_conf;
/// Definition of configuration options
use serde::{Deserialize, Serialize};

use crate::{config, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// TODO: consider implementing encapsulation with getters and setters
pub struct MgmtConfig {
    pub student_default_qos: String,
    pub staff_default_qos: String,
    pub student_qos: Vec<String>,
    pub staff_qos: Vec<String>,
    pub valid_qos: Vec<String>,
    pub valid_slurm_groups: Vec<String>,
    pub objectclass_common: Vec<String>,
    pub compute_nodes: Vec<String>,
    pub login_shell: String,
    pub student_gid: i32,
    pub staff_gid: i32,
    pub faculty_gid: i32,
    pub sacctmgr_path: String,
    pub ldap_domain_components: Option<String>,
    pub ldap_org_unit: Option<String>,
    pub ldap_server: String,
    pub ldap_readonly_user: Option<String>,
    pub ldap_readonly_pw: Option<String>,
    pub ldap_readonly_bind: Option<String>,
    pub ldap_readonly_user_prefix: Option<String>,
    pub ldap_bind_prefix: Option<String>,
    pub ldap_bind_org_unit: Option<String>,
    pub home_host: String,
    pub nfs_host: String,
    pub head_node: String,
    pub quota_softlimit: String,
    pub quota_hardlimit: String,
    pub quota_nfs_softlimit: String,
    pub quota_nfs_hardlimit: String,
    pub quota_home_softlimit: String,
    pub quota_home_hardlimit: String,
    pub nfs_root_dir: String,
    pub compute_node_root_dir: String,
    pub filesystem: String,
    pub home_filesystem: String,
    pub nfs_filesystem: String,
    pub default_ssh_user: String,
    pub include_slurm: bool,
    pub include_ldap: bool,
    pub include_dir_mgmt: bool,
    pub use_homedir_helper: bool,
    pub run_slurm_remote: bool,
    pub ssh_port: u32,
    pub ssh_agent: bool,
}
impl MgmtConfig {
    pub fn save(&self, path: &Path) -> AppResult<PathBuf> {
        let file_path = path
            .canonicalize()
            .with_context(|| format!("Could get absolute path at {:?}", path))?
            .join(constants::NAME_CONFIG_FILE);
        let text = toml::to_string_pretty(&self)?;
        std::fs::write(&file_path, text)
            .with_context(|| format!("Could not save to path: {:?}", file_path))?;
        Ok(path.to_path_buf())
    }
}

impl Default for MgmtConfig {
    fn default() -> Self {
        MgmtConfig {
            student_default_qos: "basic".to_string(),
            staff_default_qos: "advanced".to_string(),
            student_qos: vec!["interactive".to_string(), "basic".to_string()],
            staff_qos: vec!["interactive".to_string(), "advanced".to_string()],
            valid_qos: vec![
                "interactive".to_string(),
                "basic".to_string(),
                "advanced".to_string(),
            ],
            objectclass_common: vec![
                "inetOrgPerson".to_string(),
                "ldapPublicKey".to_string(),
                "organizationalPerson".to_string(),
                "person".to_string(),
                "posixAccount".to_string(),
                "shadowAccount".to_string(),
                "slurmRole".to_string(),
                "top".to_string(),
            ],
            valid_slurm_groups: vec!["staff".to_string(), "student".to_string()],
            login_shell: "/bin/bash".to_string(),
            student_gid: 1002,
            staff_gid: 1001,
            faculty_gid: 1000,
            sacctmgr_path: "/usr/local/bin/sacctmgr".to_string(),
            ldap_domain_components: None,
            ldap_org_unit: None,
            ldap_server: "ldap://localhost:389".to_string(),
            ldap_readonly_user: None,
            ldap_readonly_pw: None,
            ldap_readonly_user_prefix: None,
            ldap_readonly_bind: None,
            ldap_bind_prefix: None,
            ldap_bind_org_unit: None,
            home_host: "localhost".to_string(),
            nfs_host: "localhost".to_string(),
            quota_softlimit: "200G".to_string(),
            quota_hardlimit: "220G".to_string(),
            quota_nfs_softlimit: "200G".to_string(),
            quota_nfs_hardlimit: "220G".to_string(),
            quota_home_softlimit: "2G".to_string(),
            quota_home_hardlimit: "3G".to_string(),
            compute_nodes: vec!["localhost".to_string()],
            nfs_root_dir: "".to_string(),
            compute_node_root_dir: "".to_string(),
            include_slurm: true,
            include_ldap: true,
            include_dir_mgmt: false,
            use_homedir_helper: true,
            filesystem: "".to_string(),
            home_filesystem: "".to_string(),
            nfs_filesystem: "".to_string(),
            default_ssh_user: "root".to_string(),
            head_node: "".to_string(),
            run_slurm_remote: false,
            ssh_port: 22,
            ssh_agent: false,
        }
    }
}

/// Tries to load  config.toml for application.
///
/// # Error
///
/// - Can not ensure if folder exits where conf.toml file exits
/// - Can not read or create a configuration file
pub fn load_config(manual_path: Option<PathBuf>) -> AppResult<LoadedMgmtConfig> {
    let path = config::get_path_to_conf(manual_path)?;

    info!("Loding configuraion file from path at {:?}", path);
    // Load (or create if nonexistent) configuration file conf.toml
    let config = confy::load_path(&path)
        .with_context(|| format!("Error in loading or creating config file at {:?}", path))?;
    let path = path.parent().unwrap().to_path_buf();
    Ok(LoadedMgmtConfig { path, config })
}

#[derive(Debug, Default)]
pub struct LoadedMgmtConfig {
    pub path: PathBuf,
    pub config: MgmtConfig,
}

impl LoadedMgmtConfig {}

pub fn config_for_save() -> String {
    toml::to_string_pretty(&MgmtConfig::default())
        .expect("Could not turn default configuration into the toml format")
}
