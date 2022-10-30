/// Module for directory management
pub mod dir {
    use log::{debug, error, info, warn};
    use ssh2::Session;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use util::io_util::user_input;

    use crate::config::config::MgmtConfig;
    use crate::{util, Entity};

    pub fn add_user_directories(entity: &Entity, config: &MgmtConfig) {
        let (username, password) = ask_credentials(&config.default_ssh_user);

        handle_compute_nodes(entity, config, &username, &password);

        handle_nfs(entity, config, &username, &password);

        handle_home(entity, config, &username, &password);
    }

    fn ask_credentials(default_user: &str) -> (String, String) {
        println!("Enter your SSH username (defaults to {}):", default_user);
        let mut username = user_input();
        if username.is_empty() {
            username = default_user.to_string();
        }
        let password = rpassword::prompt_password("Enter your SSH password: ").unwrap();
        (username, password)
    }

    /// Establish SSH connection to each compute node, make user directory and set quota
    fn handle_compute_nodes(
        entity: &Entity,
        config: &MgmtConfig,
        ssh_username: &str,
        ssh_password: &str,
    ) {
        debug!("Start handling directories on compute nodes");

        if config.compute_nodes.is_empty() {
            warn!("No compute nodes provided in config. Unable to create user directories.");
            return;
        }
        if config.compute_node_root_dir.is_empty() {
            warn!("No root directory on compute nodes provided in config. Unable to create user directories.");
            return;
        }

        if config.filesystem.is_empty() {
            warn!("No root directory on compute nodes provided in config. Unable to create user directories.");
            return;
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
            // Connect to the SSH server
            info!("Connecting to compute node {}", server);
            let tcp = TcpStream::connect(format!("{server}:22")).unwrap();
            let mut sess = Session::new().unwrap();
            sess.handshake(&tcp).unwrap();

            sess.userauth_password(ssh_username, ssh_password)
                .unwrap();

            // Create directory
            let directory = format!("{}/{}", config.compute_node_root_dir, entity.username);
            let dir_exit_code = make_directory(&sess, &directory);
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
                    );
                }
                quota_exit_codes.push(quota_exit_code);
            }
        }

        if mkdir_exit_codes.iter().all(|&x| x == 0) {
            if owner_exit_codes.iter().all(|&x| x == 0) {
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
    }

    /// Establish SSH connection to NFS host, make user directory and set quota
    fn handle_nfs(entity: &Entity, config: &MgmtConfig, ssh_username: &str, ssh_password: &str) {
        debug!("Start handling NFS user directory");

        if config.nfs_host.is_empty() {
            warn!("No NFS host provided in config. Unable to create directory.");
            return;
        }
        if config.nfs_root_dir.is_empty() {
            warn!("No root directory provided in config. Unable to create directory.");
            return;
        }

        let mut can_set_quota = true;
        if config.quota_nfs_softlimit.is_empty()
            || config.quota_nfs_hardlimit.is_empty()
            || config.nfs_filesystem.is_empty()
        {
            can_set_quota = false;
            warn!("Hard-/softlimit and/or filesystem for quota isn't properly configured. Refusing to set user quota based on these values. Please check your conf.toml");
        }

        // Connect to the SSH server
        info!("Connecting to NFS host {}", config.nfs_host);
        let tcp = TcpStream::connect(format!("{}:22", config.nfs_host)).unwrap();
        let mut sess = Session::new().unwrap();
        sess.handshake(&tcp).unwrap();

        sess.userauth_password(ssh_username, ssh_password)
            .unwrap();

        // Create directory
        let directory = format!("{}/{}", config.nfs_root_dir, entity.username);
        let dir_exit_code = make_directory(&sess, &directory);

        if dir_exit_code == 0 {
            // Give ownership to user
            let owner_exit_code = change_ownership(
                &sess,
                &directory,
                &entity.username,
                &entity.group.to_string(),
            );
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
            );
            if quota_exit_code != 0 {
                error!("NFS host did not return with exit code 0 during quota setup!")
            }
        }
    }

    fn handle_home(entity: &Entity, config: &MgmtConfig, ssh_username: &str, ssh_password: &str) {
        //  Establish ssh connection to augustiner
        // make home dir under /home/
        // Set quota setquota -u someuser 20G 22G 0 0 /dev/sdb4
        debug!("Start handling home directory");

        if config.home_host.is_empty() {
            warn!("No home host provided in config. Unable to create home directory for user.");
            return;
        }

        let mut can_set_quota = true;
        if config.quota_home_softlimit.is_empty()
            || config.quota_home_hardlimit.is_empty()
            || config.home_filesystem.is_empty()
        {
            can_set_quota = false;
            warn!("Hard-/softlimit and/or filesystem for quota isn't properly configured. Refusing to set user quota based on these values. Please check your conf.toml");
        }

        // Connect to the SSH server
        info!("Connecting to home host {}", config.home_host);
        let tcp = TcpStream::connect(format!("{}:22", config.home_host)).unwrap();
        let mut sess = Session::new().unwrap();
        sess.handshake(&tcp).unwrap();

        sess.userauth_password(ssh_username, ssh_password)
            .unwrap();

        // Create directory
        let directory = format!("/home/{}", entity.username);

        let dir_exit_code = if config.use_homedir_helper {
            make_home_directory(&sess, &entity.username)
        } else {
            make_directory(&sess, &directory)
        };

        if dir_exit_code == 0 {
            // Give ownership to user
            let owner_exit_code = change_ownership(
                &sess,
                &directory,
                &entity.username,
                &entity.group.to_string(),
            );
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
            );
            if quota_exit_code != 0 {
                error!("Home host did not return with exit code 0 during quota setup!")
            }
        }
    }

    fn make_directory(sess: &Session, directory: &str) -> i32 {
        debug!("Making directory {}", directory);

        let mut channel = sess.channel_session().unwrap();
        channel.exec(&format!("sudo mkdir -p {directory}")).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let exit_status = channel.exit_status().unwrap();

        debug!("make_directory - command output: {}", s);
        debug!("make_directory - command exit status: {}", exit_status);
        exit_status
    }

    fn make_home_directory(sess: &Session, username: &str) -> i32 {
        debug!("Making home directory using the mkhomedir_helper");

        let mut channel = sess.channel_session().unwrap();
        channel
            .exec(&format!("sudo mkhomedir_helper {username}"))
            .unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let exit_status = channel.exit_status().unwrap();

        debug!("make_home_directory - command output: {}", s);
        debug!("make_home_directory - command exit status: {}", exit_status);
        exit_status
    }

    fn change_ownership(sess: &Session, directory: &str, username: &str, group: &str) -> i32 {
        debug!("Changing ownership for directory {}", directory);

        let mut channel = sess.channel_session().unwrap();
        let cmd = format!("sudo chown {username}:{group} {directory}");
        channel.exec(&cmd).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let exit_status = channel.exit_status().unwrap();

        debug!("change_ownership - command output: {}", s);
        debug!("change_ownership - command exit status: {}", exit_status);
        exit_status
    }

    fn set_quota(
        sess: &Session,
        username: &str,
        softlimit: &str,
        hardlimit: &str,
        filesystem: &str,
    ) -> i32 {
        debug!(
            "Setting quota for user {} on filesystem {}",
            username, filesystem
        );

        let mut channel = sess.channel_session().unwrap();
        let cmd = format!("sudo setquota -u {username} {softlimit} {hardlimit} 0 0 {filesystem}");
        channel.exec(&cmd).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let exit_status = channel.exit_status().unwrap();

        debug!("change_ownership - command output: {}", s);
        debug!("change_ownership - command exit status: {}", exit_status);
        exit_status
    }
}
