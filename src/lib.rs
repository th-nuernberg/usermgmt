mod util;
mod ldap;
mod slurm;
use std::{fmt, str::FromStr};
use serde::{Serialize, Deserialize};
use clap::{Parser, Subcommand};

use crate::{slurm::slurm::{add_slurm_user, delete_slurm_user, modify_slurm_user}, ldap::ldap::{add_ldap_user, delete_ldap_user, modify_ldap_user}, util::io_util::user_input};
extern crate confy;

/// Add, delete, or modify users in LDAP and Slurm simultaneously
#[derive(Parser, Debug)]
#[clap(author = "Author: Dominik Wagner", version = "0.1.0", 
        about = "Slurm and LDAP user management", long_about = None)]
pub struct Args {
    //     #[clap(short, long)]
    // pub command: String,

    /// Operation to conduct on the user. Either add, delete or modify.
    #[clap(subcommand)]
    pub command: Commands,
    // /// Username e.g. wagnerdo.
    // #[clap(short, long)]
    // pub user: String,

    /// Manage the user in Slurm only.
    #[clap(long)]
    pub slurm_only: bool,
    /// Manage the user in LDAP only.
    #[clap(long)]
    pub ldap_only: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a user to Slurm and/or LDAP
    Add { 
        /// Username e.g. wagnerdo.
        user: String,
        /// Unix group the user belongs to e.g. staff.
        #[clap(short, long, default_value = "student")]
        group: String,
        /// Firstname of the user.
        #[clap(short, long)]
        firstname: String,
        /// Lastname of the user.
        #[clap(short, long)]
        lastname: String,
        /// User's e-mail address.
        #[clap(short, long, default_value = "")]
        mail: String,
        /// Slurm default QOS for the user e.g. basic.
        #[clap(short, long, default_value = "basic")]
        default_qos: String,
        /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). 
        #[clap(short, long, max_values(20))]
        qos: Vec<String>,
    },
    /// Modify a user in Slurm and/or LDAP
    Modify { 
        /// A valid username e.g. wagnerdo.
        user: String, 
         /// Firstname of the user.
        #[clap(short, long)]
        firstname: Option<String>,
        /// Lastname of the user.
        #[clap(short, long)]
        lastname: Option<String>,
        /// User's e-mail address.
        #[clap(short, long)]
        mail: Option<String>,
        /// Slurm default QOS for the user e.g. basic.
        #[clap(short, long)]
        default_qos: Option<String>,
        /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). 
        #[clap(short, long)]
        qos: Vec<String>
    },
    /// Delete a user from Slurm and/or LDAP
    Delete { 
        /// A valid username e.g. wagnerdo.
        user: String 
    },
}

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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MgmtConfig {
    pub student_default_qos: String,
    pub staff_default_qos: String,
    pub student_qos: Vec<String>,
    pub staff_qos: Vec<String>,
    pub valid_qos: Vec<String>,
    pub valid_slurm_groups: Vec<String>,
    pub student_gid: i32,
    pub staff_gid: i32,
    pub faculty_gid: i32,
    pub sacctmgr_path: String,
    pub ldap_path: String,
    pub ldap_domain_components: String,
}

impl Default for MgmtConfig {
    fn default() -> Self {
        MgmtConfig {
            student_default_qos: "basic".to_string(), 
            staff_default_qos: "advanced".to_string(),
            student_qos: vec!["interactive".to_string(), "basic".to_string(), "gpubasic".to_string()],
            staff_qos: vec!["interactive".to_string(), "advanced".to_string(), "gpubasic".to_string()],
            valid_qos: vec!["interactive".to_string(), "basic".to_string(), 
                             "advanced".to_string(), "ultimate".to_string(), 
                            "bigmem".to_string(), "gpubasic".to_string(), 
                            "gpuultimate".to_string()],
            valid_slurm_groups: vec!["staff".to_string(), "student".to_string()],
            student_gid: 1002,
            staff_gid: 1001,
            faculty_gid: 1000,
            sacctmgr_path: "/usr/local/bin/sacctmgr".to_string(),
            ldap_path: "".to_string(),
            ldap_domain_components: "dc=informatik,dc=fh-nuernberg,dc=de".to_string(),
        }
    }
}

