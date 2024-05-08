use usermgmt_lib::prelude::{anyhow, AppResult, Context};

use crate::user_input;

pub fn ask_cli_username(default_username: Option<&str>) -> AppResult<String> {
    let default_prompt_name = default_username.unwrap_or("admin");
    println!(
        "Enter your LDAP username (defaults to {}):",
        default_prompt_name
    );
    let username =
        user_input::line_input_from_user()?.unwrap_or_else(|| default_prompt_name.to_string());
    Ok(username)
}

pub fn ask_cli_password() -> AppResult<String> {
    let password = user_input::cli_ask_for_password("Enter your LDAP password: ")
        .context("Failed to retrieve password from user in a terminal")?
        .ok_or_else(|| anyhow!("No password provided"))?;
    Ok(password)
}
