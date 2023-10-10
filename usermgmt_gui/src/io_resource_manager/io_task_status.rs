use usermgmt_lib::prelude::AppError;

#[derive(Debug)]
pub enum IoTaskStatus {
    NotStarted,
    Loading,
    Successful,
    Failed(AppError),
}

impl Default for IoTaskStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}
