use usermgmt_lib::{
    cli::OnWhichSystem,
    config::{LoadedMgmtConfig, MgmtConfig},
    ldap::LdapSimpleCredential,
    prelude::{anyhow, AppResult},
    ssh::SshGivenCredential,
};

use crate::{
    io_resource_manager::{IoResourceManager, IoTaskStatus},
    usermgmt_window::UsermgmtWindow,
};

pub struct PreparationBeforIoTask {
    pub ldap_cred: LdapSimpleCredential,
    pub ssh_cred: SshGivenCredential,
    pub config: MgmtConfig,
    pub on_which_sys: OnWhichSystem,
}

pub fn prep_conf_creds<T>(
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
        let ldap_cred = if which_sys.is_ldap_needed() {
            window
                .create_ldap_credentials()
                .ok_or_else(|| anyhow!("LDAP credentials are missing."))?
        } else {
            Default::default()
        };
        let ssh_cred = if which_sys.is_ssh_cred_needed(supports_dir) {
            window
                .create_ssh_credentials()
                .ok_or_else(|| anyhow!("Ssh credentials are missing."))?
        } else {
            Default::default()
        };
        let on_which_sys = window.which_sys.create_on_which_system();
        if let IoTaskStatus::Successful(LoadedMgmtConfig { config, .. }) =
            &window.conf_state.io_conf.status()
        {
            let config = config.clone();
            return Ok(PreparationBeforIoTask {
                config,
                on_which_sys,
                ldap_cred,
                ssh_cred,
            });
        }
        unreachable!();
    }
}
