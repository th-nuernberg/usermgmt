//! TODO: Implement LDAP credential Struct for centralizing username and password acquisition.

mod ldap_config;
mod ldap_paths;
mod ldap_readonly_config;
mod text_list_output;
pub use ldap_config::LDAPConfig;
#[cfg(test)]
pub mod testing;
use crate::prelude::AppResult;
use crate::util::user_input;
use crate::util::{get_new_uid, hashset_from_vec_str};
use crate::MgmtConfig;
use crate::{prelude::*, Entity, NewEntity};
/// LDAP operations using the ldap3 lib
use ldap3::controls::{MakeCritical, RelaxRules};
use ldap3::{LdapConn, LdapError, LdapResult, Mod, Scope, SearchEntry};
use log::{debug, info, warn};
use maplit::hashset;
use std::collections::HashSet;

use self::ldap_readonly_config::LdapReadonlyConfig;

fn make_ldap_connection(server: &str) -> Result<LdapConn, LdapError> {
    LdapConn::new(server)
}

pub fn add_ldap_user(entity: &NewEntity, config: &MgmtConfig) -> AppResult {
    let ldap_config = LDAPConfig::new(config, &None, &None)?;

    let exitence_of_username = username_exists(entity.username.as_ref(), &ldap_config)?;
    if exitence_of_username {
        warn!(
            "User {} already exists in LDAP. Skipping LDAP user creation.",
            &entity.username
        );
        return Ok(());
    }

    let uid_number = find_next_available_uid(&ldap_config, entity.group.id())
        .context("No users found or LDAP query failed. Unable to assign uid. Aborting...")?;

    let mut ldap_connection = make_ldap_connection(&ldap_config.ldap_server)?;

    ldap_connection.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;
    debug!("LDAP connection established to {}", ldap_config.bind());

    add_to_ldap_db(entity, uid_number, ldap_connection, config, &ldap_config)?;

    info!("Added LDAP user {}", entity.username);
    return Ok(());

    fn add_to_ldap_db(
        entity: &NewEntity,
        uid: u32,
        mut ldap_connection: LdapConn,
        config: &MgmtConfig,
        ldap_config: &LDAPConfig,
    ) -> AppResult {
        let un = entity.username.as_ref().as_str();
        let gid = entity.group.gid().to_string();
        let uid = uid.to_string();
        let ln = entity.lastname.as_ref().as_str();
        let gn = entity.firstname.as_ref().as_str();
        let mail: &str = entity
            .mail
            .as_ref()
            .map(|trimmmed| trimmmed.as_ref().as_str())
            .unwrap_or("");

        let def_qos = entity.default_qos.as_ref().as_str();
        let home = &format!("/home/{}", entity.username);
        let qos: HashSet<&str> = (&entity.qos)
            .into_iter()
            .map(|qos| qos.as_ref().as_str())
            .collect();
        let pubkey = entity
            .publickey
            .as_ref()
            .map(|trimmmed| trimmmed.as_ref().as_str())
            .unwrap_or("");

        let result_form_adding = ldap_connection.add(
            &format!("uid={},{}", entity.username, ldap_config.base()),
            vec![
                ("cn", hashset! {un}),
                (
                    "objectClass",
                    hashset_from_vec_str(&config.objectclass_common).to_owned(),
                ),
                ("gidNumber", hashset! {gid.as_str()}),
                ("uidNumber", hashset! {uid.as_str()}),
                ("uid", hashset! {un}),
                ("sn", hashset! {ln}),
                ("givenName", hashset! {gn}),
                ("mail", hashset! {mail}),
                ("slurmDefaultQos", hashset! {def_qos}),
                ("homeDirectory", hashset! {home.as_str()}),
                ("slurmQos", qos),
                ("sshPublicKey", hashset! {pubkey}),
                ("loginShell", hashset! {config.login_shell.as_str()}),
            ],
        );

        ldap_is_success(result_form_adding).context("Unable to create LDAP user!")?;
        Ok(())
    }
}

