# Simultaneous User Management for LDAP and Slurm

The [Slurm](https://slurm.schedmd.com/overview.html) cluster management system lacks direct LDAP integration, which can make user management quite cumbersome. 
Slurm is not automatically aware of any users in the system and what their resource limits in the cluster should be. 
Hence, a new user must be added to the LDAP instance and the Slurm database, which leads to double bookkeeping and is error-prune (e.g. user might exist in Slurm but not in LDAP or vice versa). 

Ideally, the LDAP instance is the single source of truth for what individual users are able to do on the system and even configurations specific to Slurm (e.g. resource limits) should be managed via LDAP. 

This commandline tool allows for the simultaneous creation, modification, and deletion of LDAP and Slurm entities. 
Under the hood is a simple wrapper around the ldapUtils package and Slurm's `sacctmgr` utility, which are called as sub-processes. 

The `usermgmt` tool expects an auxiliary LDAP `ObjectClass` called `slurmRole`. 
The `ObjectClass` unlocks access to several `AttributeTypes` that can be used to manage Slurm-specific things like quality-of-service (QOS). 

Currently the following `AttributeTypes` are supported:

- `slurmDefaultQos`: Specifies the user's default QOS. Can only exist once per user. 
- `slurmQos`: Specifies the QOS available to the user. Can be added multiple times to a specific user. 

## Build and Install 🦀 
You can build the `usermgmt` tool using Cargo:
```
cargo build
```

The following examples show how you can run the program with Cargo: 

```shell
# Show available arguments
cargo run -- --help
# Add a user
cargo run -- add teststaff123 --group staff --firstname Martina --lastname Musterfrau
# Modify the user
cargo run -- modify teststaff123 -f Martha -m bla@blubb.de -d interactive
# Delete the user
cargo run -- delete teststaff123
```

The tool is installed as follows:
tbd

## Configuration
tbd

## Usage
tbd


Make sure that you are executing the `usermgmt` tool with a user who has administrative rights for `sacctmgr`. 
You can check available users and their admin level via `sacctmgr list user`. 

## External Dependencies
- [Slurm Account Manager](https://slurm.schedmd.com/sacctmgr.html) as part of Slurmdbd
- [LDAP Utils](https://wiki.debian.org/LDAP/LDAPUtils)
