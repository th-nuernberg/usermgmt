use crate::prelude::AppResult;

use super::LdapCredential;

#[derive(Debug, Default, Clone)]
pub struct LdapSimpleCredential {
    username: String,
    password: String,
}

impl LdapSimpleCredential {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl LdapCredential for LdapSimpleCredential {
    fn username(&self) -> AppResult<&str> {
        Ok(&self.username)
    }

    fn password(&self) -> AppResult<&str> {
        Ok(&self.password)
    }

    fn set_password(&mut self, new: String) {
        self.password = new;
    }
}
