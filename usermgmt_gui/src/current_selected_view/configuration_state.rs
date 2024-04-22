use std::path::PathBuf;

use usermgmt_lib::config::LoadedMgmtConfig;

use crate::{drawing::configuration::CacheForConfFields, io_resource_manager::IoResourceManager};

#[derive(Debug, Default)]
pub struct ConfigurationState {
    pub gui_field_cache: CacheForConfFields,
    pub io_conf: IoResourceManager<LoadedMgmtConfig>,
    pub io_save_conf: IoResourceManager<PathBuf>,
}
