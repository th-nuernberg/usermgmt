use log::info;
use once_cell::sync::OnceCell;
use usermgmt_lib::cli::OptFilePath;
use usermgmt_lib::prelude::*;

use usermgmt_lib::ssh::SshKeyPair;
use usermgmt_lib::{
    config::MgmtConfig,
    prelude::{anyhow, AppResult},
    ssh::SshCredentials,
};

use crate::user_input;

#[derive(Debug, Clone)]
pub struct CliSshCredential {
    default_ssh_user: String,
    username: OnceCell<String>,
    password: OnceCell<String>,
    ssh_key_path: Option<SshKeyPair>,
}

impl CliSshCredential {
    pub fn new(config: &MgmtConfig, on_which_sys: &OptFilePath) -> Self {
        let ssh_key_path = on_which_sys
            .as_ref()
            .cloned()
            .or_else(|| config.ssh_key_path.clone())
            .map(SshKeyPair::from_one_path);
        Self {
            username: Default::default(),
            password: Default::default(),
            default_ssh_user: config.default_ssh_user.clone(),
            ssh_key_path,
        }
    }
}

impl SshCredentials for CliSshCredential {
    /// Returns given username of user or the default user name if the user has given no username
    fn username(&self) -> AppResult<&str> {
        let username = self.username.get_or_try_init(|| {
            user_input::ask_for_line_from_user_over_term(
                "Enter your SSH username",
                Some(self.default_ssh_user.as_str()),
            )
        })?;

        Ok(username)
    }
    fn password(&self) -> AppResult<&str> {
        let password = self.password.get_or_try_init(|| {
            let from_prompt = user_input::cli_ask_for_password("Enter your SSH password: ")?;
            Ok::<String, AppError>(from_prompt.unwrap_or_default())
        })?;

        Ok(password)
    }

    fn auth_agent_resolve(
        &self,
        many_keys: Vec<usermgmt_lib::ssh::SshPublicKeySuggestion>,
    ) -> AppResult<usize> {
        let length = many_keys.len();
        let last_index = length.saturating_sub(1);
        println!("Found more than one key in ssh agent !");
        println!("Choose one between {} and {} ssh key", 0, last_index);
        println!("===========================================");

        for (index, next) in many_keys.iter().enumerate() {
            let comment = next.comment();
            println!("{} => comment: {}", index, comment);
        }

        let user_choice: usize = user_input::line_input_from_user()?
            .ok_or_else(|| anyhow!("No number supplied"))?
            .parse()?;

        if last_index < user_choice {
            Err(anyhow!("Choice should between {} and {}", 0, last_index))
        } else {
            info!("{}. ssh key is chosen", user_choice);
            Ok(last_index)
        }
    }

    fn ssh_paths_pair_key(&self) -> Option<&SshKeyPair> {
        self.ssh_key_path.as_ref()
    }
}
