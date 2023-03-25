use crate::util::io_util;
use log::debug;
mod ssh_connection;
mod ssh_credential;

pub use ssh_connection::SshSession;

pub use ssh_credential::SshCredential;

/// Asks user over the terminal for a username and password which are meant to be used for the
/// authentication in a ssh connection
fn ask_credentials_for_ssh(default_user: &str) -> (String, String) {
    println!("Enter your SSH username (defaults to {}):", default_user);

    let mut username = io_util::user_input();
    if username.is_empty() {
        username = default_user.to_string();
    }
    let password = rpassword::prompt_password("Enter your SSH password: ").unwrap();
    (username, password)
}
/// Executes given command `cmd` on remote machine over ssh
///
/// TODO: move checking if for exit code inside here and return result instead of error code
/// directly
pub fn run_remote_command(sess: &SshSession, cmd: &str) -> i32 {
    debug!("Running command {}", cmd);

    let (s, exit_status) = sess.exec(cmd);

    debug!("command output: {}", s);
    debug!("command exit status: {}", exit_status);
    exit_status
}
