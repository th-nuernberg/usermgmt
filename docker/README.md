# Docker set up

**Do not use this in production !**

It is meant to be used for local development.
While it response to the app as a ldap and slurm endpoint, the set up 
is not designed for production ready security !

**Do not use this [ssh_host_rsa_key](./ssh_host_rsa_key) in any production environment or for your own private usage !**

The private [ssh_host_rsa_key](./ssh_host_rsa_key) is located for convenience in this repository.
It makes sure that the public key fingerprint is always the same for ssh client.

## Limitations

This docker set up provides a local API for ldap and slurm functionality. 
It does not however support the directory management system. 
For example I could not make setting the quota of a created directory work properly in a docker container.

## Setting of conf.toml for docker

By default the conf.toml is set up for docker properly in the repository.
In development the key "include_dir_mgmt" needs to false in the conf.toml. 
The key must be "sacctmgr_path"  in the conf.toml file. 
In the docker container the PATH environment variable includes the sacctmgr executable.

## How to use docker development set up

1. Go into docker folder relative to the project root

```bash
cd ./docker
```

2. build docker image from local files. Only needs to be done once.

```bash
./docker_build_set_up.sh
```

3. start docker container 

```bash
./run_dev_docker.sh
```

4. Wait a bit until slurmdb and slurmctl as services are ready. Then start this post script.

```bash
./after_container_start.sh
```

---

For stopping all containers 

```bash
./tear_dev_docker_down.sh
```

For removing all volumes to clean slurm db and LDAP db

```bash
./throw_away_volumes.sh
```

## dev_user as slurm admin during development

In the docker set up there is a user called "dev_user". The password of the user is "password"
for ssh login. 

Use this username and password when the app prompts for ssh credentials during development.

This user has admin rights in the slurmdb and can be used to add/modify/delete users in the slurmdb 
of the docker set up. 

### Authorize as dev_user via ssh-agent

You can also authenticate as the user "dev_user" via a ssh agent.
If you want to use or test the functionality with the ssh agent then type the following command: 

```bash
ssh-add docker/slurm-docker-cluster/dev_user_ed25519
```

It is presumed that you are at the project root before running the snippet.

## Initial data and specs for slurmdb

- This is accomplished via this [script](./slurm-docker-cluster/add_slurm_data.sh)
  If changes/addition of slurmdb data are required then make changes there.

## Git clone of slurm-docker-cluster folder

The [folder for slurm cluster](./slurm-docker-cluster) came from git clone 
from this remote Github [repository](https://github.com/giovtorres/slurm-docker-cluster). 

It was not included as a sub module because several files needed to modified.

One could make this folder  a sub module pointing to our own public fork. This fork should be in the control
of university th-nürnberg however.
