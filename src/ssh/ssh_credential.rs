use once_cell::unsync::OnceCell;

use crate::config::config::MgmtConfig;

pub struct SshCredential<'a> {
    default_ssh_user: &'a str,
    username_password: OnceCell<(String, String)>,
}

impl<'a> SshCredential<'a> {
    pub fn new(config: &'a MgmtConfig) -> Self {
        Self {
            username_password: OnceCell::new(),
            default_ssh_user: &config.default_ssh_user,
        }
    }
    pub fn username(&self) -> &str {
        let (username, _) = self
            .username_password
            .get_or_init(|| super::ask_credentials(self.default_ssh_user));

        username
    }
    pub fn password(&self) -> &str {
        let (_, password) = self
            .username_password
            .get_or_init(|| super::ask_credentials(self.default_ssh_user));

        password
    }
}
