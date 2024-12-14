use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;
use log::{debug, info};
use once_cell::sync::Lazy;

use crate::prelude::*;

static HOME_LOCATIONS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let os_config = dirs::config_dir();
    let mut paths: Vec<PathBuf> = Vec::with_capacity(2);
    if let Some(config) = os_config {
        let with_app_name = config.join("usermgmt");
        paths.push(with_app_name);
    }
    if cfg!(unix) {
        paths.push(PathBuf::from("~/.usermgmt"));
    }
    paths
});

static SYSTEM_LOCATIONS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let mut paths = Vec::with_capacity(1);
    if cfg!(unix) {
        paths.push(PathBuf::from("/usr/usermgmt"))
    }
    paths
});

/// Gets the first absolute path to an existing configuration file with named `NAME_CONFIG_FILE`.
///
/// At first It searches all places for a specific user. These paths are `HOME_LOCATIONS`.
/// If nothing is found there, all places for system settings are searched aka `SYSTEM_LOCATIONS`
/// If nothing is found there too, it searches CWD for the configuration file.
///
/// # Errors
///
/// - If no configuration file could be found anywhere.
/// - If CWD can not be determined after the configuration file could not be found under the home and
/// system paths.
///
pub fn get_path_to_conf(manual_path: Option<PathBuf>) -> AppResult<PathBuf> {
    get_path_to_conf_with_dep(
        manual_path,
        try_resolve_paths_with_home,
        |to_check| may_return_path_to_conf(io_try_exists, to_check),
        try_cwd_as_last_resort,
        &SYSTEM_LOCATIONS,
    )
}

fn get_path_to_conf_with_dep(
    manual_path: Option<PathBuf>,
    on_resolve_paths_with_home: impl Fn(Option<PathBuf>, &[PathBuf]) -> Vec<PathBuf>,
    on_try_exits: impl Fn(&Path) -> Option<PathBuf>,
    on_try_cwd_as_last_resort: impl Fn() -> AppResult<PathBuf>,
    system_paths: &[PathBuf],
) -> AppResult<PathBuf> {
    let mut home_dirs: Vec<PathBuf> = on_resolve_paths_with_home(dirs::home_dir(), &HOME_LOCATIONS);

    home_dirs = if let Some(manual) = manual_path {
        let mut new = vec![manual];
        new.extend(home_dirs);
        new
    } else {
        home_dirs
    };
    let first_find: Option<PathBuf> = home_dirs
        .into_iter()
        .chain(
            system_paths
                .iter()
                .map(|to_resolve| PathBuf::from(&to_resolve)),
        )
        .flat_map(|to_check| on_try_exits(&to_check))
        .next();

    match first_find {
        Some(first_match) => Ok(first_match),
        None => on_try_cwd_as_last_resort(),
    }
}

fn try_resolve_paths_with_home(home_folder: Option<PathBuf>, folders: &[PathBuf]) -> Vec<PathBuf> {
    match home_folder {
        Some(dir) => {
            let resolve_dir = dir;
            folders
                .iter()
                .map(|to_resolve| match to_resolve.strip_prefix("~") {
                    Ok(stripped) => PathBuf::from(&resolve_dir).join(stripped),
                    Err(error) => {
                        debug!(
                            "Stripping prefix did not happen for {:?}. Details: {}",
                            to_resolve, error
                        );
                        to_resolve.clone()
                    }
                })
                .collect()
        }
        None => {
            debug!(
                "Could not find any home directory location. Skipping checking for locations {:?}",
                HOME_LOCATIONS
            );

            Vec::new()
        }
    }
}

fn try_cwd_as_last_resort() -> AppResult<PathBuf> {
    debug!(
        "No configuration file found in previous paths. Trying to get configuration file in cwd."
    );
    let cwd = std::env::current_dir().context(
                "No folder found, cwd directory as last alternative could not be retrieved.\n No conf.toml could be found",
            )?;
    let last_resort = may_return_path_to_conf(io_try_exists, &cwd).ok_or(anyhow::anyhow!(
        "path at {:?} as last alternative does also not exist.\n No conf.toml could be found",
        cwd
    ))?;
    info!(
        "Using cwd path {:?} for configuration file as last resort.",
        last_resort
    );

    Ok(last_resort)
}

