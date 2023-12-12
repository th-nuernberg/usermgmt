use std::path::Path;

use super::connection_state::ConnectionState;

#[derive(Debug, Default)]
pub struct SshConnectionState {
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssh_key_pair: Option<String>,
}

impl ConnectionState for SshConnectionState {
    fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
    fn ssh_key_pair(&self) -> Option<&Path> {
        self.ssh_key_pair.as_deref().map(Path::new)
    }
}
