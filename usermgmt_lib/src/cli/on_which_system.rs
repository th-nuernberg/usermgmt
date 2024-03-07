use std::path::PathBuf;

use clap::Args;
use getset::{CopyGetters, Getters};

use crate::config::MgmtConfig;
pub type OptFilePath = Option<PathBuf>;

/// Same as [`OnWhichSystem`] except without considering options from a configuration file.
#[derive(Args, CopyGetters, Debug)]
pub struct OnWhichSystemCli {
    #[command(flatten)]
    ldap_slurm: OnSlurmLdapOnlyCli,
    #[getset(get_copy = "pub")]
    /// If true then the action will be performed on user directory too, otherwise nothing happens on user directory.
    /// Overrides the default which is provided by the conf.toml
    #[clap(long, verbatim_doc_comment)]
    dirs: Option<bool>,
}

/// Same as [`OnWhichSystemCli`] except without directory management.
/// Actions like deleting an user are not supported for directory management.
/// For those actions this option struct is used as CLI arguments.
#[derive(Args, CopyGetters, Getters, Debug)]
pub struct OnSlurmLdapOnlyCli {
    /// If true then the action will be performed on Slurm too, otherwise nothing happens on Slurm.
    /// Overrides the default which is provided by the conf.toml
    #[clap(long, verbatim_doc_comment)]
    #[getset(get_copy = "pub")]
    slurm: Option<bool>,
    /// If true then the action will be performed on LDAP too, otherwise nothing happens on Ldap.
    /// Overrides the default which is provided by the conf.toml
    #[clap(long, verbatim_doc_comment)]
    #[getset(get_copy = "pub")]
    ldap: Option<bool>,
    /// Path where to find key pair to be used for ssh connection.
    /// Has priority over the path from the configuration file.
    #[arg(long, verbatim_doc_comment)]
    #[getset(get = "pub")]
    ssh_path: Option<PathBuf>,
}

/// Information on which systems an action like creating an user should happen.
/// Ensures flexibility for user to toggle systems via CLI and options from configuration file.
/// CLI option have priority over default values from configuration file.
#[derive(CopyGetters, Getters, Debug)]
pub struct OnWhichSystem {
    #[getset(get_copy = "pub")]
    slurm: bool,
    #[getset(get_copy = "pub")]
    ldap: bool,
    #[getset(get_copy = "pub")]
    dirs: bool,
    #[getset(get = "pub")]
    ssh_path: OptFilePath,
}

impl OnWhichSystem {
    pub fn new(slurm: bool, ldap: bool, dirs: bool, ssh_path: OptFilePath) -> Self {
        Self {
            slurm,
            ldap,
            dirs,
            ssh_path,
        }
    }

    pub fn from_config_for_all(config: &MgmtConfig, from_cli: &OnWhichSystemCli) -> Self {
        let mut slurm_ldap = Self::from_config_for_slurm_ldap(config, &from_cli.ldap_slurm);
        slurm_ldap.dirs = Self::use_cli_over_config(from_cli.dirs(), config.include_dir_mgmt);
        slurm_ldap
    }

    pub fn from_config_for_slurm_ldap(config: &MgmtConfig, from_cli: &OnSlurmLdapOnlyCli) -> Self {
        Self {
            ldap: Self::use_cli_over_config(from_cli.ldap(), config.include_ldap),
            slurm: Self::use_cli_over_config(from_cli.slurm(), config.include_slurm),
            dirs: false,
            ssh_path: from_cli
                .ssh_path()
                .as_ref()
                .cloned()
                .or_else(|| config.ssh_key_path.clone()),
        }
    }

    pub fn needs_ssh(&self) -> bool {
        self.slurm() || self.ldap()
    }

    fn use_cli_over_config<T>(cli: Option<T>, config_val: T) -> T {
        match cli {
            Some(cli_over_config) => cli_over_config,
            None => config_val,
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn should_never_allow_dirs_for_ldap_slurm_only() {
        let actual = OnWhichSystem::from_config_for_slurm_ldap(
            &MgmtConfig {
                include_dir_mgmt: true,
                ..Default::default()
            },
            &OnSlurmLdapOnlyCli {
                ldap: Some(true),
                slurm: Some(true),
                ssh_path: None,
            },
        );

        assert_eq!(
            (true, true, false),
            (actual.ldap(), actual.slurm(), actual.dirs())
        );
    }
    #[test]
    fn should_use_cli_over_config_slurm_ldap() {
        // Default config without cli override
        assert_case(
            OnWhichSystemCli {
                ldap_slurm: OnSlurmLdapOnlyCli {
                    ldap: None,
                    slurm: None,
                    ssh_path: None,
                },
                dirs: None,
            },
            MgmtConfig::default(),
            (true, true, false),
        );
        // default config with slurm override
        assert_case(
            OnWhichSystemCli {
                ldap_slurm: OnSlurmLdapOnlyCli {
                    ldap: None,
                    slurm: Some(false),
                    ssh_path: None,
                },
                dirs: None,
            },
            MgmtConfig::default(),
            (true, false, false),
        );
        // non config default with slurm and ldap override
        assert_case(
            OnWhichSystemCli {
                ldap_slurm: OnSlurmLdapOnlyCli {
                    ldap: Some(false),
                    slurm: Some(false),
                    ssh_path: None,
                },
                dirs: None,
            },
            MgmtConfig {
                include_dir_mgmt: true,
                ..Default::default()
            },
            (false, false, true),
        );
        // only config values
        assert_case(
            OnWhichSystemCli {
                ldap_slurm: OnSlurmLdapOnlyCli {
                    ldap: None,
                    slurm: None,
                    ssh_path: None,
                },
                dirs: None,
            },
            MgmtConfig {
                include_dir_mgmt: true,
                include_slurm: false,
                ..Default::default()
            },
            (true, false, true),
        );
        // only cli override
        assert_case(
            OnWhichSystemCli {
                ldap_slurm: OnSlurmLdapOnlyCli {
                    ldap: Some(false),
                    slurm: Some(false),
                    ssh_path: None,
                },
                dirs: Some(true),
            },
            MgmtConfig {
                ..Default::default()
            },
            (false, false, true),
        );

        fn assert_case(
            from_cli: OnWhichSystemCli,
            from_config: MgmtConfig,
            expected: (bool, bool, bool),
        ) {
            let actual = OnWhichSystem::from_config_for_all(&from_config, &from_cli);
            let (actual_ldap, actual_slurm, actual_dirs) =
                (actual.ldap(), actual.slurm(), actual.dirs());
            assert_eq!(
                expected,
                (actual_ldap, actual_slurm, actual_dirs),
                "(ldap, slurm, dirs) !"
            );
        }
    }
}
