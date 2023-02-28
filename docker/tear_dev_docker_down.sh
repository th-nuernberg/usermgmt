#!/bin/bash

docker compose -f ./docker-compose.ldap.yml -f ./slurm-docker-cluster/docker-compose.yml down 
