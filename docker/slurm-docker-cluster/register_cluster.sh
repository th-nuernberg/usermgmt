#!/bin/bash
set -e

docker exec slurmctld bash -c "bash /usr/local/bin/add_slurm_data.sh" && \
docker compose restart slurmdbd slurmctld
