use log::debug;

use crate::{
    cli::{OnWhichSystem, UserToAdd},
    config::MgmtConfig,
    dir,
    ldap::{self, text_list_output, LDAPConfig, LdapCredential, LdapSession},
    slurm,
    ssh::{SshConnection, SshCredentials},
    AppResult, ChangesToUser, NewEntity,
};

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
    debug!("Start adding user");

    let entity = NewEntity::new_user_addition_conf(to_add, config)?;

    perform_action_on_context(
        on_which_sys,
        config,
        ldap_credentials.clone(),
        &ssh_credentials,
        |session| ldap::add_ldap_user(&entity, config, session),
        |ssh_con| slurm::add_slurm_user(&entity, config, ssh_con),
        |_| dir::add_user_directories(&entity, config, &ssh_credentials),
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
        |ldap_session| ldap::delete_ldap_user(user, ldap_session),
        |ssh_connection| slurm::delete_slurm_user(user, config, ssh_connection),
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
        |ldap_session| ldap::modify_ldap_user(&modifiable, ldap_session),
        |ssh_connection| slurm::modify_slurm_user(&modifiable, config, ssh_connection),
    )?;

    debug!("Finished modify_user");
    Ok(())
}

pub fn print_list_of_users<T, C>(
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
        |ssh_connection| {
            let output = slurm::list_users(config, ssh_connection, false)?;
            println!("{}", output);
            Ok(())
        },
    )?;

    Ok(())
}

/// Performs an action on all the three systems on the cluster.
///
/// - LDAP
/// - Slurm
/// - Directory management
///
/// # Errors
///
/// - If getting of credentials for LDAP fails. See [`LdapSession::new`]
/// - If establishing the ssh connection fails
/// - If one of three actions fails `on_ldap_action`, `on_slurm_action` or `on_dir_action`.
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

/// Same as [`perform_action_on_context`] except no directory management is performed.
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
