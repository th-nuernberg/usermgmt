![Build](https://github.com/th-nuernberg/usermgmt/actions/workflows/build.yml/badge.svg) ![Release](https://github.com/th-nuernberg/usermgmt/actions/workflows/release.yml/badge.svg)

# Simultaneous User Management for LDAP and Slurm

The [Slurm](https://slurm.schedmd.com/overview.html) cluster management system lacks direct LDAP integration, which can make user management quite cumbersome. 
Slurm is not automatically aware of any users in the system and what their resource limits in the cluster should be. 
Hence, a new user must be added to the LDAP instance and the Slurm database, which requires double bookkeeping and is error-prone (e.g. user might exist in Slurm but has been deleted in LDAP or vice versa). 

Ideally, the LDAP instance is the single source of truth for what individual users are able to do on the system and even configurations specific to Slurm (e.g. resource limits) should be managed via LDAP. 

This application allows for the simultaneous creation, modification, and deletion of LDAP and Slurm entities. 
Under the hood the `ldap3` client library is used to manage users in LDAP and Slurm's `sacctmgr` utility is called as a subprocess to add/modify/delete users in Slurm. 

Additionally, this application allows for the creation of various directories on the cluster (e.g. home, nfs share etc.) and sets the appropriate user quotas for these directories via `setquota`.  

**Note:**

The `usermgmt` application expects an auxiliary LDAP `ObjectClass` (e.g. called `slurmRole`). 
The `ObjectClass` must unlock access to several `AttributeTypes` that can be used to manage Slurm-specific things like quality-of-service (QOS). 

Currently, `usermgmt` expects the following `AttributeTypes` to present in your LDAP instance:

- `slurmDefaultQos`: Specifies the user's default QOS. Can only exist once per user. 
- `slurmQos`: Specifies the QOS available to the user. Can be added multiple times to a specific user. 

## Requirements

### LDAP

The LDAP instance needs an [auxiliary ObjectClass](https://ldap.com/object-classes/) (e.g. called `slurmRole`), which provides the [AttributeTypes](https://ldap.com/attribute-types/) `slurmDefaultQos` and `slurmQos`. 

See documentations like [this](https://www.gurkengewuerz.de/openldap-neue-schema-hinzufuegen/?cookie-state-change=1638436473037) or [this](https://www.cyrill-gremaud.ch/how-to-add-new-schema-to-openldap-2-4/) for details about the creation of new schemas in LDAP. 

### Slurm
The only dependency to Slurm is the `sacctmgr` ([Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html)), which interacts with the interface provided by `slurmdbd` (Slurm Database Daemon). 
The `sacctmgr` tool should be available on the control host of your cluster. 

You need to point to the `sacctmgr` binary location in the `/etc/usermgmt/conf.toml` file. 

### Directory management

The application relies on SSH and common commands such as `mkdir` and `setquota` to be available on each target node. 
During execution of the directory management module, you will be prompted for a username and password to establish SSH connections with. 
Note that the provided username and password must be the same on all nodes you want to manage directories for. 

Also make sure that the user can execute the following commands using sudo privileges **without password** (interactive password prompts via SSH are a bit of a hassle):

- `mkdir`
- `chown`
- `setquota`
- `mkhomedir_helper`

One way to accomplish this is by adding these commands to the `/etc/sudoers` file: 

```bash
# /etc/sudoers
username ALL = (root) NOPASSWD: /usr/bin/mkdir
username ALL = (root) NOPASSWD: /usr/bin/chown
username ALL = (root) NOPASSWD: /usr/sbin/setquota
username ALL = (root) NOPASSWD: /usr/sbin/mkhomedir_helper
```

Replace `username` by the user you want to execute the commands with and make sure the paths to the executables are correct.  

**Note:** Use `sudo visudo` to change the sudoers file!

## Build and Install 🦀 

You can build the `usermgmt` tool using Cargo:

```bash
cargo build
```

The following examples show how you can run the program with Cargo: 

```bash
# Show available arguments
cargo run -- --help

# Add a user
cargo run -- add teststaff123 --group staff --firstname Martina --lastname Musterfrau --publickey key.pub

# Modify user
cargo run -- modify teststaff123 -f Martha -m bla@blubb.de -d interactive

# Delete user
cargo run -- delete teststaff123

# Run with different log-level
# Available are: error, warn, info, debug, and trace. 
# Error represents the highest-priority log messages and trace the lowest. 
# The default is info
RUST_LOG=warn cargo run -- delete teststaff123

# Add user in LDAP only
cargo run -- --ldap-only add teststaff123 --group staff --firstname Martina --lastname Musterfrau
```

### Create Debian Package

We use [cargo-deb](https://github.com/kornelski/cargo-deb) to automatically create a Debian package for production usage. 

The package creation and installation steps are listed below:

```bash
# Install cargo-deb
cargo install cargo-deb
# Create Debian package in Debian package target/debian/<project_name>_<version>_<arch>.deb
cargo deb
# Install package
dpkg -i target/debian/*.deb
# For previously installed packages, don't forget to update your conf.toml,
# in case there were config changes
cp conf.toml /etc/usermgmt
```

## Configuration

A basic configuration file (`conf.toml`) will be created upon first start of the application. 
In development mode, the file can be found at `./conf.toml`, in release mode the file will be located at `/etc/usermgmt/conf.toml`. 
You can savely modify the file after its initial creation. 

The `conf.toml` file looks as follows:

```toml
# Default value of the Slurm default QOS for the student group
student_default_qos = 'basic'
# Default value of the Slurm default QOS for the staff group
staff_default_qos = 'advanced'
# Default value of the Slurm QOS for the student group
student_qos = [
    'interactive',
    'basic',
    'gpubasic',
]
# Default value of the Slurm QOS for the staff group
staff_qos = [
    'interactive',
    'advanced',
    'gpubasic',
]
# A list of QOS against which user inputs are validated. 
# Note that the values set here must also exist as actual QOS in Slurm. 
valid_qos = [
    'interactive',
    'basic',
    'advanced',
]
# A list of groups against which user inputs are validated. 
# Note that the values set here must also exist as actual Accounts in Slurm. 
valid_slurm_groups = [
    'staff',
    'student',
]
# Common object class values each user entity in LDAP needs to have
objectclass_common = [
    'inetOrgPerson',
    'ldapPublicKey',
    'organizationalPerson',
    'person',
    'posixAccount',
    'shadowAccount',
    'slurmRole',
    'top',       
]
# List of compute nodes on your cluster
# Will be used to create user directories on local disks
compute_nodes = [
    'machine.test.de',
]
# Gid of the student group
student_gid = 1002
# Gid of the staff group
staff_gid = 1001
# Gid of the faculty group 
# (faculty users will be treated the same as the staff group in Slurm)
faculty_gid = 1000
# Path to sacctmgr binary
sacctmgr_path = '/usr/local/bin/sacctmgr'
# Domain components used for LDAP queries
# Will be used in combination with ldap_org_unit 
# and the cn of the username you provided for ldap login
ldap_domain_components = 'cn=department,dc=company,dc=com'
# Default login shell for the user
login_shell = '/bin/bash'
# Organizational unit in LDAP used to apply operations under
# This value is combined with ldap_domain_components like
# 'ou={ldap_org_unit},{ldap_domain_components}'
ldap_org_unit = 'people'
# Protocol, host and port of your LDAP server
ldap_server = 'ldap://<hostname>:<port>'
# Default user for SSH login during directory management. 
# You can always enter a different username during application runtime
default_ssh_user = 'serveradmin'
# Hostname of the server that provides the home directories
# Assumes that a single host is responsible for home directories 
# and that they are shared via nfs
home_host = 'home.server.de'
# Hostname of an nfs server to be used by cluster users
nfs_host = 'nfs.server.de'
# Root directory of the shared folders on the nfs host
nfs_root_dir = '/mnt/md0/scratch'
# Root directory of user folders on each compute node
# (must be the same on each node)
compute_node_root_dir = '/mnt/md0/user'
# Filesystem (or mountpoint) under which user quotas are to be set on the compute nodes
filesystem = '/mnt/md0'
# Filesystem (or mountpoint) under which user quotas on the user's home directory are to be set
home_filesystem = '/dev/sdb4'
# Filesystem (or mountpoint) under which user quotas are to be set on the NFS
nfs_filesystem = '/dev/sda1'
# Quota softlimit on compute nodes
quota_softlimit = '200G'
# Quota hardlimit on compute nodes
quota_hardlimit = '220G'
# Quota softlimit on nfs
quota_nfs_softlimit = '200G'
# Quota hardlimit on compute nfs
quota_nfs_hardlimit = '220G'
# Quota softlimit on user home
quota_home_softlimit = '20G'
# Quota hardlimit on user home
quota_home_hardlimit = '22G'
# Use the directory management module of the application 
# Note that this is somewhat experimental and quite specific to 
# the THN cluster and therefore might not be suitable for 
# other cluster environments
include_dir_mgmt = true
# Use the mkhomedir_helper tool to create the user home 
# directory (recommended). When false, the directory will 
# be created using mkdir and no skeleton configs (e.g. .bashrc) will be copied
use_homedir_helper = true
```

The values for `student_default_qos`, `staff_default_qos`, `student_qos`, and `staff_qos` will be used when `--default-qos` and `--qos` are not explicitely set. 

## Usage

The following examples show the basic usage of the `usermgmt` tool:
```bash
# Show available arguments
usermgmt --help
# Show help for modify subcommand
usermgmt modify --help
# Add a user
usermgmt add teststaff123 --group staff --firstname Martina --lastname Musterfrau
# Modify user
usermgmt modify teststaff123 --firstname Martha --mail bla@blubb.de --default-qos interactive
# Delete user
usermgmt delete teststaff123
```

### Log-level
The log-level can be changed using the `RUST_LOG` environment variable. 
Available log-levels are *error*, *warn*, *info*, *debug*, and *trace*. 
*Error* represents the highest-priority log messages and *trace* the lowest. 
The default log-level is *info*. 
You'll receive the most verbose output when you set it to *debug*. 

```bash
# Delete user with log-level debug
RUST_LOG=debug usermgmt delete teststaff123
```

### Adding Users

The uid integer value will be automatically determined based on the `--group` parameter provided. 
Currently you can choose between the two groups *staff* and *student*. 

The uid for a new user will be determined based on the following rules:
- Uids for *staff* start with 1000
- Uids for *student* start with 10000
- The uid will be 1 plus the highest uid currently present in LDAP

The gids are determined based on the string provided in `--group` using the values in `conf.toml`. 
Therefore, a gid for each valid group must be present in the `/etc/usermgmt/conf.toml` file. 

When no `--default-qos` or `--qos` parameter is set, the default values provided in the `/etc/usermgmt/conf.toml` file will be used based on the `--group` parameter given. 

### Modifying Users

A list of modifiable values can be obtained via `usermgmt modify --help`.  

### Deleting Users

User can be deleted via `usermgmt delete <username>`.  

## Pitfalls 

Make sure you execute the `usermgmt` tool with a user who has **administrative rights** for `sacctmgr`. 
You can check available users and their admin level via `sacctmgr list user`. 

When you attempt LDAP operations, you will be prompted for a username and a password. 
Make sure the user has sufficient rights to add, modify, and delete entities in LDAP. 

## External Dependencies

- [Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html) as part of slurmdbd. Make sure this is installed on the host you're executing this tool from. 

## Release

You need to include the current version of your release in the `CHANGELOG.md` because the [github action](https://github.com/taiki-e/upload-rust-binary-action) picks it up from there. 

Also change the version number in `Cargo.toml` and `src/cli.rs` to keep everything consistent. 

To add a release, you need to tag the branch with the current version and then push the tag:

```bash
git tag <version> main

git push origin <version>
```

`<version>` is the version of your release (e.g. `v0.3.0`).  

### Build for Mac M1


```bash
cargo build --release --target aarch64-apple-darwin
cp README.md target/aarch64-apple-darwin
cp README.md target/aarch64-apple-darwin
cd target/aarch64-apple-darwin

```

## Todo

- Add functionality to delete directories
- Run sacctmgr commands remotely so we can use this as a client application
