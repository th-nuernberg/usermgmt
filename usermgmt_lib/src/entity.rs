use crate::{
    cli::{CommonUserFields, Modifiable, UserToAdd},
    prelude::AppError,
    util::{ResolvedGid, ValidGroupOfQos, ValidQos},
};
use anyhow::{ensure, Context};
use log::debug;
use std::{fs, path::Path, str::FromStr};

use crate::{config::MgmtConfig, prelude::AppResult, util::TrimmedNonEmptyText, Group};

/// Representation of a user entity.
/// It contains all information necessary to add/modify/delete the user.
#[derive(Debug, Clone)]
pub struct Entity {
    pub username: TrimmedNonEmptyText,
    pub firstname: Option<TrimmedNonEmptyText>,
    pub lastname: Option<TrimmedNonEmptyText>,
    pub mail: Option<TrimmedNonEmptyText>,
    pub group: Option<ResolvedGid>,
    pub default_qos: Option<ValidQos>,
    /// TODO: Add validation if a present publickey is in valid format, OpenSsh
    pub publickey: Option<TrimmedNonEmptyText>,
    pub qos: Option<ValidGroupOfQos>,
}

impl Entity {
    pub fn new(
        firstname: Option<TrimmedNonEmptyText>,
        lastname: Option<TrimmedNonEmptyText>,
        to_add: CommonUserFields,
        config: &MgmtConfig,
    ) -> AppResult<Self> {
        Self::new_inner(firstname, lastname, to_add, config, |path| {
            fs::read_to_string(path).with_context(|| {
                format!(
                    "Unable to read PublicKey from file from path {} !",
                    path.to_string_lossy()
                )
            })
        })
    }

    pub fn new_inner(
        firstname: Option<TrimmedNonEmptyText>,
        lastname: Option<TrimmedNonEmptyText>,
        to_add: CommonUserFields,
        config: &MgmtConfig,
        on_load_pubkey: impl Fn(&Path) -> AppResult<String>,
    ) -> AppResult<Self> {
        // group: Option<TrimmedNonEmptyText>,
        // /// User's e-mail address.
        // #[clap(short, long, value_parser = trimmed_non_empty)]
        // mail: Option<TrimmedNonEmptyText>,
        // /// Slurm default QOS for the user e.g. basic.
        // #[clap(short, long, value_parser = trimmed_non_empty)]
        // default_qos: Option<TrimmedNonEmptyText>,
        // /// Path to SSH publickey.
        // #[clap(short, long, value_parser = trimmed_non_empty)]
        // publickey: Option<TrimmedNonEmptyText>,
        // /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). QOS need to be provided as a whitespace separated list (e.g. interactive basic).
        // #[clap(short, long, num_args(0..=20))]
        // qos: Vec<String>,
        let (username, group, mail, default_qos, publickey, qos) = to_add.into();
        let group = group
            .map(|group| {
                let group_id = Group::from_str(group.as_ref().as_str())
                    .context("Error in mapping name to group id")?;
                Ok::<ResolvedGid, AppError>(ResolvedGid::new(group_id, config))
            })
            .transpose()?;

        let qos = if qos.is_empty() {
            None
        } else {
            let qos = qos
                .iter()
                .map(|to_validate| TrimmedNonEmptyText::try_from(to_validate.as_str()))
                .collect::<AppResult<_>>()?;
            let qos = ValidGroupOfQos::new(qos, &config.valid_qos)?;
            Some(qos)
        };

        let default_qos = default_qos
            .map(|to_validate| ValidQos::new(to_validate.into(), &config.valid_qos))
            .transpose()?;
        if let (Some(qos), Some(default_qos)) = (&qos, &default_qos) {
            ensure!(
                qos.contains_other_qos(default_qos),
                "Qos ({:?}) do not contain the default qos ({})",
                qos,
                default_qos
            );
        }

        let publickey = publickey
            .map(|path| {
                debug!("Trying to load the public key from path at {} .", path);

                let content = on_load_pubkey(Path::new(path.as_ref()))?;
                TrimmedNonEmptyText::try_from(content)
            })
            .transpose()?;

        Ok(Entity {
            username,
            firstname,
            lastname,
            group,
            default_qos,
            publickey,
            qos,
            mail,
        })
    }

    pub fn new_modifieble_conf(modif: Modifiable, conf: &MgmtConfig) -> AppResult<Self> {
        let (firstname, lastname, common_user_fields) = modif.into();
        Self::new(firstname, lastname, common_user_fields, conf)
    }

    pub fn new_user_addition_conf(to_add: UserToAdd, conf: &MgmtConfig) -> AppResult<Self> {
        let (firstname, lastname, common_user_fields) = to_add.into();
        let (firstname, lastname) = (Some(firstname), Some(lastname));
        Self::new(firstname, lastname, common_user_fields, conf)
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn error_for_not_valid_default_qos() {
        let mut input = CommonUserFields::new("SomeUser".try_into().unwrap());
        input.default_qos = Some("NotValid".try_into().unwrap());
        let actual = Entity::new_inner(None, None, input, &MgmtConfig::default(), |_| panic!());

        insta::assert_debug_snapshot!(actual);
    }
    #[test]
    fn error_for_not_valid_group_of_qos() {
        let mut input = CommonUserFields::new("SomeUser".try_into().unwrap());
        input.qos = vec!["valid".into(), "not_valid".into()];
        let actual = Entity::new_inner(
            None,
            None,
            input,
            &MgmtConfig {
                valid_qos: vec!["valid".into()],
                ..MgmtConfig::default()
            },
            |_| panic!(),
        );

        insta::assert_debug_snapshot!(actual);
    }

    #[test]
    fn ok_with_valid_default_and_group_of_qos_pubkey() {
        let mut input = CommonUserFields::new("Some_User".try_into().unwrap());
        input.group = Some("staff".try_into().unwrap());
        input.default_qos = Some("staff".try_into().unwrap());
        input.qos = vec!["valid".into(), "basic".into()];
        input.default_qos = Some("valid".try_into().unwrap());
        input.publickey = Some("Some_path".try_into().unwrap());
        input.mail = Some("faculty@xxx.de".try_into().unwrap());
        let actual = Entity::new_inner(
            Some("First".try_into().unwrap()),
            None,
            input,
            &MgmtConfig {
                valid_qos: vec!["valid".into(), "basic".into()],
                ..MgmtConfig::default()
            },
            |_path| Ok("xxxxxx".to_string()),
        );

        insta::assert_debug_snapshot!(actual);
    }
}
