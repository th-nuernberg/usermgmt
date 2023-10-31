use std::{cell::RefCell, rc::Rc};

use usermgmt_lib::{config::LoadedMgmtConfig, prelude::*};

pub type MayBeResource<T> = Option<AppResult<T>>;
pub type UnsyncSharedResources = Rc<RefCell<IoResources>>;
#[derive(Debug, Default)]
pub struct IoResources {
    pub configuration: MayBeResource<LoadedMgmtConfig>,
}
