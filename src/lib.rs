pub mod cli;
pub mod config;
pub mod dir;
pub mod util;

mod ldap;
mod slurm;
mod ssh;
use cli::{Commands, Modifiable, OnWhichSystem, UserToAdd};

use config::MgmtConfig;
use log::{debug, error, info, warn};
use prelude::*;
use std::{collections::HashSet, fmt, fs, str::FromStr};

pub mod app_error;

pub mod prelude {
    pub use crate::app_error;
    pub use anyhow::{anyhow, bail, Context};
    pub type AnyError = anyhow::Error;
    pub type AppError = AnyError;
    pub type AppResult<T = ()> = Result<T, AnyError>;
}

use crate::{
    dir::add_user_directories,
    ldap::{add_ldap_user, delete_ldap_user, modify_ldap_user},
    ssh::SshCredential,
};
extern crate confy;

// TODO: git rid of unwraps. Replace them with expects or better with result if possible.
// TODO: implement struct or function to remove redundancy for opening up tcp/ssh connection
// A code block as example in the file slurm under function add_slurm_user is repeated quite often

#[derive(Clone, PartialEq)]
pub enum Group {
    Staff,
    Student,
    Faculty,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Group::Staff => write!(f, "staff"),
            Group::Student => write!(f, "student"),
            Group::Faculty => write!(f, "faculty"),
        }
    }
}

impl FromStr for Group {
    type Err = ();
    fn from_str(input: &str) -> Result<Group, Self::Err> {
        match input {
            "Staff" | "staff" => Ok(Group::Staff),
            "Student" | "student" => Ok(Group::Student),
            "Faculty" | "faculty" => Ok(Group::Student),
            _ => Err(()),
        }
    }
}

/// Representation of a user entity.
/// It contains all information necessary to add/modify/delete the user.
/// TODO: Add proper encapsulation via getter and setters
pub struct Entity {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub gid: i32,
    pub group: Group,
    pub default_qos: String,
    /// TODO: Add validation if a present publickey is in valid format, OpenSsh
    pub publickey: String,
    pub qos: Vec<String>,
}

pub struct User {}

impl Entity {
    fn new(to_add: &UserToAdd, config: &MgmtConfig) -> Self {
        let staff_qos = &config.staff_qos;
        let student_qos = &config.student_qos;
        let valid_qos = &config.valid_qos;

        let staff_default_qos = &config.staff_default_qos;
        let student_default_qos = &config.student_default_qos;

        let mut default_qos = &to_add.default_qos;
        let mut group_str = to_add.group.to_lowercase();
        let mut qos: &Vec<String> = &to_add.qos;

        if !is_valid_group(&group_str, &config.valid_slurm_groups) {
            warn!(
                "Invalid group specified: {}. Group field will be student",
                group_str
            );
            group_str = "student".to_string();
        }

        let group = Group::from_str(&group_str).unwrap();
        let gid = Entity::map_groupname_to_gid(group.clone(), config).unwrap();

        if qos.is_empty() || !is_valid_qos(qos, valid_qos) {
            info!("Specified QOS are either invalid or empty. Using defaults in conf.toml.");
            match group {
                Group::Staff => qos = staff_qos,
                Group::Student => qos = student_qos,
                Group::Faculty => qos = staff_qos,
            }
        }

        if default_qos.is_empty() || !is_valid_qos(&[default_qos.clone()], valid_qos) {
            warn!(
                "Specified default QOS is invalid or empty. Using the value specified in config."
            );
            match group {
                Group::Staff => default_qos = staff_default_qos,
                Group::Student => default_qos = student_default_qos,
                Group::Faculty => default_qos = staff_default_qos,
            }
        }

        let mut pubkey_from_file = "".to_string();
        if !to_add.publickey.is_empty() {
            debug!("Received PublicKey file path {}", to_add.publickey);
            let pubkey_result = fs::read_to_string(&to_add.publickey);
            match pubkey_result {
                Ok(result) => pubkey_from_file = result,
                Err(e) => error!("Unable to read PublicKey from file! {}", e),
            }
        }

        Entity {
            username: to_add.user.to_lowercase(),
            firstname: to_add.firstname.to_string(),
            lastname: to_add.lastname.to_string(),
            group,
            default_qos: default_qos.to_string(),
            publickey: pubkey_from_file.trim().to_string(),
            qos: qos.to_vec(),
            gid,
            mail: to_add.mail.to_string(),
        }
    }

