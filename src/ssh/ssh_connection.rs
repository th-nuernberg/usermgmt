use anyhow::Context;
use std::io::Read;
use std::net::TcpStream;

use once_cell::unsync::OnceCell;

use log::info;
use ssh2::Session;

use crate::config::MgmtConfig;
use crate::prelude::AppResult;

use super::SshCredential;

pub struct SshSession<'a, 'b> {
    endpoint: &'a str,
    port: u32,
    credentials: &'b SshCredential<'b>,
    session: OnceCell<Session>,
}

impl<'a, 'b> SshSession<'a, 'b> {
    pub fn new(endpoint: &'a str, port: u32, credentials: &'b SshCredential) -> Self {
        Self {
            endpoint,
            port,
            credentials,
            session: OnceCell::new(),
        }
    }

    pub fn from_head_node(config: &'a MgmtConfig, credentials: &'b SshCredential) -> Self {
        Self::new(&config.head_node, config.ssh_port, credentials)
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

            let (username, password) = (self.credentials.username()?, self.credentials.password()?);
            sess.userauth_password(username, password)
                .context("Authentication has failed with provided username/password.")?;

            Ok(sess)
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
