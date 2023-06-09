use getset::Getters;
use log::info;
const DEFAULT_ORG_UNIT: &str = "";
const DEFAULT_BIND_ORG_UNIT: &str = "";
const DEFAULT_BIND_PREFIX: &str = "cn";

#[derive(Debug, Default, Getters)]
#[getset(get = "pub")]
pub struct LdapPaths {
    /// Path in which users are added/deleted and modified in ldap
    /// Example: ou=people,dc=example,dc=com
    /// if ldap_dc is dc=example,dc=com and ldap_org_unit is ou=people
    base: String,
    /// path to user who should log in ldap
    /// Example: uid=example,ou=special,dc=example,dc=com
    /// if ldap_dc is dc=example,dc=com,ldap_bind_org_unit is ou=special, ldap_user_prefix is uid
    /// and ldap_user is example
    bind: String,
    username: String,
}

impl LdapPaths {
    pub fn new(
        dc: Option<String>,
        org_unit: Option<String>,
        bind: Option<String>,
        prefix: Option<String>,
        username: String,
    ) -> Self {
        let org_unit = by_config_or_default(&org_unit, DEFAULT_ORG_UNIT);
        let prefix = by_config_or_default(&prefix, DEFAULT_BIND_PREFIX);
        let bind_org_unit = by_config_or_default(&bind, DEFAULT_BIND_ORG_UNIT);
        // create ldap paths by concat ldap components by comma
        let ldap_prefix_with_user_name = format!("{}={}", prefix, username);
        let ldap_dc = dc.unwrap_or(String::new());
        let ldap_base = concat_by_comma_if_both_not_empty(org_unit, ldap_dc.clone());
        let ldap_bind = {
            let dc_and_bind_org = concat_by_comma_if_both_not_empty(bind_org_unit, ldap_dc.clone());
            concat_by_comma_if_both_not_empty(ldap_prefix_with_user_name, dc_and_bind_org)
        };

        info!("({}) ldap domain components dn.", &ldap_dc);
        info!(
            "({}) Ldap dn under which a user is created/deleted/modified.",
            &ldap_base
        );
        info!("({}) Ldap dn binding for user log in.", &ldap_bind);

        return Self {
            base: ldap_base,
            bind: ldap_bind,
            username,
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
}
