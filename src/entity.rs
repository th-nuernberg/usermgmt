use crate::util;
use anyhow::Context;
use log::{debug, error, info, warn};
use std::{fs, str::FromStr};

use crate::{
    cli::UserToAdd, config::MgmtConfig, prelude::AppResult, util::TrimmedNonEmptyText, Group,
};

/// Representation of a user entity.
/// It contains all information necessary to add/modify/delete the user.
/// TODO: Add proper encapsulation via getter and setters
pub struct Entity {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub gid: i32,
    pub group: Group,
    pub default_qos: String,
    /// TODO: Add validation if a present publickey is in valid format, OpenSsh
    pub publickey: String,
    pub qos: Vec<String>,
}

impl Entity {
    pub fn new(to_add: &UserToAdd, config: &MgmtConfig) -> AppResult<Self> {
        let staff_qos = trim_and_check_qos_collection(
            &config.staff_qos,
            "qos for staff from field (staff_qos) in config",
        )?;
        let student_qos = trim_and_check_qos_collection(
            &config.student_qos,
            "qos for students from field (student_qos) in config",
        )?;
        let valid_qos = trim_and_check_qos_collection(
            &config.valid_qos,
            "general valid qos from field (valid_qos) in config",
        )?;

        let staff_default_qos = &config.staff_default_qos;
        let student_default_qos = &config.student_default_qos;

        let mut default_qos = &to_add.default_qos;
        let mut group_str = to_add.group.as_ref().to_lowercase();
        let mut qos = trim_and_check_qos_collection(&to_add.qos, "inidivuel qow's for user")?;

        if !util::is_valid_group(&group_str, &config.valid_slurm_groups) {
            warn!(
                "Invalid group specified: {}. Group field will be student",
                group_str
            );
            group_str = "student".to_string();
        }

        let group = Group::from_str(&group_str).unwrap();
        let gid = Entity::map_groupname_to_gid(group.clone(), config).unwrap();

        if qos.is_empty() || !util::is_valid_qos(&qos, &valid_qos) {
            info!(
                "Specified QOS {:?} are either invalid or empty. Using defaults in conf.toml.",
                qos
            );
            match group {
                Group::Staff => qos = staff_qos,
                Group::Student => qos = student_qos,
                Group::Faculty => qos = staff_qos,
            }
        }

        default_qos = {
            let no_default_qos = default_qos.is_empty();
            let no_valid_default_qos = !util::is_valid_qos(&[default_qos.clone()], &valid_qos);
            if no_default_qos {
                warn!(
                    "Specified default QOS {} is empty. Using the value specified in config.",
                    default_qos
                );
            }
            if no_valid_default_qos {
                warn!(
                  "Specified default QOS {} is not one of the valid qos. Using the value specified in config.
                   Valid qos are {:?}", 
                   default_qos,
                   valid_qos
                );
            }

            if no_default_qos && no_valid_default_qos {
                match group {
                    Group::Staff => staff_default_qos,
                    Group::Student => student_default_qos,
                    Group::Faculty => staff_default_qos,
                }
            } else {
                default_qos
            }
        };

        let mut pubkey_from_file = "".to_string();
        if !to_add.publickey.is_empty() {
            debug!("Received PublicKey file path {}", to_add.publickey);
            let pubkey_result = fs::read_to_string(&to_add.publickey);
            match pubkey_result {
                Ok(result) => pubkey_from_file = result,
                Err(e) => error!("Unable to read PublicKey from file! {}", e),
            }
        }

        return Ok(Entity {
            username: to_add.user.as_ref().to_lowercase(),
            firstname: to_add.firstname.clone().into(),
            lastname: to_add.lastname.clone().into(),
            group,
            default_qos: default_qos.to_string(),
            publickey: pubkey_from_file.trim().to_string(),
            qos: qos.to_vec(),
            gid,
            mail: to_add.mail.to_string(),
        });

        fn trim_and_check_qos_collection(
            qos: &[impl AsRef<str>],
            qos_name: &str,
        ) -> AppResult<Vec<String>> {
            qos.iter()
                .map(|to_trim| {
                    TrimmedNonEmptyText::try_from(to_trim.as_ref())
                        .map(String::from)
                        .with_context(|| format!("Invalid field in {}", qos_name))
                })
                .collect::<AppResult<Vec<String>>>()
        }
    }

    /// Convert Group enum into integer gid
    fn map_groupname_to_gid(group: Group, config: &MgmtConfig) -> Result<i32, ()> {
        match group {
            Group::Staff => Ok(config.staff_gid),
            Group::Student => Ok(config.student_gid),
            Group::Faculty => Ok(config.faculty_gid),
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            username: "".to_string(),
            firstname: "".to_string(),
            lastname: "".to_string(),
            mail: "".to_string(),
            gid: -1,
            group: Group::Student,
            default_qos: "basic".to_string(),
            publickey: "".to_string(),
            qos: vec![],
        }
    }
}
