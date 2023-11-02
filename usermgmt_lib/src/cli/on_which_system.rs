use clap::Args;
use getset::CopyGetters;

use crate::config::MgmtConfig;
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

#[derive(Args, CopyGetters, Debug)]
#[getset(get_copy = "pub")]
pub struct OnSlurmLdapOnlyCli {
    /// If true then the action will be performed on Slurm too, otherwise nothing happens on Slurm.
    /// Overrides the default which is provided by the conf.toml
    #[clap(long, verbatim_doc_comment)]
    slurm: Option<bool>,
    /// If true then the action will be performed on LDAP too, otherwise nothing happens on Ldap.
    /// Overrides the default which is provided by the conf.toml
    #[clap(long, verbatim_doc_comment)]
    ldap: Option<bool>,
}

/// Information on which systems an action like creating an user should happen.
/// Ensures flexibility for user to toggle systems via CLI and config
/// CLI option have priority over default values from conf.toml
#[derive(CopyGetters, Debug)]
#[getset(get_copy = "pub")]
pub struct OnWhichSystem {
    slurm: bool,
    ldap: bool,
    dirs: bool,
}

impl OnWhichSystem {
    pub fn new(slurm: bool, ldap: bool, dirs: bool) -> Self {
        Self { slurm, ldap, dirs }
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
        }
    }

    fn use_cli_over_config(cli: Option<bool>, config_val: bool) -> bool {
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
