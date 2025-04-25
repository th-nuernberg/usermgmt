/// Module for directory management
use log::{debug, info, warn};

use crate::util::ResultAccumulator;
use crate::config::MgmtConfig;
use crate::prelude::AppResult;
use crate::ssh::{self, SshConnection, SshCredentials};
use crate::{Group, NewEntity};

pub fn add_user_directories<T>(
    entity: &NewEntity,
    config: &MgmtConfig,
    credentials: &T,
) -> AppResult
where
    T: SshCredentials,
{
    handle_compute_nodes(entity, config, credentials)?;

    handle_nfs(entity, config, credentials)?;

    handle_home(entity, config, credentials)?;

    Ok(())
}

pub fn delete_user_directories<T>(
    username: &str,
    config: &MgmtConfig,
    credentials: &T,
) -> AppResult
where
    T: SshCredentials,
{
    delete_home_dir(username, config, credentials)?;
    delete_nfs_dir(username, config, credentials)?;
    delete_node_local_dir(username, config, credentials)?;

    Ok(())
}

/// Establish SSH connection to each compute node and delete user directory
fn delete_node_local_dir<T>(username: &str, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
    info!("Start deleting directories on compute nodes");

    if config.compute_nodes.is_empty() {
        warn!("No compute nodes provided in config. Unable to delete user directories on nodes.");
        return Ok(());
    }
    if config.compute_node_root_dir.is_empty() {
        warn!("No root directory on compute nodes provided in config. Unable to delete user directories on nodes.");
        return Ok(());
    }

    if config.filesystem.is_empty() {
        warn!("No root directory on compute nodes provided in config. Unable to delete user directories on nodes.");
        return Ok(());
    }
    
    let mut rm_exit_codes = Vec::new();
    for server in config.compute_nodes.iter() {
        info!("{}", format!("Connecting to compute node {} for directory deletion", server));
        let sess = SshConnection::new(server, config, credentials.clone());
        // Delete directory
        let directory = format!("{}/{}", config.compute_node_root_dir, username);
        let (dir_exit_code, _) = delete_directory(&sess, &directory)?;
        rm_exit_codes.push(dir_exit_code);
    }

    let mut errors_from_codes =
        ResultAccumulator::new("Failed to delete all directories on compute nodes".to_owned());

    let all_exit_codes_are_zero = rm_exit_codes.iter().all(|&x| x == 0);

    errors_from_codes.add_err_if_false(
        all_exit_codes_are_zero,
        "Not all compute nodes returned exit code 0 during directory deletion!".to_owned(),
    );
    
    if !errors_from_codes.errs.is_empty() {
        warn!("{}: {}", errors_from_codes.base_err_msg, errors_from_codes.errs.join("\n"));
    } else {
        info!("Successfully deleted directories on compute nodes.");
    }
    
    Ok(())
}


/// Establish SSH connection to NFS hosts and delete user directory
fn delete_nfs_dir<T>(username: &str, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
    debug!("Start deleting NFS user directories");

    if config.nfs_host.is_empty() {
        warn!("No NFS host provided in config. Unable to create directory.");
        return Ok(());
    }
    if config.nfs_root_dir.is_empty() {
        warn!("No root directory provided in config. Unable to create directory.");
        return Ok(());
    }

    let mut detected_errors =
        ResultAccumulator::new("Errors during NFS directory deletion occurred".to_owned());
    for i in 0..config.nfs_host.len() {
        let current_nfs_host = &config.nfs_host[i];
        let current_nfs_root_dir = &config.nfs_root_dir[i];

        info!("Connecting to NFS host {} for directory deletion", current_nfs_host);
        let sess = SshConnection::new(current_nfs_host, config, credentials.clone());

        // Infer user group
        let mut group_dir = "staff";
        if username.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            group_dir = "students";
        }
        
        let directory = format!("{}/{}/{}", current_nfs_root_dir, group_dir, username);
        let (dir_exit_code, _) = delete_directory(&sess, &directory)?;

        if dir_exit_code != 0 {
            detected_errors.add_err(format!(
                "NFS host {} did not return with exit code 0 during directory deletion!",
                current_nfs_host
            ));
        } else {
            info!(
                    "{}",
                    format!(
                        "Successfully deleted user directory on NFS host {}.",
                        current_nfs_host
                    )
                );
        }
    }

    if !detected_errors.errs.is_empty() {
        warn!("{}: {}", detected_errors.base_err_msg, detected_errors.errs.join("\n"));
    }

    Ok(())
}

