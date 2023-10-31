use usermgmt_lib::{config::LoadedMgmtConfig, ldap::LdapSearchResult, prelude::anyhow};

use crate::task::AppTask;

#[derive(Debug)]
pub struct PendingJobs {
    pub listing_ldap_users: AppTask<LdapSearchResult>,
    pub load_config: AppTask<LoadedMgmtConfig>,
}

impl Default for PendingJobs {
    fn default() -> Self {
        let on_panic = |error| anyhow!("Thread has panicked: {error:?}");
        Self {
            listing_ldap_users: AppTask::new(on_panic),
            load_config: AppTask::new(on_panic),
        }
    }
}
