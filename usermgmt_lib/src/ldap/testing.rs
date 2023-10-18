use crate::ldap::{
    ldap_search_result::LdapSearchResult, ldap_simple_credential::LdapSimpleCredential,
};

use super::*;
use maplit::hashmap;

struct ExpectedLdapPaths {
    pub ldap_bind: String,
    pub ldap_base: String,
}

#[test]
fn should_give_correct_ldap_paths() {
    {
        let given = MgmtConfig {
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(
            &given,
            LdapSimpleCredential::new("xxx".to_owned(), "xxxx".to_owned()),
        )
        .unwrap();
        assert_case(
            ExpectedLdapPaths {
                ldap_base: "".to_owned(),
                ldap_bind: "cn=xxx".to_owned(),
            },
            &actual,
        );
    }

    {
        let given = MgmtConfig {
            ldap_domain_components: Some("dc=example,dc=com".to_owned()),
            ldap_org_unit: Some("ou=it,ou=people".to_owned()),
            ldap_bind_org_unit: Some("ou=special".to_owned()),

            ..MgmtConfig::default()
        };

        let actual = LDAPConfig::new(
            &given,
            LdapSimpleCredential::new("alice".to_owned(), "xxxx".to_owned()),
        )
        .unwrap();

        assert_case(
            ExpectedLdapPaths {
                ldap_base: "ou=it,ou=people,dc=example,dc=com".to_owned(),
                ldap_bind: "cn=alice,ou=special,dc=example,dc=com".to_owned(),
            },
            &actual,
        );
    }

    {
        let given = MgmtConfig {
            ldap_domain_components: Some("dc=example,dc=com".to_owned()),
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(
            &given,
            LdapSimpleCredential::new("alice".to_owned(), "xxxx".to_owned()),
        )
        .unwrap();
        assert_case(
            ExpectedLdapPaths {
                ldap_base: "dc=example,dc=com".to_owned(),
                ldap_bind: "cn=alice,dc=example,dc=com".to_owned(),
            },
            &actual,
        );
    }

    {
        let given = MgmtConfig {
            ldap_domain_components: Some("dc=example,dc=com".to_owned()),
            ldap_org_unit: Some("aaa=department,bbb=level".to_owned()),
            ldap_bind_prefix: Some("uid".to_string()),
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(
            &given,
            LdapSimpleCredential::new("alice".to_owned(), "xxxx".to_owned()),
        )
        .unwrap();
        assert_case(
            ExpectedLdapPaths {
                ldap_base: "aaa=department,bbb=level,dc=example,dc=com".to_owned(),
                ldap_bind: "uid=alice,dc=example,dc=com".to_owned(),
            },
            &actual,
        );
    }

    fn assert_case(expected: ExpectedLdapPaths, actual: &LDAPConfig<LdapSimpleCredential>) {
        assert_eq!(
            expected.ldap_base,
            actual.base(),
            "Did not produce correct base with  org path + dc."
        );
        assert_eq!(
            expected.ldap_bind,
            actual.bind(),
            "Correct user binding, prefix_user + user + bind_org + dc"
        );
    }
}

#[test]
fn should_produce_simple_output() {
    let given_entries = vec![
        hashmap! {
            "qos".to_string() => vec!["basic".to_string(), "default".to_string()],
            "name".to_string() => vec!["Mr. X".to_string()],
            "age".to_string() => vec!["2".to_string()]
        },
        hashmap! {
            "name".to_string() => vec!["example".to_string()]
        },
        hashmap! {
            "qos".to_string() => vec!["default".to_string()],
            "name".to_string() => vec!["example_man".to_string()],
        },
    ];
    let given_search_entries = vec!["qos", "name"];
    let ldap_search_result = LdapSearchResult::new(given_search_entries, given_entries);
    let actual = text_list_output::ldap_simple_output(&ldap_search_result);
    insta::assert_display_snapshot!(actual);
}

#[test]
fn should_produce_table_from_ldap_search() {
    let given_entries = vec![
        hashmap! {
            "qos".to_string() => vec!["basic".to_string(), "default".to_string()],
            "name".to_string() => vec!["Mr. X".to_string()],
            "age".to_string() => vec!["2".to_string()]
        },
        hashmap! {
            "name".to_string() => vec!["example".to_string()]
        },
    ];

    let given_search_entries = vec!["qos", "name"];
    let ldap_search_result = LdapSearchResult::new(given_search_entries, given_entries);
    let actual = text_list_output::ldap_search_to_pretty_table(&ldap_search_result);
    insta::assert_display_snapshot!(actual);
}
