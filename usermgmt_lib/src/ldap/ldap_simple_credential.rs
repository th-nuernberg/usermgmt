use super::LdapCredential;

#[derive(Debug)]
pub struct LdapSimpleCredential {
    username: String,
    password: String,
}

impl LdapCredential for LdapSimpleCredential {
    fn username(&self) -> &str {
        &self.username
    }

    fn password(&self) -> &str {
        &self.password
    }
}
