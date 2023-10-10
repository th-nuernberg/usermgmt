use once_cell::unsync::OnceCell;
use usermgmt_lib::{
    ldap::{self, LdapCredential},
    prelude::AppResult,
};

#[derive(Debug, Default)]
pub struct LdapCliCredential {
    username: OnceCell<String>,
    password: OnceCell<String>,
}

impl LdapCredential for LdapCliCredential {
    fn username(&self) -> AppResult<&str> {
        self.username
            .get_or_try_init(ldap::ask_cli_username)
            .map(|string| string.as_str())
    }

    fn password(&self) -> AppResult<&str> {
        self.password
            .get_or_try_init(ldap::ask_cli_password)
            .map(|string| string.as_str())
    }
}
