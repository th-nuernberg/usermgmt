use clap::StructOpt;
use env_logger::Env;
use log::error;
use std::fs;
use std::path::Path;
use usermgmt::cli::cli::Args;
use usermgmt::config::config::MgmtConfig;
use usermgmt::run_mgmt;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let config_file_basedir = if cfg!(target_os = "linux") && cfg!(not(debug_assertions)) {
        "/etc/usermgmt".to_owned()
    } else {
        ".".to_owned()
    };

    fs::create_dir_all(&config_file_basedir).unwrap();
    let path_string = config_file_basedir + "/conf.toml";
    let path = Path::new(&path_string);

    // Load (or create if nonexistent) configuration file conf.toml
    let cfg: Result<MgmtConfig, confy::ConfyError> = confy::load_path(path);
    match cfg {
        Ok(config) => {
            let args = Args::parse();
            run_mgmt(args, config);
        }
        Err(e) => error!("Configuration error: {:?}", e),
    }
}
