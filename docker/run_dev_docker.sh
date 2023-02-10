#!/bin/bash

# A docker compose file might be needed if more than LDAP instance is required
docker run \
  -p 389:389 -p 636:636 \
  --volume ./bootstrap_lidfs:/container/service/slapd/assets/config/bootstrap/ldif/custom \
  --detach \
  osixia/openldap:1.5.0 --copy-service  
  


