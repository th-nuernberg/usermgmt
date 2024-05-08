use once_cell::unsync::OnceCell;
use usermgmt_lib::{config::MgmtConfig, ldap::LdapCredential, prelude::AppResult};

use crate::cli_user_input;

#[derive(Debug, Clone)]
pub struct LdapCliCredential {
    default_username: Option<String>,
    username: OnceCell<String>,
    password: OnceCell<String>,
}

impl LdapCliCredential {
    pub fn new(conf: &MgmtConfig) -> Self {
        dbg!();
        let default_username = conf.ldap_default_user.to_owned();
        Self {
            default_username,
            username: Default::default(),
            password: Default::default(),
        }
    }
}

impl LdapCredential for LdapCliCredential {
    fn username(&self) -> AppResult<&str> {
        let a = self
            .username
            .get_or_try_init(|| cli_user_input::ask_cli_username(self.default_username.as_deref()))
            .map(|string| string.as_str());
        dbg!(self);
        a
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
