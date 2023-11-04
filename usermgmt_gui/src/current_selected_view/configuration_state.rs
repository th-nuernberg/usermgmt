use usermgmt_lib::config::LoadedMgmtConfig;

use crate::{drawing::configuration::CacheForConfFiels, io_resource_manager::IoResourceManager};

#[derive(Debug, Default)]
pub struct ConfigurationState {
    pub gui_field_cache: CacheForConfFiels,
    pub io_conf: IoResourceManager<LoadedMgmtConfig>,
    pub io_save_conf: IoResourceManager,
}
