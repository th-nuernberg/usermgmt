use super::*;
use maplit::hashmap;

struct ExpectedLdapPaths {
    pub ldap_dc: String,
    pub ldap_bind: String,
    pub ldap_base: String,
}

#[test]
fn should_give_correct_ldap_paths() {
    {
        let given = MgmtConfig {
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(&given, &Some("xxx".to_owned()), &Some("xxxx".to_owned()));
        assert_case(
            ExpectedLdapPaths {
                ldap_dc: "".to_owned(),
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
            ldap_bind_prefix: Some("uid".to_owned()),
            ..MgmtConfig::default()
        };

        let actual = LDAPConfig::new(&given, &Some("alice".to_owned()), &Some("xxxx".to_owned()));

        assert_case(
            ExpectedLdapPaths {
                ldap_dc: "dc=example,dc=com".to_owned(),
                ldap_base: "ou=it,ou=people,dc=example,dc=com".to_owned(),
                ldap_bind: "uid=alice,ou=special,dc=example,dc=com".to_owned(),
            },
            &actual,
        );
    }
    {
        let given = MgmtConfig {
            ldap_domain_components: Some("dc=example,dc=com".to_owned()),
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(&given, &Some("alice".to_owned()), &Some("xxxx".to_owned()));
        assert_case(
            ExpectedLdapPaths {
                ldap_dc: "dc=example,dc=com".to_owned(),
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
            ..MgmtConfig::default()
        };
        let actual = LDAPConfig::new(&given, &Some("alice".to_owned()), &Some("xxxx".to_owned()));
        assert_case(
            ExpectedLdapPaths {
                ldap_dc: "dc=example,dc=com".to_owned(),
                ldap_base: "aaa=department,bbb=level,dc=example,dc=com".to_owned(),
                ldap_bind: "cn=alice,dc=example,dc=com".to_owned(),
            },
            &actual,
        );
    }

    fn assert_case(expected: ExpectedLdapPaths, actual: &LDAPConfig) {
        assert_eq!(
            expected.ldap_dc, actual.ldap_dc,
            "Did not produce correct ldap dc."
        );
        assert_eq!(
            expected.ldap_base, actual.ldap_base,
            "Did not produce correct base with  org path + dc."
        );
        assert_eq!(
            expected.ldap_bind, actual.ldap_bind,
            "Correct user binding, prefix_user + user + bind_org + dc"
        );
    }
}

#[test]
fn should_produce_simple_output() {
    let given_entries = vec![
        hashmap! {
            "qos" => vec!["basic", "default"],
            "name" => vec!["Mr. X"],
            "age" => vec!["2"]
        },
        hashmap! {
            "name" => vec!["example"]
        },
        hashmap! {
            "qos" => vec!["default"],
            "name" => vec!["example_man"],
        },
    ];
    let given_search_entries = vec!["qos", "name"];
    let actual =
        contruct_simple_output_from_vec_hash_map(&given_search_entries, given_entries.as_slice());
    insta::assert_display_snapshot!(actual);
}

#[test]
fn should_produce_table_from_ldap_search() {
    let given_entries = vec![
        hashmap! {
            "qos" => vec!["basic", "default"],
            "name" => vec!["Mr. X"],
            "age" => vec!["2"]
        },
        hashmap! {
            "name" => vec!["example"]
        },
    ];

    let given_search_entries = vec!["qos", "name"];
    let actual = contruct_table_from_vec_hash_map(&given_search_entries, given_entries.as_slice());
    insta::assert_display_snapshot!(actual);
}
