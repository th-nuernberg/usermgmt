use usermgmt_lib::prelude::AppError;

#[derive(Debug)]
pub enum IoTaskStatus {
    NotStarted,
    Loading,
    Successful,
    Failed(AppError),
}

impl PartialEq for IoTaskStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Failed(_), Self::Failed(_)) => true,
            (Self::Failed(_), _) | (_, Self::Failed(_)) => false,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Default for IoTaskStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}
