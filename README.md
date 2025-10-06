![Checks](https://github.com/th-nuernberg/usermgmt/actions/workflows/check.yml/badge.svg) ![Release](https://github.com/th-nuernberg/usermgmt/actions/workflows/release.yml/badge.svg)

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

## Project Structure

This project consists of three crates:

- [`usermgmt`](./usermgmt): The CLI tool for simultaneous user management for LDAP and Slurm.
- [`usermgmt_gui`](./usermgmt_gui): The GUI frontend for simultaneous user management for LDAP and Slurm.
- [`usermgmt_lib`](./usermgmt_lib): Shared code between the binaries, `usermgmt` and `usermgmt_gui`.

## Requirements

### LDAP

The LDAP instance needs an [auxiliary ObjectClass](https://ldap.com/object-classes/) (e.g. called `slurmRole`), which provides the [AttributeTypes](https://ldap.com/attribute-types/) `slurmDefaultQos` and `slurmQos`. 

See documentations like [this](https://www.gurkengewuerz.de/openldap-neue-schema-hinzufuegen/?cookie-state-change=1638436473037) for details about the creation of new schemas in LDAP. 

### Slurm

The only dependency to Slurm is the `sacctmgr` ([Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html)), which interacts with the interface provided by `slurmdbd` (Slurm Database Daemon). 
The `sacctmgr` tool should be available on the control host of your cluster. 

You need to point to the `sacctmgr` binary location in the `/etc/usermgmt/conf.toml` file. 

### Directory Management

The application relies on SSH and common commands such as `mkdir` and `setquota` to be available on each target node. 
During execution of the directory management module, you will be prompted for a username and password to establish SSH connections with. 
Note that the provided username and password must be the same on all nodes you want to manage directories for. 

Also make sure that the user can execute the following commands using sudo privileges **without password** (interactive password prompts via SSH are a bit of a hassle):

- `mkdir`
- `chown`
- `rm`
- `setquota`
- `mkhomedir_helper`

One way to accomplish this is by adding these commands to the `/etc/sudoers` file: 

```bash
# /etc/sudoers
username ALL = (root) NOPASSWD: /usr/bin/mkdir
username ALL = (root) NOPASSWD: /usr/bin/rm
username ALL = (root) NOPASSWD: /usr/bin/chown
username ALL = (root) NOPASSWD: /usr/sbin/setquota
username ALL = (root) NOPASSWD: /usr/sbin/mkhomedir_helper
```

Replace `username` by the user you want to execute the commands with and make sure the paths to the executables are correct.  

**Note:** Use `sudo visudo` to change the sudoers file!

## Build and Install 🦀 

You can build the `usermgmt` CLI tool using Cargo (install the Rust toolchain via [rustup](https://rustup.rs/) first):

```bash
# Maybe:
rustup update

cargo build
```

The following examples show how you can run the program with Cargo: 

```bash
# Show available arguments
cargo cli --help

# Add a user
cargo cli add teststaff123 --group staff --firstname Martina --lastname Musterfrau --publickey key.pub

# Modify user
cargo cli modify teststaff123 -f Martha -m bla@blubb.de -d interactive

# Delete user
cargo cli delete teststaff123

# List users in LDAP and Slurm
cargo cli list

# Run with different log-level
# Available are: error, warn, info, debug, and trace. 
# Error represents the highest-priority log messages and trace the lowest. 
# The default is info
RUST_LOG=warn cargo cli -- delete teststaff123

# Add user in LDAP only
cargo cli --ldap true --slurm false --dirs false add teststaff123 --group staff --firstname Martina --lastname Musterfrau

# Specify key pair for SSH connection
# Example of listing all users while using key pair at "~/.ssh/some_user.pub" and "~/.ssh/some_user"
cargo cli list --ssh-path "~/.ssh/some_user"

```

### Install From Source

To directly install from source, you must install the Rust toolchain locally, which can be installed via [rustup](https://rustup.rs/).

Install the CLI version

```bash
cd usermgmt
cargo install --path '.' --force
```

Install the GUI version

```bash
cd usermgmt_gui
cargo install --path '.' --force
```

- `cargo install` normally installs a crate binary into `~/.cargo/bin`
- `--path '.'` installs from the local crate at the current directory instead of from `crates.io`. 
- `--force` overwrites any existing installation of the same binary. 

### Create Debian Package

We use [cargo-deb](https://github.com/kornelski/cargo-deb) to automatically create a Debian package for production usage. 

The package creation and installation steps are listed below:

```bash
# Install cargo-deb
cargo install cargo-deb
# Go into project for the CLI tool.
cd usermgmt
# Create Debian package in Debian package target/debian/<project_name>_<version>_<arch>.deb
cargo deb
# Install package
dpkg -i ../target/debian/*.deb
# Don't forget to update your conf.toml, in case there have been config changes
```

### Install Prebuilt Binary

You can use the [`install.sh`](install.sh) script to download and install one of the prebuilt binaries provided under [releases](https://github.com/th-nuernberg/usermgmt/releases). 
Check `install.sh --help` for usage infos. 

## GUI 

We also provide an experimental GUI version.
More information can be found under the [GUI README](./usermgmt_gui/README.md).

You can start the GUI version via the following command:

```bash
cargo gui
```

## Configuration

### Configuration File Location

A configuration file (`conf.toml`) is loaded during runtime, determining most of the behaviour. 
The program searches for the configuration file in several places.
The configuration file that is first found is loaded.
The search is conducted in the following order: 

- OS-specific configuration locations.
  - Config location depending on the OS (see [here](https://docs.rs/dirs/latest/dirs/fn.config_dir.html))
    The name of the folder is `usermgmt` within the config location.
  -  Mac/Linux: `~/.usermgmt` 
- OS-specific system-wide locations:
  - Linux: `/usr/usermgmt`
- The `CWD` as the last resort 

Note that the configuration file is not created automatically!
However, you can use the `generate-config` command to generate a default configuration file:

```sh
usermgmt generate-config > /home/foo/conf.toml
```

### Content of Configuration File

The `conf.toml` file looks as follows:

```toml
# If true, a timestamp is created in LDAP, showing the creation date of the user entry. 
# Make sure to also include the field `createdAtRole` in the array "`objectclass_common`" in the config file. 
# The timestamp is saved in the format of RFC 3339 (https://www.rfc-editor.org/rfc/rfc3339) with the UTC time zone (e.g. 2024-05-09T10:49:34.545686277+00:00)
ldap_add_created_at = true
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
    'createdAtRole',
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
# LDAP username used by default if no username for LDAP login is provided.
ldap_default_user = 'admin'
# Domain components used for LDAP queries
# Will be used in combination with ldap_org_unit 
# and the cn of the username you provided for ldap login
ldap_domain_components = 'cn=department,dc=company,dc=com'
# Default login shell for the user
login_shell = '/bin/bash'
# Organizational unit in LDAP used to apply operations under
# This value is combined with ldap_domain_components like
# Is optional and can be omitted.
# '[ldap_org_unit,]{ldap_domain_components}'
ldap_org_unit = 'people'
# User bind prefix to be used when establishing LDAP connections. 
# Binding goes like: cn=admin... or uid=someuser...
ldap_bind_prefix = 'cn'
# Use a different OU for establishing connections to you LDAP server
# Is optional and can be omitted.
# The resulting ldap path for logging is: {ldap_bind_prefix}=<ldap_user_name>,[ldap_bind_prefix,][ldap_domain_components] 
ldap_bind_org_unit = 'ou=people'
# Protocol, host and port of your LDAP server
ldap_server = 'ldap://<hostname>:<port>'
# Read only user for ldap search queries (e.g. usermgmt list ldap)
# Is optional and can be omitted.
ldap_readonly_user = 'readonlyuser'
# Read only user password
# Is optional and can be omitted.
ldap_readonly_pw = 'secret'
# Can be used for connection with read access only
# Is optional and can be omitted.
# User bind prefix to be used when establishing LDAP connections. 
# Binding goes like: cn=admin... or uid=someuser...
ldap_readonly_user_prefix = "read_only_uid"
# Can be used for connection with read access only
# Is optional and can be omitted.
# The resulting ldap path for logging is: {ldap_bind_prefix}=<ldap_user_name>,[ldap_bind_prefix,][ldap_domain_components] 
ldap_readonly_bind = "ou=readonly,ou=realm"
# Default user for SSH login during directory management. 
# You can always enter a different username during application runtime
default_ssh_user = 'serveradmin'
# Hostname of the server that provides the home directories
# Assumes that a single host is responsible for home directories 
# and that they are shared via NFS
home_host = 'home.server.de'
# Hostnames of NFS servers
nfs_host = ['nfs.server.de']
# Slurm head node (where sacctmgr is installed)
# Required when run_slurm_remote=true
head_node = 'head.node.de'
# Root directories of the shared folders on the NFS hosts
nfs_root_dir = ['/mnt/md0/scratch']
# Root directory of user folders on each compute node
# (must be the same on each node)
compute_node_root_dir = '/mnt/md0/user'
# Filesystem (or mountpoint) under which user quotas are to be set on the compute nodes
filesystem = '/mnt/md0'
# Filesystem (or mountpoint) under which user quotas on the user's home directory are to be set
home_filesystem = '/dev/sdb4'
# Filesystems (or mountpoints) under which user quotas are to be set on the NFS
nfs_filesystem = ['/dev/sda1']
# Quota softlimit on compute nodes
quota_softlimit = '200G'
# Quota hardlimit on compute nodes
quota_hardlimit = '220G'
# Quota softlimit on NFS hosts
quota_nfs_softlimit = ['200G']
# Quota hardlimit on NFS hosts
quota_nfs_hardlimit = ['220G']
# Quota softlimit on user home
quota_home_softlimit = '20G'
# Quota hardlimit on user home
quota_home_hardlimit = '22G'
# Create/delete/modify user on the Slurm database by default
# Can be overridden via CLI option for a command
include_slurm = true
# Create/delete/modify user on the LDAP database by default
# Can be overridden via CLI option for a command
include_ldap = true
# Use the directory management module of the application 
# Note that this is somewhat experimental and quite specific to 
# the THN cluster and therefore might not be suitable for 
# other cluster environments
include_dir_mgmt = true
# Use the mkhomedir_helper tool to create the user home 
# directory (recommended). When false, the directory will 
# be created using mkdir and no skeleton configs (e.g. .bashrc) will be copied
use_homedir_helper = true
# Execute Slurm commands from a remote client via SSH or directly on the server
run_slurm_remote = true
# Port to be used when connecting via ssh to any node
ssh_port = 22
# If true, the application will try to authenticate via SSH agent before the simple password authentication
ssh_agent = false
# Path to SSH key pair to be used if no SSH agent is used.
# Path points to base name of the private and public key. 
# Example: For private key ~/.ssh/some_key_pair, there should be a corresponding public key "~/.ssh/some_key_pair.pub"
ssh_key_path = "~/.ssh/some_key_pair"
```

The values for `student_default_qos`, `staff_default_qos`, `student_qos`, and `staff_qos` will be used when `--default-qos` and `--qos` 
are not explicitly set. 

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

### Adding Users

The uid integer value will be automatically determined based on the `--group` parameter provided. 
Currently, you can choose between the two groups *staff* and *student*. 

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

A user can be deleted via `usermgmt delete <username>`.  

## Tips and Advanced Usage

### Add User Creation Date to LDAP

To preserve the backwards compatibility with earlier versions, this features must be opted in.

Set up the use of creation dates in LDAP via:

1. Set the field value `ldap_add_created_at` to `true` in `conf.toml`.
2. Add the value `createdAtRole` to `objectclass_common` in `conf.toml`.

### Use SSH Agent for Authentication 

To save yourself from entering passwords for SSH authentication over and over again, 
you can let the application use a running SSH agent. 

Activate this feature via setting the field `ssh_agent` to `true` in `conf.toml`.

Start your SSH agent in the terminal via the command (often started automatically during system boot):

```sh
ssh-agent
```

Add your private SSH key to the agent. 
You might be asked for the password to decrypt this key if a password was set during creation of the key pair. 

```sh
ssh-agent <path_to_private_ssh_key>
```

If authentication over SSH is requested, the application will try to use one of your keys registered within the SSH agent and does not ask for a password.
If more than one key is registered within the agent, you will be prompted to select the key you want to use.

**Additional optional steps on macOS:**

Ensure macOS remembers the key (optional, via Keychain): 

```sh
ssh-add --apple-use-keychain <path_to_private_ssh_key>
```

Persist keys across reboots (add to config) by editing `~/.ssh/config`:

```sh
Host *
  UseKeychain yes
  AddKeysToAgent yes
  IdentityFile <path_to_private_ssh_key>
```
This integrates with the macOS keychain and ensures the key is added automatically. 


### Show More Log Output

The log-level can be changed using the `RUST_LOG` environment variable. 
Available log-levels are *error*, *warn*, *info*, *debug*, and *trace*. 
*Error* represents the highest-priority log messages and *trace* the lowest. 
The default log-level is *info* during production. During development the default is debug. 
You'll receive the most verbose output when you set it to *debug*. 

```bash
# Delete user with log-level debug
RUST_LOG=debug usermgmt delete teststaff123
```

Logs are also written to the data folder of the application according to OS-specific conventions.
If this is not possible, it tries to log to the location of the executable.
If this also fails, the application only writes to the terminal via stderr.
See the [docs](https://docs.rs/dirs/latest/dirs/fn.data_dir.html) for details.

### Show Stacktrace

Errors reported by the application, can be displayed including their stack trace.
Since the stack trace is disabled by default, you need to set the environment variable `RUST_BACKTRACE=1`.

## Pitfalls 

Make sure you execute the `usermgmt` tool with a user who has **administrative rights** for `sacctmgr`. 
You can check available users and their admin level via `sacctmgr list user`. 

When you attempt LDAP operations, you will be prompted for a username and a password. 
Make sure the user has sufficient rights to add, modify, and delete entities in LDAP. 

## External Dependencies

- [Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html) as part of slurmdbd. Make sure this is installed on the host you're executing this tool from. 

## Release

You need to include the current version of your release in the `CHANGELOG.md` because the [github action](https://github.com/taiki-e/upload-rust-binary-action) picks it up from there. 

Also change the version numbers in `Cargo.toml` files at [`./usermgmt`](./usermgmt/Cargo.toml), [`./usermgmt_lib`](./usermgmt_lib/Cargo.toml), and [`./usermgmt_gui`](./usermgmt_gui/Cargo.toml) to keep everything consistent. 

To add a release, you need to tag the branch with the current version and then push the tag:

```bash
# Maybe:
rustup update
# Make sure the project is formatted properly so the git workflow doesn't fail
cargo fmt --all
cargo clippy --fix --all
# Verify with: 
cargo fmt --check --all
cargo clippy --all

git add .
git commit -m "my commit"
git tag <version>

git push <version>
```

`<version>` is the version of your release (e.g. `v0.6.0`). 

**Note:** Don't forget to merge into main and push your commits to main. 

### Build for Mac M1

You can build the application natively on a Mac M1:

```bash
cargo build --release --target aarch64-apple-darwin
```

If you want to make a release file similar to the ones created automatically via the github action do:

```bash
cp README.md target/aarch64-apple-darwin/release
cp LICENSE target/aarch64-apple-darwin/release

cd target/aarch64-apple-darwin/release
tar -cvzf usermgmt-aarch64-apple-darwin.tar.gz usermgmt README.md LICENSE
tar -cvzf usermgmt_gui-aarch64-apple-darwin.tar.gz usermgmt_gui README.md LICENSE
```

Finally, upload the `.tar.gz` files to [releases](https://github.com/th-nuernberg/usermgmt/releases). 

## Local Development with Docker

Development of this app can be done locally via a docker container set up. As moment of writing 
the functionality for LDAP and Slurm can be used via docker. Directory management does not work in
docker. Read this [Readme](./docker/README.md) for how to set up local development via docker.

## Changelogs

The [changelog](CHANGELOG.md) 
at the project root lists all changes for the `usermgmt_lib` and `usermgmt` crate.
Changes to the GUI version are documented in its own [changelog](./usermgmt_gui/CHANGELOG.md)

## License

This project is licensed under [MIT](./LICENSE)

