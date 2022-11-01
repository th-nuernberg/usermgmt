/// LDAP operations using the ldap3 lib
pub mod ldap {

    use ldap3::controls::{MakeCritical, RelaxRules};
    use ldap3::{LdapConn, LdapError, Mod, Scope, SearchEntry};
    use log::{debug, error, info, warn};
    use maplit::hashset;
    use std::collections::HashSet;

    use crate::util::io_util::{get_new_uid, hashset_from_vec_str};
    use crate::{util::io_util::user_input, Entity, MgmtConfig, Modifiable};

    #[derive(Debug, Default)]
    pub struct LDAPConfig {
        pub ldap_server: String,
        pub ldap_bind: String,
        pub ldap_user: String,
        pub ldap_pass: String,
        pub ldap_base: String,
        pub ldap_dc: String,
    }

    impl LDAPConfig {
        fn new(
            ldap_server: &String,
            dc: &Option<String>,
            org_unit: &Option<String>,
            username: &Option<String>,
            password: &Option<String>,
        ) -> Self {
            let (ldap_user, ldap_pass);
            match username {
                Some(u) => match password {
                    Some(p) => (ldap_user, ldap_pass) = (u.clone(), p.clone()),
                    None => (ldap_user, ldap_pass) = Self::ask_credentials(),
                },
                None => {
                    (ldap_user, ldap_pass) = Self::ask_credentials();
                }
            }

            let org_unit_str = match org_unit {
                Some(ou) => ou.to_owned(),
                None => "people".to_string(),
            };

            let ldap_bind: String;
            let ldap_base: String;
            let ldap_dc: String;
            match dc {
                Some(x) => {
                    ldap_bind = format!("cn={ldap_user},{x}");
                    ldap_base = format!("ou={org_unit_str},{x}");
                    ldap_dc = x.to_string();
                }
                None => {
                    ldap_dc = "dc=informatik,dc=fh-nuernberg,dc=de".to_string();
                    ldap_bind = format!("cn={ldap_user},{ldap_dc}");
                    ldap_base = format!("ou={org_unit_str},{ldap_dc}");
                }
            }

            LDAPConfig {
                ldap_server: ldap_server.to_string(),
                ldap_bind,
                ldap_user,
                ldap_pass,
                ldap_base,
                ldap_dc,
            }
        }

        fn ask_credentials() -> (String, String) {
            println!("Enter your LDAP username (defaults to admin):");
            let mut username = user_input();
            if username.is_empty() {
                username = "admin".to_string();
            }
            let password = rpassword::prompt_password("Enter your LDAP password: ").unwrap();
            (username, password)
        }
    }

    fn make_ldap_connection(server: &str) -> Result<LdapConn, LdapError> {
        LdapConn::new(server)
    }

    pub fn add_ldap_user(entity: &Entity, config: &MgmtConfig) {
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &None,
            &None,
        );

        if username_exists(
            &entity.username,
            config,
            &ldap_config.ldap_user,
            &ldap_config.ldap_pass,
        ) {
            warn!(
                "User {} already exists in LDAP. Skipping LDAP user creation.",
                &entity.username
            );
            return;
        }

