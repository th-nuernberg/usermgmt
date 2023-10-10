use std::thread::{self, spawn, JoinHandle};

use log::{error, info};
use usermgmt_lib::prelude::{anyhow, AppError, AppResult};

#[derive(Debug, Default)]
pub struct IoBackgroundWorker<T = ()> {
    thread: Option<JoinHandle<AppResult<T>>>,
}

impl<T> Drop for IoBackgroundWorker<T> {
    fn drop(&mut self) {
        if let Some(task) = self.thread.take() {
            match task.join() {
                Ok(Err(error)) => error!("Left over thread returned with error {:?}", error),
                Err(error_data) => error!("Thread paniced with error {:?}", error_data),
                _ => info!("Left over thread joined."),
            }
        }
    }
}

impl<T> IoBackgroundWorker<T>
where
    T: Send + 'static,
{
    fn spawn<F>(&mut self, task: F, thread_name: String) -> AppResult<bool>
    where
        F: Fn() -> AppResult<T> + Send + 'static,
    {
        if self.thread.is_none() {
            let new_thread = thread::Builder::new()
                .name(thread_name)
                .spawn(task)
                .map_err(|error| anyhow!("{:?}", error))?;
            self.thread = Some(new_thread);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn is_loading(&self) -> bool {
        self.thread.is_some()
    }

    fn get_task_result(&mut self) -> Option<AppResult<T>> {
        if self
            .thread
            .as_ref()
            .map(|to_bool| to_bool.is_finished())
            .unwrap_or(false)
        {
            match self.thread.take().unwrap().join() {
                Ok(result) => Some(result),
                Err(error) => Some(Err(anyhow!("Task paniced !, details: {:?}", error))),
            }
        } else {
            None
        }
    }
}
