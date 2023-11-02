use eframe::egui;
use usermgmt_lib::cli::OnWhichSystem;

use crate::draw_selected_view::util;

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
}

pub fn draw_which_system(ui: &mut egui::Ui, state: &mut WhichSystem, supports_dir: bool) {
    util::draw_box_group(ui, "On which system", |ui| {
        ui.checkbox(&mut state.ldap, "LDAP");
        ui.checkbox(&mut state.slurm, "Slurm");
        if supports_dir {
            ui.checkbox(&mut state.dir, "Directory");
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
