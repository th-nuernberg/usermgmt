use crate::prelude::*;

#[derive(Default, Debug)]
pub struct ModifyState {
    pub username: String,
    pub lastname: String,
    pub firstname: String,
    pub group: String,
    pub mail: String,
    pub default_qos: String,
    pub publickey: String,
    pub qos: Vec<String>,
    pub res_io: IoResourceManager,
    pub last_added_username: String,
}
