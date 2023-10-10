use crate::prelude::AppResult;

use super::LdapCredential;

#[derive(Debug)]
pub struct LdapSimpleCredential {
    username: String,
    password: String,
}

impl LdapCredential for LdapSimpleCredential {
    fn username(&self) -> AppResult<&str> {
        Ok(&self.username)
    }

    fn password(&self) -> AppResult<&str> {
        Ok(&self.password)
    }
}
