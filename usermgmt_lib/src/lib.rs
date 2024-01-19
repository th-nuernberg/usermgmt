pub use entity::Entity;
pub use new_entity::NewEntity;

pub mod app_error;
pub mod changes_to_user;
pub mod cli;
pub mod config;
pub mod constants;
pub mod dir;
pub mod logging;
pub mod util;

pub mod entity;
pub mod ldap;
pub mod new_entity;
pub mod operations;
pub mod slurm;
pub mod ssh;

pub use changes_to_user::ChangesToUser;

use config::MgmtConfig;
use log::warn;
use prelude::*;
use std::{collections::HashSet, fmt, str::FromStr};

pub mod prelude {
    pub use crate::app_error;
    pub use crate::constants;
    pub use anyhow::{anyhow, bail, Context};
    pub type AnyError = anyhow::Error;
    pub type AppError = AnyError;
    pub type AppResult<T = ()> = Result<T, AnyError>;
}
pub mod app_panic_hook;

extern crate confy;

// TODO: git rid of unwraps. Replace them with expects or better with result if possible.
// TODO: implement struct or function to remove redundancy for opening up tcp/ssh connection
// A code block as example in the file slurm under function add_slurm_user is repeated quite often

#[derive(Clone, PartialEq, Copy, Debug, Eq)]
pub enum Group {
    Staff,
    Student,
    Faculty,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Group::Staff => write!(f, "staff"),
            Group::Student => write!(f, "student"),
            Group::Faculty => write!(f, "faculty"),
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::Student
    }
}

impl FromStr for Group {
    type Err = AppError;
    fn from_str(input: &str) -> Result<Group, Self::Err> {
        match input {
            "Staff" | "staff" => Ok(Group::Staff),
            "Student" | "student" => Ok(Group::Student),
            "Faculty" | "faculty" => Ok(Group::Student),
            _ => Err(anyhow!("given group name ({}) is not valid", input)),
        }
    }
}

/// Removes all invalid elements of `qos`. An element is valid if `valid_qos` contains it.
/// Filters out duplicates too.
/// Returns an empty vector if `qos` or `valid_qos` is empty.
pub fn filter_invalid_qos<S>(qos: &[S], valid_qos: &[S]) -> Vec<S>
where
    S: AsRef<str> + PartialEq + Clone + std::fmt::Display,
{
    let mut filtered_qos: Vec<S> = Vec::with_capacity(qos.len());
    // Just keep references to prevent another heap allocation.
    let mut found: HashSet<&str> = HashSet::with_capacity(qos.len());

    for to_inspect in qos {
        let as_str: &str = to_inspect.as_ref();
        if valid_qos.contains(to_inspect) {
            if found.insert(as_str) {
                filtered_qos.push(to_inspect.clone());
            } else {
                warn!(
                    "QOS {} has a duplicate and will not be added another time !",
                    to_inspect
                )
            }
        } else {
            let s: &str = to_inspect.as_ref();
            warn!("QOS {} is invalid and will be removed!", s)
        }
    }

    filtered_qos.into_iter().collect()
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn should_determine_if_valid_qos() {
        assert_case(&["student"], &["student", "staff", "faculty"], true);
        assert_case(&["worker"], &["student", "staff", "faculty"], false);
        assert_case(
            &["student", "worker"],
            &["student", "staff", "faculty"],
            false,
        );
        assert_case(&["student"], &[], false);
        assert_case(&[], &["student"], true);
        assert_case(&[], &[], true);

        fn assert_case(qos: &[&str], valid_qos: &[&str], expected: bool) {
            let actual = util::is_valid_qos(qos, valid_qos);
            assert_eq!(
                expected, actual,
                "expected: {} with qos: {:?} and valid_qos: {:?}",
                expected, qos, valid_qos
            );
        }
    }

    #[test]
    fn should_filter_out_invalid_qos() {
        assert_case(&["student", "worker"], &["student"], vec!["student"]);
        // With duplicates
        assert_case(
            &["student", "student", "worker"],
            &["student"],
            vec!["student"],
        );
        // left == right
        assert_case(
            &["student", "worker"],
            &["student", "worker"],
            vec!["student", "worker"],
        );
        // contains only valid elements
        assert_case(
            &["student", "worker"],
            &["student", "worker", "staff"],
            vec!["student", "worker"],
        );
        // No valid element
        assert_case(&["npc", "worker"], &["student"], vec![]);
        // Edge cases for empty lists
        assert_case(&["student"], &[], vec![]);
        assert_case(&[], &["student"], vec![]);

        fn assert_case(qos: &[&str], filter: &[&str], expected: Vec<&str>) {
            let actual = filter_invalid_qos(qos, filter);
            assert_eq!(expected, actual, "qos: {:?} and filter: {:?}", qos, filter);
        }
    }
}
