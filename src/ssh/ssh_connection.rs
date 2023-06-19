use anyhow::{anyhow, bail, Context};
use std::io::Read;
use std::net::TcpStream;

use once_cell::unsync::OnceCell;

use log::{info, warn};
use ssh2::Session;

use crate::config::MgmtConfig;
use crate::prelude::AppResult;
use crate::ssh::{self, EntitiesAndSshAgent};

use super::SshCredential;

pub struct SshConnection<'a, 'b> {
    endpoint: &'a str,
    port: u32,
    ssh_agent: bool,
    credentials: &'b SshCredential<'b>,
    session: OnceCell<Session>,
}

impl<'a, 'b> SshConnection<'a, 'b> {
    pub fn new(endpoint: &'a str, config: &MgmtConfig, credentials: &'b SshCredential) -> Self {
        Self {
            endpoint,
            port: config.ssh_port,
            ssh_agent: config.ssh_agent,
            credentials,
            session: OnceCell::new(),
        }
    }

    pub fn from_head_node(config: &'a MgmtConfig, credentials: &'b SshCredential) -> Self {
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
        let session = self.session.get_or_try_init(|| -> AppResult<Session> {
            info!("Connecting to host {}", self.endpoint);

            let mut sess = Session::new().context("Could not build up ssh session")?;

            {
                let tcp = TcpStream::connect(format!("{}:{}", self.endpoint, self.port))
                    .with_context(|| {
                        format!(
                            "Could not connect over tcp to endpoint: {} over port: {}",
                            self.endpoint, self.endpoint
                        )
                    })?;
                sess.set_tcp_stream(tcp);
            }

            sess.handshake()
                .context("Could not perform ssh handshake")?;

            let username = self.credentials.username()?;

            auth(self, &mut sess, username)?;

            return Ok(sess);

            fn simple_password_auth(
                session: &mut Session,
                session_connection: &SshConnection,
                username: &str,
            ) -> AppResult {
                let password = session_connection.credentials.password()?;
                session
                    .userauth_password(username, password)
                    .context("Authentication has failed with provided username/password.")?;
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
            fn auth(
                connection: &SshConnection,
                session: &mut Session,
                username: &str,
            ) -> AppResult {
                if connection.ssh_agent {
                    match try_authenticate_via_ssh_agent(session, username) {
                        Ok(_) => {
                            info!(
                                "Authentication via ssh agent successeded with username {}",
                                username
                            );
                        }
                        Err(agent_error) => {
                            warn!(
                                "Authentication via ssh agent failed with username ({}).
                                \n Details: {}",
                                username, agent_error
                            );
                            simple_password_auth(session, connection, username)?;
                        }
                    }
                } else {
                    simple_password_auth(session, connection, username)?;
                }

                Ok(())
            }
        })?;

        let mut channel = session
            .channel_session()
            .context("Could not create channel for ssh session")?;

        channel
            .exec(cmd)
            .context("Execution of command on remote machine over ssh has failed.")?;

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .context("Could not read output of executed commmand over ssh channel")?;
        let exit_status = channel
            .exit_status()
            .context("Could not retrieve exit code of executed command over ssh")?;

        Ok((output, exit_status))
    }
}

/// Tries to authenticate an user via an active ssh agent.
/// If more than one key is registered in the ssh agent, user is asked which one to use via prompt in the
/// terminal.
///
/// # Errors
///
/// - If no ssh agent is accessible.
/// - If no key is registered withing ssh agent
/// - If the selection from user is not within the available range of ssh keys registered withing
/// ssh agent .
fn try_authenticate_via_ssh_agent(session: &mut Session, username: &str) -> AppResult<()> {
    let keys = ssh::get_agent_with_all_entities(session)?;

    let (agent, chosen_key) = match keys {
        EntitiesAndSshAgent::None => bail!("No keys could be found on the ssh agent."),
        EntitiesAndSshAgent::One(agent, only_key) => (agent, only_key),
        EntitiesAndSshAgent::Many(agent, to_choose_from) => {
            let length = to_choose_from.len();
            println!("Found more than one key in ssh agent !");
            println!("Chooose one between {} and {} ssh key", 0, length);
            println!("===========================================");

            for (index, next) in to_choose_from.iter().enumerate() {
                let comment = next.comment();
                println!("{} => comment: {}", index, comment);
            }

            let user_choice: usize = crate::util::user_input::line_input_from_user()?
                .ok_or_else(|| anyhow!("No number supplied"))?
                .parse()?;

            if length <= user_choice {
                bail!("Choice should between {} and {}", 0, length);
            } else {
                info!("{}. ssh key is chosen", user_choice);
                (agent, to_choose_from.into_iter().nth(user_choice).unwrap())
            }
        }
    };

    info!(
        "Using the sh key with comment ({}) from the ssh agent",
        chosen_key.comment()
    );

    agent.userauth(username, &chosen_key)?;
    Ok(())
}
