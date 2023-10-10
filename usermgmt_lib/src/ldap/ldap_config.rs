use log::info;

use crate::{prelude::AppResult, MgmtConfig};

use super::{ldap_paths::LdapPaths, LdapCredential};
#[derive(Debug, Default)]
/// Contains all information for creating/deleting and modifying an user aka writting actions
/// TODO: consider implementing encapsulation with getters and setters
pub struct LDAPConfig<T> {
    pub ldap_server: String,
    ldap_credentails: T,
    pub ldap_paths: LdapPaths,
}

impl<T> LDAPConfig<T>
where
    T: LdapCredential,
{
    pub fn new_readonly(config: &MgmtConfig, mut credentials: T) -> AppResult<Self> {
        let ldap_server = config.ldap_server.clone();
        dbg!("aaaa");
        let (ldap_user, ldap_pass) = super::ask_credentials_if_not_provided(
            config.ldap_readonly_user.as_deref(),
            config.ldap_readonly_pw.as_deref(),
            &credentials,
        )?;
        credentials.set_password(ldap_pass);

        let (bind, prefix) = (
            config.ldap_readonly_bind.clone().or_else(|| {
                info!(
                    "No org bind for readonly user provided, falling back to normal user bind org unit."
                );
                config.ldap_bind_org_unit.clone()
            }),
            config.ldap_readonly_user_prefix.clone().or_else(|| {
                info!("No prefix for readonly user provided, falling back to normal user prefix.");
                config.ldap_bind_prefix.clone()
            }),
        );
        let ldap_paths = LdapPaths::new(
            config.ldap_domain_components.clone(),
            config.ldap_org_unit.clone(),
            bind,
            prefix,
            ldap_user,
        );

        Ok(Self {
            ldap_paths,
            ldap_credentails: credentials,
            ldap_server,
        })
    }
    pub fn new(config: &MgmtConfig, credentials: T) -> AppResult<Self> {
        let (bind_prefix, ldap_server, dc, org_unit, bind_org_unit) = (
            &config.ldap_bind_prefix,
            &config.ldap_server,
            &config.ldap_domain_components,
            &config.ldap_org_unit,
            &config.ldap_bind_org_unit,
        );

        let ldap_user = credentials.username()?;

        let ldap_paths = LdapPaths::new(
            dc.clone(),
            org_unit.clone(),
            bind_org_unit.clone(),
            bind_prefix.clone(),
            ldap_user.to_string(),
        );

        Ok(Self {
            ldap_server: ldap_server.to_string(),
            ldap_credentails: credentials,
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
    pub fn password(&self) -> AppResult<&str> {
        self.ldap_credentails.password()
    }
}
