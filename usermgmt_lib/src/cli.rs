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
pub const fn links() -> &'static str {
    concatcp!(
        "Wheret to report bugs: ", constants::ISSUE_LINK, ".\n",
        "Source code: ", constants::REPOSITORY_LINK, " .\n",
        "License: MIT => ", constants::MIT_LINK, ".\n",
        "Readme: ", constants::README_LINK, ".\n"
    )
}

pub const fn long_about() -> &'static str {
    concatcp!(short_about(), ". \n\n", links())
}

#[derive(Parser, Debug)]
#[clap(author = "Authors: dwgnr and BoolPurist", version = env!("CARGO_PKG_VERSION"),
            about = long_about(), long_about = Some(long_about()))]
/// Add, delete, or modify users in LDAP and Slurm simultaneously
pub struct GeneralArgs {
    /// Operation to conduct on the user. Either add, delete or modify.
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a user to Slurm and/or LDAP
    #[clap(visible_alias = "a")]
    Add {
        #[command(flatten)]
        to_add: UserToAdd,
        #[command(flatten)]
        on_which_sys: OnWhichSystemCli,
    },
    /// Modify a user in Slurm and/or LDAP
    #[clap(visible_alias = "m")]
    Modify {
        #[command(flatten)]
        data: Modifiable,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// Delete a user from Slurm and/or LDAP
    #[clap(visible_alias = "d")]
    Delete {
        /// A valid username e.g. wagnerdo.
        #[clap(value_parser = trimmed_non_empty)]
        user: TrimmedNonEmptyText,
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
    },
    /// List users in Slurm and/or LDAP
    #[clap(visible_alias = "l")]
    List {
        #[command(flatten)]
        on_which_sys: OnSlurmLdapOnlyCli,
        /// Prints out comma separated list instead of table
        /// Is meant to be used for automation
        #[clap(long, verbatim_doc_comment)]
        simple_output_for_ldap: Option<bool>,
    },
    #[clap(visible_alias = "gc")]
    /// Outputs a default configuration, aka conf.toml, to stdout.
    /// Pipe it to a path for a file to generate a permanent configuration somewhere.
    GenerateConfig,
}

/// Defines options that can be modified
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
    /// List of QOS assigned to the user (must be valid QOS i.e. they must exist in valid_qos of conf.toml). QOS need to be provided as a whitespace separated list (e.g. interactive basic).
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
/// the argument is not empty and white spaces are trimmed off from it
pub fn trimmed_non_empty(s: &str) -> AppResult<TrimmedNonEmptyText> {
    let to_validate = TrimmedNonEmptyText::try_from(s)?;
    Ok(to_validate)
}
