mod ldap_config;
mod ldap_credential;
mod ldap_paths;
mod ldap_search_result;
mod ldap_session;
mod ldap_simple_credential;
pub mod text_list_output;

use chrono::Utc;
pub use ldap_config::LDAPConfig;
pub use ldap_credential::LdapCredential;
pub use ldap_search_result::LdapSearchResult;
pub use ldap_session::LdapSession;
pub use ldap_simple_credential::LdapSimpleCredential;
use once_cell::sync::Lazy;

#[cfg(test)]
pub mod testing;
use crate::prelude::AppResult;
use crate::util::{get_new_uid, hashset_from_vec_str};
use crate::{prelude::*, NewEntity};
use crate::{ChangesToUser, MgmtConfig};
use ldap3::controls::{MakeCritical, RelaxRules};
use ldap3::{LdapConn, LdapError, LdapResult, Mod, Scope, SearchEntry, SearchResult};
use log::{debug, info, warn};
use maplit::hashset;
use std::collections::HashSet;

/// Tries to connect to a LDAP instance and authenticates as an user there.
///
/// # Errors
///
/// - If the connection to the LDAP instance fails.
/// - If the binding as the user fails aka authentication
pub fn make_ldap_connection<T>(ldap_config: &LDAPConfig<T>) -> AppResult<LdapConn>
where
    T: LdapCredential,
{
    let mut ldap = LdapConn::new(ldap_config.ldap_server())?;
    let _ = ldap
        .simple_bind(ldap_config.bind(), ldap_config.password()?)
        .with_context(|| {
            format!(
                "Failed to establish ldap connection via the bind {}",
                ldap_config.bind()
            )
        })?;
    Ok(ldap)
}

/// # Errors
///
/// - If the existence of the user can not be checked. See [`username_exists`]
/// - If determining the next UID fails. See [`find_next_available_uid`]
/// - If the adding of an user in the LDAP database failed.
pub fn add_ldap_user<T>(
    entity: &NewEntity,
    config: &MgmtConfig,
    ldap_session: &mut LdapSession<T>,
) -> AppResult
where
    T: LdapCredential,
{
    let exitence_of_username = username_exists(entity.username.as_ref(), ldap_session.config())?;
    if exitence_of_username {
        warn!(
            "User {} already exists in LDAP. Skipping LDAP user creation.",
            &entity.username
        );
        return Ok(());
    }

    let uid_number = find_next_available_uid(ldap_session, entity.group.id())
        .context("No users found or LDAP query failed. Unable to assign uid. Aborting...")?;

    debug!(
        "LDAP connection established to {}",
        ldap_session.config().bind()
    );

    add_to_ldap_db(entity, uid_number, ldap_session, config)?;

    info!("Added LDAP user {}", entity.username);
    return Ok(());

    fn add_to_ldap_db<T>(
        entity: &NewEntity,
        uid: u32,
        ldap_session: &mut LdapSession<T>,
        config: &MgmtConfig,
    ) -> AppResult
    where
        T: LdapCredential,
    {
        fn add_fields<T>(
            connection: &mut LdapConn,
            entity: &NewEntity,
            ldap_config: &LDAPConfig<T>,
            fields: Vec<(&str, HashSet<&str>)>,
        ) -> AppResult
        where
            T: LdapCredential,
        {
            let result_from_adding = connection.add(
                &format!("uid={},{}", entity.username, ldap_config.base()),
                fields,
            );

            ldap_is_success(result_from_adding).context("Unable to create LDAP user!")?;
            Ok(())
        }

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

        ldap_session.action(|connection, ldap_config| {
            let mut fields = vec![
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
            ];

            if config.ldap_add_created_at {
                let created_at = Utc::now().to_rfc3339();
                let attr = hashset! {created_at.as_str()};
                fields.push(("createdAt", attr));

                add_fields(connection, entity, ldap_config, fields)?;
                Ok(())
            } else {
                add_fields(connection, entity, ldap_config, fields)?;
                Ok(())
            }
        })
    }
}

/// # Errors
///
/// - If finding the LDAP-DN by the UID fails. See [`find_dn_by_uid`]
/// - If the deletion of an user in the LDAP database failed.
pub fn delete_ldap_user<T>(username: &str, ldap_session: &mut LdapSession<T>) -> AppResult
where
    T: LdapCredential,
{
    let dn = find_dn_by_uid(username, ldap_session)
        .with_context(|| format!("No DN found for username {}!", username))?;
    debug!(
        "LDAP connection established to {}",
        ldap_session.config().bind()
    );

    match &dn {
        Some(dn_to_delete) => {
            ldap_session.action(|ldap, _| {
                let result = ldap.delete(dn_to_delete);
                ldap_is_success(result)?;
                info!("Successfully deleted DN {}", dn_to_delete);
                Ok(())
            })?;
        }
        None => {
            warn!("No dn found to delete under the username {}", username);
        }
    }

    Ok(())
}