pub fn delete_ldap_user(username: &str, config: &MgmtConfig) -> AppResult {
    let ldap_config = LDAPConfig::new(config, &None, &None)?;
    // get dn for uid
    let dn = find_dn_by_uid(username, &ldap_config)
        .with_context(|| format!("No DN found for username {}!", username))?;
    let mut ldap = make_ldap_connection(&ldap_config.ldap_server)?;
    debug!("LDAP connection established to {}", ldap_config.bind());

    let _ = ldap
        .simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)
        .with_context(|| {
            format!(
                "Failed to establish ldap connection via the bind {}",
                ldap_config.bind()
            )
        })?;

    match &dn {
        Some(dn_to_delete) => {
            ldap_is_success(ldap.delete(dn_to_delete)).context("User deletion in LDAP failed!")?;
            info!("Successfully deleted DN {}", dn_to_delete);
        }
        None => {
            warn!("No dn found to delete under the username {}", username);
        }
    }

    Ok(())
}

pub fn modify_ldap_user(modifiable: &Entity, config: &MgmtConfig) -> AppResult {
    let ldap_config = LDAPConfig::new(config, &None, &None)?;

    let dn = find_dn_by_uid(modifiable.username.as_ref(), &ldap_config)
        .with_context(|| {
            format!(
                "No DN found for username {}! Unable to modify user.",
                modifiable.username.as_ref()
            )
        })?
        .ok_or(anyhow!("No dn found for uid"))?;

    let mut ldap = make_ldap_connection(&ldap_config.ldap_server)?;
    ldap.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;
    debug!("LDAP connection established to {}", ldap_config.bind());

    // Prepare replace operation
    let old_qos = find_qos_by_uid(
        modifiable.username.as_ref(),
        config,
        ldap_config.username(),
        &ldap_config.ldap_pass,
    )?;
    let mod_vec = make_modification_vec(modifiable, &old_qos);

    // Replace userPassword at given dn
    let res = ldap
        .with_controls(RelaxRules.critical())
        .modify(&dn, mod_vec);

    ldap_is_success(res).context("User modification in LDAP failed!")?;
    info!("Successfully modified user {} in LDAP", modifiable.username);
    Ok(())
}

/// List all LDAP users and some attributes
///
/// It currently outputs all values in line separated by commas.
pub fn list_ldap_users(config: &MgmtConfig, simple_output_ldap: bool) -> AppResult {
    let ldap_config = LdapReadonlyConfig::new(config)?;

    // Establish LDAP connection and bind
    let mut ldap = make_ldap_connection(ldap_config.ldap_server())
        .context("Error while connecting via LDAP !")?;

    ldap.simple_bind(ldap_config.bind(), ldap_config.ldap_pass())
        .context("Error during LDAP binding!")?;

    debug!(
        "LDAP connection established to {}. Will search under {}",
        ldap_config.bind(),
        ldap_config.base()
    );
    let attrs = {
        // Make sure the keys are sorted alphabetic
        // This way the order fields in the final output deterministic
        let mut to_sort = vec![
            "uid",
            "uidNumber",
            "givenName",
            "sn",
            "mail",
            "slurmDefaultQos",
            "slurmQos",
        ];
        to_sort.sort();
        to_sort
    };

    // Search for all entities under base dn
    let search_result = ldap
        .search(
            ldap_config.base(),
            Scope::OneLevel,
            "(objectclass=*)",
            attrs.clone(),
        )
        .context("Error during LDAP search!")?;

    let output = if simple_output_ldap {
        text_list_output::ldap_simple_output(&attrs, &search_result)
    } else {
        text_list_output::ldap_search_to_pretty_table(&attrs, &search_result)
    };

    println!("{}", output);
    Ok(())
}

