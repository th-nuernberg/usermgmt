use anyhow::anyhow;
use std::{io, process::Output};
use thiserror::Error;

/// Error which is used in application. It contains a backstrace, an error message and an exit
/// code. Most of the time it is used to report information to the user of this
/// application in case of an error.
#[derive(Debug, Error)]
#[error("Error: {error:#?}.\nExit code: {exit_code}")]
pub struct AppError {
    #[source]
    error: anyhow::Error,
    exit_code: i32,
}

impl AppError {
    pub fn new_with_exit_code(error: anyhow::Error, exit_code: i32) -> Self {
        Self { error, exit_code }
    }

    /// Defaults to 1 for some error.
    /// May return own defined exit code.
    /// If the cause of this error is an io error, then it returns the os exit code of the io error.
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        match value.downcast_ref::<io::Error>() {
            // Make sure that exit code is sync with os error.
            Some(io_error) => Self {
                exit_code: io_error.raw_os_error().unwrap_or(1),
                error: value,
            },
            None => Self {
                error: value,
                exit_code: 1,
            },
        }
    }
}

impl From<Output> for AppError {
    fn from(value: Output) -> Self {
        let out = String::from_utf8_lossy(&value.stdout);
        let err = String::from_utf8_lossy(&value.stderr);

        return AppError::new_with_exit_code(
            anyhow!("stderr: {}\nstdout: {}", err, out),
            value.status.code().unwrap_or(1),
        );
    }
}
