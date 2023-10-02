use std::process::Command;

use anyhow::{anyhow, Context};
use log::{debug, info};

mod commmand_builder;
use crate::{config::MgmtConfig, prelude::AppResult, ssh};

use self::commmand_builder::CommandBuilder;

use crate::ssh::{SshConnection, SshCredential};
use crate::{Entity, NewEntity};

/// Creates a user in slurm database on a remote machine over ssh
pub fn add_slurm_user(
    entity: &NewEntity,
    config: &MgmtConfig,
    session: &SshConnection,
) -> AppResult {
    let action = CommandBuilder::new_add(
        entity.username.to_string(),
        entity.group.id(),
        entity.default_qos.to_string(),
        entity.qos.clone().into(),
    );

    run_slurm_action(action, config, session).with_context(|| {
        format!(
            "Failed to add user {} with account {}",
            entity.username, entity.group
        )
    })?;

    info!(
        "Added user {} with account {}, qos {:?} and default qos {}",
        entity.username, entity.group, entity.qos, entity.default_qos
    );

    Ok(())
}

/// Deletes a user in a slurm database  via SSH session on a remote machine
pub fn delete_slurm_user(user: &str, config: &MgmtConfig, session: &SshConnection) -> AppResult {
    let action = CommandBuilder::new_delete(user.to_string());
    run_slurm_action(action, config, session)
        .with_context(|| format!("Failed to delete user with name {}", user))?;
    info!("Deleted user with name {}", user);
    Ok(())
}

/// Modifies a user in a slurm database via SSH session on a remote machine
/// It currently only modifies the quality of services of a user !
pub fn modify_slurm_user(
    modifiable: &Entity,
    config: &MgmtConfig,
    session: &SshConnection,
) -> AppResult {
    if let Some(ref default_qos) = modifiable.default_qos {
        debug!("Slurm: Start modifying user default qos");
        let action = CommandBuilder::new_modify(
            modifiable.username.to_string(),
            commmand_builder::Modifier::DefaultQOS,
            vec![default_qos.to_string()],
        );
        run_slurm_action(action, config, session)?;
        info!("Slurm: Successfully modified the default qos of user");
    }
    if let Some(ref qos) = modifiable.qos {
        debug!("Slurm: Start modifying user qos");
        let action = CommandBuilder::new_modify(
            modifiable.username.to_string(),
            commmand_builder::Modifier::Qos,
            qos.clone().into(),
        );
        run_slurm_action(action, config, session)?;
        info!("Slurm: Successfully modified the qos of user");
    }

    Ok(())
}

/// Lists all users in slurm database on a remote machine
pub fn list_users(config: &MgmtConfig, credentials: &SshCredential) -> AppResult {
    let action = CommandBuilder::new_show();
    let session = SshConnection::from_head_node(config, credentials);
    let output = run_slurm_action(action, config, &session)?;

    println!("{}", output);

    Ok(())
}

fn run_slurm_action(
    mut actions: CommandBuilder,
    config: &MgmtConfig,
    session: &SshConnection,
) -> AppResult<String> {
    let mut output = String::new();
    actions = actions
        .immediate(true)
        .sacctmgr_path(config.sacctmgr_path.clone());
    if config.run_slurm_remote {
        for cmd in actions.remote_command() {
            debug!("Run remote slurm command ({})", &cmd);
            let next_output = run_remote_report_slurm_cmd(session, &cmd)?;
            output.push_str(&next_output);
        }
    } else {
        for cmd in actions.local_command() {
            let next_output = run_local_and_report_slurm_cmd(cmd)?;
            output.push_str(&next_output);
        }
    }
    Ok(output)
}

fn run_remote_report_slurm_cmd(session: &SshConnection, cmd: &str) -> AppResult<String> {
    let (exit_code, output) = ssh::run_remote_command(session, cmd)
        .with_context(|| format!("Error: For remote slurm command ({}).", cmd,))?;

    if exit_code == 0 {
        debug!("Success: For remote slurm command ({})", cmd);
        Ok(output)
    } else {
        Err(anyhow!(
            "Error: For remote slurm command ({}) with error code {}",
            cmd,
            exit_code
        ))
    }
}

fn run_local_and_report_slurm_cmd(mut command: Command) -> AppResult<String> {
    let output = command.output().context(
        "Unable to execute sacctmgr command. Is the path specified in your config correct?",
    )?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
