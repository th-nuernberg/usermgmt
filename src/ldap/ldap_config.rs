use crate::{prelude::AppResult, MgmtConfig};

use super::ldap_paths::LdapPaths;
#[derive(Debug, Default)]
/// Contains all information for creating/deleting and modifying an user aka writting actions
/// TODO: consider implementing encapsulation with getters and setters
pub struct LDAPConfig {
    pub ldap_server: String,
    pub ldap_pass: String,
    pub ldap_paths: LdapPaths,
}

impl LDAPConfig {
    pub fn new(
        config: &MgmtConfig,
        username: &Option<String>,
        password: &Option<String>,
    ) -> AppResult<Self> {
        let (bind_prefix, ldap_server, dc, org_unit, bind_org_unit) = (
            &config.ldap_bind_prefix,
            &config.ldap_server,
            &config.ldap_domain_components,
            &config.ldap_org_unit,
            &config.ldap_bind_org_unit,
        );

        let (ldap_user, ldap_pass) = super::ask_credentials_if_not_provided(
            username.as_deref(),
            password.as_deref(),
            super::ask_credentials_in_tty,
        )?;

        let ldap_paths = LdapPaths::new(
            dc.clone(),
            org_unit.clone(),
            bind_org_unit.clone(),
            bind_prefix.clone(),
            ldap_user,
        );

        Ok(Self {
            ldap_server: ldap_server.to_string(),
            ldap_pass,
            ldap_paths,
        })
    }

    pub fn bind(&self) -> &str {
        self.ldap_paths.bind()
    }
    pub fn base(&self) -> &str {
        self.ldap_paths.base()
    }
    pub fn username(&self) -> &str {
        self.ldap_paths.username()
    }
}