    /// Convert Group enum into integer gid
    fn map_groupname_to_gid(group: Group, config: &MgmtConfig) -> Result<i32, ()> {
        match group {
            Group::Staff => Ok(config.staff_gid),
            Group::Student => Ok(config.student_gid),
            Group::Faculty => Ok(config.faculty_gid),
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            username: "".to_string(),
            firstname: "".to_string(),
            lastname: "".to_string(),
            mail: "".to_string(),
            gid: -1,
            group: Group::Student,
            default_qos: "basic".to_string(),
            publickey: "".to_string(),
            qos: vec![],
        }
    }
}

/// Main function that handles user management
pub fn run_mgmt(args: cli::GeneralArgs, config: MgmtConfig) -> AppResult {
    match &args.command {
        Commands::Add {
            to_add,
            on_which_sys,
        } => add_user(
            to_add,
            &OnWhichSystem::from_config_for_all(&config, on_which_sys),
            &config,
        )?,
        Commands::Modify { data, on_which_sys } => modify_user(
            data.clone(),
            &OnWhichSystem::from_config_for_slurm_ldap(&config, on_which_sys),
            &config,
        )?,
        Commands::Delete { user, on_which_sys } => delete_user(
            user,
            &OnWhichSystem::from_config_for_slurm_ldap(&config, on_which_sys),
            &config,
        )?,
        Commands::List {
            on_which_sys,
            simple_output_for_ldap,
        } => list_users(
            &config,
            &OnWhichSystem::from_config_for_slurm_ldap(&config, on_which_sys),
            simple_output_for_ldap.unwrap_or(false),
        )?,
    };

    Ok(())
}

/// Check if sequence `qos` contains only valid QOS values.
/// A value in `qos` is valid if `valid_qos` contains it.
/// Valid QOS are defined in conf.toml
///
/// # Returns
///
/// - true if all values in `qos` are valid
/// - false if at least one element in `qos` is invalid
/// - true if `qos` and `valid_qos` are empty
/// - true if `qos` is empty
/// - false if `valid_qos` is empty
///
fn is_valid_qos<S>(qos: &[S], valid_qos: &[S]) -> bool
where
    S: AsRef<str> + PartialEq,
{
    for q in qos {
        if !valid_qos.contains(q) {
            return false;
        }
    }
    true
}
/// Removes all invalid elements of `qos`. An element is valid if `valid_qos` contains it.
/// Filters out duplicates too.
/// Returns an empty vector if `qos` or `valid_qos` is empty.
fn filter_invalid_qos<S>(qos: &[S], valid_qos: &[S]) -> Vec<S>
where
    S: AsRef<str> + PartialEq + Clone + std::fmt::Display,
{
    let mut filtered_qos: Vec<S> = Vec::with_capacity(qos.len());
    // Just keep references to prevent another heap allocation.
    let mut found: HashSet<&str> = HashSet::with_capacity(qos.len());

    for to_inspect in qos {
        let as_str: &str = to_inspect.as_ref();
        if valid_qos.contains(to_inspect) {
            if found.insert(as_str) {
                filtered_qos.push(to_inspect.clone());
            } else {
                warn!(
                    "QOS {} has a duplicate and will not be added another time !",
                    to_inspect
                )
            }
        } else {
            let s: &str = to_inspect.as_ref();
            warn!("QOS {} is invalid and will be removed!", s)
        }
    }

    filtered_qos.into_iter().collect()
}

fn is_valid_group(group: &String, valid_groups: &[String]) -> bool {
    valid_groups.contains(group)
}

/// TODO: reduce argument count
fn add_user(to_add: &UserToAdd, on_which_sys: &OnWhichSystem, config: &MgmtConfig) -> AppResult {
    debug!("Start add_user");

    let sacctmgr_path = config.sacctmgr_path.clone();

    let entity = Entity::new(to_add, config);

    let ssh_credentials = SshCredential::new(config);

    if on_which_sys.ldap() {
        add_ldap_user(&entity, config);
    }

    if on_which_sys.slurm() {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::add_slurm_user(&entity, config, &ssh_credentials)?;
        } else {
            // Call sacctmgr binary directly via subprocess
            slurm::local::add_slurm_user(&entity, &sacctmgr_path)?;
        }
    }

    if on_which_sys.dirs() {
        add_user_directories(&entity, config, &ssh_credentials)?;
    } else {
        debug!("include_dir_mgmt in conf.toml is false (or not set). Not creating directories.");
    }

    debug!("Finished add_user");

    Ok(())
}

