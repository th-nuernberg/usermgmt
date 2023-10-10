use anyhow::anyhow;
use once_cell::unsync::OnceCell;

use crate::{config::MgmtConfig, prelude::AppResult, util::user_input};

/// Fetches username and password lazy at the first time.
/// The fetching of username and password happens only once !
pub struct SshCredential<'a> {
    default_ssh_user: &'a str,
    username: OnceCell<String>,
    password: OnceCell<String>,
}

impl<'a> SshCredential<'a> {
    pub fn new(config: &'a MgmtConfig) -> Self {
        Self {
            username: Default::default(),
            password: Default::default(),
            default_ssh_user: &config.default_ssh_user,
        }
    }
    /// Returns given username of user or the default user name if the user has given no username
    pub fn username(&self) -> AppResult<&str> {
        let username = self.username.get_or_try_init(|| {
            user_input::ask_for_line_from_user_over_term(
                "Enter your SSH username",
                Some(self.default_ssh_user),
            )
        })?;

        Ok(username)
    }
    pub fn password(&self) -> AppResult<&str> {
        let password = self.password.get_or_try_init(|| {
            let maybe_password = user_input::cli_ask_for_password("Enter your SSH password: ")?;
            maybe_password.ok_or_else(|| anyhow!("No password provided"))
        })?;

        Ok(password)
    }
}
