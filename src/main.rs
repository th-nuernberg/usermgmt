use clap::Parser;
use env_logger::Env;
use log::error;
use std::process::ExitCode;
use usermgmt::cli::GeneralArgs;
use usermgmt::prelude::*;
use usermgmt::run_mgmt;

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    if let Err(error) = execute_command() {
        error!("Error: {:?}", error);
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}

/// Executes action adding/deleting/changing user with arguments from CLI and values from
/// configuration file
fn execute_command() -> AppResult {
    let args = GeneralArgs::parse();
    run_mgmt(args)
}
