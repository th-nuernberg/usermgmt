# Docker set up

Do not use this in production ! It is meant to be used for local development.
While it response to the app as a ldap and slurm endpoint, the set up 
is designed for production ready security !

## Limitations

This docker set up provides a local API for ldap and slurm functionality. 
It does not however support the directory management system. 
For example I could not make setting the quota of a created directory work properly in a docker container.

## Setting of conf.toml for docker

By default the conf.toml has set up those keys for docker properly in the repository.
In development you want to deactivate the directory management by setting the key "include_dir_mgmt" to false 
in the conf.toml. 
Set key "sacctmgr_path" to sacctmgr in the conf.toml file. 
In the docker container the PATH environment variable includes the sacctmgr executable.

## How to use docker development set up

1. build docker image from local files. Only needs to be done once.

```bash
./docker_build_set_up.sh
```

2. start docker container 

```bash
./run_dev_docker.sh
```

3. post docker container start. Installs slurm specs

```bash
./after_container_start.sh
```

For stopping all containers 

```bash
./tear_dev_docker_down.sh
```

For removing all volumes to restart slurm db

```bash
./throw_away_volumes.sh
```

## User for ssh

In docker set up development there is a user called "dev_user". The password of the user is "password"
for ssh login. 
This user has admin rights in the slurmdb and can be used add/modify/delete users in the slurmdb of the docker set up
Login in as this user in the ssh prompt of app.

## Initial data and specs for slurmdb

- This is accomplished via this [script](./slurm-docker-cluster/add_slurm_data.sh)
  If changes/addition of slurmdb data are required then make changes there.
  You need to build the docker file again to provide the docker container with its new version.

## Git clone of slurm-docker-cluster folder
The [folder for slurm cluster](./slurm-docker-cluster) came from git clone 
from this remote Github [repository](https://github.com/giovtorres/slurm-docker-cluster). 

It was not included as a sub module because several files needed to modified.

One could make this folder  a sub module pointing to our own public fork. This fork should be in the control
of university th-nürnberg however.
