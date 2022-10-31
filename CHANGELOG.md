# Changelog
All changes to this project.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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