/// Establish SSH connection to home host and delete user home directory
fn delete_home_dir<T>(username: &str, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
    debug!("Start deleting home directory");

    if config.home_host.is_empty() {
        warn!("No home host provided in config. Unable to delete user home directory.");
        return Ok(());
    }
    
    info!(
        "{}",
        format!("Connecting to home host {} for directory deletion", &config.home_host)
    );
    let sess = SshConnection::new(&config.home_host, config, credentials.clone());

    // Delete directory
    let directory = format!("/home/{}", username);
    let (dir_exit_code, _) = delete_directory(&sess, &directory)?;

    if dir_exit_code == 0 {
        info!("Successfully deleted user home directory.");
        
    } else {
        warn!("{}", format!("Failed to delete user home directory: {}", &directory));
    }
    
    Ok(())
}



/// Establish SSH connection to each compute node, make user directory and set quota
fn handle_compute_nodes<T>(entity: &NewEntity, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
    info!("Start handling directories on compute nodes");

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
        info!("{}", format!("Connecting to compute node {}", server));
        let sess = SshConnection::new(server, config, credentials.clone());
        // Create directory
        let directory = format!("{}/{}", config.compute_node_root_dir, entity.username);
        let (dir_exit_code, _) = make_directory(&sess, &directory)?;
        mkdir_exit_codes.push(dir_exit_code);

        if dir_exit_code == 0 {
            // Give ownership to user
            let owner_exit_code = change_ownership(
                &sess,
                &directory,
                entity.username.as_ref(),
                &entity.group.to_string(),
            );
            owner_exit_codes.push(owner_exit_code);

            // Set user quota
            let mut quota_exit_code = 1;
            if can_set_quota {
                (quota_exit_code, _) = set_quota(
                    &sess,
                    entity.username.as_ref(),
                    &config.quota_softlimit,
                    &config.quota_hardlimit,
                    &config.filesystem,
                )?;
            }
            quota_exit_codes.push(quota_exit_code);
        }
    }

    let mut errors_from_codes =
        ResultAccumulator::new("Failed to create directories on compute nodes.".to_owned());

    let all_exit_codes_are_zero = mkdir_exit_codes.iter().all(|&x| x == 0);

    errors_from_codes.add_err_if_false(
        all_exit_codes_are_zero,
        "Not all compute nodes returned exit code 0 during directory creation!".to_owned(),
    );

    let all_owner_exit_codes_are_zero = owner_exit_codes
        .iter()
        .all(|x| x.as_ref().is_ok_and(|(code, _)| *code == 0));

    errors_from_codes.add_err_if_false(
        all_owner_exit_codes_are_zero,
        "Not all compute nodes returned exit code 0 during ownership change!".to_owned(),
    );

    let all_quota_exit_codes_are_zero = quota_exit_codes.iter().all(|&x| x == 0);

    errors_from_codes.add_err_if_false(
        all_quota_exit_codes_are_zero,
        "Not all compute nodes returned exit code 0 during quota setup!".to_owned(),
    );

    AppResult::from(errors_from_codes)?;

    info!("Successfully created directories on compute nodes.");

    Ok(())
}

/// Establish SSH connection to NFS hosts, make user directory and set quota
fn handle_nfs<T>(entity: &NewEntity, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
    debug!("Start handling NFS user directories");

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

    let mut detected_errors =
        ResultAccumulator::new("Errors during NFS directory creation occurred!".to_owned());
    for i in 0..config.nfs_host.len() {
        let current_nfs_host = &config.nfs_host[i];
        let current_nfs_root_dir = &config.nfs_root_dir[i];
        let current_quota_nfs_softlimit = &config.quota_nfs_softlimit[i];
        let current_nfs_quota_nfs_hardlimit = &config.quota_nfs_hardlimit[i];
        let current_nfs_filesystem = &config.nfs_filesystem[i];

        info!("Connecting to NFS host {}", current_nfs_host);
        let sess = SshConnection::new(current_nfs_host, config, credentials.clone());

        // Create directory
        let mut group_dir = "staff";
        if entity.group.id() == Group::Student {
            group_dir = "students"
        }
        let directory = format!("{}/{}/{}", current_nfs_root_dir, group_dir, entity.username);
        let (dir_exit_code, _) = make_directory(&sess, &directory)?;

        let no_error_make_dir = dir_exit_code == 0;
        if no_error_make_dir {
            // Give ownership to user
            let (owner_exit_code, _) = change_ownership(
                &sess,
                &directory,
                entity.username.as_ref(),
                &entity.group.to_string(),
            )?;
            if owner_exit_code != 0 {
                detected_errors.add_err(format!(
                    "NFS host {} did not return with exit code 0 during ownership change!",
                    current_nfs_host
                ));
            } else {
                info!(
                    "{}",
                    format!(
                        "Successfully created user directory on NFS host {}.",
                        current_nfs_host
                    )
                );
            }
        } else {
            detected_errors.add_err(format!(
                "NFS host {} did not return with exit code 0 during directory creation!",
                current_nfs_host
            ));
        }

        // Set user quota
        if can_set_quota {
            let (quota_exit_code, _) = set_quota(
                &sess,
                entity.username.as_ref(),
                current_quota_nfs_softlimit,
                current_nfs_quota_nfs_hardlimit,
                current_nfs_filesystem,
            )?;

            detected_errors.add_err_if_false(
                quota_exit_code == 0,
                format!(
                    "NFS host {} did not return with exit code 0 during quota setup!",
                    current_nfs_host
                ),
            )
        }
    }

    AppResult::from(detected_errors)?;

    Ok(())
}