/// # Errors
///
/// - If finding the DN-LDAP  by the UID fails. See [`find_dn_by_uid`]
/// - If finding the quality of service by the UID fails. See [`find_qos_by_uid`]
pub fn modify_ldap_user<T>(
    modifiable: &ChangesToUser,
    ldap_session: &mut LdapSession<T>,
) -> AppResult
where
    T: LdapCredential,
{
    let dn = find_dn_by_uid(modifiable.username.as_ref(), ldap_session)
        .with_context(|| {
            format!(
                "No DN found for username {}! Unable to modify user.",
                modifiable.username.as_ref()
            )
        })?
        .ok_or(anyhow!("No dn found for uid"))?;

    // Prepare replace operation

    let old_qos = match &modifiable.qos {
        Some(_) => find_qos_by_uid(modifiable.username.as_ref(), ldap_session),
        None => Ok(Vec::default()),
    }?;
    let mod_vec = make_modification_vec(modifiable, &old_qos);

    // Replace userPassword at given dn
    ldap_session.action(|ldap_connection, _| {
        let result = ldap_connection
            .with_controls(RelaxRules.critical())
            .modify(&dn, mod_vec);
        ldap_is_success(result).context("User modification in LDAP failed!")
    })?;

    info!("Successfully modified user {} in LDAP", modifiable.username);
    Ok(())
}

/// List all LDAP users and some attributes
///
/// It currently outputs all values in line separated by commas.
///
/// # Errors
///
/// - If the connection to the LDAP instance fails. See [`make_ldap_connection`]
/// - If the searching in LDAP failed
pub fn list_ldap_users<T>(ldap_config: &LDAPConfig<T>) -> AppResult<LdapSearchResult>
where
    T: LdapCredential,
{
    // Establish LDAP connection and bind
    let mut ldap =
        make_ldap_connection(ldap_config).context("Error while connecting via LDAP !")?;

    debug!(
        "LDAP connection established to {}. Will search under {}",
        ldap_config.bind(),
        ldap_config.base()
    );

    let attrs = SORTED_LDAP_LISTING_ATTRIBUTES.as_slice();
    // Search for all entities under base dn
    let search_result = ldap
        .search(
            ldap_config.base(),
            Scope::OneLevel,
            "(objectclass=*)",
            attrs,
        )
        .context("Error during LDAP search!")?;

    let search_result = LdapSearchResult::from_ldap_raw_search(attrs.iter(), &search_result);

    Ok(search_result)
}

/// Creates modification parameters which are used by `ldap3` library to modify an user in LDAP.
fn make_modification_vec<'a>(
    modifiable: &'a ChangesToUser,
    old_qos: &'a Vec<String>,
) -> Vec<Mod<&'a str>> {
    macro_rules! may_push_simple_modification {
        ($name:expr, $modifiable:ident, $modification:ident, $field:ident) => {
            if let Some(val) = &$modifiable.$field {
                info_log($name);
                ($modification).push(Mod::Replace($name, HashSet::from([val.as_ref().as_str()])))
            }
        };
    }
    let mut modifications: Vec<Mod<&str>> = Vec::new();

    let modifiable = modifiable.as_ref();
    may_push_simple_modification!("givenName", modifiable, modifications, firstname);
    may_push_simple_modification!("sn", modifiable, modifications, lastname);
    may_push_simple_modification!("mail", modifiable, modifications, mail);
    may_push_simple_modification!("slurmDefaultQos", modifiable, modifications, default_qos);
    may_push_simple_modification!("publickey", modifiable, modifications, publickey);

    let replace_old_with_new_qos = !old_qos.is_empty();
    if replace_old_with_new_qos {
        // first we delete all old qos
        const SLURM_QOS: &str = "slurmQos";
        info_log(SLURM_QOS);
        for q in old_qos {
            modifications.push(Mod::Delete(SLURM_QOS, HashSet::from([q.as_str()])))
        }
        // then we add all new qos
        for q in modifiable.qos.iter() {
            let q: HashSet<&str> = q.into_iter().map(|qos| qos.as_ref().as_str()).collect();
            modifications.push(Mod::Add(SLURM_QOS, q))
        }
    }
    return modifications;

    fn info_log(field: &str) {
        info!("Changing the field: {}", field)
    }
}