fn make_modification_vec<'a>(
    modifiable: &'a Entity,
    old_qos: &'a Vec<String>,
) -> Vec<Mod<&'a str>> {
    let mut modifications: Vec<Mod<&str>> = Vec::new();

    if let Some(firstname) = &modifiable.firstname {
        modifications.push(Mod::Replace(
            "givenName",
            HashSet::from([firstname.as_ref().as_str()]),
        ))
    }

    if let Some(lastname) = &modifiable.lastname {
        modifications.push(Mod::Replace(
            "sn",
            HashSet::from([lastname.as_ref().as_str()]),
        ))
    }

    if let Some(mail) = &modifiable.mail {
        modifications.push(Mod::Replace(
            "mail",
            HashSet::from([mail.as_ref().as_str()]),
        ))
    }

    if let Some(default_qos) = &modifiable.default_qos {
        modifications.push(Mod::Replace(
            "slurmDefaultQos",
            HashSet::from([default_qos.as_ref().as_str()]),
        ))
    }

    if let Some(publickey) = &modifiable.publickey {
        debug!("Pushing modifiable publickey {}", publickey);
        modifications.push(Mod::Replace(
            "sshPublicKey",
            HashSet::from([publickey.as_ref().as_str()]),
        ))
    }

    if !old_qos.is_empty() {
        // first we delete all old qos
        for q in old_qos {
            modifications.push(Mod::Delete("slurmQos", HashSet::from([q.as_str()])))
        }
        // then we add all new qos
        for q in modifiable.qos.iter() {
            let q: HashSet<&str> = q.into_iter().map(|qos| qos.as_ref().as_str()).collect();
            modifications.push(Mod::Add("slurmQos", q))
        }
    }
    modifications
}

/// Do a LDAP search to determine the next available uid
fn find_next_available_uid(ldap_config: &LDAPConfig, group: crate::Group) -> AppResult<u32> {
    let mut ldap =
        make_ldap_connection(&ldap_config.ldap_server).context("Error during uid search!")?;

    debug!(
        "Binding with dn: {}, pw: {}",
        ldap_config.bind(),
        ldap_config.ldap_pass
    );

    let bind = ldap.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;
    debug!(
        "find_next_available_uid: LDAP connection established to {}, {}",
        ldap_config.bind(),
        bind
    );

    debug!("Search under {}", ldap_config.base());

    // Search for all uidNumbers under base dn
    let search_result = ldap
        .search(
            ldap_config.base(),
            Scope::OneLevel,
            "(objectclass=*)",
            vec!["uidNumber"],
        )
        .context("Error during uid search!")?;

    let mut uids: Vec<u32> = Vec::new();
    for elem in search_result.0.iter() {
        let search_result = SearchEntry::construct(elem.to_owned());
        debug!("UID: {:?}", SearchEntry::construct(elem.to_owned()));
        let uid = &search_result.attrs["uidNumber"][0].parse::<u32>().unwrap();
        uids.push(*uid);
    }

    get_new_uid(&uids, group)
}

/// Search for a specific uid and return the corresponding dn.
fn find_dn_by_uid(username: &str, ldap_config: &LDAPConfig) -> AppResult<Option<String>> {
    let mut ldap = make_ldap_connection(&ldap_config.ldap_server)?;
    ldap.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;
    debug!("LDAP connection established to {}", ldap_config.bind());

    // Search for all uids under base dn and return dn of user
    let search = ldap.search(
        ldap_config.base(),
        Scope::OneLevel,
        &format!("(uid={username})"),
        vec!["dn"],
    )?;

    let entry = search
        .0
        .into_iter()
        .next()
        .with_context(|| format!("No LDAP entry found for user {}", username))?;

    let sr = SearchEntry::construct(entry);
    debug!("SR for deletion: {:?}", sr);

    Ok(Some(sr.dn))
}

