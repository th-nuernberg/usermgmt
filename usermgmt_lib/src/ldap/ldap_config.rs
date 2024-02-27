use log::info;

use crate::{prelude::AppResult, MgmtConfig};

use super::{ldap_paths::LdapPaths, LdapCredential};
#[derive(Debug, Default)]
/// Contains all information for creating/deleting and modifying an user aka writting actions
pub struct LDAPConfig<T> {
    ldap_server: String,
    ldap_credentails: T,
    ldap_paths: LdapPaths,
}

impl<T> LDAPConfig<T>
where
    T: LdapCredential,
{
    pub fn new_readonly(config: &MgmtConfig, mut credentials: T) -> AppResult<Self> {
        let ldap_server = config.ldap_server.clone();
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
        let _trigger_password_resolvement = credentials.password()?;

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

    pub fn ldap_server(&self) -> &str {
        &self.ldap_server
    }
}

#[cfg(test)]
mod testing {
    use crate::ldap::ldap_simple_credential::LdapSimpleCredential;

    use super::*;

    #[test]
    fn takes_all_from_readonly_conf() {
        let config = MgmtConfig {
            ldap_readonly_pw: Some("Password".to_string()),
            ldap_readonly_user: Some("User".to_string()),
            ..Default::default()
        };

        let ldap_config = LDAPConfig::new_readonly(
            &config,
            LdapSimpleCredential::new(String::from("Password"), String::from("User")),
        )
        .unwrap();
        assert_eq!(
            ("User", "Password"),
            (ldap_config.username(), ldap_config.password().unwrap())
        );
    }

    #[test]
    fn takes_username_from_readonly_conf() {
        const EXPECTED_PASSWORD: &str = "Password user provided";
        const EXPECTED_USERNAME: &str = "User from user provided";
        let config = MgmtConfig {
            ldap_readonly_user: Some(EXPECTED_USERNAME.to_string()),
            ..Default::default()
        };

        let ldap_config = LDAPConfig::new_readonly(
            &config,
            LdapSimpleCredential::new(
                String::from(EXPECTED_USERNAME),
                String::from(EXPECTED_PASSWORD),
            ),
        )
        .unwrap();
        assert_eq!(
            (EXPECTED_USERNAME, EXPECTED_PASSWORD),
            (ldap_config.username(), ldap_config.password().unwrap())
        );
    }

    #[test]
    fn takes_password_from_readonly_conf() {
        const EXPECTED_USERNAME: &str = "Username from user provided";
        const EXPECTED_PASSWORD: &str = "Password from config";
        let config = MgmtConfig {
            ldap_readonly_pw: Some(EXPECTED_PASSWORD.to_string()),
            ..Default::default()
        };

        let ldap_config = LDAPConfig::new_readonly(
            &config,
            LdapSimpleCredential::new(
                String::from(EXPECTED_USERNAME),
                String::from("Password from user provided"),
            ),
        )
        .unwrap();
        assert_eq!(
            (EXPECTED_USERNAME, EXPECTED_PASSWORD),
            (ldap_config.username(), ldap_config.password().unwrap())
        );
    }
}
