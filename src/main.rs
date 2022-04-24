use std::path::Path;
use clap::StructOpt;
use usermgmt::{MgmtConfig, Args, run_mgmt};
extern crate confy;

//TODO
// implement ldap user modification
// refactor lib into cli.rs and config.rs

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
        Err(e) => println!("Config error: {:?}", e),
    }
}