fn delete_user(user: &str, on_which_sys: &OnWhichSystem, config: &MgmtConfig) -> AppResult {
    debug!("Start delete_user");

    let credentials = SshCredential::new(config);

    if on_which_sys.ldap() {
        delete_ldap_user(user, config);
    }

    if on_which_sys.slurm() {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::delete_slurm_user(user, config, &credentials)?;
        } else {
            // Call sacctmgr binary directly via subprocess
            slurm::local::delete_slurm_user(user, &config.sacctmgr_path);
        }
    }

    debug!("Finished delete_user");
    Ok(())
}

/// TODO: reduce argument count
fn modify_user(
    mut data: Modifiable,
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
) -> AppResult {
    debug!("Start modify_user for {}", data.username);

    if let Some(ref s) = data.default_qos {
        if !is_valid_qos(&[s.clone()], &config.valid_qos) {
            warn!("Specified default QOS {s} is invalid and will be removed!");
            data.default_qos = None;
        }
    }

    let mut pubkey_from_file = None;

    if let Some(ref pubk) = data.publickey {
        debug!("Matched pubkey file {}", pubk);
        if !pubk.is_empty() {
            debug!("Reading publickey from {}", pubk);
            let pubkey_result = fs::read_to_string(pubk);
            match pubkey_result {
                Ok(result) => pubkey_from_file = Some(result),
                Err(e) => error!("Unable to read publickey from file! {}", e),
            }
        }
    }

    {
        let filtered_qos = filter_invalid_qos(&data.qos, &config.valid_qos);
        data.qos = filtered_qos;

        debug!("Received pubkey as modifiable {:?}", pubkey_from_file);
        data.publickey = pubkey_from_file;
    }

    let sacctmgr_path = config.sacctmgr_path.clone();

    let credential = SshCredential::new(config);
    if on_which_sys.ldap() {
        modify_ldap_user(&data, config)?;
    }
    if on_which_sys.slurm() {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::modify_slurm_user(&data, config, &credential)?;
        } else {
            // Call sacctmgr binary directly via subprocess
            slurm::local::modify_slurm_user(&data, &sacctmgr_path);
        }
    }

    debug!("Finished modify_user");
    Ok(())
}

fn list_users(
    config: &MgmtConfig,
    on_which_sys: &OnWhichSystem,
    simple_output_ldap: bool,
) -> AppResult {
    let credentials = SshCredential::new(config);

    if on_which_sys.ldap() {
        ldap::list_ldap_users(config, simple_output_ldap)?;
    }

    if on_which_sys.slurm() {
        if config.run_slurm_remote {
            slurm::remote::list_users(config, &credentials)?;
        } else {
            slurm::local::list_users(&config.sacctmgr_path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn should_determine_if_valid_qos() {
        assert_case(&["student"], &["student", "staff", "faculty"], true);
        assert_case(&["worker"], &["student", "staff", "faculty"], false);
        assert_case(
            &["student", "worker"],
            &["student", "staff", "faculty"],
            false,
        );
        assert_case(&["student"], &[], false);
        assert_case(&[], &["student"], true);
        assert_case(&[], &[], true);

        fn assert_case(qos: &[&str], valid_qos: &[&str], expected: bool) {
            let actual = is_valid_qos(qos, valid_qos);
            assert_eq!(
                expected, actual,
                "expected: {} with qos: {:?} and valid_qos: {:?}",
                expected, qos, valid_qos
            );
        }
    }

    #[test]
    fn should_filter_out_invalid_qos() {
        assert_case(&["student", "worker"], &["student"], vec!["student"]);
        // With duplicates
        assert_case(
            &["student", "student", "worker"],
            &["student"],
            vec!["student"],
        );
        // left == right
        assert_case(
            &["student", "worker"],
            &["student", "worker"],
            vec!["student", "worker"],
        );
        // contains only valid elements
        assert_case(
            &["student", "worker"],
            &["student", "worker", "staff"],
            vec!["student", "worker"],
        );
        // No valid element
        assert_case(&["npc", "worker"], &["student"], vec![]);
        // Edge cases for empty lists
        assert_case(&["student"], &[], vec![]);
        assert_case(&[], &["student"], vec![]);

        fn assert_case(qos: &[&str], filter: &[&str], expected: Vec<&str>) {
            let actual = filter_invalid_qos(qos, filter);
            assert_eq!(expected, actual, "qos: {:?} and filter: {:?}", qos, filter);
        }
    }
}
