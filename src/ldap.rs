/// LDAP operations 
pub mod ldap {
    use log::{debug, error, warn, info};
    use tempfile::tempdir;

    use crate::{MgmtConfig, Entity, util::io_util::{user_input, read_ldif_template, write_tmp_ldif, write_to_tmp_file}, Modifiable};
    use std::{process::Command};

    /// Defines fields necessary to establish an LDAP connection
    pub struct LDAPConn {
        pub ldap_bind: String,
        pub ldap_pass: String,
        pub ldap_base: String,
        pub ldap_dc: String,
    }

    impl LDAPConn {
        fn new(dc: &Option<String>) -> Self {
            let (ldap_user, ldap_pass) = Self::ask_credentials();

            let ldap_bind: String;
            match dc {
                Some(x) => ldap_bind = format!("cn={ldap_user},{x}"),
                None => ldap_bind = format!("cn={ldap_user},dc=informatik,dc=fh-nuernberg,dc=de"),
            }

            LDAPConn {
                ldap_bind,
                ldap_pass,
                ..Default::default()
            }
        }

        fn ask_credentials() -> (String, String) {
            println!("Enter your LDAP username (defaults to admin):");
            let mut username = user_input();
            if username.len() < 1 {
                username = "admin".to_string();
            }
            let password = rpassword::prompt_password("Enter your LDAP password: ").unwrap();
            (username, password)
        }
    }

    impl Default for LDAPConn {
        fn default() -> Self {
            LDAPConn {
                ldap_bind: "cn=connector,dc=informatik,dc=fh-nuernberg,dc=de".to_string(),
                ldap_pass: "".to_string(),
                ldap_base: "ou=people,dc=informatik,dc=fh-nuernberg,dc=de".to_string(),
                ldap_dc: "dc=informatik,dc=fh-nuernberg,dc=de".to_string(),
            }
        }
    }

