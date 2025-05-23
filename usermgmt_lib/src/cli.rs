use std::path::PathBuf;

pub use on_which_system::{OnSlurmLdapOnlyCli, OnWhichSystem, OnWhichSystemCli, OptFilePath};

mod on_which_system;

use clap::{Args, Parser, Subcommand};
use const_format::concatcp;
use derive_more::Into;

use crate::prelude::*;
use crate::util::TrimmedNonEmptyText;

pub const fn short_about() -> &'static str {
    "Simultaneous user management for Slurm and LDAP"
}

#[rustfmt::skip]
pub const fn links_about_project_for_end_users() -> &'static str {
    concatcp!(
        "Bug reports: ", constants::ISSUE_LINK, ".\n",
        "Source code: ", constants::REPOSITORY_LINK, " .\n",
        "License: MIT => ", constants::MIT_LINK, ".\n",
        "Readme: ", constants::README_LINK, ".\n"
    )
}

pub const fn long_about() -> &'static str {
    concatcp!(short_about(), ". \n\n", links_about_project_for_end_users())
}

#[derive(Parser, Debug)]
#[clap(author = "Authors: dwgnr and BoolPurist", version = env!("CARGO_PKG_VERSION"),
            about = long_about(), long_about = Some(long_about()))]
/// Add, delete, or modify users in LDAP and Slurm simultaneously
pub struct GeneralArgs {
    /// Operation to perform on the user. Either add, delete or modify.
    #[clap(subcommand)]
    pub command: Commands,
    #[arg(long)]
    /// Provide the path to a .toml configuration file.
    /// If absent, an attempt is made to locate the configuration file in various places on your system.
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
/// CLI sub commands for operations on users in LDAP and Slurm database
pub enum Commands {
    /// Add user to Slurm and/or LDAP and/or create user directories
    #[clap(visible_alias = "a")]
    Add {
        #[command(flatten)]
        to_add: UserToAdd,
        #[command(flatten)]
        on_which_sys: OnWhichSystemCli,
    },
    /// Modify user in Slurm and/or LDAP
    #[clap(visible_alias = "m")]
    Modify {
        #[command(flatten)]
        data: Modifiable,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// Delete user from Slurm and/or LDAP and/or delete user directories
    #[clap(visible_alias = "d")]
    Delete {
        /// A valid username e.g. wagnerdo.
        #[clap(value_parser = trimmed_non_empty)]
        user: TrimmedNonEmptyText,
        #[command(flatten)]
        on_which_sys: OnWhichSystemCli,
    },
    /// List users in Slurm and/or LDAP
    #[clap(visible_alias = "l")]
    List {
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
        /// Print a comma separated list of LDAP users instead of formatted table
        #[clap(long, verbatim_doc_comment)]
        simple_output_for_ldap: Option<bool>,
    },
    #[clap(visible_alias = "gc")]
    /// Prints a default configuration (conf.toml) to stdout.
    GenerateConfig,
}

/// Defines options for modifying a user
#[derive(Args, Debug, Clone, Into)]
pub struct Modifiable {
    /// Firstname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    firstname: Option<TrimmedNonEmptyText>,
    /// Lastname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    lastname: Option<TrimmedNonEmptyText>,
    #[command(flatten)]
    common_user_fields: CommonUserFields,
}

impl Modifiable {
    pub fn new(username: TrimmedNonEmptyText) -> Self {
        Self {
            firstname: Default::default(),
            lastname: Default::default(),
            common_user_fields: CommonUserFields::new(username),
        }
    }
}

/// Defines options for adding a user
#[derive(Args, Debug, Clone, Into)]
pub struct UserToAdd {
    /// Firstname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    firstname: TrimmedNonEmptyText,
    /// Lastname of the user.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    lastname: TrimmedNonEmptyText,
    #[command(flatten)]
    common_user_fields: CommonUserFields,
}

impl UserToAdd {
    pub fn new(
        firstname: TrimmedNonEmptyText,
        lastname: TrimmedNonEmptyText,
        common_user_fields: CommonUserFields,
    ) -> Self {
        Self {
            firstname,
            lastname,
            common_user_fields,
        }
    }

    pub fn common_user_fields(&self) -> &CommonUserFields {
        &self.common_user_fields
    }
}

#[derive(Args, Debug, Clone, Into)]
/// User attributes required for operations on users (adding, deleting or modifying).
pub struct CommonUserFields {
    /// Username e.g. wagnerdo.
    #[clap(value_parser = trimmed_non_empty)]
    pub username: TrimmedNonEmptyText,
    /// Unix group the user belongs to e.g. staff.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub group: Option<TrimmedNonEmptyText>,
    /// User's e-mail address.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub mail: Option<TrimmedNonEmptyText>,
    /// Slurm default QOS for the user e.g. basic.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub default_qos: Option<TrimmedNonEmptyText>,
    /// Path to SSH publickey.
    #[clap(short, long, value_parser = trimmed_non_empty)]
    pub publickey: Option<TrimmedNonEmptyText>,
    /// List of QoS assigned to the user (must be valid QoS i.e., they must exist in valid_qos of conf.toml). Arguments must be provided as a whitespace separated list (e.g. interactive basic).
    #[clap(short, long, num_args(0..=20))]
    pub qos: Vec<String>,
}

impl CommonUserFields {
    pub fn new(username: TrimmedNonEmptyText) -> Self {
        Self {
            username,
            group: Default::default(),
            mail: Default::default(),
            default_qos: Default::default(),
            publickey: Default::default(),
            qos: Default::default(),
        }
    }
}

/// Used by argument parser to ensure that
/// the argument is not empty and white spaces are trimmed
pub fn trimmed_non_empty(s: &str) -> AppResult<TrimmedNonEmptyText> {
    let to_validate = TrimmedNonEmptyText::try_from(s)?;
    Ok(to_validate)
}
