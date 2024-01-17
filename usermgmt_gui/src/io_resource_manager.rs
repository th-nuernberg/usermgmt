use log::{error, warn};
use usermgmt_lib::prelude::{AppError, AppResult};

use self::io_background_worker::IoBackgroundWorker;

mod io_background_worker;
mod io_task_status;

pub use io_task_status::IoTaskStatus;

#[derive(Default, Debug)]
/// This contains concept of an IO background task.
/// Via methods [`status`] and [`status_mut`] one can query if a task is running, has failed of
/// succeeded.
/// The method [`spawn_task`] allows to initiate an IO background task.
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
    /// Returns if an IO task is running, has failed or succeeded.
    pub fn status(&self) -> &IoTaskStatus<T> {
        &self.status
    }
    pub fn is_loading(&self) -> bool {
        self.status.is_loading()
    }
    pub fn is_there(&self) -> bool {
        self.status.is_there()
    }
    pub fn status_mut(&mut self) -> &mut IoTaskStatus<T> {
        &mut self.status
    }
    pub fn set_error(&mut self, error: AppError) {
        if self.is_loading() {
            warn!("Tried to set failure for loading io resource. Failure is not set because the resource is still loading.");
            return;
        }
        self.status = IoTaskStatus::Failed(error);
    }
    /// Allows to set a successful io task in the GUI rendering without spawning an actual OS
    /// Thread. There are cases where we can set the result immediately.
    pub fn set_success(&mut self, success: T) {
        if self.is_loading() {
            warn!("Tried to set success for loading io resource. Failure is not set because the resource is still loading.");
            return;
        }
        self.status = IoTaskStatus::Successful(success);
    }

    /// # Parameters
    /// - [`task`]: Closure which will be completed once the IO task finished.
    /// - [`thread_name`]: Name of thread used for IO background task. Useful for logging and
    /// debugging.
    ///
    /// ## Returns
    /// - True: if a new task has spawned.
    /// - False: if a task is already running or spawning a new task has failed for other
    /// reasons.
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

    /// # Returns
    /// - Some if an IO task finished this frame. Next call will then return None.
    /// - None if there is no IO task which finished during this frame.
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
                    error!("Error: {}", error);
                    self.status = IoTaskStatus::Failed(error);
                    None
                }
            }
        } else {
            None
        }
    }
    /// Same as method [`query_task`] expect it takes the result out of the
    /// manager for this task. If a resource is taken out of a manager then
    /// the task counts as uninitialized after.
    ///
    /// # Returns
    /// Same as method [`query_task`]
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
