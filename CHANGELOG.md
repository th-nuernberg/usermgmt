# Changelog
All changes to the CLI tool and library of the usermgmt project.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Changelog for binary "usermgmt" and library "usermgmt_lib".

## [Unreleased]

### Added

- Added conf field "ldap_add_created_at" to opt in for creating LDAP users with creation date
  Note: the field value field "ldapAttributes" under the array field "objectclass_common" is needed
  within the config file. Otherwise one gets an error while creating an user within LDAP""
- Added configuration field "ldap_default_user". Allows to define username for LDAP login used by default.
- Added CLI option to specify the configuration file
- Logging also performed to logging file
- Ssh key pair can be provided by field within configuration file or as a CLI argument.

### Changed

- In Cli, connection to LDAP and Slurm is checked before any attempt at performing adding/removing/listing or modifying/listing.
- In CLI, user can just press enter to provide no password during password prompt for ssh. 
  Useful for ssh key pair without password.

## [0.1.0] - 2022-06-20

### Added

- Usermanagement via subprocess calls to [LDAPUtils](https://wiki.debian.org/LDAP/LDAPUtils) and `sacctmgr`

## [0.2.0] - 2022-10-29

### Added
- Publickeys can now be added and modified via the CLI

### Changed
- Switch from calling [LDAPUtils](https://wiki.debian.org/LDAP/LDAPUtils) to native [ldap3](https://docs.rs/ldap3/latest/ldap3/) library (code looks much cleaner and there is no need for LDAP Utils being installed anymore)
- Fix typos in log output and readme
- Use `rustfmt` and `clippy`

### Removed
- `LDAPUtils` dependency

## [0.3.0] - 2022-10-30

### Added
- Experimental directory management functionalities that include creating directories on NFS and compute nodes and setting quotas 

## [0.3.3] - 2022-10-31

### Added
- Add changelog and github actions

## [0.3.4] - 2022-10-31

### Added
- Add `aarch64-apple-darwin` to release
- Add `LICENSE` and `README.md` to release

## [0.3.5] - 2022-10-31

### Added
- Add `aarch64-unknown-linux-gnu` to release

## [0.3.6] - 2022-10-31

### Added
- Cross compile `aarch64-apple-darwin` release

## [0.3.7] - 2022-10-31

### Added
- Add cross-compilation tools

## [0.3.8] - 2022-10-31

### Removed
- Cross-compilation tools

## [0.3.9] - 2022-10-31

### Removed
- Broken `aarch64-apple-darwin` target (missing openssl lib)

## [0.3.10] - 2022-10-31

### Removed
- Broken `aarch64-unknown-linux-gnu` target (missing openssl lib)

## [0.4.0] - 2022-11-01

### Added
- Slurm remote execution via SSH 
- Options for listing users in Slurm and/or LDAP

### Changed
- Fix for broken `--slurm-only` and `--ldap-only` flags during user creation
- Config file location now depends on the target OS, as well as debug assertions

## [0.4.1] - 2022-11-01

### Changed
- Ask credentials during LDAP search, when no readonly user or password is supplied via config

## [0.4.5] - 2022-11-14

### Changed
- Adjust directory creation on NFS from `/nfs/scratch` to `/nfs/scratch/<students|staff>` 

## [0.4.6] - 2023-03-15

### Fixed
- Fix for [#13](https://github.com/th-nuernberg/usermgmt/issues/13)

### Added
- New config parameters `ldap_bind_prefix` and `ldap_bind_org_unit` to allow more flexibility regarding user binding for establishing LDAP connections
- Unit tests by @BoolPurist

### Changed
- Improved listing of Slurm users. It now executes `sacctmgr show assoc format=User%30,Account,DefaultQOS,QOS%80`.  
- Various improvements by @BoolPurist

## [0.4.7] - 2023-03-15

### Changed
- Try M1 release build

## [0.4.8] - 2023-03-16

### Fixed

- Fix for SSH credential reuse by @BoolPurist ([#8](https://github.com/th-nuernberg/usermgmt/issues/8))
- Warning for missing ssh public key is only shown if LDAP is involved, since it is only used in LDAP.
- Command "add" respects options "--ldap-only", "--slurm-only" and "--dirs-only" now.

### Changed
- Order of Slurm QOS modification during user creation
- Pick default QOS from `conf.toml` when no value is provided and remove default value from CLI


## [0.5.0] - 2023-04-19

### Changed

- Output of for LDAP user via the sub command list is now presented in a nicer ASCII table format
- CLI argument for slurm, ldap and directory management can be toggled via cli and conf.toml individually CLI Option for a system is only available if a sub command supports it. slurm-only, ldap-only and dirs-only were replaced by new options
- LDAP bind OU and general LDAP OU are more configurable.
  One can now specify several dn parts like ou=people,ou=department.
  Before one could only specify a value for one fixed dn key part like people => ou=people.

### Added

- Added possibility to output listing of LDAP user via pretty table format and the old machine-readable format via CLI flag in subcommand list
- Used base dc, dn for user manipulation and dn for user bind are now logged.
- Port for ssh connection can now specified via configuration file

## [0.6.0] - 2023-11-08

### Changed

- Configuration file is not created automatically anymore
- Username and passwords which are empty or only white spaces and have default value are invalid 
- Output of for LDAP user via the sub command list is now presented in a nicer ASCII table format
- CLI argument for slurm, ldap and directory management can be toggled via cli and conf.toml individually
  CLI Option for a system is only available if a sub command supports it.
  slurm-only, ldap-only and dirs-only were replaced by new options
- LDAP bind OU and general LDPA OU are more configurable. 
  One can now specify several dn parts like ou=people,ou=department.
  Before one could only specify a value for one fixed dn key part like people => ou=people.

### Added

- Subcommands can be issued via shortcuts which consists of one or two letters.
- Configuration file can be located in several places. 
  It always try user specific configuration locations, then system configuration places and the CWD as the last resort.
- Default configuration can be generated via new subcommand "generate-config".
- Option to try authentication via ssh agent before simple password authentication.
- Added option to use different user prefix and org unit binding for readonly LDAP user.
- Made username and password optional for readonly user to allow for prompting these credentials.
- Used base dc, dn for user manipulation and dn for user bind are now logged.
- Port for ssh connection can now specified via configuration file

### Fixed

- CLI tool now enforces the following:
  - Default Qos must be contained in the collection of Qos for the operations add, remove and modify
  - Default Qos along with the collection of Qos must be given both or none of them for the operations add, remove and modify.
- This prevents the problem where sacctmgr CLI, slurm CLI, did not update default Qos or collection of Qos.
- The prompt showing allowed number for selection for several keys under ssh agent is now correctly displayed. 
  Before the maximum index selection was too big by one.
- Fixed faulty determining inside os config folder space. 
  Example: before It searched under ~/.config instead of ~/.config/usermgmt

[0.6.0]: https://github.com/th-nuernberg/usermgmt/releases/tag/v0.6.0 