/// Search for a specific uid and return the corresponding qos.
fn find_qos_by_uid(
    username: &str,
    config: &MgmtConfig,
    ldap_user: &str,
    ldap_pass: &str,
) -> AppResult<Vec<String>> {
    let ldap_config = LDAPConfig::new(
        config,
        &Some(ldap_user.to_string()),
        &Some(ldap_pass.to_string()),
    )?;
    let mut fetched_all_qos: Vec<String> = Vec::new();

    let ldap_server = &ldap_config.ldap_server;
    let mut ldap_connection = make_ldap_connection(ldap_server)
        .with_context(|| format!("Connection to {} failed", ldap_server))?;

    ldap_connection.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;

    debug!("LDAP connection established to {}", ldap_config.bind());

    // Search for all uid under base dn and return dn of user
    let search = ldap_connection
        .search(
            ldap_config.base(),
            Scope::OneLevel,
            &format!("(uid={})", username),
            vec!["slurmQos"],
        )
        .with_context(|| {
            format!(
                "search did not find any slurmQos for the user with uid {}",
                username
            )
        })?;

    for elem in search.0.iter() {
        let search_result = SearchEntry::construct(elem.to_owned());
        let q = &search_result.attrs["slurmQos"];
        for one_qos in q {
            debug!("Fetched QOS: {:?}", one_qos);
            fetched_all_qos.push(one_qos.clone());
        }
    }

    Ok(fetched_all_qos)
}

/// Check if username already exists in ldap.
/// Must be an exact match on the uid attribute.
/// TODO: Bubble up error instead of just logging it
fn username_exists(username: &String, ldap_config: &LDAPConfig) -> AppResult<bool> {
    let mut username_exists = false;
    let mut ldap = make_ldap_connection(&ldap_config.ldap_server)?;
    ldap.simple_bind(ldap_config.bind(), &ldap_config.ldap_pass)?;
    debug!("LDAP connection established to {}", ldap_config.bind());

    // Search for all uid under base dn and return dn of user
    let search_result = ldap.search(
        ldap_config.base(),
        Scope::OneLevel,
        &format!("(uid={username})"),
        vec!["dn"],
    )?;
    match search_result.0.into_iter().next() {
        Some(entry) => {
            // User found. Good.
            debug!("Found user: {:?}", SearchEntry::construct(entry));
            username_exists = true
        }
        None => debug!("No LDAP entry found for user {}", username),
    }
    Ok(username_exists)
}

/// If ok is returned then ldap operation happened with zero error code, LDAP_SUCCESS
///
/// Even if a call to ldap returns ok it has an error code inside it. Only if the code is zero
/// then the operation really happened successfully.
/// Link: https://docs.rs/ldap3/latest/ldap3/result/struct.LdapResult.html
fn ldap_is_success(to_check: Result<LdapResult, LdapError>) -> Result<(), LdapError> {
    match to_check {
        Ok(might_have_non_zero_error_code) => match might_have_non_zero_error_code.success() {
            Ok(_with_zero_error_code) => Ok(()),
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    }
}

fn ask_credentials_if_not_provided(
    username: Option<&str>,
    password: Option<&str>,
    on_credentials: impl FnOnce() -> AppResult<(String, String)>,
) -> AppResult<(String, String)> {
    let (ldap_user, ldap_pass) = match username {
        Some(u) => match password {
            Some(p) => Ok((u.to_owned(), p.to_owned())),
            None => on_credentials(),
        },
        None => on_credentials(),
    }?;

    return Ok((ldap_user.trim().to_owned(), ldap_pass.trim().to_owned()));
}

fn ask_credentials_in_tty() -> AppResult<(String, String)> {
    println!("Enter your LDAP username (defaults to admin):");
    let username = user_input::line_input_from_user()?.unwrap_or_else(|| "admin".to_string());
    let password = user_input::ask_for_password("Enter your LDAP password: ")
        .context("Failed to retrieve password from user in a terminal")?
        .ok_or_else(|| anyhow!("No password provided"))?;
    Ok((username, password))
}