/// Does a LDAP search to determine the next available UID needed by a new user.
/// The parameter `group` determines in which range a next available UID is found.
///
/// # Errors
///
/// - If establishing the connection to the LDAP instance fails.
/// - If the new UID can not be valid. See [`get_new_uid`] for more details
pub fn find_next_available_uid<T>(
    ldap_session: &mut LdapSession<T>,
    group: crate::Group,
) -> AppResult<u32>
where
    T: LdapCredential,
{
    {
        let config = ldap_session.config();
        debug!(
            "find_next_available_uid: LDAP connection established to {}",
            config.bind(),
        );

        debug!("Search under {}", config.base());
    }

    // Search for all uidNumbers under base dn
    let search_result = ldap_session.action(|connection, config| {
        connection
            .search(
                config.base(),
                Scope::OneLevel,
                "(objectclass=*)",
                vec!["uidNumber"],
            )
            .context("Error during uid search!")
    })?;
    let mut uids: Vec<u32> = Vec::new();
    for elem in search_result.0.iter() {
        let search_result = SearchEntry::construct(elem.to_owned());
        debug!("UID: {:?}", SearchEntry::construct(elem.to_owned()));
        let uid = {
            const ATTRIBUTE: &str = "uidNumber";
            let unparsed = &search_result.attrs[ATTRIBUTE].first().ok_or_else(|| {
                anyhow!(
                    "No uid under the attribute `{}` in the LDPA search ",
                    ATTRIBUTE
                )
            })?;
            unparsed.parse::<u32>().with_context(|| format!("Uid `{}` for ldap operation could not be parsed into unsigned integer 32 value", unparsed))?
        };

        uids.push(uid);
    }

    get_new_uid(&uids, group)
}

/// Search for a specific UID and return the corresponding dn.
///
/// # Errors
///
/// - If the connection to a LDAP instance can not be established
/// - If nothing is found in the LDAP query under the given user aka parameter `username`
pub fn find_dn_by_uid<T>(
    username: &str,
    ldap_session: &mut LdapSession<T>,
) -> AppResult<Option<String>>
where
    T: LdapCredential,
{
    debug!(
        "LDAP connection established to {}",
        ldap_session.config().bind()
    );

    // Search for all uids under base dn and return dn of user
    let search: SearchResult = ldap_session.action(|con, config| {
        con.search(
            config.base(),
            Scope::OneLevel,
            &format!("(uid={username})"),
            vec!["dn"],
        )
        .context("LDAP search failed")
    })?;

    let entry = search
        .0
        .into_iter()
        .next()
        .with_context(|| format!("No LDAP entry found for user {}", username))?;

    let sr = SearchEntry::construct(entry);
    debug!("Search result for deletion: {:?}", sr);

    Ok(Some(sr.dn))
}

/// Search for a specific uid and return the corresponding qos.
/// # Errors
///
/// - If the connection to the LDAP instance fails
/// - If nothing is found in the LDAP query under the given user aka parameter `username`
pub fn find_qos_by_uid<T>(
    username: &str,
    ldap_session: &mut LdapSession<T>,
) -> AppResult<Vec<String>>
where
    T: LdapCredential,
{
    let mut fetched_all_qos: Vec<String> = Vec::new();

    debug!(
        "LDAP connection established to {}",
        ldap_session.config().bind()
    );

    // Search for all uid under base dn and return dn of user
    let search = ldap_session.action(|ldap_connection, ldap_config| {
        ldap_connection
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
            })
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
///
/// # Errors
///
/// - If the connection to the LDAP instance fails
/// - If nothing is found in the LDAP query under the given user aka parameter `username`
pub fn username_exists<T>(username: &String, ldap_config: &LDAPConfig<T>) -> AppResult<bool>
where
    T: LdapCredential,
{
    let mut username_exists = false;
    let mut ldap = make_ldap_connection(ldap_config)?;
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

/// # Errors
///
/// - If the username could not be retrieved
/// - If the password could not be retrieved
fn ask_credentials_if_not_provided<T>(
    username: Option<&str>,
    password: Option<&str>,
    credential: &T,
) -> AppResult<(String, String)>
where
    T: LdapCredential,
{
    let (ldap_user, ldap_pass) = match (username, password) {
        (None, None) => (credential.username()?, credential.password()?),
        (Some(username), None) => (username, credential.password()?),
        (None, Some(password)) => (credential.username()?, password),
        (Some(username), Some(password)) => (username, password),
    };

    Ok((ldap_user.trim().to_owned(), ldap_pass.trim().to_owned()))
}

static SORTED_LDAP_LISTING_ATTRIBUTES: Lazy<Vec<&str>> = Lazy::new(|| {
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
        "createdAt",
    ];
    to_sort.sort();
    to_sort
});
