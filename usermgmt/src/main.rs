use clap::Parser;
use cli_ssh_credential::CliSshCredential;
use ldap_cli_credential::LdapCliCredential;
use log::error;
use std::process::ExitCode;
use usermgmt_lib::cli::{self, Commands, GeneralArgs, OnWhichSystem};
use usermgmt_lib::config::{self};
use usermgmt_lib::{operations, prelude::*, ChangesToUser, Entity};

mod cli_ssh_credential;
mod cli_user_input;
mod ldap_cli_credential;
mod user_input;
fn main() -> ExitCode {
    usermgmt_lib::app_panic_hook::set_app_panic_hook();
    // Logger handler in variable so background thread for file logging is not stopped until the
    // end of application.
    let _keep_logger_handler = usermgmt_lib::logging::set_up_logging(env!("CARGO_PKG_NAME"))
        .expect("Failed to initilize logger");

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
    let ldap_credential = LdapCliCredential::default();
    match args.command {
        Commands::GenerateConfig => {
            println!("{}", config::config_for_save())
        }
        Commands::Add {
            to_add,
            on_which_sys,
        } => {
            let config = config::load_config(None)?.config;
            let on_which_sys = &OnWhichSystem::from_config_for_all(&config, &on_which_sys);
            let cli_ssh_credential = CliSshCredential::new(&config, on_which_sys.ssh_path());
            operations::add_user(
                to_add,
                on_which_sys,
                &config,
                ldap_credential,
                cli_ssh_credential,
            )?
        }
        Commands::Modify { data, on_which_sys } => {
            let config = config::load_config(None)?.config;
            let on_which_sys = &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys);
            let cli_ssh_credential = CliSshCredential::new(&config, on_which_sys.ssh_path());
            let data = Entity::new_modifieble_conf(data, &config)?;
            let data = ChangesToUser::try_new(data)?;
            operations::modify_user(
                data,
                on_which_sys,
                &config,
                ldap_credential,
                cli_ssh_credential,
            )?
        }
        Commands::Delete { user, on_which_sys } => {
            let config = config::load_config(None)?.config;
            let on_which_sys = &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys);
            let cli_ssh_credential = CliSshCredential::new(&config, on_which_sys.ssh_path());
            operations::delete_user(
                user.as_ref(),
                on_which_sys,
                &config,
                ldap_credential,
                cli_ssh_credential,
            )?;
        }
        Commands::List {
            on_which_sys,
            simple_output_for_ldap,
        } => {
            let config = config::load_config(None)?.config;
            let on_which_sys = &OnWhichSystem::from_config_for_slurm_ldap(&config, &on_which_sys);
            let cli_ssh_credential = CliSshCredential::new(&config, on_which_sys.ssh_path());
            operations::list_users(
                &config,
                on_which_sys,
                simple_output_for_ldap.unwrap_or(false),
                ldap_credential,
                cli_ssh_credential,
            )?
        }
    };

    Ok(())
}
