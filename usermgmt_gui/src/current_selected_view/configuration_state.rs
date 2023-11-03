use usermgmt_lib::config::LoadedMgmtConfig;

use crate::io_resource_manager::IoResourceManager;

#[derive(Debug, Default)]
pub struct ConfigurationState {
    pub io_conf: IoResourceManager<LoadedMgmtConfig>,
    pub io_save_conf: IoResourceManager,
}
