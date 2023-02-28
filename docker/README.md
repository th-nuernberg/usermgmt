# How to use docker development set up
1. build docker image from local files. Only needs to be done once.
```bash
./docker_build_set_up.sh
```

2. start docker container 
```bash
./run_dev_docker.sh
```

For stopping all containers 
```bash
./tear_dev_docker_down.sh
```

## Git clone of slurm-docker-cluster folder
The [folder for slurm cluster](./slurm-docker-cluster) came from git clone 
from this remote Github [repository](https://github.com/giovtorres/slurm-docker-cluster). 

It was not included as a sub module because several files needed to modified.

One could make this folder  a sub module pointing to our own public fork. This fork should be in the control
of university th-nürnberg however.