/// Establish SSH connection to home host, make user directory and set quota
fn handle_home<T>(entity: &NewEntity, config: &MgmtConfig, credentials: &T) -> AppResult
where
    T: SshCredentials,
{
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

    info!(
        "{}",
        format!("Connecting to home host {}", &config.home_host)
    );
    let sess = SshConnection::new(&config.home_host, config, credentials.clone());

    // Create directory
    let directory = format!("/home/{}", entity.username);

    let (dir_exit_code, _) = if config.use_homedir_helper {
        make_home_directory(&sess, entity.username.as_ref())
    } else {
        make_directory(&sess, &directory)
    }?;

    let mut detected_errors = ResultAccumulator::new(format!(
        "Errors during home directory creation occurred on host {}",
        &config.home_host
    ));

    if dir_exit_code == 0 {
        // Give ownership to user
        let (owner_exit_code, _) = change_ownership(
            &sess,
            &directory,
            entity.username.as_ref(),
            &entity.group.to_string(),
        )?;
        if owner_exit_code != 0 {
            detected_errors.add_err(
                "Home host did not return with exit code 0 during ownership change!".to_owned(),
            );
        } else {
            info!("Successfully created user home directory.");
        }
    } else {
        detected_errors.add_err(
            "Home host did not return with exit code 0 during directory creation!".to_owned(),
        );
    }

    // Set user quota
    if can_set_quota {
        let (quota_exit_code, _) = set_quota(
            &sess,
            entity.username.as_ref(),
            &config.quota_home_softlimit,
            &config.quota_home_hardlimit,
            &config.home_filesystem,
        )?;
        detected_errors.add_err_if_false(
            quota_exit_code == 0,
            format!(
                "Home host did not return with exit code 0 (actual exit code: {}) during quota setup!",
                quota_exit_code
            ),
        );
    }

    AppResult::from(detected_errors)?;

    Ok(())
}

fn make_directory<C>(sess: &SshConnection<C>, directory: &str) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!("Making directory {}", directory);

    let cmd = format!("sudo mkdir -p {directory}");
    ssh::run_remote_command(sess, &cmd)
}

fn delete_directory<C>(sess: &SshConnection<C>, directory: &str) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!("Deleting directory {}", directory);

    let cmd = format!("sudo rm -r {directory}");
    ssh::run_remote_command(sess, &cmd)
}

fn make_home_directory<C>(sess: &SshConnection<C>, username: &str) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!("Making home directory using the mkhomedir_helper utility");

    let cmd = format!("sudo mkhomedir_helper {username}");
    ssh::run_remote_command(sess, &cmd)
}

fn change_ownership<C>(
    sess: &SshConnection<C>,
    directory: &str,
    username: &str,
    group: &str,
) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!("Changing ownership for directory {}", directory);

    let cmd = format!("sudo chown {username}:{group} {directory}");
    ssh::run_remote_command(sess, &cmd)
}

fn set_quota<C>(
    sess: &SshConnection<C>,
    username: &str,
    softlimit: &str,
    hardlimit: &str,
    filesystem: &str,
) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!(
        "Setting quota for user {} on filesystem {}",
        username, filesystem
    );

    let cmd = format!("sudo setquota -u {username} {softlimit} {hardlimit} 0 0 {filesystem}");

    ssh::run_remote_command(sess, &cmd)
}
