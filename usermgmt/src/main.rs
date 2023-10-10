use clap::Parser;
use env_logger::Env;
use log::error;
use std::process::ExitCode;
use usermgmt_lib::cli::{self, Commands, GeneralArgs, OnWhichSystem};
use usermgmt_lib::config::{self};
use usermgmt_lib::{prelude::*, Entity};

mod ldap_cli_credential;
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

/// Main function that handles user management
pub fn run_mgmt(args: cli::GeneralArgs) -> AppResult {
    match args.command {
        Commands::GenerateConfig => {
            println!("{}", config::config_for_save())
        }
        Commands::Add {
            to_add,
            on_which_sys,
        } => {
            let config = config::load_config()?;
            usermgmt_lib::add_user(
                to_add,
                &OnWhichSystem::from_config_for_all(&config, &on_which_sys),
                &config,
            )?
        }
        Commands::Modify { data, on_which_sys } => {
            let config = config::load_config()?;
            let data = Entity::new_modifieble_conf(data, &config)?;
            usermgmt_lib::modify_user(
                data,
                &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys),
                &config,
            )?
        }
        Commands::Delete { user, on_which_sys } => {
            let config = config::load_config()?;
            usermgmt_lib::delete_user(
                user.as_ref(),
                &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys),
                &config,
            )?;
        }
        Commands::List {
            on_which_sys,
            simple_output_for_ldap,
        } => {
            let config = config::load_config()?;
            usermgmt_lib::list_users(
                &config,
                &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys),
                simple_output_for_ldap.unwrap_or(false),
            )?
        }
    };

    Ok(())
}
