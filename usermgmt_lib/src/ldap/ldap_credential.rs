use crate::prelude::AppResult;

pub trait LdapCredential {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
}
