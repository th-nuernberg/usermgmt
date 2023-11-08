use crate::{current_selected_view::SshConnectionState, prelude::*};
use usermgmt_lib::{cli::OnWhichSystem, config::MgmtConfig};

use crate::drawing::draw_utils;

#[derive(Debug)]
pub struct WhichSystem {
    pub ldap: bool,
    pub slurm: bool,
    pub dir: bool,
}

impl WhichSystem {
    pub fn create_on_which_system(&self) -> OnWhichSystem {
        let (slurm, ldap, dirs) = (self.ldap, self.slurm, self.dir);
        OnWhichSystem::new(slurm, ldap, dirs)
    }
    pub fn is_ssh_cred_needed(&self, supports_dir: bool) -> bool {
        let operates_on_dir = supports_dir && self.dir;
        operates_on_dir || self.slurm
    }
    pub fn is_ldap_needed(&self) -> bool {
        self.ldap
    }

    pub fn is_ssh_cred_provided(
        &self,
        app_state: &UsermgmtWindow,
        config: &MgmtConfig,
        supports_dir: bool,
    ) -> bool {
        let ssh_state = &app_state.ssh_state;
        return !self.is_ssh_cred_needed(supports_dir)
            || creds_ssh_agent(config, ssh_state)
            || simple_creds(ssh_state);

        fn creds_ssh_agent(config: &MgmtConfig, cred: &SshConnectionState) -> bool {
            config.ssh_agent && cred.username().is_some()
        }
        fn simple_creds(cred: &SshConnectionState) -> bool {
            cred.username().is_some() && cred.password().is_some()
        }
    }
}

pub fn draw_which_system(
    ui: &mut egui::Ui,
    settings: &Settings,
    state: &mut WhichSystem,
    supports_dir: bool,
) {
    let text = settings.texts();
    draw_utils::draw_box_group(ui, text.mode_main_title(), |ui| {
        ui.checkbox(&mut state.ldap, text.mode_ldap());
        ui.checkbox(&mut state.slurm, text.mode_slurm());
        if supports_dir {
            ui.checkbox(&mut state.dir, text.mode_directory());
        }
    });
}

impl Default for WhichSystem {
    fn default() -> Self {
        Self {
            ldap: true,
            slurm: true,
            dir: true,
        }
    }
}
