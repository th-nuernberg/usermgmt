use crate::prelude::*;

pub fn query(window: &mut UsermgmtWindow) {
    if let Some(conf) = window.conf_state.io_conf.query_task() {
        let listing_state = &mut window.listin_state;
        let ssh_state = &mut window.ssh_state;
        let path = &mut window.conf_path;
        *path = conf.path.to_owned();
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
    }
    let _ = window.conf_state.io_save_conf.query_task();
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
