use log::warn;

use crate::{
    cli::UserToAdd,
    config::MgmtConfig,
    prelude::AppResult,
    util::{ResolvedGid, TrimmedNonEmptyText, ValidGroupOfQos, ValidQos},
    Entity, Group,
};

/// Contains attributes used for adding users in various systems like LDAP or slurm database
pub struct NewEntity {
    pub username: TrimmedNonEmptyText,
    pub firstname: TrimmedNonEmptyText,
    pub lastname: TrimmedNonEmptyText,
    pub mail: Option<TrimmedNonEmptyText>,
    pub group: ResolvedGid,
    pub default_qos: ValidQos,
    pub publickey: Option<TrimmedNonEmptyText>,
    pub qos: ValidGroupOfQos,
}

impl NewEntity {
    /// # Errors
    ///
    /// - If first or last name is not provided.
    pub fn new(entity: Entity, config: &MgmtConfig) -> AppResult<Self> {
        let (firstname, lastname) = match (entity.firstname, entity.lastname) {
            (Some(first), Some(last)) => Ok((first, last)),
            _ => Err(anyhow::anyhow!("Last and first name need to be provided")),
        }?;

        let (mail, publickey) = (entity.mail, entity.publickey);

        let group = entity.group.unwrap_or_else(|| {
            let group = Group::default();
            ResolvedGid::new(group, config)
        });
        let default_qos = entity
            .default_qos
            .unwrap_or_else(|| ValidQos::default_qos_from_conf(group.id(), config));
        let qos = entity
            .qos
            .map(Ok)
            .unwrap_or_else(|| ValidGroupOfQos::from_group(group.id(), config))?;

        if publickey.is_none() {
            warn!("No public key was supplied for new user. Remember to add it later via modification");
        }

        Ok(Self {
            username: entity.username,
            default_qos,
            group,
            firstname,
            lastname,
            mail,
            publickey,
            qos,
        })
    }

    /// # Errors
    ///
    /// - If an user entity could not be created. See [`Entity::new`]
    pub fn new_user_addition_conf(to_add: UserToAdd, conf: &MgmtConfig) -> AppResult<Self> {
        let (firstname, lastname, common_user_fields) = to_add.into();
        let (firstname, lastname) = (Some(firstname), Some(lastname));
        let entity = Entity::new(firstname, lastname, common_user_fields, conf)?;
        Self::new(entity, conf)
    }
}
