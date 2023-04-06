use clap::Parser;
use env_logger::Env;
use log::error;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use usermgmt::cli::GeneralArgs;
use usermgmt::config::config::MgmtConfig;
use usermgmt::prelude::*;
use usermgmt::run_mgmt;

/// Name of the file in which all values for configuration of this app are located
/// besides the CLI arguments.
const NAME_CONFIG_FILE: &str = "conf.toml";

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let config_file_basedir = if cfg!(target_os = "linux") && cfg!(not(debug_assertions)) {
        Path::new("/etc/usermgmt")
    } else {
        Path::new(".")
    };

    if let Err(error) = execute_command(config_file_basedir) {
        error!("Error: {:?}", error);
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}

/// Executes action adding/deleting/changing user with arguments from CLI and values from
/// configuration file
fn execute_command(config_file_basedir: &Path) -> AppResult {
    let config = load_config(config_file_basedir).context("Configuration error")?;
    let args = GeneralArgs::parse();
    run_mgmt(args, config)
}

/// Tries to load  config.toml for application.
///
/// # Error
///
/// - Can not ensure if folder exits where conf.toml file exits
/// - Can not read or create a configuration file
fn load_config(config_file_basedir: &Path) -> AppResult<MgmtConfig> {
    fs::create_dir_all(config_file_basedir)
        .with_context(|| format!("Could not create folder for {:?}", config_file_basedir))?;
    let path = config_file_basedir.join(NAME_CONFIG_FILE);

    // Load (or create if nonexistent) configuration file conf.toml
    confy::load_path(&path)
        .with_context(|| format!("Error in loading or creating config file at {:?}", path))
}
