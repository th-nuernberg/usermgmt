use crate::prelude::*;

use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use once_cell::unsync::OnceCell;

use log::{info, warn};
use ssh2::Session;

use crate::config::MgmtConfig;
use crate::prelude::AppResult;
use crate::ssh::{self, EntitiesAndSshAgent, SshPublicKeySuggestion};

use super::SshCredentials;

pub struct SshConnection<'a, T> {
    endpoint: &'a str,
    port: u32,
    ssh_agent: bool,
    credentials: T,
    session: OnceCell<Session>,
}

impl<'a, T> SshConnection<'a, T>
where
    T: SshCredentials,
{
    pub fn password(&self) -> AppResult<&str> {
        self.credentials.password()
    }

    pub fn username(&self) -> AppResult<&str> {
        self.credentials.username()
    }

    pub fn new(endpoint: &'a str, config: &MgmtConfig, credentials: T) -> Self {
        Self {
            endpoint,
            port: config.ssh_port,
            ssh_agent: config.ssh_agent,
            credentials,
            session: OnceCell::new(),
        }
    }

    pub fn from_head_node(config: &'a MgmtConfig, credentials: T) -> Self {
        Self::new(&config.head_node, config, credentials)
    }

    /// Tries to execute a given command on a remote machine over ssh
    ///
    /// # Error
    ///
    /// - If connection over tcp to endpoint failed.
    /// - If Authentication failed.
    /// - If remote command could not be executed.
    /// - If output or exit code of executed remote command could not be retrieved.
    ///
    pub fn exec(&self, cmd: &str) -> AppResult<(String, i32)> {
        let session = self
            .session
            .get_or_try_init(|| -> AppResult<Session> { self.establish_connection() })?;

        let mut channel = session
            .channel_session()
            .context("Unable to create channel for SSH session")?;

        channel
            .exec(cmd)
            .context("Execution of command on remote machine over SSH has failed.")?;

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .context("Could not read output of executed command over SSH channel")?;
        let exit_status = channel
            .exit_status()
            .context("Could not retrieve exit code of executed command over SSH")?;

        Ok((output, exit_status))
    }

    pub fn establish_connection(&self) -> AppResult<Session> {
        info!("Connecting to host {}", self.endpoint);

        let mut sess = Session::new().context("Unable to build SSH session")?;
        let timeout = constants::SSH_TIME_OUT_MILL_SECS;
        let socket_addr: String = format!("{}:{}", self.endpoint, self.port);
        let socket_addr: SocketAddr = socket_addr
            .parse()
            .with_context(|| format!("Socket address is not valid: {}", socket_addr))?;
        sess.set_timeout(timeout);

        {
            let tcp =
                TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout as u64))
                    .with_context(|| {
                        format!(
                            "Unable to connect over tcp to endpoint {} via port {}",
                            self.endpoint, self.port
                        )
                    })?;
            sess.set_tcp_stream(tcp);
        }

        sess.handshake()
            .context("Unable to perform SSH handshake")?;

        auth(self, &mut sess, &self.credentials)?;

        return Ok(sess);

        fn simple_password_auth<T>(
            session: &mut Session,
            session_connection: &SshConnection<T>,
            username: &str,
        ) -> AppResult
        where
            T: SshCredentials,
        {
            let password = session_connection.password()?;
            session
                .userauth_password(username, password)
                .context("Authentication has failed with provided username/password.")?;
            Ok(())
        }

        fn direct_key_path_auth<T>(
            session: &mut Session,
            session_connection: &SshConnection<T>,
            credentials: &impl SshCredentials,
        ) -> AppResult
        where
            T: SshCredentials,
        {
            info!("Trying to authenticate over SSH by using key pair");
            let username = credentials.username()?;
            let password = session_connection.password()?;
            let pair = credentials
                .ssh_paths_pair_key()
                .ok_or_else(|| anyhow!("No key pair provided"))?;
            let (public, private) = (pair.pub_key(), pair.private_key());
            info!(
                "SSH key pair at ({:?}) and ({:?}) is used for authentication",
                public, private
            );
            session.userauth_pubkey_file(username, Some(public), private, Some(password))?;
            Ok(())
        }

        /// Conducts the authentication on the session.
        /// It tries to first authenticate via ssh agent if allowed in the configuration.
        /// After a failed ssh agent authentication or without it,
        /// a simple username and password authentication is done as an alternative
        ///
        /// # Errors
        ///
        /// If none of authentication methods succeeded, ssh agent and password
        /// authentication.
        ///
        fn auth<T>(
            connection: &SshConnection<T>,
            session: &mut Session,
            cred: &impl SshCredentials,
        ) -> AppResult
        where
            T: SshCredentials,
        {
            let username = cred.username()?;
            if connection.ssh_agent {
                match try_authenticate_via_ssh_agent(session, &connection.credentials, username) {
                    Ok(_) => {
                        info!(
                            "Authentication via SSH agent succeeded with username {}",
                            username
                        );
                    }
                    Err(agent_error) => {
                        warn!(
                            "Authentication via SSH agent failed with username ({}).
                                \n Details: {}",
                            username, agent_error
                        );
                        pub_key_file_or_simple_auth(connection, session, cred)?;
                    }
                }
            } else {
                pub_key_file_or_simple_auth(connection, session, cred)?;
            }

            return Ok(());

            fn pub_key_file_or_simple_auth<T>(
                connection: &SshConnection<T>,
                session: &mut Session,
                cred: &impl SshCredentials,
            ) -> AppResult
            where
                T: SshCredentials,
            {
                let username = cred.username()?;
                if let Err(error) = direct_key_path_auth(session, connection, cred) {
                    warn!(
                        "Could not connect over SSH via key file's path\n Details: {}",
                        error
                    );
                    simple_password_auth(session, connection, username)?;
                }
                Ok(())
            }
        }
    }
}

/// Tries to authenticate a user via an active SSH agent.
/// If more than one key is registered in the SSH agent, user is asked which one to use via prompt in the
/// terminal.
///
/// # Errors
///
/// - If no SSH agent is accessible.
/// - If no key is registered within ssh agent
/// - If the selection from user is not within the available range of SSH keys registered within
///     SSH agent .
fn try_authenticate_via_ssh_agent(
    session: &mut Session,
    credentials: &impl SshCredentials,
    username: &str,
) -> AppResult<()> {
    let keys = ssh::get_agent_with_all_entities(session)?;

    let (agent, chosen_key) = match keys {
        EntitiesAndSshAgent::None => bail!("No keys found in SSH agent session."),
        EntitiesAndSshAgent::One(agent, only_key) => (agent, only_key),
        EntitiesAndSshAgent::Many(agent, to_choose_from) => {
            let choice: Vec<SshPublicKeySuggestion> = to_choose_from
                .iter()
                .map(SshPublicKeySuggestion::from)
                .collect();

            let user_choice = credentials.auth_agent_resolve(choice)?;
            (
                agent,
                to_choose_from.into_iter().nth(user_choice).expect(
                    "Function for retrieving entities guarantees that we have elements at this point.",
                ),
            )
        }
    };

    info!("Using SSH key '{}' from SSH agent", chosen_key.comment());

    agent.userauth(username, &chosen_key)?;
    Ok(())
}
