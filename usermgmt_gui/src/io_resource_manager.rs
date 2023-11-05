use log::{error, warn};
use usermgmt_lib::prelude::{AppError, AppResult};

use self::io_background_worker::IoBackgroundWorker;

mod io_background_worker;
mod io_task_status;

pub use io_task_status::IoTaskStatus;

#[derive(Default, Debug)]
pub struct IoResourceManager<T = ()>
where
    T: Send + 'static,
{
    status: IoTaskStatus<T>,
    task: IoBackgroundWorker<T>,
}

impl<T> IoResourceManager<T>
where
    T: Send + 'static,
{
    pub fn set_error(&mut self, error: AppError) {
        if self.is_loading() {
            warn!("Tried to set failure for loading io resource. Failure is not set because the resource is still loading.");
            return;
        }
        self.status = IoTaskStatus::Failed(error);
    }
    pub fn status_mut(&mut self) -> &mut IoTaskStatus<T> {
        &mut self.status
    }

    pub fn is_loading(&self) -> bool {
        self.status.is_loading()
    }
    pub fn is_there(&self) -> bool {
        self.status.is_there()
    }
}
impl<T> IoResourceManager<T>
where
    T: Send + 'static,
{
    pub fn status(&self) -> &IoTaskStatus<T> {
        &self.status
    }
    pub fn spawn_task<F>(&mut self, task: F, thread_name: String) -> bool
    where
        F: FnOnce() -> AppResult<T> + Send + 'static,
    {
        let did_spawn = self.task.spawn(task, thread_name);
        match did_spawn {
            Ok(did_spawn) => {
                self.status = IoTaskStatus::Loading;
                did_spawn
            }
            Err(error) => {
                self.status = IoTaskStatus::Failed(error);
                false
            }
        }
    }

    pub fn query_task(&mut self) -> Option<&T> {
        if let Some(result) = self.task.get_task_result() {
            match result {
                Ok(to_return) => {
                    self.status = IoTaskStatus::Successful(to_return);
                    if let IoTaskStatus::Successful(to_return) = &self.status {
                        return Some(to_return);
                    }
                    unreachable!()
                }
                Err(error) => {
                    self.status = IoTaskStatus::Failed(error);
                    None
                }
            }
        } else {
            None
        }
    }
    pub fn query_task_and_take(&mut self) -> Option<T> {
        if let Some(result) = self.task.get_task_result() {
            match result {
                Ok(to_return) => {
                    self.status = IoTaskStatus::Successful(to_return);
                    let taken = std::mem::take(&mut self.status);
                    if let IoTaskStatus::Successful(to_return) = taken {
                        return Some(to_return);
                    }
                    unreachable!()
                }
                Err(error) => {
                    error!(
                        "Task ({}) failed with error: {:?}",
                        self.task.get_thread_name(),
                        &error
                    );
                    self.status = IoTaskStatus::Failed(error);
                    None
                }
            }
        } else {
            None
        }
    }
}
