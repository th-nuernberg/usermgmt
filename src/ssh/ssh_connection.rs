use std::io::Read;
use std::net::TcpStream;

use once_cell::unsync::OnceCell;

use log::info;
use ssh2::Session;

use crate::config::config::MgmtConfig;

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

    pub fn exec(&self, cmd: &str) -> (String, i32) {
        let session = self.session.get_or_init(|| {
            let tcp = TcpStream::connect(format!("{}:{}", self.endpoint, self.port)).unwrap();

            info!("Connecting to host {}", self.endpoint);

            let mut sess = Session::new().expect("Could not build up ssh session");

            sess.handshake(&tcp)
                .expect("Could not perform ssh handshake");

            let (username, password) = (self.credentials.username(), self.credentials.password());
            sess.userauth_password(username, password)
                .unwrap_or_else(|error| {
                    panic!(
                        "Authentication has failed with username: {} and password: {}\n Error: {}",
                        username, password, error
                    )
                });

            sess
        });

        let mut channel = session
            .channel_session()
            .expect("Could not create channel for ssh session");

        channel
            .exec(cmd)
            .expect("Execution of command on remote machine over ssh has failed.");

        let mut output = String::new();
        channel
            .read_to_string(&mut output)
            .expect("Could read output of executed commmand over ssh channel");
        let exit_status = channel
            .exit_status()
            .expect("Could retrieve exit code of executed command over ssh");

        (output, exit_status)
    }
}
