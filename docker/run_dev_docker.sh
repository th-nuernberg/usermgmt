#!/bin/bash

echo "Strating ldap docker container with name ldap."
docker compose -f ./docker-compose.ldap.yml -f ./slurm-docker-cluster/docker-compose.yml up --detach

