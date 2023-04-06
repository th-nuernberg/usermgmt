const DEFAULT_ORG_UNIT: &str = "";
const DEFAULT_BIND_ORG_UNIT: &str = "";
const DEFAULT_BIND_PREFIX: &str = "cn";
use crate::util::io_util;
use crate::MgmtConfig;
use log::info;
#[derive(Debug, Default)]
/// Contains all information for creating/listing/deleting and modifying an user
/// TODO: consider implementing encapsulation with getters and setters
pub struct LDAPConfig {
    pub ldap_server: String,
    pub ldap_user: String,
    pub ldap_pass: String,
    /// Base part of every part in ldap
    /// Example: dc=example,dc=com
    pub ldap_dc: String,
    /// Path in which users are added/deleted and modified in ldap
    /// Example: ou=people,dc=example,dc=com
    /// if ldap_dc is dc=example,dc=com and ldap_org_unit is ou=people
    pub ldap_base: String,
    /// path to user who should log in ldap
    /// Example: ou=special,dc=example,dc=com
    /// if ldap_dc is dc=example,dc=com and ldap_bind_org_unit is ou=special
    pub ldap_bind: String,
}

impl LDAPConfig {
    pub fn new(config: &MgmtConfig, username: &Option<String>, password: &Option<String>) -> Self {
        let (bind_prefix, ldap_server, dc, org_unit, bind_org_unit) = (
            &config.ldap_bind_prefix,
            &config.ldap_server,
            &config.ldap_domain_components,
            &config.ldap_org_unit,
            &config.ldap_bind_org_unit,
        );

        let (ldap_user, ldap_pass) = {
            let (ldap_user, ldap_pass) = match username {
                Some(u) => match password {
                    Some(p) => (u.clone(), p.clone()),
                    None => Self::ask_credentials(),
                },
                None => Self::ask_credentials(),
            };

            (ldap_user.trim().to_owned(), ldap_pass.trim().to_owned())
        };

        // Take values from config or instead use defaults.
        let org_unit_str = by_config_or_default(org_unit, DEFAULT_ORG_UNIT);
        let bind_prefix_str = by_config_or_default(bind_prefix, DEFAULT_BIND_PREFIX);
        let bind_org_unit_str = by_config_or_default(bind_org_unit, DEFAULT_BIND_ORG_UNIT);

        // create ldap paths by concat ldap components by comma
        let ldap_user_end_point = format!("{bind_prefix_str}={ldap_user}");
        let ldap_dc = dc.clone().unwrap_or(String::new());
        let ldap_base = concat_by_comma_if_both_not_empty(org_unit_str.clone(), ldap_dc.clone());
        let ldap_bind = {
            let dc_and_bind_org =
                concat_by_comma_if_both_not_empty(bind_org_unit_str.clone(), ldap_dc.clone());
            concat_by_comma_if_both_not_empty(ldap_user_end_point, dc_and_bind_org)
        };

        info!("({}) ldap domain components dn.", &ldap_dc);
        info!(
            "({}) Ldap dn under which a user is created/deleted/modified.",
            &ldap_base
        );
        info!("({}) Ldap dn binding for user log in.", &ldap_bind);

        return LDAPConfig {
            ldap_server: ldap_server.to_string(),
            ldap_bind,
            ldap_user,
            ldap_pass,
            ldap_base,
            ldap_dc,
        };

        fn by_config_or_default(to_resolve: &Option<String>, default_val: &str) -> String {
            match to_resolve {
                Some(from_config) => from_config.to_owned(),
                None => default_val.to_owned(),
            }
        }

        fn concat_by_comma_if_both_not_empty(left_part: String, right_part: String) -> String {
            match (left_part.is_empty(), right_part.is_empty()) {
                (true, true) => String::new(),
                (true, false) => right_part,
                (false, true) => left_part,
                (false, false) => format!("{},{}", left_part, right_part),
            }
        }
    }

    fn ask_credentials() -> (String, String) {
        println!("Enter your LDAP username (defaults to admin):");
        let mut username = io_util::user_input();
        if username.is_empty() {
            username = "admin".to_string();
        }
        let password = rpassword::prompt_password("Enter your LDAP password: ")
            .expect("Failed to retrieve password from user in a terminal");
        (username, password)
    }
}
