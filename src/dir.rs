/// Module for directory management
use log::{debug, error, info, warn};

use crate::config::MgmtConfig;
use crate::prelude::AppResult;
use crate::ssh::{self, SshCredential, SshSession};
use crate::{Entity, Group};

pub fn add_user_directories(
    entity: &Entity,
    config: &MgmtConfig,
    credentials: &SshCredential,
) -> AppResult {
    handle_compute_nodes(entity, config, credentials)?;

    handle_nfs(entity, config, credentials)?;

    handle_home(entity, config, credentials)?;

    Ok(())
}

/// TODO: Bubble up errors instead of just logging
/// Establish SSH connection to each compute node, make user directory and set quota
fn handle_compute_nodes(
    entity: &Entity,
    config: &MgmtConfig,
    credentials: &SshCredential,
) -> AppResult {
    debug!("Start handling directories on compute nodes");

    if config.compute_nodes.is_empty() {
        warn!("No compute nodes provided in config. Unable to create user directories.");
        return Ok(());
    }
    if config.compute_node_root_dir.is_empty() {
        warn!("No root directory on compute nodes provided in config. Unable to create user directories.");
        return Ok(());
    }

    if config.filesystem.is_empty() {
        warn!("No root directory on compute nodes provided in config. Unable to create user directories.");
        return Ok(());
    }

    let mut can_set_quota = true;
    if config.quota_softlimit.is_empty()
        || config.quota_hardlimit.is_empty()
        || config.filesystem.is_empty()
    {
        can_set_quota = false;
        warn!("Hard-/softlimit and/or filesystem for quotas isn't properly configured. Refusing to set user quotas based on these values. Please check your conf.toml");
    }

    let mut mkdir_exit_codes = Vec::new();
    let mut owner_exit_codes = Vec::new();
    let mut quota_exit_codes = Vec::new();
    for server in config.compute_nodes.iter() {
        info!("Connecting to a compute node");
        let sess = SshSession::new(server, config.ssh_port, credentials);
        // Create directory
        let directory = format!("{}/{}", config.compute_node_root_dir, entity.username);
        let dir_exit_code = make_directory(&sess, &directory)?;
        mkdir_exit_codes.push(dir_exit_code);

        if dir_exit_code == 0 {
            // Give ownership to user
            let owner_exit_code = change_ownership(
                &sess,
                &directory,
                &entity.username,
                &entity.group.to_string(),
            );
            owner_exit_codes.push(owner_exit_code);

            // Set user quota
            let mut quota_exit_code = 1;
            if can_set_quota {
                quota_exit_code = set_quota(
                    &sess,
                    &entity.username,
                    &config.quota_softlimit,
                    &config.quota_hardlimit,
                    &config.filesystem,
                )?;
            }
            quota_exit_codes.push(quota_exit_code);
        }
    }

    if mkdir_exit_codes.iter().all(|&x| x == 0) {
        if owner_exit_codes
            .iter()
            .all(|x| x.as_ref().is_ok_and(|code| *code == 0))
        {
            if quota_exit_codes.iter().all(|&x| x == 0) {
                info!("Successfully created directories on compute nodes.");
            } else if can_set_quota {
                error!("Not all compute nodes returned exit code 0 during quota setup!");
            }
        } else {
            error!("Not all compute nodes returned exit code 0 during ownership change!");
        }
    } else {
        error!("Not all compute nodes returned exit code 0 during directory creation!");
    }

    Ok(())
}

