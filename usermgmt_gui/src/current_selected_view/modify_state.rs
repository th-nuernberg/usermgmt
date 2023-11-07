use usermgmt_lib::{
    cli::CommonUserFields, config::MgmtConfig, util::TrimmedNonEmptyText, ChangesToUser, Entity,
};

use crate::prelude::*;

#[derive(Default, Debug)]
pub struct ModifyState {
    pub username: String,
    pub lastname: String,
    pub firstname: String,
    pub group: String,
    pub mail: String,
    pub default_qos: String,
    pub publickey: String,
    pub qos: Vec<String>,
    pub res_io: IoResourceManager<String>,
    pub last_added_username: String,
}

impl ModifyState {
    pub fn create_changes_to_user(&self, config: &MgmtConfig) -> AppResult<ChangesToUser> {
        let (firstname, lastname) = (
            general_utils::some_if_not_blank_str(&self.firstname),
            general_utils::some_if_not_blank_str(&self.lastname),
        );
        let common_fields = CommonUserFields {
            username: TrimmedNonEmptyText::try_from(self.username.clone())?,
            group: general_utils::some_if_not_blank_str(&self.group),
            mail: general_utils::some_if_not_blank_str(&self.mail),
            default_qos: general_utils::some_if_not_blank_str(&self.default_qos),
            publickey: general_utils::some_if_not_blank_str(&self.publickey),
            qos: self.qos.clone(),
        };
        let entity = Entity::new(firstname, lastname, common_fields, config)?;
        let changes = ChangesToUser::try_new(entity)?;
        Ok(changes)
    }
}
