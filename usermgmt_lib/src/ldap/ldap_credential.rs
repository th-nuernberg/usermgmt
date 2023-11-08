use crate::prelude::AppResult;

pub trait LdapCredential {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
    fn set_password(&mut self, new: String);
}
