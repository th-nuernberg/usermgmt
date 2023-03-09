use crate::util::io_util;
mod ssh_credential;
pub use ssh_credential::SshCredential;

fn ask_credentials(default_user: &str) -> (String, String) {
    println!("Enter your SSH username (defaults to {}):", default_user);
    let mut username = io_util::user_input();
    if username.is_empty() {
        username = default_user.to_string();
    }
    let password = rpassword::prompt_password("Enter your SSH password: ").unwrap();
    (username, password)
}