    pub fn add_ldap_user(entity: &Entity, config: &MgmtConfig) {

        let ldap_conn = LDAPConn::new(&Some(config.ldap_domain_components.clone()));

        if username_exists(&ldap_conn, &entity.username) {
            warn!("User {} already exists in LDAP. Skipping LDAP user creation.", &entity.username);
            return
        }

        let uid_result = find_next_available_uid(&ldap_conn, entity.group.clone());
        let uid_number : i32;
        match uid_result {
            Some(r) => {
                uid_number = r
            },
            None => {
                error!( "No users found or LDAP query failed. Unable to assign uid. Aborting..." );
                return
            }
        }

        let template = read_ldif_template(&config.ldif_template_path);
        let mut custom_elems: Vec<String> = Vec::new();

        custom_elems.push(format!("dn: uid={},{}", entity.username, ldap_conn.ldap_base));
        custom_elems.push(format!("cn: {}", entity.username));
        custom_elems.push(format!("gidNumber: {}", entity.gid));
        custom_elems.push(format!("homeDirectory: /home/{}", entity.username));
        custom_elems.push(format!("sn: {}", entity.lastname));
        custom_elems.push(format!("uid: {}", entity.username));
        custom_elems.push(format!("uidNumber: {}", uid_number));
        custom_elems.push(format!("givenName: {}", entity.firstname));
        custom_elems.push(format!("slurmDefaultQos: {}", entity.default_qos));

        for qos in &entity.qos {
            custom_elems.push(format!("slurmQos: {}", qos));
        }

        if entity.mail.len() > 0 {
            custom_elems.push(format!("mail: {}", entity.mail));
        }

        let tmpdir = tempdir().unwrap();
        let tmp_file = write_tmp_ldif(&tmpdir, template, custom_elems).unwrap();
        let output = Command::new("ldapadd")
            .arg("-x")
            .arg("-w")
            .arg(ldap_conn.ldap_pass)
            .arg("-D")
            .arg(ldap_conn.ldap_bind)
            .arg("-f")
            .arg(tmp_file)
            .output()
            .expect("Unable to execute ldapadd command.");
        debug!("add_ldap_user: {}", String::from_utf8_lossy(&output.stdout));
    
        if output.status.success() {
            info!("Added user {} to LDAP.", entity.username);
        } else {
            warn!("LDAP user creation did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("ldapadd stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.len() > 0 {
                error!("ldapadd stderr: {}", err);
            }
        }
    }

    pub fn delete_ldap_user(username: &str, config: &MgmtConfig) {

        let ldap_conn = LDAPConn::new(&Some(config.ldap_domain_components.clone()));
        let maybe_dn = find_dn_by_uid(&ldap_conn, username);
        let dn: String;
        
        match maybe_dn {
            Some(maybe_dn) => dn = maybe_dn,
            None => {
                error!("Unable to find DN for user {}. Maybe the user does not exist in LDAP or you used invalid credentials.", username);
                return
            }
        }

        let output = Command::new("ldapdelete")
            .arg("-x")
            .arg("-w")
            .arg(ldap_conn.ldap_pass)
            .arg("-D")
            .arg(ldap_conn.ldap_bind)
            .arg(format!("{}", dn))
            .output()
            .expect("Unable to execute ldapdelete command.");
        
        debug!("delete_ldap_user: {}", String::from_utf8_lossy(&output.stdout));

        if output.status.success() {
            info!("Deleted {username} from LDAP.")
        } else {
            warn!("LDAP user deletion did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("ldapdelete stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.len() > 0 {
                error!("ldapdelete stderr: {}", err);
            }
        }
    }
    
    pub fn modify_ldap_user(modifiable: &Modifiable, config: &MgmtConfig) {

        let ldap_conn = LDAPConn::new(&Some(config.ldap_domain_components.clone()));

        let mut modifiable_elems: Vec<String> = Vec::new();

        let maybe_dn = find_dn_by_uid(&ldap_conn, &modifiable.username);
        let dn: String;
        
        match maybe_dn {
            Some(maybe_dn) => dn = maybe_dn,
            None => {
                error!("Unable to find DN for user {}. Maybe the user does not exist in LDAP or you used invalid credentials.", modifiable.username);
                return
            }
        }
        
        modifiable_elems.push(format!("dn: {}", dn));
        modifiable_elems.push("changetype: modify".to_string());

        // Todo we should also change the cn according to the changes made to givenName and sn
        // let mut new_cn = "";
        if let Some(firstname) = &modifiable.firstname {
            modifiable_elems.push("replace: givenName".to_string());
            modifiable_elems.push(format!("givenName: {}", firstname));
            modifiable_elems.push("-".to_string());
        }

        if let Some(lastname) = &modifiable.lastname {
                modifiable_elems.push("replace: sn".to_string());
                modifiable_elems.push(format!("sn: {}", lastname));
                modifiable_elems.push("-".to_string());
        }

        if let Some(mail) = &modifiable.mail {
            modifiable_elems.push("replace: mail".to_string());
            modifiable_elems.push(format!("mail: {}", mail));
            modifiable_elems.push("-".to_string());
        }

        if let Some(default_qos) = &modifiable.default_qos {
            modifiable_elems.push("replace: slurmDefaultQos".to_string());
            modifiable_elems.push(format!("slurmDefaultQos: {}", default_qos));
            modifiable_elems.push("-".to_string());
        }

        if !modifiable.qos.is_empty() {
            // first we delete all old qos
            modifiable_elems.push("delete: slurmQos".to_string());
            modifiable_elems.push("-".to_string());
            // then we add all new qos
            modifiable_elems.push("add: slurmQos".to_string());
            for q in &modifiable.qos {
                modifiable_elems.push(format!("slurmQos: {}", q));
            }
            modifiable_elems.push("-".to_string());

        }

        let tmpdir = tempdir().unwrap();
        let tmp_file = write_to_tmp_file(&tmpdir, modifiable_elems).unwrap();

        let output = Command::new("ldapmodify")
            .arg("-x")
            .arg("-w")
            .arg(ldap_conn.ldap_pass)
            .arg("-D")
            .arg(ldap_conn.ldap_bind)
            .arg("-f")
            .arg(tmp_file)
            .output()
            .expect("Unable to execute ldapmodify command.");
        
        debug!("modify_ldap_user: {}", String::from_utf8_lossy(&output.stdout));

        if output.status.success() {    
            info!("Modified user {} in LDAP", modifiable.username);   
        } else {
            warn!("LDAP user modification did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("ldapmodify stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.len() > 0 {
                error!("ldapmodify stderr: {}", err);
            } 
        }
    }

    /// Check if username already exists in ldap. 
    /// Must be an exact match on the uid attribute. 
    fn username_exists(ldap_conn: &LDAPConn, username: &String) -> bool {
        let output = Command::new("ldapsearch")
            .arg("-LLL")
            .arg("-D")
            .arg(&ldap_conn.ldap_bind)
            .arg("-x")
            .arg("-w")
            .arg(&ldap_conn.ldap_pass)
            .arg("-b")
            .arg(&ldap_conn.ldap_base)
            .arg("-o")
            .arg("ldif-wrap=no")
            .arg("(objectclass=*)")
            .arg("uid")
            .output()
            .expect("Unable to execute ldapsearch command. Is the path specified in your config correct?");
        
        let search_result = String::from_utf8_lossy(&output.stdout);
        let search_result_split = search_result.split("\n");
        
        for s in search_result_split {
            if s.contains("uid:") {
                let split : Vec<&str> = s.split(" ").collect();
                if username == split[1].trim() {
                    return true;
                }
            }
        }
        false
    }

    /// Search for a specific uid and return the corresponding dn. 
    fn find_dn_by_uid(ldap_conn: &LDAPConn, username: &str) -> Option<String> {
        let output = Command::new("ldapsearch")
            .arg("-LLL")
            .arg("-D")
            .arg(&ldap_conn.ldap_bind)
            .arg("-x")
            .arg("-w")
            .arg(&ldap_conn.ldap_pass)
            .arg("-b")
            .arg(&ldap_conn.ldap_base)
            .arg("-o")
            .arg("ldif-wrap=no")
            .arg(format!("(uid={username})"))
            .arg("dn")
            .output()
            .expect("Unable to execute ldapsearch command. Is the path specified in your config correct?");
        
        let search_result = String::from_utf8_lossy(&output.stdout);
        let search_result_split = search_result.split("\n");

        if !output.status.success() { 
            warn!("LDAP search query did not return with success.");
            let out = String::from_utf8_lossy(&output.stdout);
            if out.len() > 0 {
                warn!("ldapsearch stdout: {}", out);
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if err.len() > 0 {
                error!("ldapsearch stderr: {}", err);
            }    
        }

        for s in search_result_split {
            if s.contains("dn:") {
                let split : Vec<&str> = s.split(" ").collect();
                return Some(split[1].trim().to_string())
            }
        }
        None  
    }

    /// Do an LDAP search to determine the next available uid
    fn find_next_available_uid(ldap_conn: &LDAPConn, group: crate::Group) -> Option<i32> {

        let output = Command::new("ldapsearch")
            .arg("-LLL")
            .arg("-D")
            .arg(&ldap_conn.ldap_bind)
            .arg("-x")
            .arg("-w")
            .arg(&ldap_conn.ldap_pass)
            .arg("-b")
            .arg(&ldap_conn.ldap_base)
            .arg("-o")
            .arg("ldif-wrap=no")
            .arg("(objectclass=*)")
            .arg("uidNumber")
            .output()
            .expect("Unable to execute ldapsearch command. Is the path specified in your config correct?");
        
        let search_result = String::from_utf8_lossy(&output.stdout);
        let search_result_split = search_result.split("\n");
        
        let mut uids : Vec<i32> = Vec::new();
        for s in search_result_split {
            if s.contains("uidNumber") {
                let split : Vec<&str> = s.split(" ").collect();
                uids.push(split[1].parse::<i32>().unwrap());
            }
        }
        // students start at 10000, staff at 1000
        if group == crate::Group::Student {
            uids = uids.into_iter().filter(|&i|i >= 10000 ).collect::<Vec<_>>();
        } else {
            uids = uids.into_iter().filter(|&i|i >= 1000 && i < 10000 ).collect::<Vec<_>>();
        }

        let max_value = uids.iter().max();
        match max_value {
            Some(max) => {
                debug!( "Next available uid is: {}", max + 1);
                Some(max + 1)
            },
            None => {
                None
            },
        }
    }
}
