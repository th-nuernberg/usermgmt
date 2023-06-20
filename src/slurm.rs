use anyhow::{anyhow, Context};
use log::info;

use crate::{
    prelude::AppResult,
    ssh::{self, SshConnection},
};

pub mod local {
    use std::process::Command;

    use log::{debug, info, warn};

    use crate::prelude::*;
    use crate::{Entity, Modifiable};
    use anyhow::Context;

    /// Uses local sacctmgr binary to add user on the slurm db of the local machine.
    ///
    /// # Errors
    ///
    /// - If no local sacctmgr binary can be found
    /// - If the execution of the local sacctmgr binary returns with non zero error code.
    /// - if modifying the quality of service fails
    ///  
    pub fn add_slurm_user(entity: &Entity, sacctmgr_path: &str) -> AppResult {
        let account_spec = format!("Account={}", entity.group);
        let cmd = &[
            "add",
            "user",
            &entity.username,
            &account_spec,
            "--immediate",
        ];

        let output = Command::new(sacctmgr_path)
            .args(cmd)
            .output()
            .context("could not execute sacctmgr on local machine. Is sacctmgr in $Path ?")?;

        debug!(
            "add_slurm_user: {}\n Executed",
            String::from_utf8_lossy(&output.stdout)
        );

        if output.status.success() {
            info!("Added user {} to Slurm", entity.username);
            info!("Executed command for adding user: {}", cmd.join(" "))
        } else {
            _ = app_error::output_to_result(output).with_context(|| {
                format!(
                    "Failed to create Slurm user. Command '{}' did not exit with 0.",
                    cmd.join(" ")
                )
            })?;
        }

        debug!("Modifying user qos");
        // Note: The order of execution is important here!
        // Slurm expects the user to have QOS, before it can set the default QOS
        modify_qos(entity, sacctmgr_path, false)?;
        modify_qos(entity, sacctmgr_path, true)?;

        Ok(())
    }

