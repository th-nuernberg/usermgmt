use crate::io_resource_manager::IoResourceManager;

#[derive(Debug, Default)]
pub struct RemoveState {
    pub username: String,
    pub remove_res_io: IoResourceManager<String>,
    pub last_username: String,
}
