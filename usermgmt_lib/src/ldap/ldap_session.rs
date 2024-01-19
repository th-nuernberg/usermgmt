use crate::{AppError, AppResult};
use ldap3::LdapConn;

use crate::config::MgmtConfig;
use crate::ldap;

use super::{LDAPConfig, LdapCredential};

pub struct LdapSession<T> {
    config: LDAPConfig<T>,
    connection: Option<AppResult<LdapConn>>,
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

    pub fn config(&self) -> &LDAPConfig<T> {
        &self.config
    }

    pub fn action<RT>(
        &mut self,
        action: impl FnOnce(&mut LdapConn, &LDAPConfig<T>) -> AppResult<RT>,
    ) -> Result<RT, AppError> {
        self.establish_connection()?;
        let connection = self.connection.as_mut().unwrap().as_mut().unwrap();
        action(connection, &self.config)
    }

    pub fn establish_connection(&mut self) -> Result<(), AppError> {
        match self.connection.as_mut() {
            Some(Ok(_)) => Ok(()),
            Some(Err(error)) => Err(anyhow::format_err!(
                "Establishing connection failed\n{:?}",
                error
            )),
            None => {
                let connection = ldap::make_ldap_connection(&self.config)?;
                self.connection = Some(Ok(connection));
                Ok(())
            }
        }
    }
}
