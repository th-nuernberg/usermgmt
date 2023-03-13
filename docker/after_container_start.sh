#!/bin/bash

echo "Populate slurmdb with th cluster specs like accounts, qos etc ..."
docker exec -it slurmctld bash /usr/local/bin/add_slurm_data.sh

echo "Adding dev_user as admin in slurmdb"
docker exec -it slurmctld bash -c "sacctmgr add user dev_user account=root adminlevel=admin --immediate"
