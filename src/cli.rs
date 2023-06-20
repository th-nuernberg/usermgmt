use clap::{Args, Parser, Subcommand};
mod on_which_system;

pub use on_which_system::{OnSlurmLdapOnlyCli, OnWhichSystem, OnWhichSystemCli};

use crate::{prelude::AppResult, util::TrimmedNonEmptyText};
/// Add, delete, or modify users in LDAP and Slurm simultaneously
#[derive(Parser, Debug)]
#[clap(author = "Authors: dwgnr and BoolPurist", version = env!("CARGO_PKG_VERSION"),
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
        #[clap(value_parser = trimmed_non_empty)]
        user: TrimmedNonEmptyText,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// List users in Slurm and/or LDAP
    List {
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
        /// Prints out comma separated list instead of table
        /// Is meant to be used for automation
        #[clap(long, verbatim_doc_comment)]
        simple_output_for_ldap: Option<bool>,
    },
}

/// Defines options that can be modified
/// TODO: consider encapsulation with getters and setters.
#[derive(Args, Debug, Clone)]
pub struct Modifiable {
    /// A valid username e.g. wagnerdo.
    #[clap(value_parser = trimmed_non_empty)]
    pub username: TrimmedNonEmptyText,
    /// Firstname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub firstname: Option<TrimmedNonEmptyText>,
    /// Lastname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub lastname: Option<TrimmedNonEmptyText>,
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
    #[clap(value_parser = trimmed_non_empty)]
    pub user: TrimmedNonEmptyText,
    /// Unix group the user belongs to e.g. staff.
    #[clap(short, long, default_value = "student", value_parser = trimmed_non_empty)]
    pub group: TrimmedNonEmptyText,
    /// Firstname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub firstname: TrimmedNonEmptyText,
    /// Lastname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub lastname: TrimmedNonEmptyText,
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

/// Used by argument parser to ensure that
/// the argument is not empty and white spaces are trimmed off from it
pub fn trimmed_non_empty(s: &str) -> AppResult<TrimmedNonEmptyText> {
    let to_validate = TrimmedNonEmptyText::try_from(s)?;
    Ok(to_validate)
}
