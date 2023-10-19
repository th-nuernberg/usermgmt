#[derive(Debug, Default)]
pub struct SshConnectionState {
    pub username: Option<String>,
    pub password: Option<String>,
}
