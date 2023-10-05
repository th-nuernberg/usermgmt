use crate::{config::MgmtConfig, prelude::AppResult};

use super::ldap_paths::LdapPaths;
use getset::Getters;
use log::info;

#[derive(Getters, Debug)]
#[getset(get = "pub")]
/// Used for action over connection with read only access
pub struct LdapReadonlyConfig {
    ldap_server: String,
    ldap_pass: String,
    ldap_paths: LdapPaths,
}

impl LdapReadonlyConfig {
    pub fn new(config: &MgmtConfig) -> AppResult<Self> {
        Self::create(config, super::ask_credentials_in_tty)
    }

    fn create(
        config: &MgmtConfig,
        on_credentials: impl FnOnce() -> AppResult<(String, String)>,
    ) -> AppResult<Self> {
        let ldap_server = config.ldap_server.clone();
        let (ldap_user, ldap_pass) = super::ask_credentials_if_not_provided(
            config.ldap_readonly_user.as_deref(),
            config.ldap_readonly_pw.as_deref(),
            on_credentials,
        )?;

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
            ldap_pass,
            ldap_server,
        })
    }

    pub fn bind(&self) -> &str {
        self.ldap_paths.bind()
    }
    pub fn base(&self) -> &str {
        self.ldap_paths.base()
    }
}
#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn fall_back_for_default() {
        let given = LdapReadonlyConfig::create(&MgmtConfig::default(), || {
            Ok(("user".to_owned(), "password".to_owned()))
        })
        .unwrap();
        assert_eq!("password", given.ldap_pass);
        assert_eq!("user", given.ldap_paths.username());
    }
    #[test]
    fn use_all_readonly_options() {
        const USER: &str = "readonly_user";
        const PASSWORD: &str = "readonly_password";
        const BIND: &str = "ou=people,ou=department";
        const PREFIX: &str = "zzz";
        let config = MgmtConfig {
            ldap_readonly_user: Some(USER.to_owned()),
            ldap_readonly_pw: Some(PASSWORD.to_owned()),
            ldap_readonly_bind: Some(BIND.to_owned()),
            ldap_domain_components: Some("dc=example,dc=com".to_owned()),
            ldap_readonly_user_prefix: Some(PREFIX.to_owned()),
            ..Default::default()
        };
        let given = LdapReadonlyConfig::create(&config, || {
            panic!("Should fall back for asking credentials")
        })
        .unwrap();
        assert_eq!(PASSWORD, given.ldap_pass);
        assert_eq!(USER, given.ldap_paths.username());
        let expected_bind = format!(
            "{}={},{},{}",
            PREFIX,
            USER,
            BIND,
            config.ldap_domain_components.unwrap()
        );
        assert_eq!(expected_bind, given.bind());
    }
}
