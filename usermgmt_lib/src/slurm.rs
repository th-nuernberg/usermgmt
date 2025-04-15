use std::process::Command;

use anyhow::{anyhow, Context};
use log::{debug, info};

mod command_builder;
mod listed_user;
use crate::{config::MgmtConfig, prelude::AppResult, ssh};

use self::command_builder::CommandBuilder;

use crate::ssh::{SshConnection, SshCredentials};
use crate::{ChangesToUser, NewEntity};

pub use listed_user::ListedUser;

/// Creates a user in a slurm database on a remote machine over ssh
pub fn add_slurm_user<C>(
    entity: &NewEntity,
    config: &MgmtConfig,
    session: &SshConnection<C>,
) -> AppResult
where
    C: SshCredentials,
{
    let action = CommandBuilder::new_add(
        entity.username.to_string(),
        entity.group.id(),
        entity.default_qos.to_string(),
        entity.qos.clone().into(),
    );

    run_slurm_action(action, config, session).with_context(|| {
        format!(
            "Failed to add user {} with account {} to Slurm",
            entity.username, entity.group
        )
    })?;

    info!(
        "Added user {} with account {}, QoS {:?} and default QoS {} to Slurm",
        entity.username, entity.group, entity.qos, entity.default_qos
    );

    Ok(())
}

/// Deletes a user in a slurm database
///
/// # Errors
///
/// - See [`run_slurm_action`]
pub fn delete_slurm_user<C>(
    user: &str,
    config: &MgmtConfig,
    session: &SshConnection<C>,
) -> AppResult
where
    C: SshCredentials,
{
    let action = CommandBuilder::new_delete(user.to_string());
    run_slurm_action(action, config, session)
        .with_context(|| format!("Failed to delete user {} from Slurm", user))?;
    info!("Deleted user {} from Slurm", user);
    Ok(())
}

/// Modifies a user in a slurm database via SSH session on a remote machine
/// It currently only modifies the quality of services of a user !
///
/// # Errors
///
/// - See [`run_slurm_action`]
pub fn modify_slurm_user<C>(
    modifiable: &ChangesToUser,
    config: &MgmtConfig,
    session: &SshConnection<C>,
) -> AppResult
where
    C: SshCredentials,
{
    if let Some((qos, default_qos)) = modifiable.may_qos_and_default_qos() {
        let action = CommandBuilder::new_modify_qos_default_qows(
            modifiable.username.to_string(),
            default_qos,
            qos,
        );

        run_slurm_action(action, config, session)?;
    }
    Ok(())
}

/// Lists all users in slurm database
///
/// # Errors
///
/// See [`run_slurm_action`]
pub fn list_users<T>(
    config: &MgmtConfig,
    session: &SshConnection<T>,
    parseable: bool,
) -> AppResult<String>
where
    T: SshCredentials,
{
    let action = CommandBuilder::new_show(parseable);
    let output = run_slurm_action(action, config, session)?;

    Ok(output)
}

/// Runs the slurm command on a local machine or remotely somewhere else.
/// Whether run remotely or locally depends on the parameter `config`.
///
/// # Errors
///
/// - If running the command remotely fails. See [`run_remote_report_slurm_cmd`]
/// - If running the command on the local machine. See [`run_remote_report_slurm_cmd`]
pub fn run_slurm_action<C>(
    mut actions: CommandBuilder,
    config: &MgmtConfig,
    session: &SshConnection<C>,
) -> AppResult<String>
where
    C: SshCredentials,
{
    let mut output = String::new();
    actions = actions
        .immediate(true)
        .sacctmgr_path(config.sacctmgr_path.clone());
    if config.run_slurm_remote {
        for cmd in actions.remote_commands() {
            debug!("Running remote Slurm command: {}", &cmd);
            let next_output = run_remote_report_slurm_cmd(session, &cmd)?;
            output.push_str(&next_output);
        }
    } else {
        for cmd in actions.local_commands() {
            let next_output = run_local_and_report_slurm_cmd(cmd)?;
            output.push_str(&next_output);
        }
    }
    Ok(output)
}

/// # Errors
///
/// - If execution of the command fails. See [`SshConnection::exec`].
/// - If the exit code of executed command is an error code.
pub fn run_remote_report_slurm_cmd<C>(session: &SshConnection<C>, cmd: &str) -> AppResult<String>
where
    C: SshCredentials,
{
    let (exit_code, output) = ssh::run_remote_command(session, cmd)
        .with_context(|| format!("Error during remote Slurm command execution ({}).", cmd,))?;

    if exit_code == 0 {
        debug!("Successfully executed remote Slurm command:{}", cmd);
        Ok(output)
    } else {
        Err(anyhow!(
            "Error during remote Slurm command execution! Command '{}' returned exit code {}",
            cmd,
            exit_code
        ))
    }
}

/// # Errors
///
/// - If output of command could not be retrieved
pub fn run_local_and_report_slurm_cmd(mut command: Command) -> AppResult<String> {
    let output = command.output().context(
        "Unable to execute sacctmgr command. Is the path specified in your config correct?",
    )?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
