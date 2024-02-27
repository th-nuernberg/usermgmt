use crate::{AppError, AppResult};
use anyhow::anyhow;
use ldap3::LdapConn;

use crate::config::MgmtConfig;
use crate::ldap;

use super::{LDAPConfig, LdapCredential};

/// API of the ldap crate requires us to mutate the connection
/// to establish a session.
/// We can not work with once_cell here because of it.
type MutableLdapConnection = Option<AppResult<LdapConn>>;

pub struct LdapSession<T> {
    config: LDAPConfig<T>,
    connection: MutableLdapConnection,
}

impl<T> LdapSession<T>
where
    T: LdapCredential,
{
    pub fn new(config: &MgmtConfig, credentials: T) -> AppResult<Self> {
        let config = LDAPConfig::new(config, credentials)?;
        let connection = None;
        Ok(Self { config, connection })
    }
    pub fn from_ldap_readonly_config(config: &MgmtConfig, credentials: T) -> AppResult<Self> {
        let config = LDAPConfig::new_readonly(config, credentials)?;
        let connection = None;
        Ok(Self { config, connection })
    }

    pub fn config(&self) -> &LDAPConfig<T> {
        &self.config
    }

    pub fn action<RT>(
        &mut self,
        action: impl FnOnce(&mut LdapConn, &LDAPConfig<T>) -> AppResult<RT>,
    ) -> Result<RT, AppError> {
        self.establish_connection()?;
        let config = &self.config;
        let connection = {
            let is_there = self
                .connection
                .as_mut()
                .expect("Is Some because of establishing connection");
            is_there
                .as_mut()
                .expect("Is ok because of establishing connection")
        };
        action(connection, config)
    }

    pub fn establish_connection(&mut self) -> AppResult {
        let _ = self
            .connection
            .get_or_insert_with(|| ldap::make_ldap_connection(&self.config))
            .as_mut()
            .map_err(|error| anyhow!("{}", error))?;
        Ok(())
    }
}