        let uid_result = find_next_available_uid(&ldap_config, entity.group.clone());
        let uid_number = match uid_result {
            Some(r) => r,
            None => {
                error!("No users found or LDAP query failed. Unable to assign uid. Aborting...");
                return;
            }
        };

        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(_bind) => debug!("LDAP connection established to {}", ldap_config.ldap_bind),
                    Err(e) => error!("{}", e),
                }
                let un = &*entity.username.to_owned();
                let gid = &*format!("{}", entity.gid);
                let uid = &*format!("{}", uid_number);
                let ln = &*entity.lastname.to_owned();
                let gn = &*entity.firstname.to_owned();
                let mail = &*entity.mail.to_owned();
                let def_qos = &*entity.default_qos.to_owned();
                let home = &*format!("/home/{}", entity.username);
                let qos = entity.qos.to_owned();
                let pubkey = &*entity.publickey.to_owned();

                let ldap_result = ldap.add(
                    &format!("uid={},{}", entity.username, ldap_config.ldap_base),
                    vec![
                        ("cn", hashset! {un}),
                        (
                            "objectClass",
                            hashset_from_vec_str(&config.objectclass_common).to_owned(),
                        ),
                        ("gidNumber", hashset! {gid}),
                        ("uidNumber", hashset! {uid}),
                        ("uid", hashset! {un}),
                        ("sn", hashset! {ln}),
                        ("givenName", hashset! {gn}),
                        ("mail", hashset! {mail}),
                        ("slurmDefaultQos", hashset! {def_qos}),
                        ("homeDirectory", hashset! {home}),
                        ("slurmQos", hashset_from_vec_str(&qos).to_owned()),
                        ("sshPublicKey", hashset! {pubkey}),
                        ("loginShell", hashset! {config.login_shell.as_str()}),
                    ],
                );

                match ldap_result {
                    Ok(_) => info!("Added LDAP user {}", entity.username),
                    Err(e) => error!("Unable to create LDAP user! {e}"),
                }
            }
            Err(e) => error!("{}", e),
        }

        debug!("add_ldap_user done");
    }

    pub fn delete_ldap_user(username: &str, config: &MgmtConfig) {
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &None,
            &None,
        );
        // get dn for uid
        match find_dn_by_uid(
            username,
            config,
            &ldap_config.ldap_user,
            &ldap_config.ldap_pass,
        ) {
            Some(dn) => {
                match make_ldap_connection(&ldap_config.ldap_server) {
                    Ok(mut ldap) => {
                        match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                            Ok(_bind) => {
                                debug!("LDAP connection established to {}", ldap_config.ldap_bind)
                            }
                            Err(e) => error!("{}", e),
                        }
                        // delete user by dn
                        match ldap.delete(&dn) {
                            Ok(_) => info!("Successfully deleted DN {}", dn),
                            Err(e) => error!("User deletion in LDAP failed! {}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            None => error!("No DN found for username {}!", username),
        }
        debug!("delete_ldap_user done");
    }

    pub fn modify_ldap_user(modifiable: &Modifiable, config: &MgmtConfig) {
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &None,
            &None,
        );
        // get dn for uid
        match find_dn_by_uid(
            &modifiable.username,
            config,
            &ldap_config.ldap_user,
            &ldap_config.ldap_pass,
        ) {
            Some(dn) => {
                match make_ldap_connection(&ldap_config.ldap_server) {
                    Ok(mut ldap) => {
                        match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                            Ok(_bind) => {
                                debug!("LDAP connection established to {}", ldap_config.ldap_bind)
                            }
                            Err(e) => error!("{}", e),
                        }
                        // Prepare replace operation
                        let old_qos = find_qos_by_uid(
                            &modifiable.username,
                            config,
                            &ldap_config.ldap_user,
                            &ldap_config.ldap_pass,
                        );
                        let mod_vec = make_modification_vec(modifiable, &old_qos);

                        // Replace userPassword at given dn
                        let res = ldap
                            .with_controls(RelaxRules.critical())
                            .modify(&*dn, mod_vec);

                        match res {
                            Ok(_) => {
                                info!("Successfully modified user {} in LDAP", modifiable.username)
                            }
                            Err(e) => info!("User modification in LDAP failed! {}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            None => error!(
                "No DN found for username {}! Unable to modify user.",
                modifiable.username
            ),
        }
        debug!("modify_ldap_user done");
    }

    /// List all LDAP users and some attributes
    pub fn list_ldap_users(config: &MgmtConfig) {
        // ldapsearch -x -b "ou=people,dc=informatik,dc=fh-nuernberg,dc=de" -H ldap://localhost 
        // -D "cn=ldapconnector,dc=informatik,dc=fh-nuernberg,dc=de" "objectClass=posixAccount" cn uid mail -W
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &Some(config.ldap_readonly_user.clone()),
            &Some(config.ldap_readonly_pw.clone()),
        );

        // Establish LDAP connection and bind
        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(_bind) => {
                        debug!("LDAP connection established to {}. Will search under {}", ldap_config.ldap_bind, ldap_config.ldap_base);
                        let attrs = vec!["uid","uidNumber", "givenName", "sn", "mail", "slurmDefaultQos", "slurmQos"];
                        // Search for all entities under base dn
                        let search_result = ldap.search(
                            &ldap_config.ldap_base,
                            Scope::OneLevel,
                            "(objectclass=*)",
                            attrs.clone(),
                        );
                        match search_result {
                            // Parse search results and print
                            Ok(result) => {
                                for elem in result.0.iter() {
                                    let search_result = SearchEntry::construct(elem.to_owned());
                                    
                                    let mut output_str = "".to_string();
                                    for a in attrs.iter() {
                                        if search_result.attrs.contains_key(*a) {
                                            if *a == "slurmQos" {
                                                let qos = &search_result.attrs["slurmQos"];
                                                let elem = qos.join("|"); 
                                                output_str += &format!("{}={},", a, elem);
                                            } else {
                                                let elem = search_result.attrs[*a][0].clone();
                                                output_str += &format!("{}={},", a, elem);
                                            }
                                        }
                                    }
                                    println!("{}", output_str);
                                }
                            }
                            Err(e) => error!("Error during LDAP search! {}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
    }

    fn make_modification_vec<'a>(
        modifiable: &'a Modifiable,
        old_qos: &'a Vec<String>,
    ) -> Vec<Mod<&'a str>> {
        let mut modifications: Vec<Mod<&str>> = Vec::new();

        if let Some(firstname) = &modifiable.firstname {
            modifications.push(Mod::Replace(
                "givenName",
                HashSet::from([&*firstname.as_str()]),
            ))
        }

        if let Some(lastname) = &modifiable.lastname {
            modifications.push(Mod::Replace("sn", HashSet::from([&*lastname.as_str()])))
        }

        if let Some(mail) = &modifiable.mail {
            modifications.push(Mod::Replace("mail", HashSet::from([&*mail.as_str()])))
        }

        if let Some(default_qos) = &modifiable.default_qos {
            modifications.push(Mod::Replace(
                "slurmDefaultQos",
                HashSet::from([&*default_qos.as_str()]),
            ))
        }

        if let Some(publickey) = &modifiable.publickey {
            debug!("Pushing modifiable publickey {}", publickey);
            modifications.push(Mod::Replace(
                "sshPublicKey",
                HashSet::from([&*publickey.as_str()]),
            ))
        }

        if !old_qos.is_empty() {
            // first we delete all old qos
            for q in old_qos {
                modifications.push(Mod::Delete("slurmQos", HashSet::from([&*q.as_str()])))
            }
            // then we add all new qos
            for q in &modifiable.qos {
                modifications.push(Mod::Add("slurmQos", HashSet::from([&*q.as_str()])))
            }
        }
        modifications
    }

    /// Do a LDAP search to determine the next available uid
    fn find_next_available_uid(ldap_config: &LDAPConfig, group: crate::Group) -> Option<i32> {
        let mut new_uid = None;

        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                debug!(
                    "Binding with dn: {}, pw: {}",
                    ldap_config.ldap_bind, ldap_config.ldap_pass
                );
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(r) => debug!(
                        "find_next_available_uid: LDAP connection established to {}, {}",
                        ldap_config.ldap_bind, r
                    ),
                    Err(e) => error!("{}", e),
                }
                debug!("Search under {}", ldap_config.ldap_base);
                // Search for all uidNumbers under base dn
                let search_result = ldap.search(
                    &ldap_config.ldap_base,
                    Scope::OneLevel,
                    "(objectclass=*)",
                    vec!["uidNumber"],
                );
                match search_result {
                    // Parse search results into ints
                    Ok(result) => {
                        let mut uids: Vec<i32> = Vec::new();
                        for elem in result.0.iter() {
                            let search_result = SearchEntry::construct(elem.to_owned());
                            debug!("UID: {:?}", SearchEntry::construct(elem.to_owned()));
                            let uid = &search_result.attrs["uidNumber"][0].parse::<i32>().unwrap();
                            uids.push(*uid);
                        }
                        // Find max uid and add 1
                        new_uid = get_new_uid(uids, group);
                    }
                    Err(e) => error!("Error during uid search! {}", e),
                }
            }
            Err(e) => error!("Error during uid search! {}", e),
        }
        new_uid
    }

    /// Search for a specific uid and return the corresponding dn.
    fn find_dn_by_uid(
        username: &str,
        config: &MgmtConfig,
        ldap_user: &String,
        ldap_pass: &String,
    ) -> Option<String> {
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &Some(ldap_user.clone()),
            &Some(ldap_pass.clone()),
        );
        let mut dn_result = None;
        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(_bind) => debug!("LDAP connection established to {}", ldap_config.ldap_bind),
                    Err(e) => error!("{}", e),
                }

                // Search for all uids under base dn and return dn of user
                let search = ldap.search(
                    &*ldap_config.ldap_base,
                    Scope::OneLevel,
                    &format!("(uid={username})"),
                    vec!["dn"],
                );

                match search {
                    Ok(result) => {
                        // Only deal with the first element retrieved from search
                        match result.0.into_iter().next() {
                            Some(entry) => {
                                let sr = SearchEntry::construct(entry);
                                debug!("SR for deletion: {:?}", sr);
                                dn_result = Some(sr.dn);
                            }
                            None => error!("No LDAP entry found for user {}", username),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
        dn_result
    }

    /// Search for a specific uid and return the corresponding qos.
    fn find_qos_by_uid(
        username: &str,
        config: &MgmtConfig,
        ldap_user: &String,
        ldap_pass: &String,
    ) -> Vec<String> {
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &Some(ldap_user.clone()),
            &Some(ldap_pass.clone()),
        );
        let mut qos: Vec<String> = Vec::new();

        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(_bind) => debug!("LDAP connection established to {}", ldap_config.ldap_bind),
                    Err(e) => error!("{}", e),
                }

                // Search for all uid under base dn and return dn of user
                let search = ldap.search(
                    &*ldap_config.ldap_base,
                    Scope::OneLevel,
                    &format!("(uid={username})"),
                    vec!["slurmQos"],
                );

                match search {
                    Ok(result) => {
                        for elem in result.0.iter() {
                            let search_result = SearchEntry::construct(elem.to_owned());
                            let q = &search_result.attrs["slurmQos"][0];
                            debug!("QOS: {:?}", SearchEntry::construct(elem.to_owned()));
                            qos.push(q.to_string().clone());
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
        qos
    }

    /// Check if username already exists in ldap.
    /// Must be an exact match on the uid attribute.
    fn username_exists(
        username: &String,
        config: &MgmtConfig,
        ldap_user: &String,
        ldap_pass: &String,
    ) -> bool {
        let mut username_exists = false;
        let ldap_config = LDAPConfig::new(
            &config.ldap_server,
            &Some(config.ldap_domain_components.clone()),
            &Some(config.ldap_org_unit.clone()),
            &Some(ldap_user.clone()),
            &Some(ldap_pass.clone()),
        );
        match make_ldap_connection(&ldap_config.ldap_server) {
            Ok(mut ldap) => {
                match ldap.simple_bind(&ldap_config.ldap_bind, &ldap_config.ldap_pass) {
                    Ok(_bind) => debug!("LDAP connection established to {}", ldap_config.ldap_bind),
                    Err(e) => error!("{}", e),
                }

                // Search for all uid under base dn and return dn of user
                let search = ldap.search(
                    &*ldap_config.ldap_base,
                    Scope::OneLevel,
                    &format!("(uid={username})"),
                    vec!["dn"],
                );

                match search {
                    Ok(result) => {
                        // Only deal with the first element retrieved from search
                        match result.0.into_iter().next() {
                            Some(entry) => {
                                // User found. Good.
                                debug!("Found user: {:?}", SearchEntry::construct(entry));
                                username_exists = true
                            }
                            None => debug!("No LDAP entry found for user {}", username),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
        username_exists
    }
}
