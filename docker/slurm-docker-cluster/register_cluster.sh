#!/bin/bash
set -e

# add_slurm_data is copied into containers during docker build
# We need a lot of commands to initial slurmdb. So we want to put the commands in a file.
# exec seems not to accept bash files on side of the invoker/host
docker exec slurmctld bash -c "bash /usr/local/bin/add_slurm_data.sh" && \
docker compose restart slurmdbd slurmctld
