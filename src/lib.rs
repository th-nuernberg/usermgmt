pub mod util;
pub mod config;
pub mod cli;
mod ldap;
mod slurm;
use std::{fmt, str::FromStr};
use cli::cli::Commands;
use config::config::MgmtConfig;

use crate::{slurm::slurm::{add_slurm_user, delete_slurm_user, modify_slurm_user}, ldap::ldap::{add_ldap_user, delete_ldap_user, modify_ldap_user}};
extern crate confy;

#[derive(Clone, PartialEq)]
pub enum Group {
    Staff,
    Student,
    Faculty
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Group::Staff => write!(f, "Staff"),
            Group::Student => write!(f, "Student"),
            Group::Faculty => write!(f, "Faculty"),
        }
    }
}

impl FromStr for Group {
    type Err = ();
    fn from_str(input: &str) -> Result<Group, Self::Err> {
        match input {
            "Staff"   | "staff" => Ok(Group::Staff),
            "Student" | "student"  => Ok(Group::Student),
            "Faculty" | "faculty"  => Ok(Group::Student),
            _      => Err(()),
        }
    }
}

/// Representation of a user entity. 
/// It contains all information necessary to add/modify/delete the user. 
pub struct Entity {
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub gid: i32,
    pub group: Group,
    pub default_qos: String,
    pub qos: Vec<String>
}

impl Entity {
    fn new(user: &String, group: &String, firstname: &String, lastname: &String, mail: &String, default_qos: &String, qos: &Vec<String>, config: &MgmtConfig) -> Self {
        
        let staff_qos = &config.staff_qos;
        let student_qos = &config.student_qos;
        let valid_qos = &config.valid_qos;

        let staff_default_qos = &config.staff_default_qos;
        let student_default_qos = &config.student_default_qos;

        let mut default_qos = default_qos;
        let mut qos: &Vec<String> = qos; 


        let mut group_str = group.to_lowercase();
        // let mut default_qos = args.default_qos;
        // let mut qos = args.qos;
        
        if !is_valid_group(&group_str, &config.valid_slurm_groups) {
            println!("Invalid group specified: {}. Group field will be student", group_str);
            group_str = "student".to_string();
        }

        let group = Group::from_str(&group_str).unwrap();
        let gid = Entity::map_groupname_to_gid(group.clone(), config).unwrap();

        if qos.is_empty() || !is_valid_qos(&qos, &valid_qos) {
            println!("Specified QOS are either invalid or empty. Using defaults specified in config.");
            match group {
                Group::Staff => qos = &staff_qos,
                Group::Student => qos = &student_qos,
                Group::Faculty => qos = &staff_qos
            }
        } 

        if !is_valid_qos(&vec![default_qos.clone()], &valid_qos) {
            println!("Specified default QOS is invalid. Using the value specified in config.");
            match group {
                Group::Staff => default_qos = &staff_default_qos,
                Group::Student => default_qos = &student_default_qos,
                Group::Faculty => default_qos = &staff_default_qos
            }
        } 

        Entity {
            username: user.to_lowercase(),
            firstname: firstname.to_string(),
            lastname: lastname.to_string(),
            group,
            default_qos: default_qos.to_string(),
            qos: qos.to_vec(),
            gid,
            mail: mail.to_string()
        }
    }

    fn map_groupname_to_gid(group : Group, config: &MgmtConfig) -> Result<i32, ()> {
        match group {
            Group::Staff => Ok(config.staff_gid),
            Group::Student => Ok(config.student_gid),
            Group::Faculty => Ok(config.faculty_gid)
        }
    }

}

impl Default for Entity {
    fn default() -> Self {
        Entity {
            username: "".to_string(),
            firstname: "".to_string(),
            lastname: "".to_string(),
            mail: "".to_string(),
            gid: -1,
            group: Group::Student,
            default_qos: "basic".to_string(),
            qos: vec![],
        }
    }
}

pub struct Modifiable {
    pub username: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
    pub default_qos: Option<String>,
    pub qos: Vec<String>
}

impl Modifiable {
    fn new(username: &String, firstname: &Option<String>, lastname: &Option<String>, 
        mail: &Option<String>, default_qos: &Option<String>, qos: &Vec<String>) -> Self {

        Modifiable { 
            username: username.clone(),
            firstname: firstname.clone(), 
            lastname: lastname.clone(), 
            mail: mail.clone(), 
            default_qos: default_qos.clone(), 
            qos: qos.clone() 
        }
    }
}

/// Main function that handles user management
pub fn run_mgmt(args: cli::cli::Args, config: MgmtConfig) {

    let is_slurm_only = args.slurm_only.clone();
    let is_ldap_only = args.slurm_only.clone();
    let sacctmgr_path = config.sacctmgr_path.clone(); 

    match &args.command {
        Commands::Add {user, group, firstname, lastname, mail, default_qos, qos } => {
            add_user(user, group, firstname, lastname, mail, default_qos, qos, 
                    &is_slurm_only, &is_ldap_only, &config)
        }
        Commands::Modify { user, firstname, lastname, mail, default_qos, qos } => {
            modify_user(user, firstname, lastname, mail, default_qos, qos, 
                        &is_slurm_only, &is_ldap_only, &config)
        },
        Commands::Delete { user } => delete_user(user, &is_slurm_only, &is_ldap_only, &sacctmgr_path, &config),
    }
}

fn is_valid_qos(qos: &Vec<String>, valid_qos: &Vec<String>) -> bool {
    for q in qos {
        if !valid_qos.contains(q) { 
            return false
        }
    }
    true
}

fn is_valid_group(group: &String, valid_groups: &Vec<String>) -> bool {
    valid_groups.contains(group)
}

fn add_user(user: &String, group: &String, firstname: &String, 
            lastname: &String, mail: &String, default_qos: &String, 
            qos: &Vec<String>, is_slurm_only: &bool, is_ldap_only: &bool, 
            config: &MgmtConfig) {
    println!("Start add_user");

    // println!("{:#?}", config);
    // println!("{:#?}", args);
    let sacctmgr_path = config.sacctmgr_path.clone(); 

    let entity = Entity::new(user, group, firstname, lastname, mail, default_qos, qos, config);

    if !is_ldap_only {
        add_slurm_user(&entity, &sacctmgr_path);
    }

    if !is_slurm_only {
        add_ldap_user(&entity, &config);
    }
    println!("Finished add_user");
}

fn delete_user(user: &String, is_slurm_only: &bool, is_ldap_only: &bool, 
                sacctmgr_path: &String, config: &MgmtConfig) {
    println!("Start delete_user");

    if !is_ldap_only {
        delete_slurm_user(user, sacctmgr_path);
    }

    if !is_slurm_only {
        delete_ldap_user(user, &config);
    }
    println!("Finished delete_user");
}

fn modify_user(user: &String, firstname: &Option<String>, lastname: &Option<String>, 
                mail: &Option<String>, default_qos: &Option<String>, qos: &Vec<String>, 
                is_slurm_only: &bool, is_ldap_only: &bool, config: &MgmtConfig) {
    
    println!("Start modify_user for {}", user);

    let modifiable = Modifiable::new(user, firstname, lastname, mail, default_qos, qos);

    let sacctmgr_path = config.sacctmgr_path.clone(); 

    if !is_ldap_only {
        modify_slurm_user(&modifiable, &sacctmgr_path);
    }

    if !is_slurm_only {
        modify_ldap_user(&modifiable, &config);
    }
    println!("Finished modify_user");
}
