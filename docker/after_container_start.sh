#!/bin/bash

docker exec -it slurmctld bash -c "sacctmgr add user dev_user account=root adminlevel=admin --immediate"

