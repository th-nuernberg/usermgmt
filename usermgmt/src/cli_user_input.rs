use usermgmt_lib::prelude::{anyhow, AppResult, Context};

use crate::user_input;

pub fn ask_cli_username() -> AppResult<String> {
    println!("Enter your LDAP username (defaults to admin):");
    let username = user_input::line_input_from_user()?.unwrap_or_else(|| "admin".to_string());
    Ok(username)
}

pub fn ask_cli_password() -> AppResult<String> {
    let password = user_input::cli_ask_for_password("Enter your LDAP password: ")
        .context("Failed to retrieve password from user in a terminal")?
        .ok_or_else(|| anyhow!("No password provided"))?;
    Ok(password)
}
