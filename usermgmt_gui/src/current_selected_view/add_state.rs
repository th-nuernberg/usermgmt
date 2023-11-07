use usermgmt_lib::{
    cli::{CommonUserFields, UserToAdd},
    prelude::AppResult,
    util::TrimmedNonEmptyText,
};

use crate::io_resource_manager::IoResourceManager;

#[derive(Default, Debug)]
pub struct AddState {
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub group: String,
    pub mail: String,
    pub default_qos: String,
    pub publickey: String,
    pub qos: Vec<String>,
    pub adding_res_io: IoResourceManager<String>,
    pub last_added_username: String,
}

impl AddState {
    pub fn all_needed_fields_filled(&self) -> bool {
        return is_not_blank(&self.firstname)
            && is_not_blank(&self.lastname)
            && is_not_blank(&self.username);

        fn is_not_blank(input: &str) -> bool {
            !input.trim().is_empty()
        }
    }
    pub fn create_user_to_add(&self) -> AppResult<UserToAdd> {
        let (firstname, lastname, username, qos) = (
            self.firstname.clone(),
            self.lastname.clone(),
            self.username.clone(),
            self.qos.clone(),
        );
        let (firstname, lastname, username) = (
            firstname.try_into()?,
            lastname.try_into()?,
            username.try_into()?,
        );
        let user = UserToAdd {
            firstname,
            lastname,
            common_user_fields: CommonUserFields {
                username,
                group: some_if_not_blank_str(&self.group),
                mail: some_if_not_blank_str(&self.mail),
                default_qos: some_if_not_blank_str(&self.default_qos),
                publickey: some_if_not_blank_str(&self.publickey),
                qos,
            },
        };

        return Ok(user);
        fn some_if_not_blank_str(input: &str) -> Option<TrimmedNonEmptyText> {
            input.try_into().ok()
        }
    }
}
