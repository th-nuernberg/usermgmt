use std::path::Path;
use clap::StructOpt;
use usermgmt::config::config::MgmtConfig;
use usermgmt::run_mgmt;
use usermgmt::cli::cli::Args;
extern crate confy;

//TODO
// make deb package
// maybe make slurmQos and slurmDefaultQos atributetype names configurable

fn main() {
    let path = Path::new("conf.toml");
    let cfg:Result<MgmtConfig, confy::ConfyError> = confy::load_path(path);
    match cfg {
        Ok(config) => {
            // println!("Config ok: {:?}", config);
            let args = Args::parse();
            // println!("{:?}", args);
            run_mgmt(args, config);
        },
        Err(e) => println!("Configuration error: {:?}", e),
    }
}






