pub mod cli;
pub mod config;
pub mod dir;
mod ldap;
mod slurm;
mod ssh;
pub mod util;
use cli::cli::Commands;
use config::config::MgmtConfig;
use log::{debug, error, info, warn};
use prelude::*;
use std::{fmt, fs, str::FromStr};

mod app_error;

mod prelude {
    pub use anyhow::{anyhow, bail};
    pub type AppError = crate::app_error::AppError;
    pub type AppResult<T = ()> = Result<T, AppError>;
    pub type AnyError = anyhow::Error;
}

use crate::{
    dir::dir::add_user_directories,
    ldap::ldap::{add_ldap_user, delete_ldap_user, modify_ldap_user},
    ssh::SshCredential,
    // slurm::local::{add_slurm_user, delete_slurm_user, modify_slurm_user},
};
extern crate confy;

// TODO: git rif of unwraps. Replace them with expects or better with result if possible.
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

impl Entity {
    fn new(
        user: &String,
        group: &String,
        firstname: &String,
        lastname: &String,
        mail: &String,
        default_qos: &String,
        publickey: &String,
        qos: &Vec<String>,
        config: &MgmtConfig,
    ) -> Self {
        let staff_qos = &config.staff_qos;
        let student_qos = &config.student_qos;
        let valid_qos = &config.valid_qos;

        let staff_default_qos = &config.staff_default_qos;
        let student_default_qos = &config.student_default_qos;

        let mut default_qos = default_qos;
        let mut group_str = group.to_lowercase();
        let mut qos: &Vec<String> = qos;

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

        if default_qos.is_empty() || !is_valid_qos(&vec![default_qos.clone()], valid_qos) {
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
        if !publickey.is_empty() {
            debug!("Received PublicKey file path {}", publickey);
            let pubkey_result = fs::read_to_string(publickey);
            match pubkey_result {
                Ok(result) => pubkey_from_file = result,
                Err(e) => error!("Unable to read PublicKey from file! {}", e),
            }
        }

        Entity {
            username: user.to_lowercase(),
            firstname: firstname.to_string(),
            lastname: lastname.to_string(),
            group,
            default_qos: default_qos.to_string(),
            publickey: pubkey_from_file.trim().to_string(),
            qos: qos.to_vec(),
            gid,
            mail: mail.to_string(),
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

/// Defines options that can be modified
/// TODO: consider encapsulation with getters and setters.
pub struct Modifiable {
    pub username: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
    pub default_qos: Option<String>,
    pub publickey: Option<String>,
    pub qos: Vec<String>,
}

impl Modifiable {
    fn new(
        username: &String,
        firstname: &Option<String>,
        lastname: &Option<String>,
        mail: &Option<String>,
        default_qos: &Option<String>,
        publickey: &Option<String>,
        qos: &[String],
    ) -> Self {
        Modifiable {
            username: username.clone(),
            firstname: firstname.clone(),
            lastname: lastname.clone(),
            mail: mail.clone(),
            default_qos: default_qos.clone(),
            publickey: publickey.clone(),
            qos: qos.to_vec(),
        }
    }
}

/// Main function that handles user management
pub fn run_mgmt(args: cli::cli::Args, config: MgmtConfig) {
    let is_slurm_only = args.slurm_only;
    let is_ldap_only = args.ldap_only;
    let directories_only = args.dirs_only;
    let sacctmgr_path = config.sacctmgr_path.clone();

    match &args.command {
        Commands::Add {
            user,
            group,
            firstname,
            lastname,
            mail,
            default_qos,
            publickey,
            qos,
        } => add_user(
            user,
            group,
            firstname,
            lastname,
            mail,
            default_qos,
            publickey,
            qos,
            &is_slurm_only,
            &is_ldap_only,
            &directories_only,
            &config,
        ),
        Commands::Modify {
            user,
            firstname,
            lastname,
            mail,
            default_qos,
            publickey,
            qos,
        } => modify_user(
            user,
            firstname,
            lastname,
            mail,
            default_qos,
            publickey,
            qos,
            &is_slurm_only,
            &is_ldap_only,
            &config,
        ),
        Commands::Delete { user } => {
            delete_user(user, &is_slurm_only, &is_ldap_only, &sacctmgr_path, &config)
        }
        Commands::List {
            slurm_users,
            ldap_users,
        } => list_users(&config, slurm_users, ldap_users),
    }
}

/// Check if a Vector contains invalid QOS values
/// Valid QOS are defined in conf.toml
fn is_valid_qos(qos: &Vec<String>, valid_qos: &[String]) -> bool {
    for q in qos {
        if !valid_qos.contains(q) {
            return false;
        }
    }
    true
}

fn filter_invalid_qos(qos: &Vec<String>, valid_qos: &[String]) -> Vec<String> {
    let mut filtered_qos: Vec<String> = Vec::new();
    for q in qos {
        if is_valid_qos(&vec![q.clone()], valid_qos) {
            filtered_qos.push(q.clone());
        } else {
            warn!("QOS {q} is invalid and will be removed!")
        }
    }
    filtered_qos
}

fn is_valid_group(group: &String, valid_groups: &[String]) -> bool {
    valid_groups.contains(group)
}

/// TODO: reduce argument count
fn add_user(
    user: &String,
    group: &String,
    firstname: &String,
    lastname: &String,
    mail: &String,
    default_qos: &String,
    publickey: &String,
    qos: &Vec<String>,
    is_slurm_only: &bool,
    is_ldap_only: &bool,
    directories_only: &bool,
    config: &MgmtConfig,
) {
    debug!("Start add_user");

    let sacctmgr_path = config.sacctmgr_path.clone();

    let entity = Entity::new(
        user,
        group,
        firstname,
        lastname,
        mail,
        default_qos,
        publickey,
        qos,
        config,
    );

    let ssh_credentials = SshCredential::new(config);
    let wants_slurm = !is_ldap_only && !directories_only;
    if wants_slurm {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::add_slurm_user(&entity, config, &ssh_credentials);
        } else {
            // Call sacctmgr binary directly via subprocess

            let after = slurm::local::add_slurm_user(&entity, &sacctmgr_path);
            exit_if_app_error(&after);
        }
    }

    let wants_ldap = !is_slurm_only && !directories_only;
    if wants_ldap {
        add_ldap_user(&entity, config);
    }

    if config.include_dir_mgmt {
        let wants_user_directories = !is_slurm_only && !is_ldap_only;
        if wants_user_directories {
            add_user_directories(&entity, config, &ssh_credentials);
        }
    } else {
        debug!("include_dir_mgmt in conf.toml is false (or not set). Not creating directories.");
    }

    debug!("Finished add_user");
}

fn delete_user(
    user: &String,
    is_slurm_only: &bool,
    is_ldap_only: &bool,
    sacctmgr_path: &String,
    config: &MgmtConfig,
) {
    debug!("Start delete_user");

    let credentials = SshCredential::new(config);
    if !is_ldap_only {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::delete_slurm_user(user, config, &credentials);
        } else {
            // Call sacctmgr binary directly via subprocess
            slurm::local::delete_slurm_user(user, sacctmgr_path);
        }
    }

    if !is_slurm_only {
        delete_ldap_user(user, config);
    }
    debug!("Finished delete_user");
}

/// TODO: reduce argument count
fn modify_user(
    user: &String,
    firstname: &Option<String>,
    lastname: &Option<String>,
    mail: &Option<String>,
    mut default_qos: &Option<String>,
    publickey: &Option<String>,
    qos: &Vec<String>,
    is_slurm_only: &bool,
    is_ldap_only: &bool,
    config: &MgmtConfig,
) {
    debug!("Start modify_user for {}", user);

    if let Some(s) = default_qos {
        if !is_valid_qos(&vec![s.clone()], &config.valid_qos) {
            warn!("Specified default QOS {s} is invalid and will be removed!");
            default_qos = &None;
        }
    }

    let mut pubkey_from_file = None;

    if let Some(pubk) = publickey {
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

    let filtered_qos = filter_invalid_qos(qos, &config.valid_qos);
    debug!("Received pubkey as modifiable {:?}", pubkey_from_file);
    let modifiable = Modifiable::new(
        user,
        firstname,
        lastname,
        mail,
        default_qos,
        &pubkey_from_file,
        &filtered_qos,
    );

    let sacctmgr_path = config.sacctmgr_path.clone();

    let credential = SshCredential::new(config);
    if !is_ldap_only {
        if config.run_slurm_remote {
            // Execute sacctmgr commands via SSH session
            slurm::remote::modify_slurm_user(&modifiable, config, &credential);
        } else {
            // Call sacctmgr binary directly via subprocess
            slurm::local::modify_slurm_user(&modifiable, &sacctmgr_path);
        }
    }

    if !is_slurm_only {
        modify_ldap_user(&modifiable, config);
    }
    debug!("Finished modify_user");
}

fn list_users(config: &MgmtConfig, slurm: &bool, ldap: &bool) {
    let credentials = SshCredential::new(config);
    if *slurm {
        if config.run_slurm_remote {
            slurm::remote::list_users(config, &credentials);
        } else {
            slurm::local::list_users(&config.sacctmgr_path);
        }
    }

    if *ldap {
        ldap::ldap::list_ldap_users(config);
    }
}

/// If given parmater `maybe_error` is an error
/// then the application prints out error information to the user and
/// exits with error code of provided error.
fn exit_if_app_error<T>(maybe_error: &AppResult<T>) {
    if let Err(error) = maybe_error {
        error!("{:?}", error);

        std::process::exit(error.exit_code());
    }
}
