use crate::prelude::AppResult;
use log::debug;
mod ssh_connection;

use ssh2::{Agent, PublicKey, Session};
mod ssh_credentials;
mod ssh_given_credential;
mod ssh_public_key_suggestion;

pub use ssh_connection::SshConnection;
pub use ssh_credentials::SshCredentials;
pub use ssh_given_credential::SshGivenCredential;
pub use ssh_public_key_suggestion::SshPublicKeySuggestion;

/// Contains all accessible ssh keys with their ssh agent.
pub enum EntitiesAndSshAgent {
    /// No identity was added to ssh agent
    None,
    /// Agent with the ssh key which is the only one, registered into the ssh agent
    One(Agent, PublicKey),
    /// Agent with with more than one ssh key, registered into the ssh agent
    Many(Agent, Vec<PublicKey>),
}

/// Executes given command `cmd` on remote machine over ssh
pub fn run_remote_command<C>(sess: &SshConnection<C>, cmd: &str) -> AppResult<(i32, String)>
where
    C: SshCredentials,
{
    debug!("Running command {}", cmd);

    let (s, exit_status) = sess.exec(cmd)?;

    debug!("command output: {}", s);
    debug!("command exit status: {}", exit_status);
    Ok((exit_status, s))
}

/// Tries get all identities, pub keys, from the active ssh agent.
///
/// # Errors
///
/// - If agent could not retrieve identities because no agent found, connection could not be
/// established and so on
pub fn get_agent_with_all_entities(session: &mut Session) -> AppResult<EntitiesAndSshAgent> {
    let mut agent = session.agent()?;
    agent.connect()?;
    agent.list_identities()?;
    let keys = agent.identities()?;

    if keys.is_empty() {
        Ok(EntitiesAndSshAgent::None)
    } else if keys.len() == 1 {
        Ok(EntitiesAndSshAgent::One(
            agent,
            keys.into_iter().next().unwrap(),
        ))
    } else {
        Ok(EntitiesAndSshAgent::Many(agent, keys))
    }
}
