use clap::{Args, Parser, Subcommand};
mod on_which_system;

pub use on_which_system::{OnSlurmLdapOnlyCli, OnWhichSystem, OnWhichSystemCli};
/// Add, delete, or modify users in LDAP and Slurm simultaneously
#[derive(Parser, Debug)]
#[clap(author = "Author: Dominik Wagner", version = env!("CARGO_PKG_VERSION"),
            about = "Simultaneous user management for Slurm and LDAP", long_about = None)]
pub struct GeneralArgs {
    /// Operation to conduct on the user. Either add, delete or modify.
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a user to Slurm and/or LDAP
    Add {
        #[command(flatten)]
        to_add: UserToAdd,
        #[command(flatten)]
        on_which_sys: OnWhichSystemCli,
    },
    /// Modify a user in Slurm and/or LDAP
    Modify {
        #[command(flatten)]
        data: Modifiable,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// Delete a user from Slurm and/or LDAP
    Delete {
        /// A valid username e.g. wagnerdo.
        user: String,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// List users in Slurm and/or LDAP
    List {
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
}

/// Defines options that can be modified
/// TODO: consider encapsulation with getters and setters.
#[derive(Args, Debug, Clone)]
pub struct Modifiable {
    /// A valid username e.g. wagnerdo.
    pub username: String,
    /// Firstname of the user.
    #[clap(short, long)]
    pub firstname: Option<String>,
    /// Lastname of the user.
    #[clap(short, long)]
    pub lastname: Option<String>,
    /// User's e-mail address.
    #[clap(short, long)]
    pub mail: Option<String>,
    /// Slurm default QOS for the user e.g. basic.
    #[clap(short, long)]
    pub default_qos: Option<String>,
    /// Path to SSH publickey.
    #[clap(short, long)]
    pub publickey: Option<String>,
    /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). Max 20 values allowed.
    #[clap(short, long, num_args(0..=20))]
    pub qos: Vec<String>,
}

/// TODO: consider encapsulation with getters and setters.
#[derive(Args, Debug, Clone)]
pub struct UserToAdd {
    /// Username e.g. wagnerdo.
    pub user: String,
    /// Unix group the user belongs to e.g. staff.
    #[clap(short, long, default_value = "student")]
    pub group: String,
    /// Firstname of the user.
    #[clap(short, long)]
    pub firstname: String,
    /// Lastname of the user.
    #[clap(short, long)]
    pub lastname: String,
    /// User's e-mail address.
    #[clap(short, long, default_value = "")]
    pub mail: String,
    /// Slurm default QOS for the user e.g. basic.
    #[clap(short, long, default_value = "")]
    pub default_qos: String,
    /// Path to SSH publickey.
    #[clap(short, long, default_value = "")]
    pub publickey: String,
    /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). QOS need to be provided as a whitespace separated list (e.g. interactive basic).
    #[clap(short, long, num_args(0..=20))]
    pub qos: Vec<String>,
}
