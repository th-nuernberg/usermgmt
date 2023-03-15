# Changelog
All changes to this project.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Warning for missing ssh public key is only shown if LDAP is involved, since it is only used LDAP.
- Command "add" respects options "--ldap-only", "--slurm-only" and "--dirs-only" now.

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