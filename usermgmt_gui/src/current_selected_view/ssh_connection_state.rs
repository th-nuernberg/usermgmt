#[derive(Debug, Default)]
pub struct SshConnectionState {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl SshConnectionState {
    pub fn all_fields_filled(&self) -> Option<(&str, &str)> {
        match (self.username.as_deref(), self.password.as_deref()) {
            (Some(username), Some(password)) => Some((username, password)),
            _ => None,
        }
    }

    pub fn fields_filled(&self) -> bool {
        self.all_fields_filled().is_some()
    }
}
