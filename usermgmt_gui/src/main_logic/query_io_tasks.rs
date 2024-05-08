use crate::prelude::*;

/// Checks every frame if some background IO Task as a Os thread is finished and applies its result
/// to the global state of this application.
pub fn query(window: &mut UsermgmtWindow) {
    if let Some(conf) = window.conf_state.io_conf.query_task() {
        let listing_state = &mut window.listin_state;
        let ssh_state = &mut window.ssh_state;
        let ldap_cred = &mut window.ldap_state;
        let path = &mut window.conf_path;
        path.clone_from(&conf.path);
        let config = &conf.config;
        if listing_state.rw_user_name.is_none() {
            if let Some(rw_user) = config.ldap_readonly_user.as_deref() {
                listing_state.rw_user_name = Some(rw_user.to_owned());
            }
        }
        if listing_state.rw_pw.is_none() {
            if let Some(rw_password) = config.ldap_readonly_pw.as_deref() {
                listing_state.rw_pw = Some(rw_password.to_owned());
            }
        }
        if ssh_state.username.is_none() && !config.default_ssh_user.is_empty() {
            debug!("GUI: Ssh user name taken from default ssh user in loaded config");
            ssh_state.username = Some(config.default_ssh_user.to_owned());
        }
        if ldap_cred.username.is_none() {
            if let Some(ldap_user_name) = config.ldap_default_user.as_deref() {
                debug!("GUI: ldap user name taken from default ldap user in loaded config");
                ldap_cred.username = Some(ldap_user_name.to_owned());
            }
        }
    }
    if let Some(path) = window.conf_state.io_save_conf.query_task() {
        window.conf_path = path.to_path_buf();
    }
    let _ = window.listin_state.list_ldap_res.query_task();
    let _ = window.listin_state.list_slurm_user_res.query_task();
    let _ = window.adding_state.adding_res_io.query_task();
    let _ = window.remove_state.remove_res_io.query_task();
    let _ = window.modify_state.res_io.query_task();

    #[cfg(debug_assertions)]
    {
        if let Some(new_settings) = window.settings_watcher.tick() {
            info!("Applied new settings from changed settings file.");
            window.settings = new_settings;
        }
    }
}
