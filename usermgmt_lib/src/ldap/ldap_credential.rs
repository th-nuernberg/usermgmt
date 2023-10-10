pub trait LdapCredential {
    fn username(&self) -> &str;
    fn password(&self) -> &str;
}
