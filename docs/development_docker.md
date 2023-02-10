To create typical users in ldap of the th cluster we need add add a few extra ldap schemas

To find out if a custom schema was added after starting the docker container

Go into the docker container vai
```text
docker run exec -it <docker_container_id_or_name> bash
```
And the execute this command:
```text
ldapsearch -b "cn=schema,cn=config" -H ldapi:/// -LLL -Q -Y EXTERNAL dn
```
You need to be in the docker container because the endpoint ldapi:/// with the -Y EXTERNAL authentication
only works if run it on the machine where the ldap server is running.

You should then see the added schema alongside with the others.
