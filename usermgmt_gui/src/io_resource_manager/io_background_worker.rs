use std::thread::{self, JoinHandle};

use log::{error, info, warn};
use usermgmt_lib::prelude::{anyhow, AppResult};

#[derive(Debug, Default)]
pub struct IoBackgroundWorker<T = ()>
where
    T: Send + 'static,
{
    thread: Option<JoinHandle<AppResult<T>>>,
    thread_name: String,
}

impl<T> Drop for IoBackgroundWorker<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        if let Some(task) = self.thread.take() {
            let thread_name = &self.thread_name;
            match task.join() {
                Ok(Err(error)) => error!(
                    "Left over thread ({}) returned with error {:?}",
                    thread_name, error
                ),
                Err(error_data) => error!(
                    "Left over Thread ({}) panicked with error {:?}",
                    thread_name, error_data
                ),
                _ => {
                    warn!("Left over thread ({}) joined.", thread_name)
                }
            }
        }
    }
}

impl<T> IoBackgroundWorker<T>
where
    T: Send + 'static,
{
    pub fn get_thread_name(&self) -> &str {
        &self.thread_name
    }
    pub fn spawn<F>(&mut self, task: F, thread_name: String) -> AppResult<bool>
    where
        F: FnOnce() -> AppResult<T> + Send + 'static,
    {
        if self.thread.is_none() {
            let new_thread = thread::Builder::new()
                .name(thread_name.clone())
                .spawn(task)
                .map_err(|error| {
                    error!("Failed to spawn thread with name: {}", thread_name);
                    anyhow!("{:?}", error)
                })?;
            self.thread = Some(new_thread);
            info!("Started background task in thread ({})", thread_name);
            self.thread_name = thread_name;
            Ok(true)
        } else {
            warn!("Start of thread ({}) was rejected because a thread for this responsility is still running", thread_name);
            Ok(false)
        }
    }

    pub fn _is_loading(&self) -> bool {
        self.thread.is_some()
    }

    pub fn get_task_result(&mut self) -> Option<AppResult<T>> {
        if self
            .thread
            .as_ref()
            .map(|to_bool| to_bool.is_finished())
            .unwrap_or(false)
        {
            match self.thread.take().unwrap().join() {
                Ok(result) => Some(result),
                Err(error) => Some(Err(anyhow!("Task panicked !, details: {:?}", error))),
            }
        } else {
            None
        }
    }
}
