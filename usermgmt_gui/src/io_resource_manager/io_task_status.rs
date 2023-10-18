use usermgmt_lib::prelude::AppError;

#[derive(Debug)]
pub enum IoTaskStatus<T> {
    NotStarted,
    Loading,
    Successful(T),
    Failed(AppError),
}

impl<T> IoTaskStatus<T> {
    pub fn _is_loading(&self) -> bool {
        matches!(self, IoTaskStatus::Loading)
    }
    pub fn is_there(&self) -> bool {
        matches!(self, IoTaskStatus::Successful(_))
    }
}

impl<T> Default for IoTaskStatus<T> {
    fn default() -> Self {
        Self::NotStarted
    }
}
