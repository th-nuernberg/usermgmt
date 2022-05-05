use std::fs;
use std::path::Path;
use clap::StructOpt;
use env_logger::Env;
use log::error;
use usermgmt::config::config::MgmtConfig;
use usermgmt::run_mgmt;
use usermgmt::cli::cli::Args;
use usermgmt::util::io_util::make_ldif_template;
use std::error::Error;
extern crate confy;

fn main() {

    env_logger::Builder::from_env(Env::default()
        .default_filter_or("info"))
        .format_timestamp(None).init();

    #[cfg(debug_assertions)]
    let config_file_basedir: String = ".".to_owned();

    #[cfg(not(debug_assertions))]
    let config_file_basedir = "/etc/usermgmt".to_owned();

    fs::create_dir_all(&config_file_basedir).unwrap();
    let path_string = config_file_basedir + "/conf.toml";
    let path = Path::new(&path_string);

    // Try to create the template.ldif file required to execute LDAP commands
    let maybe_ldif_template_path : Result<String, Box<dyn Error>> = make_ldif_template();
    let mut ldif_template_path: String = "".to_string();
    match maybe_ldif_template_path {
        Ok(p) => ldif_template_path = p,
        Err(e) => error!("Configuration error: {:?}", e),
    }

    // Load or create the main configuration file conf.toml
    let cfg : Result<MgmtConfig, confy::ConfyError> = confy::load_path(path);
    match cfg {
        Ok(mut config) => {
            let args = Args::parse();
            config.ldif_template_path = ldif_template_path;

            run_mgmt(args, config);
        },
        Err(e) => error!("Configuration error: {:?}", e),
    }
}






