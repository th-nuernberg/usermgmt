#[derive(Debug, Default)]
pub struct LdapConnectionState {
    pub username: Option<String>,
    pub password: Option<String>,
}
