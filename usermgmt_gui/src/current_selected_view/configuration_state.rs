use usermgmt_lib::config::MgmtConfig;

use crate::io_resource_manager::IoResourceManager;

#[derive(Debug, Default)]
pub struct ConfigurationState {
    pub io_conf: IoResourceManager<MgmtConfig>,
}