    /// Calls sacctmgr binary directly via subprocess
    pub fn delete_slurm_user(user: &str, sacctmgr_path: &str) -> AppResult {
        let output = Command::new(sacctmgr_path)
            .arg("delete")
            .arg("user")
            .arg(user)
            .arg("--immediate")
            .output()
            .context(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            )?;

        debug!(
            "delete_slurm_user: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        if output.status.success() {
            info!("Deleted user {} from Slurm", user);
        } else {
            warn!("Slurm user deletion did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("sacctmgr stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.is_empty() {
                bail!("sacctmgr stderr: {}", err);
            }
        }

        Ok(())
    }

    pub fn modify_slurm_user(modifiable: &Modifiable, sacctmgr_path: &str) -> AppResult {
        debug!("Start modifying user qos");
        match &modifiable.default_qos {
            Some(m) => {
                let entity = Entity {
                    username: modifiable.username.clone(),
                    default_qos: m.to_string(),
                    ..Default::default()
                };
                info!(
                    "Modifying default QOS for user {} in Slurm to {}.",
                    modifiable.username, m
                );
                modify_qos(&entity, sacctmgr_path, true)?;
            }
            None => info!(
                "Did not modify default QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            ),
        }

        if !modifiable.qos.is_empty() {
            let entity = Entity {
                username: modifiable.username.clone(),
                qos: modifiable.qos.clone(),
                ..Default::default()
            };
            modify_qos(&entity, sacctmgr_path, false)?;
        } else {
            info!(
                "Did not modify QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            );
        }

        Ok(())
    }

    /// Lists all user in slurm database on the local machine
    pub fn list_users(sacctmgr_path: &str) -> AppResult {
        let output = Command::new(sacctmgr_path)
            .arg("show")
            .arg("assoc")
            .arg("format=User%30,Account,DefaultQOS,QOS%80")
            .output()
            .context(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            )?;
        // sacctmgr show assoc format=cluster,account,qos
        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    /// Modifies the quality of use in database on a local machine
    fn modify_qos(entity: &Entity, sacctmgr_path: &str, default_qos: bool) -> AppResult {
        let mut qos_str: String = "defaultQos=".to_owned();
        if default_qos {
            qos_str += &entity.default_qos;
        } else {
            let qos_joined = entity.qos.join(",");
            qos_str = format!("qos={}", qos_joined);
        }
        debug!("Attempting to modify user with QOS: {qos_str}");
        let output = Command::new(sacctmgr_path)
            .arg("modify")
            .arg("user")
            .arg(entity.username.clone())
            .arg("set")
            .arg(qos_str)
            .arg("--immediate")
            .output()
            .context(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            )?;

        debug!("modify_qos: {}", String::from_utf8_lossy(&output.stdout));

        if output.status.success() {
            debug!("Modified QOS for user {} in Slurm", entity.username);
        } else {
            warn!("Slurm QOS modification did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("sacctmgr stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.is_empty() {
                bail!("sacctmgr stderr: {}", err);
            }
        }

        Ok(())
    }
}

pub mod remote {

    use log::{debug, info};

    use crate::config::MgmtConfig;
    use crate::prelude::AppResult;
    use crate::ssh::{SshConnection, SshCredential};
    use crate::{Entity, Modifiable};

    /// Creates a user in slurm database on a remote machine over ssh
    pub fn add_slurm_user(
        entity: &Entity,
        config: &MgmtConfig,
        credentials: &SshCredential,
    ) -> AppResult {
        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let session = SshConnection::from_head_node(config, credentials);

        let cmd = format!(
            "{} add user {} Account={} --immediate",
            config.sacctmgr_path, entity.username, entity.group
        );
        super::run_remote_report_slurm_cmd(
            &session,
            &cmd,
            || {
                format!(
                    "Successfully created Slurm user {}:{}.",
                    entity.username, entity.group
                )
            },
            || {
                format!(
                    "Failed to create Slurm user. Command '{}' did not exit with 0.",
                    cmd
                )
            },
        )?;

        debug!("Modifying user qos");
        // Note: The order of execution is important here!
        // Slurm expects the user to have QOS, before it can set the default QOS
        modify_qos(entity, config, &session, false)?;
        modify_qos(entity, config, &session, true)?;

        Ok(())
    }

    /// Deletes a user in a slurm database  via SSH session on a remote machine
    pub fn delete_slurm_user(
        user: &str,
        config: &MgmtConfig,
        credentials: &SshCredential,
    ) -> AppResult {
        // Connect to the SSH server and authenticate
        let sess = SshConnection::from_head_node(config, credentials);

        let cmd = format!("{} delete user {} --immediate", config.sacctmgr_path, user);
        super::run_remote_report_slurm_cmd(
            &sess,
            &cmd,
            || format!("Successfully deleted Slurm user {}.", user),
            || {
                format!(
                    "Failed to delete Slurm user. Command '{}' did not exit with 0.",
                    cmd
                )
            },
        )?;

        Ok(())
    }

    /// Modifies a user in a slurm database via SSH session on a remote machine
    /// It currently only modifies the quality of services of a user !
    pub fn modify_slurm_user(
        modifiable: &Modifiable,
        config: &MgmtConfig,
        credentials: &SshCredential,
    ) -> AppResult {
        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let sess = SshConnection::from_head_node(config, credentials);

        debug!("Start modifying user default qos");
        match &modifiable.default_qos {
            Some(m) => {
                let entity = Entity {
                    username: modifiable.username.clone(),
                    default_qos: m.to_string(),
                    ..Default::default()
                };
                debug!("New defaultQOS will be {}", entity.default_qos);
                modify_qos(&entity, config, &sess, true)?;
            }
            None => info!(
                "Did not modify default QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            ),
        }

        debug!("Start modifying user qos");
        if !modifiable.qos.is_empty() {
            let entity = Entity {
                username: modifiable.username.clone(),
                qos: modifiable.qos.clone(),
                ..Default::default()
            };
            debug!("New QOS will be {:?}", entity.qos);
            modify_qos(&entity, config, &sess, false)?;
        } else {
            info!(
                "Did not modify QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            );
        }

        Ok(())
    }

    /// Lists all users in slurm database on a remote machine
    pub fn list_users(config: &MgmtConfig, credentials: &SshCredential) -> AppResult {
        let cmd = "sacctmgr show assoc format=User%30,Account,DefaultQOS,QOS%80";

        let sess = SshConnection::from_head_node(config, credentials);

        let (output, _) = sess.exec(cmd)?;
        println!("{}", output);

        Ok(())
    }

    fn modify_qos(
        entity: &Entity,
        config: &MgmtConfig,
        sess: &SshConnection,
        default_qos: bool,
    ) -> AppResult {
        let mut qos_str: String = "defaultQos=".to_owned();
        if default_qos {
            qos_str += &entity.default_qos;
        } else {
            let qos_joined = entity.qos.join(",");
            qos_str = format!("qos={}", qos_joined);
        }
        debug!("Attempting to modify user QOS with {qos_str}");

        let cmd = format!(
            "{} modify user {} set {} --immediate",
            config.sacctmgr_path, entity.username, qos_str
        );

        super::run_remote_report_slurm_cmd(
            sess,
            &cmd,
            || {
                format!(
                    "Successfully modified QOS of user {} in Slurm.",
                    entity.username
                )
            },
            || {
                format!(
                    "Failed to modify Slurm user! Command '{}' did not exit with 0.",
                    cmd
                )
            },
        )?;

        Ok(())
    }
}

fn run_remote_report_slurm_cmd(
    session: &SshConnection,
    cmd: &str,
    on_success: impl Fn() -> String,
    on_error: impl Fn() -> String,
) -> AppResult {
    let exit_code = ssh::run_remote_command(session, cmd).with_context(&on_error)?;
    if exit_code == 0 {
        info!("{}", on_success());
        Ok(())
    } else {
        Err(anyhow!(on_error()))
    }
}
