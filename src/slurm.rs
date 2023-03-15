pub mod local {
    use std::process::Command;

    use log::{debug, error, info, warn};

    use crate::{Entity, Modifiable};

    /// TODO: Bubble up error instead of just logging it
    pub fn add_slurm_user(entity: &Entity, sacctmgr_path: &str) {
        let output = Command::new(sacctmgr_path)
            .arg("add")
            .arg("user")
            .arg(entity.username.clone())
            .arg(format!("Account={}", entity.group))
            .arg("--immediate")
            .output()
            .expect(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            );

        debug!(
            "add_slurm_user: {}",
            String::from_utf8_lossy(&output.stdout)
        );

        if output.status.success() {
            info!("Added user {} to Slurm", entity.username);
        } else {
            warn!("Slurm user creation did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("sacctmgr stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.len() > 0 {
                error!("sacctmgr stderr: {}", err);
            }
        }

        debug!("Modifying user qos");
        modify_qos(entity, sacctmgr_path, true);
        modify_qos(entity, sacctmgr_path, false);
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn delete_slurm_user(user: &str, sacctmgr_path: &str) {
        let output = Command::new(sacctmgr_path)
            .arg("delete")
            .arg("user")
            .arg(user)
            .arg("--immediate")
            .output()
            .expect(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            );

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
            if err.len() > 0 {
                error!("sacctmgr stderr: {}", err);
            }
        }
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn modify_slurm_user(modifiable: &Modifiable, sacctmgr_path: &str) {
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
                modify_qos(&entity, sacctmgr_path, true);
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
            modify_qos(&entity, sacctmgr_path, false)
        } else {
            info!(
                "Did not modify QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            );
        }
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn list_users(sacctmgr_path: &str) {
        let output = Command::new(sacctmgr_path)
            .arg("show")
            .arg("assoc")
            .arg("format=User%30,Account,DefaultQOS,QOS%80")
            .output()
            .expect(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            );
        // sacctmgr show assoc format=cluster,account,qos
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    /// TODO: Bubble up error instead of just logging it
    fn modify_qos(entity: &Entity, sacctmgr_path: &str, default_qos: bool) {
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
            .expect(
                "Unable to execute sacctmgr command. Is the path specified in your config correct?",
            );

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
            if err.len() > 0 {
                error!("sacctmgr stderr: {}", err);
            }
        }
    }
}

pub mod remote {
    use std::io::Read;
    use std::net::TcpStream;

    use log::{debug, error, info};
    use ssh2::Session;

    use crate::config::config::MgmtConfig;
    use crate::ssh::SshCredential;
    use crate::{Entity, Modifiable};

    /// TODO: Bubble up error instead of just logging it
    pub fn add_slurm_user(entity: &Entity, config: &MgmtConfig, credentials: &SshCredential) {
        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let tcp = TcpStream::connect(format!("{}:22", config.head_node)).unwrap();
        let mut sess = Session::new().unwrap();

        sess.handshake(&tcp).unwrap();
        sess.userauth_password(credentials.username(), credentials.password())
            .unwrap();

        let cmd = format!(
            "{} add user {} Account={} --immediate",
            config.sacctmgr_path, entity.username, entity.group
        );
        let exit_code = run_remote_command(&sess, &cmd);

        match exit_code {
            0 => info!(
                "Successfully created Slurm user {}:{}.",
                entity.username, entity.group
            ),
            _ => error!(
                "Failed to create Slurm user. Command '{}' did not exit with 0.",
                cmd
            ),
        };

        debug!("Modifying user qos");
        modify_qos(&entity, config, &sess, true);
        modify_qos(&entity, config, &sess, false);
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn delete_slurm_user(user: &str, config: &MgmtConfig, credentials: &SshCredential) {
        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let tcp = TcpStream::connect(format!("{}:22", config.head_node)).unwrap();
        let mut sess = Session::new().unwrap();

        sess.handshake(&tcp).unwrap();
        sess.userauth_password(credentials.username(), credentials.password())
            .unwrap();

        let cmd = format!("{} delete user {} --immediate", config.sacctmgr_path, user);
        let exit_code = run_remote_command(&sess, &cmd);

        match exit_code {
            0 => info!("Successfully deleted Slurm user {}.", user),
            _ => error!(
                "Failed to delete Slurm user. Command '{}' did not exit with 0.",
                cmd
            ),
        };
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn modify_slurm_user(
        modifiable: &Modifiable,
        config: &MgmtConfig,
        credentials: &SshCredential,
    ) {
        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let tcp = TcpStream::connect(format!("{}:22", config.head_node)).unwrap();
        let mut sess = Session::new().unwrap();

        sess.handshake(&tcp).unwrap();
        sess.userauth_password(credentials.username(), credentials.password())
            .unwrap();

        debug!("Start modifying user default qos");
        match &modifiable.default_qos {
            Some(m) => {
                let entity = Entity {
                    username: modifiable.username.clone(),
                    default_qos: m.to_string(),
                    ..Default::default()
                };
                debug!("New defaultQOS will be {}", entity.default_qos);
                modify_qos(&entity, config, &sess, true);
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
            modify_qos(&entity, config, &sess, false);
        } else {
            info!(
                "Did not modify QOS for user {} in Slurm, since nothing was specified to modify.",
                modifiable.username
            );
        }
    }

    /// TODO: Bubble up error instead of just logging it
    pub fn list_users(config: &MgmtConfig, credentials: &SshCredential) {
        let cmd = "sacctmgr show assoc format=User%30,Account,DefaultQOS,QOS%80";

        // Connect to the SSH server and authenticate
        info!("Connecting to {}", config.head_node);
        let tcp = TcpStream::connect(format!("{}:22", config.head_node)).unwrap();
        let mut sess = Session::new().unwrap();

        sess.handshake(&tcp).unwrap();
        sess.userauth_password(credentials.username(), credentials.password())
            .unwrap();

        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        println!("{}", s);
    }

    /// TODO: Bubble up error instead of just logging it
    fn modify_qos(entity: &Entity, config: &MgmtConfig, sess: &Session, default_qos: bool) {
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
        let exit_code = run_remote_command(sess, &cmd);

        match exit_code {
            0 => info!(
                "Successfully modified QOS of user {} in Slurm.",
                entity.username
            ),
            _ => error!(
                "Failed to modify Slurm user! Command '{}' did not exit with 0.",
                cmd
            ),
        };
    }

    /// Executes given command `cmd` on remote machine over ssh
    ///
    /// TODO: move checking if for exit code inside here and return result instead of error code
    /// directly
    fn run_remote_command(sess: &Session, cmd: &str) -> i32 {
        debug!("Running command {}", cmd);

        let mut channel = sess.channel_session().unwrap();
        channel.exec(cmd).unwrap();

        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let exit_status = channel.exit_status().unwrap();

        debug!("command output: {}", s);
        debug!("command exit status: {}", exit_status);
        exit_status
    }
}
