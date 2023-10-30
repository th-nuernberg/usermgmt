use std::{any::Any, cell::RefCell, thread::JoinHandle};

use usermgmt_lib::prelude::AppError;

pub type AppTask<T> = Task<T, AppError>;
#[derive(Debug)]
pub struct Task<T, E> {
    job: RefCell<Option<JoinHandle<Result<T, E>>>>,
    on_panic_error: fn(Box<dyn Any + Send>) -> E,
}

impl<T, E> Task<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    pub fn new(on_panic_error: fn(Box<dyn Any + Send>) -> E) -> Self {
        Self {
            job: Default::default(),
            on_panic_error,
        }
    }

    pub fn spawn_task<F>(&self, to_perfrom: F)
    where
        F: FnOnce() -> Result<T, E> + Sync + Send + 'static,
    {
        let mut job = self.job.borrow_mut();
        if job.is_some() {
            return;
        }
        let spawned = std::thread::spawn(to_perfrom);
        *job = Some(spawned);
    }

    pub fn is_thread_running(&self) -> bool {
        self.job.borrow().is_some()
    }

    pub fn query_task(&self) -> Option<Result<T, E>> {
        let mut job = self.job.borrow_mut();
        if job.as_ref().map(|job| !job.is_finished()).unwrap_or(true) {
            None
        } else {
            let to_join = job.take().unwrap();
            match to_join.join() {
                Ok(to_return) => Some(to_return),
                Err(paniced) => Some(Err((self.on_panic_error)(paniced))),
            }
        }
    }
}
