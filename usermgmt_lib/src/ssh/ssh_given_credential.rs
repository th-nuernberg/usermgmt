use std::sync::Arc;

use super::{SshCredentials, SshKeyPair};

#[derive(Debug, Clone)]
pub struct SshGivenCredential {
    username: Arc<str>,
    password: Arc<str>,
    ssh_key_pair: Option<Arc<SshKeyPair>>,
}

impl Default for SshGivenCredential {
    fn default() -> Self {
        Self::new("", "", None)
    }
}

impl SshGivenCredential {
    pub fn new(username: &str, password: &str, path: Option<SshKeyPair>) -> Self {
        let (username, password, ssh_key_pair) =
            (username.into(), password.into(), path.map(Arc::from));
        Self {
            username,
            password,
            ssh_key_pair,
        }
    }
}

impl SshCredentials for SshGivenCredential {
    fn username(&self) -> crate::prelude::AppResult<&str> {
        Ok(&self.username)
    }

    fn password(&self) -> crate::prelude::AppResult<&str> {
        Ok(&self.password)
    }

    fn ssh_paths_pair_key(&self) -> Option<&SshKeyPair> {
        self.ssh_key_pair.as_deref()
    }
}
