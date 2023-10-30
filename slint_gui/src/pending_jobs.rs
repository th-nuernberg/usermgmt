use usermgmt_lib::prelude::{anyhow, AppError};

use crate::task::AppTask;

#[derive(Debug)]
pub struct PendingJobs {
    pub listing_ldap_users: AppTask<String>,
}

impl Default for PendingJobs {
    fn default() -> Self {
        Self {
            listing_ldap_users: AppTask::new(|error| anyhow!("Thread has panicked: {error:?}")),
        }
    }
}
