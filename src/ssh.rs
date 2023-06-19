use crate::prelude::AppResult;
use log::debug;
mod ssh_connection;
mod ssh_credential;

pub use ssh_connection::SshSession;

pub use ssh_credential::SshCredential;

/// Executes given command `cmd` on remote machine over ssh
///
/// TODO: move checking if for exit code inside here and return result instead of error code
/// directly
pub fn run_remote_command(sess: &SshSession, cmd: &str) -> AppResult<i32> {
    debug!("Running command {}", cmd);

    let (s, exit_status) = sess.exec(cmd)?;

    debug!("command output: {}", s);
    debug!("command exit status: {}", exit_status);
    Ok(exit_status)
}
