#!/bin/bash

# A docker compose file might be needed if more than LDAP instance is required
docker run \
  --volume ./bootstrap_lidfs:/container/service/slapd/assets/config/bootstrap/ldif/custom \
  osixia/openldap:1.5.0 --copy-service  
  


