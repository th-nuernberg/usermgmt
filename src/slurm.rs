pub mod slurm {
    use std::process::Command;

    use log::{debug, error, info, warn};

    use crate::{Entity, Modifiable};

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
                "Did not modify default QOS for user {} in Slurm, since nothing was specified.",
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
                "Did not modify QOS for user {} in Slurm, since nothing was specified.",
                modifiable.username
            );
        }
    }

    fn modify_qos(entity: &Entity, sacctmgr_path: &str, default_qos: bool) {
        let mut qos_str: String = "defaultQos=".to_owned();
        if default_qos {
            qos_str += &entity.default_qos;
        } else {
            let qos_joined = entity.qos.join(",");
            qos_str = format!("qos={}", qos_joined);
        }

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
