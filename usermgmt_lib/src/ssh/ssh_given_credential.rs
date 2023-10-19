use std::sync::Arc;

use super::SshCredentials;

#[derive(Debug, Clone)]
pub struct SshGivenCredential {
    username: Arc<str>,
    password: Arc<str>,
}

impl SshGivenCredential {
    #[allow(dead_code)]
    pub fn new(username: &str, password: &str) -> Self {
        let (username, password) = (username.into(), password.into());
        Self { username, password }
    }
}

impl SshCredentials for SshGivenCredential {
    fn username(&self) -> crate::prelude::AppResult<&str> {
        Ok(&self.username)
    }

    fn password(&self) -> crate::prelude::AppResult<&str> {
        Ok(&self.password)
    }

    fn auth_resolve(&self) -> bool {
        false
    }
}