fn may_return_path_to_conf(
    try_exists: impl Fn(&Path) -> io::Result<bool>,
    path: &Path,
) -> Option<PathBuf> {
    let to_check = if path.is_file() {
        debug!("Path at {:?} is a configuration file.", path);
        path.to_path_buf()
    } else {
        debug!(
            "Path at {0:?} is a directory.\n\
            Looking for a configuration file named {1} within this directory.",
            path,
            constants::NAME_CONFIG_FILE
        );
        path.join(constants::NAME_CONFIG_FILE)
    };
    match try_exists(&to_check) {
        Ok(exits) => {
            if exits {
                Some(to_check)
            } else {
                debug!("Path at {:?} does not exist!", to_check);
                None
            }
        }
        Err(error) => {
            debug!(
                "Could not check existence for path at {:?}\n. Error: {}",
                to_check, error
            );
            None
        }
    }
}

fn io_try_exists(path: &Path) -> io::Result<bool> {
    path.try_exists()
}

#[cfg(test)]
mod testing {
    use std::io::ErrorKind;

    use anyhow::anyhow;

    use super::*;

    #[test]
    fn return_empty_if_no_home_folder() {
        let actual = try_resolve_paths_with_home(None, &[]);
        assert!(actual.is_empty());
    }

    #[test]
    fn return_resolve_with_home_folder() {
        let actual = try_resolve_paths_with_home(
            Some(PathBuf::from("/home")),
            &["~/foo", "~/bar", "~/.tool", "~", "~/hello_~"]
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<PathBuf>>(),
        );
        insta::assert_yaml_snapshot!(actual);
    }

    #[test]
    fn return_none_if_io_error_for_path_check() {
        let actual = may_return_path_to_conf(
            |_| Err(io::Error::new(ErrorKind::Other, "Some error")),
            &PathBuf::from("/"),
        );

        assert!(actual.is_none());
    }
    #[test]
    fn return_none_if_path_does_not_exits() {
        let actual = may_return_path_to_conf(|_| Ok(false), &PathBuf::from("/"));

        assert!(actual.is_none());
    }
    #[test]
    fn return_some_if_path_exits() {
        let expected = Some(PathBuf::from("/home").join(constants::NAME_CONFIG_FILE));
        let actual = may_return_path_to_conf(|_| Ok(true), &PathBuf::from("/home"));

        assert_eq!(expected, actual);
    }

    #[test]
    fn error_for_no_path_found() {
        let actual =
            get_path_to_conf_with_dep(None, |_, _| Vec::new(), |_| None, || Err(anyhow!("")), &[]);
        assert!(actual.is_err());
    }
    #[test]
    fn cwd_if_home_and_others_are_none() {
        let expected = PathBuf::from("/home");
        let actual = get_path_to_conf_with_dep(
            None,
            |_, _| Vec::new(),
            |_| None,
            || Ok(PathBuf::from("/home")),
            &Vec::new(),
        );
        assert_eq!(expected, actual.unwrap());
    }
    #[test]
    fn others_before_cwd() {
        let expected = PathBuf::from("/usr/home");
        let actual = get_path_to_conf_with_dep(
            None,
            |_, _| Vec::new(),
            |to_match| {
                if to_match == &expected {
                    Some(to_match.to_owned())
                } else {
                    None
                }
            },
            || Ok(PathBuf::from("/home")),
            &["foo".into(), "/usr/home".into(), "bar".into()],
        );
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn home_before_others_before_cwd() {
        let expected = PathBuf::from("/home/bar");
        let actual = get_path_to_conf_with_dep(
            None,
            |_, _| vec![expected.clone()],
            |to_match| {
                if to_match == &expected {
                    Some(to_match.to_owned())
                } else {
                    None
                }
            },
            || Ok(PathBuf::from("/home")),
            &["foo".into(), "/usr/home".into(), "bar".into()],
        );
        assert_eq!(expected, actual.unwrap());
    }
}
