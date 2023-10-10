use usermgmt_lib::config::MgmtConfig;

use crate::{io_background_worker::IoBackgroundWorker, io_task_status::IoTaskStatus};

#[derive(Debug, Default)]
pub struct ConfigurationState {
    pub conf: Option<MgmtConfig>,
    pub io_load_conf: IoBackgroundWorker<MgmtConfig>,
    pub io_status_conf: IoTaskStatus,
}