/// Establish SSH connection to NFS host, make user directory and set quota
/// TODO: Bubble up errors instead of just logging
fn handle_nfs(entity: &Entity, config: &MgmtConfig, credentials: &SshCredential) -> AppResult {
    debug!("Start handling NFS user directory");

    if config.nfs_host.is_empty() {
        warn!("No NFS host provided in config. Unable to create directory.");
        return Ok(());
    }
    if config.nfs_root_dir.is_empty() {
        warn!("No root directory provided in config. Unable to create directory.");
        return Ok(());
    }

    let mut can_set_quota = true;
    if config.quota_nfs_softlimit.is_empty()
        || config.quota_nfs_hardlimit.is_empty()
        || config.nfs_filesystem.is_empty()
    {
        can_set_quota = false;
        warn!("Hard-/softlimit and/or filesystem for quota isn't properly configured. Refusing to set user quota based on these values. Please check your conf.toml");
    }

    info!("Connecting to NFS host");
    let sess = SshSession::new(&config.nfs_host, config.ssh_port, credentials);

    // Create directory
    let mut group_dir = "staff";
    if entity.group == Group::Student {
        group_dir = "students"
    }
    let directory = format!("{}/{}/{}", config.nfs_root_dir, group_dir, entity.username);
    let dir_exit_code = make_directory(&sess, &directory)?;

    if dir_exit_code == 0 {
        // Give ownership to user
        let owner_exit_code = change_ownership(
            &sess,
            &directory,
            &entity.username,
            &entity.group.to_string(),
        )?;
        if owner_exit_code != 0 {
            error!("NFS host did not return with exit code 0 during ownership change!");
        } else {
            info!("Successfully created user directory on NFS host.");
        }
    } else {
        error!("NFS host did not return with exit code 0 during directory creation!");
    }

    // Set user quota
    if can_set_quota {
        let quota_exit_code = set_quota(
            &sess,
            &entity.username,
            &config.quota_nfs_softlimit,
            &config.quota_nfs_hardlimit,
            &config.nfs_filesystem,
        )?;
        if quota_exit_code != 0 {
            error!("NFS host did not return with exit code 0 during quota setup!")
        }
    }

    Ok(())
}

/// Establish SSH connection to home host, make user directory and set quota
/// TODO: Bubble up errors instead of just logging
fn handle_home(entity: &Entity, config: &MgmtConfig, credentials: &SshCredential) -> AppResult {
    debug!("Start handling home directory");

    if config.home_host.is_empty() {
        warn!("No home host provided in config. Unable to create home directory for user.");
        return Ok(());
    }

    let mut can_set_quota = true;
    if config.quota_home_softlimit.is_empty()
        || config.quota_home_hardlimit.is_empty()
        || config.home_filesystem.is_empty()
    {
        can_set_quota = false;
        warn!("Hard-/softlimit and/or filesystem for quota isn't properly configured. Refusing to set user quota based on these values. Please check your conf.toml");
    }

    info!("Connecting to home host");
    let sess = SshSession::new(&config.home_host, config.ssh_port, credentials);

    // Create directory
    let directory = format!("/home/{}", entity.username);

    let dir_exit_code = if config.use_homedir_helper {
        make_home_directory(&sess, &entity.username)
    } else {
        make_directory(&sess, &directory)
    }?;

    if dir_exit_code == 0 {
        // Give ownership to user
        let owner_exit_code = change_ownership(
            &sess,
            &directory,
            &entity.username,
            &entity.group.to_string(),
        )?;
        if owner_exit_code != 0 {
            error!("Home host did not return with exit code 0 during ownership change!");
        } else {
            info!("Successfully created user home directory.");
        }
    } else {
        error!("Home host did not return with exit code 0 during directory creation!");
    }

    // Set user quota
    if can_set_quota {
        let quota_exit_code = set_quota(
            &sess,
            &entity.username,
            &config.quota_home_softlimit,
            &config.quota_home_hardlimit,
            &config.home_filesystem,
        )?;
        if quota_exit_code != 0 {
            error!("Home host did not return with exit code 0 during quota setup!")
        }
    }

    Ok(())
}

fn make_directory(sess: &SshSession, directory: &str) -> AppResult<i32> {
    debug!("Making directory {}", directory);

    let cmd = format!("sudo mkdir -p {directory}");
    ssh::run_remote_command(sess, &cmd)
}

fn make_home_directory(sess: &SshSession, username: &str) -> AppResult<i32> {
    debug!("Making home directory using the mkhomedir_helper");

    let cmd = format!("sudo mkhomedir_helper {username}");
    ssh::run_remote_command(sess, &cmd)
}

fn change_ownership(
    sess: &SshSession,
    directory: &str,
    username: &str,
    group: &str,
) -> AppResult<i32> {
    debug!("Changing ownership for directory {}", directory);

    let cmd = format!("sudo chown {username}:{group} {directory}");
    ssh::run_remote_command(sess, &cmd)
}

fn set_quota(
    sess: &SshSession,
    username: &str,
    softlimit: &str,
    hardlimit: &str,
    filesystem: &str,
) -> AppResult<i32> {
    debug!(
        "Setting quota for user {} on filesystem {}",
        username, filesystem
    );

    let cmd = format!("sudo setquota -u {username} {softlimit} {hardlimit} 0 0 {filesystem}");

    ssh::run_remote_command(sess, &cmd)
}