/// Main function that handles user management
pub fn run_mgmt(args: Args, config: MgmtConfig) {
    // let command = &args.command;
    let is_slurm_only = args.slurm_only.clone();
    let is_ldap_only = args.slurm_only.clone();
    let sacctmgr_path = config.sacctmgr_path.clone(); 

    // println!("\n\n\nList users");
    // list_ldap_users();
    match &args.command {
        Commands::Add {user, group, firstname, lastname, mail, default_qos, qos } => {
            add_user(user, group, firstname, lastname, mail, default_qos, qos, 
                    &is_slurm_only, &is_ldap_only, &config)
        }
        Commands::Modify { user, firstname, lastname, mail, default_qos, qos } => {
            modify_user(user, firstname, lastname, mail, default_qos, qos, 
                        &is_slurm_only, &is_ldap_only, &config)
        },
        Commands::Delete { user } => delete_user(user, &is_slurm_only, &is_ldap_only, &sacctmgr_path),
    }
    // match command.as_str() {
    //     "add" => add_user(args, &config),
    //     "modify" => modify_user(args, config),
    //     "delete" => delete_user(args, config),
    //     _ => println!("Invalid argument for command: {}. Must be one of 'add', 'modify' or 'delete'", command)
    // }
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
        add_ldap_user(&entity);
    }
    println!("Finished add_user");
}

fn delete_user(user: &String, is_slurm_only: &bool, is_ldap_only: &bool, sacctmgr_path: &String) {
    println!("Start delete_user");

    // let is_slurm_only = args.slurm_only.clone();
    // let is_ldap_only = args.slurm_only.clone();
    // let sacctmgr_path = config.sacctmgr_path.clone(); 

    if !is_ldap_only {
        delete_slurm_user(user, sacctmgr_path);
    }

    if !is_slurm_only {
        delete_ldap_user(user);
    }
    println!("Finished delete_user");
}

fn modify_user(user: &String, firstname: &Option<String>, lastname: &Option<String>, 
                mail: &Option<String>, default_qos: &Option<String>, qos: &Vec<String>, 
                is_slurm_only: &bool, is_ldap_only: &bool, config: &MgmtConfig) {
    
    // to do
    // we need a struct that contains all elements to be modified
    // we do this by checking if we get some in the optional
    println!("Start modify_user for {}", user);

    let modifiable = Modifiable::new(user, firstname, lastname, mail, default_qos, qos);

    let sacctmgr_path = config.sacctmgr_path.clone(); 
    // let entity = Entity::new(user, group, firstname, lastname, mail, default_qos, qos, config);

    if !is_ldap_only {
        modify_slurm_user(&modifiable, &sacctmgr_path);
    }

    if !is_slurm_only {
        modify_ldap_user(&modifiable, &config);
    }
    println!("Finished modify_user");
}


/// Another method for adding users. 
/// Might be used instead of command line arguments at some point
fn handle_add_user_questions(valid_qos: &Vec<String>) {

    println!("Enter user firstname:\n");
    let firstname = user_input();
    println!("Enter user lastname:\n");
    let lastname = user_input();
    println!("Enter e-mail:\n");
    let email = user_input();
    println!("Enter Slurm default QOS:\n");
    loop {
        let default_qos = user_input();
        if !is_valid_qos(&vec![default_qos.clone()], valid_qos) {
            println!("QOS {} is invalid. Please select one of the following:", &default_qos);
            println!("{:#?}", valid_qos);
        } else {
            break;
        }
    }
    
    let mut qos = Vec::new();
    loop {
        println!("Enter Slurm QOS (press \"f\" or \"1\" to finish):\n");
        let current_input = user_input();
        if current_input.to_lowercase() == "f" || current_input.to_lowercase() == "1" {
            println!("Finished adding QOS. Got:");
            println!("{:#?}", valid_qos);
        } else {
            if is_valid_qos(&vec![current_input.clone()], valid_qos) {
                qos.push(current_input);
            } else {
                println!("QOS {} is invalid. Please select one of the following:", current_input);
                println!("{:#?}", valid_qos);
            }
        }
    }
}
