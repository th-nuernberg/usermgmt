use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use crate::{current_selected_view::ConfigurationState, prelude::*};
use usermgmt_lib::{
    cli::OnWhichSystem,
    config::{self, LoadedMgmtConfig, MgmtConfig},
    ldap::LdapSimpleCredential,
    prelude::{anyhow, AppResult},
    ssh::SshGivenCredential,
    util::TrimmedNonEmptyText,
};

use crate::io_resource_manager::{IoResourceManager, IoTaskStatus};

pub fn error_status<T>(msg: &str, error_details: T) -> String
where
    T: Display + Debug,
{
    format!("{}. Details: \n{:?}", msg, error_details)
}

pub fn some_if_not_blank_str(input: &str) -> Option<TrimmedNonEmptyText> {
    input.try_into().ok()
}
pub fn is_some_and_not_empty(input: Option<&str>) -> bool {
    input
        .map(|input| some_if_not_blank_str(input).is_some())
        .unwrap_or(false)
}
pub fn start_load_config(conf_state: &mut ConfigurationState, path: Option<PathBuf>) {
    conf_state.io_conf.spawn_task(
        || config::load_config(path),
        "Loading configuration".to_string(),
    );
}

pub struct PreparationBeforIoTask {
    pub ldap_cred: LdapSimpleCredential,
    pub ssh_cred: SshGivenCredential,
    pub config: MgmtConfig,
    pub on_which_sys: OnWhichSystem,
}

pub fn prep_conf_creds<T: Send + 'static>(
    app: &mut UsermgmtWindow,
    on_error: impl FnOnce(&mut UsermgmtWindow) -> &mut IoResourceManager<T>,
    supports_dir: bool,
) -> Result<PreparationBeforIoTask, &'static str> {
    return match try_prep(app, supports_dir) {
        Ok(result) => Ok(result),
        Err(error) => {
            on_error(app).set_error(error);
            Err(
                "Could fetch all needed credentials, config or which system is to be affectd.
Details of error are embeded within respective io resource state.",
            )
        }
    };

    fn try_prep(
        window: &mut UsermgmtWindow,
        supports_dir: bool,
    ) -> AppResult<PreparationBeforIoTask> {
        let which_sys = &window.which_sys;
        let text = window.settings.texts();
        let (ldap_cred_missing, ssh_cred_missing) = (
            text.ldap_cred_missing().clone(),
            text.ssh_cred_missing().clone(),
        );
        let ldap_cred = if which_sys.is_ldap_needed() {
            window
                .create_ldap_credentials()
                .ok_or_else(|| anyhow!(ldap_cred_missing))?
        } else {
            Default::default()
        };
        let on_which_sys = window.which_sys.create_on_which_system();
        if let IoTaskStatus::Successful(LoadedMgmtConfig { config, .. }) =
            &window.conf_state.io_conf.status()
        {
            let config = config.clone();
            let ssh_cred = if which_sys.is_ssh_cred_provided(window, &config, supports_dir) {
                window
                    .create_ssh_credentials()
                    .ok_or_else(|| anyhow!(ssh_cred_missing))?
            } else {
                Default::default()
            };
            return Ok(PreparationBeforIoTask {
                config,
                on_which_sys,
                ldap_cred,
                ssh_cred,
            });
        }
        unreachable!("At this point, there should be a successfully loaded configuration");
    }
}
