use usermgmt_lib::prelude::AppResult;

use self::io_background_worker::IoBackgroundWorker;

mod io_background_worker;
mod io_task_status;

pub use io_task_status::IoTaskStatus;

#[derive(Default, Debug)]
pub struct IoResourceManager<T = ()> {
    status: IoTaskStatus,
    task: IoBackgroundWorker<T>,
}

impl<T> IoResourceManager<T>
where
    T: Send + 'static,
{
    pub fn status(&self) -> &IoTaskStatus {
        &self.status
    }

    pub fn spawn_task<F>(&mut self, task: F, thread_name: String) -> bool
    where
        F: Fn() -> AppResult<T> + Send + 'static,
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

    pub fn query_task(&mut self) -> Option<T> {
        if let Some(result) = self.task.get_task_result() {
            match result {
                Ok(to_return) => {
                    self.status = IoTaskStatus::Successful;
                    Some(to_return)
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
}
