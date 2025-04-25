//! Functions to acquire information from the user by asking for it in the terminal.

use std::io;

use usermgmt_lib::{
    prelude::{anyhow, AppResult, Context},
    util::TrimmedNonEmptyText,
};

fn ask_for_line_from_user(
    on_input: impl Fn() -> AppResult<Option<String>>,
    on_output: impl Fn(String),
    prompt: &str,
    placeholder: Option<&str>,
) -> AppResult<String> {
    let final_placeholder_prompt = match placeholder {
        Some(value) => format!(" (defaults to {})", value),
        None => String::new(),
    };

    let final_prompt = format!("{}{}", prompt, final_placeholder_prompt);
    on_output(final_prompt);
    let received_input = on_input()?;

    match received_input {
        Some(input) => Ok(input),
        None => match placeholder {
            Some(to_use_instead) => Ok(to_use_instead.to_owned()),
            None => Err(anyhow!("No entered input for the field".to_string())),
        },
    }
}

/// Ask the user for a line until new line is given over the terminal.
/// A prompt is shown to the user in the terminal.
/// If no input is given by the user, the placeholder is returned
///
/// # Returns
///
/// - None if input is empty or only white spaces
/// - Some if input has at least on char which is not white space. Inner value is trimmed or the
///  placeholder if no input is provided.
///
/// # Errors
///
/// - if reading from the terminal does not work. For example terminal is not accessible.
pub fn ask_for_line_from_user_over_term(
    prompt: &str,
    placeholder: Option<&str>,
) -> AppResult<String> {
    ask_for_line_from_user(
        line_input_from_user,
        |input| println!("{}", input),
        prompt,
        placeholder,
    )
}

fn trim_input(input: &str) -> Option<String> {
    TrimmedNonEmptyText::try_from(input)
        .ok()
        .map(|s| s.to_string())
}

/// Ask the user for an line until new line is given over the terminal.
///
/// # Returns
///
/// - None if input is empty or only white spaces
/// - Some if input has at least on char which is not white space. Inner value is trimmed.
///
/// # Errors
///
/// - if reading from the terminal does not work. For example terminal is not accessible.
pub fn line_input_from_user() -> AppResult<Option<String>> {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    Ok(trim_input(&input))
}

/// Ask the user for a password until new line is given  over the terminal.
///
/// # Returns
///
/// - None if input is empty or only white spaces
/// - Some if input has at least on char which is not white space. Inner value is trimmed.
///
/// # Errors
///
/// - if reading from the terminal does not work. For example terminal is not accessible.
pub fn cli_ask_for_password(prompt: &str) -> AppResult<Option<String>> {
    let password =
        rpassword::prompt_password(prompt).context("Could not retrieve password from prompt !")?;
    Ok(trim_input(&password))
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn take_placeholder_over_empty_input() {
        let placeholder = String::from("place holder ...");
        let prompt = "Some prompt";

        let actual = ask_for_line_from_user(
            || Ok(None),
            |output| {
                let expected = format!("{} (defaults to {})", prompt, placeholder);
                assert_eq!(expected, output);
            },
            "Some prompt",
            Some(&placeholder),
        );

        assert_eq!(placeholder, actual.unwrap());
    }
    #[test]
    fn take_input_over_placeholder() {
        let placeholder = String::from("place holder ...");
        let prompt = "Some prompt";

        let input = "Input !".to_owned();
        let actual = ask_for_line_from_user(
            || Ok(Some(input.clone())),
            |output| {
                let expected = format!("{} (defaults to {})", prompt, placeholder);
                assert_eq!(expected, output);
            },
            "Some prompt",
            Some(&placeholder),
        );

        assert_eq!(input, actual.unwrap());
    }
    #[test]
    fn prompt_without_default_and_input_was_given() {
        let prompt = "Some prompt";

        let input = "Input!".to_owned();
        let actual = ask_for_line_from_user(
            || Ok(Some(input.clone())),
            |output| {
                let expected = prompt.to_string();
                assert_eq!(expected, output);
            },
            "Some prompt",
            None,
        );

        assert_eq!(input, actual.unwrap());
    }

    #[test]
    fn no_input_given() {
        let prompt = "Some prompt";

        let input = None;
        let actual = ask_for_line_from_user(
            || Ok(input.clone()),
            |output| {
                let expected = prompt.to_string();
                assert_eq!(expected, output);
            },
            "Some prompt",
            None,
        );
        assert!(actual.is_err());
    }
}
