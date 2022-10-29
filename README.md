# Simultaneous User Management for LDAP and Slurm

The [Slurm](https://slurm.schedmd.com/overview.html) cluster management system lacks direct LDAP integration, which can make user management quite cumbersome. 
Slurm is not automatically aware of any users in the system and what their resource limits in the cluster should be. 
Hence, a new user must be added to the LDAP instance and the Slurm database, which requires double bookkeeping and is error-prone (e.g. user might exist in Slurm but has been deleted in LDAP or vice versa). 

Ideally, the LDAP instance is the single source of truth for what individual users are able to do on the system and even configurations specific to Slurm (e.g. resource limits) should be managed via LDAP. 

This application allows for the simultaneous creation, modification, and deletion of LDAP and Slurm entities. 
Under the hood is a simple wrapper around the `ldapUtils` package and Slurm's `sacctmgr` utility, which are called as subprocesses. 

The `usermgmt` application expects an auxiliary LDAP `ObjectClass` called `slurmRole`. 
The `ObjectClass` unlocks access to several `AttributeTypes` that can be used to manage Slurm-specific things like quality-of-service (QOS). 

Currently the following `AttributeTypes` are supported:

- `slurmDefaultQos`: Specifies the user's default QOS. Can only exist once per user. 
- `slurmQos`: Specifies the QOS available to the user. Can be added multiple times to a specific user. 

## Requirements
The `usermgmt` application interacts with LDAP via the [ldap-utils](https://wiki.debian.org/LDAP/LDAPUtils) package and calls `sacctmgr` ([Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html)) for Slurm user management. 
Hence, you'll need an LDAP instance up and running, have the `ldap-utils` package installed and have the Slurm cluster management system up and running as well.

### LDAP
The LDAP instance needs an [auxiliary ObjectClass](https://ldap.com/object-classes/) called `slurmRole`, which provides the [AttributeTypes](https://ldap.com/attribute-types/) `slurmDefaultQos` and `slurmQos`. 

See documentations like [this](https://www.gurkengewuerz.de/openldap-neue-schema-hinzufuegen/?cookie-state-change=1638436473037) or [this](https://www.cyrill-gremaud.ch/how-to-add-new-schema-to-openldap-2-4/) for details about the creation of new schemas in LDAP. 

### Slurm
The only dependency to Slurm is the `sacctmgr` ([Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html)), which interacts with the interface provided by `slurmdbd` (Slurm Database Daemon). 
The `sacctmgr` tool should be available on the control host of your cluster. 

You need to point to the `sacctmgr` binary location in the `/etc/usermgmt/conf.toml` file. 

## Build and Install 🦀 

You can build the `usermgmt` tool using Cargo:
```
cargo build
```

The following examples show how you can run the program with Cargo: 

```bash
# Show available arguments
cargo run -- --help
# Add a user
cargo run -- add teststaff123 --group staff --firstname Martina --lastname Musterfrau
# Modify user
cargo run -- modify teststaff123 -f Martha -m bla@blubb.de -d interactive
# Delete user
cargo run -- delete teststaff123
# Run with different log-level
# Available are: error, warn, info, debug, and trace. 
# Error represents the highest-priority log messages and trace the lowest. 
# The default is info
RUST_LOG=warn cargo run -- delete teststaff123
```

### Create Debian Package

We use [cargo-deb](https://github.com/kornelski/cargo-deb) to automatically create a Debian package. 

The package creation and installation steps are listed below:

```bash
# Install cargo-deb
cargo install cargo-deb
# Create Debian package in Debian package target/debian/<project_name>_<version>_<arch>.deb
cargo deb
# Install package
dpkg -i target/debian/*.deb
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
    'ultimate',
    'bigmem',
    'gpubasic',
    'gpuultimate',
]
# A list of groups against which user inputs are validated. 
# Note that the values set here must also exist as actual Accounts in Slurm. 
valid_slurm_groups = [
    'staff',
    'student',
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
ldap_domain_components = 'dc=informatik,dc=fh-nuernberg,dc=de'
# Path where a file called template.ldif will be created. 
# The template is necessary for adding users
ldif_template_path = './ldif'
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

The LDAP user creation routine utilizes the `ldapadd` command, which requires the given parameters to be translated into the [LDAP Data Interchange Format](https://en.wikipedia.org/wiki/LDAP_Data_Interchange_Format) (LDIF). 
See [./ldif/example.ldif](./ldif/example.ldif) for an example on how a user is specified in LDIF format. 

The gids are determined based on the string provided in `--group` using the values in `conf.toml`. 
Hence, a gid for each valid group must be present in the `/etc/usermgmt/conf.toml` file. 

When no `--default-qos` or `--qos` parameter is set, the default values provided in the `/etc/usermgmt/conf.toml` file will be used based on the `--group` parameter given. 

### Modifying Users

The LDAP user modification routine utilizes the `ldapmodify` command, which also requires the translation of commands into LDIF format. 
See [./ldif/example1.ldif](./ldif/example1.ldif) for an example on how modifications are expressed in LDIF format. 

A list of modifiable values can be obtained via `usermgmt modify --help`.  

### Deleting Users

The LDAP user deletion routine utilizes the `ldapdelete` command. 
The `ldapdelete` command deletes users based on their distinguished name (DN). 
The DN for the username provided in `usermgmt delete <username>` is obtained via `ldapsearch`.  

## Pitfalls 

Make sure you execute the `usermgmt` tool with a user who has administrative rights for `sacctmgr`. 
You can check available users and their admin level via `sacctmgr list user`. 

When you attempt LDAP operations, you will be prompted for a username and a password. 
Make sure the user has sufficient rights to add,modify, and delete entities in LDAP. 

## External Dependencies

- [Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html) as part of slurmdbd
- [LDAP Utils](https://wiki.debian.org/LDAP/LDAPUtils)


## New Version

```bash
# sudo visudo
mladm ALL = (root) NOPASSWD: /usr/bin/mkdir
mladm ALL = (root) NOPASSWD: /usr/bin/chown
```


