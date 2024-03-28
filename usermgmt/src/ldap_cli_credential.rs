use once_cell::unsync::OnceCell;
use usermgmt_lib::{ldap::LdapCredential, prelude::AppResult};

use crate::cli_user_input;

#[derive(Debug, Default, Clone)]
pub struct LdapCliCredential {
    username: OnceCell<String>,
    password: OnceCell<String>,
}

impl LdapCredential for LdapCliCredential {
    fn username(&self) -> AppResult<&str> {
        self.username
            .get_or_try_init(cli_user_input::ask_cli_username)
            .map(|string| string.as_str())
    }

    fn password(&self) -> AppResult<&str> {
        self.password
            .get_or_try_init(cli_user_input::ask_cli_password)
            .map(|string| string.as_str())
    }

    fn set_password(&mut self, new: String) {
        self.password = OnceCell::new();
        self.password
            .set(new)
            .expect("Once cell is cleared the line above");
    }
}
