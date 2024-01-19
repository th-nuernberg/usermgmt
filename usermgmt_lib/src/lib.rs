pub use entity::Entity;
use ldap::LdapCredential;
pub use new_entity::NewEntity;

pub mod app_error;
pub mod changes_to_user;
pub mod cli;
pub mod config;
pub mod constants;
pub mod dir;
pub mod logging;
pub mod util;

pub mod entity;
pub mod ldap;
pub mod new_entity;
pub mod slurm;
pub mod ssh;

pub use changes_to_user::ChangesToUser;
use ssh::SshCredentials;

use cli::{OnWhichSystem, UserToAdd};

use config::MgmtConfig;
use log::{debug, warn};
use prelude::*;
use std::{collections::HashSet, fmt, str::FromStr};

pub mod prelude {
    pub use crate::app_error;
    pub use crate::constants;
    pub use anyhow::{anyhow, bail, Context};
    pub type AnyError = anyhow::Error;
    pub type AppError = AnyError;
    pub type AppResult<T = ()> = Result<T, AnyError>;
}
pub mod app_panic_hook;

use crate::{
    dir::add_user_directories,
    ldap::{
        add_ldap_user, delete_ldap_user, modify_ldap_user, text_list_output, LDAPConfig,
        LdapSession,
    },
    slurm::{add_slurm_user, delete_slurm_user},
    ssh::SshConnection,
};
extern crate confy;

// TODO: git rid of unwraps. Replace them with expects or better with result if possible.
// TODO: implement struct or function to remove redundancy for opening up tcp/ssh connection
// A code block as example in the file slurm under function add_slurm_user is repeated quite often

#[derive(Clone, PartialEq, Copy, Debug, Eq)]
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

impl Default for Group {
    fn default() -> Self {
        Self::Student
    }
}

impl FromStr for Group {
    type Err = AppError;
    fn from_str(input: &str) -> Result<Group, Self::Err> {
        match input {
            "Staff" | "staff" => Ok(Group::Staff),
            "Student" | "student" => Ok(Group::Student),
            "Faculty" | "faculty" => Ok(Group::Student),
            _ => Err(anyhow!("given group name ({}) is not valid", input)),
        }
    }
}

fn perform_action_on_context<T, C>(
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
    ldap_credentials: T,
    ssh_credentials: &C,
    on_ldap_action: impl FnOnce(&mut LdapSession<T>) -> AppResult,
    on_slurm_action: impl FnOnce(&SshConnection<C>) -> AppResult,
    on_dir_action: impl FnOnce(&SshConnection<C>) -> AppResult,
) -> AppResult
where
    T: LdapCredential,
    C: SshCredentials,
{
    let ssh_session = SshConnection::from_head_node(config, ssh_credentials.clone());
    let mut ldap_session = LdapSession::new(config, ldap_credentials)?;

    if on_which_sys.slurm() {
        ssh_session.establish_connection()?;
    }

    if on_which_sys.ldap() {
        ldap_session.establish_connection()?;
        on_ldap_action(&mut ldap_session)?;
    }

    if on_which_sys.slurm() {
        on_slurm_action(&ssh_session)?;
    }

    if on_which_sys.dirs() {
        on_dir_action(&ssh_session)?;
    }

    Ok(())
}

fn perform_action_context_no_dirs<T, C>(
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
    ldap_credentials: T,
    ssh_credentials: &C,
    readonly: bool,
    on_ldap_action: impl FnOnce(&mut LdapSession<T>) -> AppResult,
    on_slurm_action: impl FnOnce(&SshConnection<C>) -> AppResult,
) -> AppResult
where
    T: LdapCredential,
    C: SshCredentials,
{
    let ssh_session = SshConnection::from_head_node(config, ssh_credentials.clone());
    let mut ldap_session = if readonly {
        LdapSession::from_ldap_readonly_config(config, ldap_credentials)?
    } else {
        LdapSession::new(config, ldap_credentials)?
    };

    if on_which_sys.slurm() {
        ssh_session.establish_connection()?;
    }

    if on_which_sys.ldap() {
        ldap_session.establish_connection()?;
        on_ldap_action(&mut ldap_session)?;
    }

    if on_which_sys.slurm() {
        on_slurm_action(&ssh_session)?;
    }

    Ok(())
}

/// Removes all invalid elements of `qos`. An element is valid if `valid_qos` contains it.
/// Filters out duplicates too.
/// Returns an empty vector if `qos` or `valid_qos` is empty.
pub fn filter_invalid_qos<S>(qos: &[S], valid_qos: &[S]) -> Vec<S>
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

pub fn add_user<T, C>(
    to_add: UserToAdd,
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
    ldap_credentials: T,
    ssh_credentials: C,
) -> AppResult
where
    T: LdapCredential + Clone,
    C: SshCredentials + Clone,
{
    debug!("Start add_user");

    let entity = NewEntity::new_user_addition_conf(to_add, config)?;

    perform_action_on_context(
        on_which_sys,
        config,
        ldap_credentials.clone(),
        &ssh_credentials,
        |session| add_ldap_user(&entity, config, session),
        |ssh_con| add_slurm_user(&entity, config, ssh_con),
        |_| add_user_directories(&entity, config, &ssh_credentials),
    )?;

    debug!("Finished add_user");

    Ok(())
}

pub fn delete_user<T, C>(
    user: &str,
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
    ldap_credentials: T,
    credentials: C,
) -> AppResult
where
    T: LdapCredential,
    C: SshCredentials,
{
    debug!("Start delete_user");

    perform_action_context_no_dirs(
        on_which_sys,
        config,
        ldap_credentials,
        &credentials,
        false,
        |ldap_session| delete_ldap_user(user, ldap_session),
        |ssh_connection| delete_slurm_user(user, config, ssh_connection),
    )?;

    debug!("Finished delete_user");
    Ok(())
}

pub fn modify_user<T, C>(
    data: ChangesToUser,
    on_which_sys: &OnWhichSystem,
    config: &MgmtConfig,
    ldap_credentials: T,
    credential: C,
) -> AppResult
where
    C: SshCredentials,
    T: LdapCredential,
{
    debug!("Start modify_user for {}", data.username);

    let modifiable = ChangesToUser::try_new(data.clone())?;
    perform_action_context_no_dirs(
        on_which_sys,
        config,
        ldap_credentials,
        &credential,
        false,
        |ldap_session| modify_ldap_user(&modifiable, ldap_session),
        |ssh_connection| slurm::modify_slurm_user(&modifiable, config, ssh_connection),
    )?;

    debug!("Finished modify_user");
    Ok(())
}

pub fn list_users<T, C>(
    config: &MgmtConfig,
    on_which_sys: &OnWhichSystem,
    simple_output_ldap: bool,
    ldap_credentials: T,
    credentials: C,
) -> AppResult
where
    T: LdapCredential,
    C: SshCredentials,
{
    perform_action_context_no_dirs(
        on_which_sys,
        config,
        ldap_credentials.clone(),
        &credentials,
        true,
        |_ldap_session| {
            let ldap_config = LDAPConfig::new_readonly(config, ldap_credentials)?;
            let search_result_data = ldap::list_ldap_users(ldap_config)?;

            let output = if simple_output_ldap {
                text_list_output::ldap_simple_output(&search_result_data)
            } else {
                text_list_output::ldap_search_to_pretty_table(&search_result_data)
            };
            println!("{}", &output);
            Ok(())
        },
        |ssh_connnection| {
            let output = slurm::list_users(config, ssh_connnection, false)?;
            println!("{}", output);
            Ok(())
        },
    )?;

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
            let actual = util::is_valid_qos(qos, valid_qos);
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
